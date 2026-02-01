use super::traits::{SettingsGroup, SettingsPersistence};
use crate::settings::events::SettingsUpdateEvent;
use anyhow::{Context, Result, anyhow};
use log::{info, warn};
use macros::inventory;
use parking_lot::{RwLock, RwLockUpgradableReadGuard};
use schemars::schema_for;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value::Null;
use serde_json::{Value, json};
use std::collections::HashMap;
use tap::Pipe;
use tokio::sync::Mutex;

pub type UpdateHandler = fn(Value, Value) -> Result<()>;

#[inventory]
pub struct UpdateHandlerInfo {
    pub key: &'static str,
    pub handler: UpdateHandler,
}

pub struct SettingsStore {
    values: RwLock<HashMap<String, Value>>,
    schemas: RwLock<HashMap<String, Value>>,
    persistence: Box<dyn SettingsPersistence>,
    write_lock: Mutex<()>,
}

impl SettingsStore {
    pub fn new(persistence: Box<dyn SettingsPersistence>) -> Self {
        Self {
            persistence,
            values: RwLock::new(HashMap::new()),
            schemas: RwLock::new(HashMap::new()),
            write_lock: Mutex::new(()),
        }
    }

    pub fn get_schemas(&self) -> HashMap<String, Value> {
        self.schemas.read().clone()
    }

    pub fn get_values(&self) -> HashMap<String, Value> {
        self.values.read().clone()
    }

    pub async fn load(&self) -> Result<()> {
        let _guard = self.write_lock.lock().await;
        let source = self.persistence.source_description();
        if let Some(content) = self.persistence.load().await? {
            let data = self.try_parse_json(&content)?;
            *self.values.write() = data;
            info!("Loaded settings from {source}");
        } else {
            info!("No settings found at {source}, will using defaults");
        }
        Ok(())
    }

    pub async fn register<T: SettingsGroup>(&self) -> Result<()> {
        self.register_schema::<T>()?;
        self.perform_evolution::<T>().await?;
        self.validate_or_default::<T>().await?;
        self.update_atomic::<T, _>(|original| {
            original.post_process();
            Ok(true)
        }).await?;
        info!("{:?}", self.get::<T>());
        Ok(())
    }

    pub fn get<T: SettingsGroup>(&self) -> T {
        self.get_value::<T>().unwrap_or_else(|e| {
            let key = T::KEY;
            warn!("Failed to get settings '{key}', will using default: {e}");
            T::default()
        })
    }

    pub async fn set<T: SettingsGroup>(&self, value: T) -> Result<()> {
        let old = self.get::<T>();
        let key = T::KEY;
        let json_value =
            serde_json::to_value(value.clone()).context("Settings serialization failed")?;
        self.update(key, json_value).await?;
        Ok(())
    }

    pub async fn set_by_key(&self, key: impl Into<String>, value: Value) -> Result<()> {
        self.update(key.into().as_str(), value).await
    }

    pub async fn update_atomic<T, F>(&self, func: F) -> Result<()>
    where
        T: SettingsGroup + Serialize + DeserializeOwned + Default,
        F: FnOnce(&mut T) -> Result<bool>,
    {
        let key = T::KEY;
        self.mutate_state(key, |json_value| {
            let mut settings: T = serde_json::from_value(json_value.clone()).unwrap_or_default();

            if func(&mut settings)? {
                *json_value = serde_json::to_value(settings).context("Settings serialization failed")?;
                Ok(true)
            } else {
                Ok(false)
            }
        }).await
    }

    async fn update(&self, key: &str, value: Value) -> Result<()> {
        self.mutate_state(key, |json_value| {
            *json_value = value.clone();
            Ok(true)
        }).await
    }

    async fn mutate_state<F>(&self, key: &str, func: F) -> Result<()>
    where
        F: FnOnce(&mut Value) -> Result<bool>,
    {
        self.validate_key_exists(key)?;

        let _file_guard = self.write_lock.lock().await;

        let mut old_value = Null;
        let mut new_value = Null;
        let mut should_save = false;

        {
            let mut values_guard = self.values.write();
            let mut current_json = values_guard.get(key).cloned().unwrap_or(Null);

            old_value = current_json.clone();

            if func(&mut current_json)? {
                should_save = true;
                new_value = current_json;
                values_guard.insert(key.to_string(), new_value.clone());
            }
        }

        if should_save {
            self.save_inner().await.context("Failed to persist settings")?;

            if old_value != Null && let Some(handler) = Self::find_update_handler(key) {
                handler(new_value.clone(), old_value)?;
            }

            self.sync_to_frontend(key, new_value).await?;
        }

        Ok(())
    }

    async fn sync_to_frontend(&self, key: &str, value: Value) -> Result<()> {
        SettingsUpdateEvent::new(key, value)?.emit()
    }

    fn find_update_handler(key: &str) -> Option<UpdateHandler> {
        inventory::iter::<UpdateHandlerInfo>
            .into_iter()
            .find(|info| info.key == key)
            .map(|info| info.handler)
    }

    fn get_value<T: SettingsGroup + DeserializeOwned + Default>(&self) -> Result<T> {
        let key = T::KEY;
        let guard = self.values.read();
        let value = guard
            .get(key)
            .context(format!("Failed to get settings '{key}'"))?;
        serde_json::from_value(value.clone())
            .context(format!("Settings deserialization failed: {key}"))
    }

    fn register_schema<T: SettingsGroup>(&self) -> Result<()> {
        let key = T::KEY;
        let schema = schema_for!(T);
        self.schemas
            .write()
            .insert(key.to_string(), serde_json::to_value(schema)?);
        Ok(())
    }

    async fn perform_evolution<T: SettingsGroup>(&self) -> Result<()> {
        let key = T::KEY;
        let all_values_guard = self.values.upgradable_read();

        let Some(mut current_val) = all_values_guard.get(key).cloned() else {
            return Ok(());
        };

        T::evolve(&mut current_val, &all_values_guard)?;

        let mut write_guard = RwLockUpgradableReadGuard::upgrade(all_values_guard);
        write_guard.insert(key.to_string(), current_val);

        Ok(())
    }

    async fn validate_or_default<T: SettingsGroup>(&self) -> Result<()> {
        let key = T::KEY;

        let refined_value = self
            .get_value::<T>()
            .unwrap_or_else(|e| {
                warn!("Settings '{key}' is corrupted or missing, resetting to default: {e:?}");
                T::default()
            })
            .pipe(|s| serde_json::to_value(s))?;

        let values = self.values.upgradable_read();
        if values.get(key) != Some(&refined_value) {
            let mut values = RwLockUpgradableReadGuard::upgrade(values);
            values.insert(key.to_string(), refined_value);
        }

        Ok(())
    }

    pub async fn save(&self) -> Result<()> {
        let _guard = self.write_lock.lock().await;
        self.save_inner().await
    }

    async fn save_inner(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&*self.values.read())?;
        self.persistence.save(json).await?;
        info!(
            "Settings saved to {}",
            self.persistence.source_description()
        );
        Ok(())
    }

    fn validate_key_exists(&self, key: &str) -> Result<()> {
        if self.schemas.read().contains_key(key) {
            Ok(())
        } else {
            Err(anyhow!("Settings '{key}' is not registered"))
        }
    }

    fn try_parse_json(&self, content: &str) -> Result<HashMap<String, Value>> {
        serde_json::from_str(content).context("Failed to parse settings JSON")
    }
}
