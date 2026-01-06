use super::traits::{ConfigGroup, ConfigPersistence};
use anyhow::{Context, Result, anyhow};
use log::{info, warn};
use macros::inventory;
use parking_lot::{RwLock, RwLockUpgradableReadGuard};
use schemars::schema_for;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use tap::Pipe;
use tokio::sync::Mutex;

pub type UpdateHandler = fn(Value, Value) -> Result<()>;

#[inventory]
pub struct UpdateHandlerInfo {
    pub key: &'static str,
    pub handler: UpdateHandler,
}

pub struct ConfigStore {
    values: RwLock<HashMap<String, Value>>,
    schemas: RwLock<HashMap<String, Value>>,
    persistence: Box<dyn ConfigPersistence>,
    write_lock: Mutex<()>,
}

impl ConfigStore {
    pub fn new(persistence: Box<dyn ConfigPersistence>) -> Self {
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
            info!("Loaded config from {source}");
        } else {
            info!("No config found at {source}, using defaults");
        }
        Ok(())
    }

    pub async fn register<T: ConfigGroup>(&self) -> Result<()> {
        self.register_schema::<T>()?;
        self.perform_evolution::<T>().await?;
        self.validate_or_default::<T>().await?;
        self.get::<T>().post_process()?;
        Ok(())
    }

    pub fn get<T: ConfigGroup>(&self) -> T {
        self.get_value::<T>().unwrap_or_else(|e| {
            let key = T::KEY;
            warn!("Failed to get config '{key}', will using default: {e}");
            T::default()
        })
    }

    pub async fn set<T: ConfigGroup>(&self, value: T) -> Result<()> {
        let old = self.get::<T>();
        let key = T::KEY;
        let json_value =
            serde_json::to_value(value.clone()).context("Config serialization failed")?;
        self.update(key, json_value).await;
        Ok(())
    }

    pub async fn set_by_key(&self, key: impl Into<String>, value: Value) -> Result<()> {
        self.update(key.into().as_str(), value).await
    }

    async fn update(&self, key: &str, value: Value) -> Result<()> {
        self.validate_key_exists(key)?;

        let old = self
            .values
            .write()
            .insert(key.to_string(), value.clone())
            .unwrap_or_default();

        self.save_config()
            .await
            .context("Failed to persist config")?;

        if let Some(handler) = Self::find_update_handler(key) {
            handler(value, old)?;
        }

        Ok(())
    }

    fn find_update_handler(key: &str) -> Option<UpdateHandler> {
        inventory::iter::<UpdateHandlerInfo>
            .into_iter()
            .find(|info| info.key == key)
            .map(|info| info.handler)
    }

    fn get_value<T: ConfigGroup + DeserializeOwned + Default>(&self) -> Result<T> {
        let key = T::KEY;
        let guard = self.values.read();
        let value = guard
            .get(key)
            .context(format!("Failed to get config '{key}'"))?;
        serde_json::from_value(value.clone())
            .context(format!("Config deserialization failed: {key}"))
    }

    fn register_schema<T: ConfigGroup>(&self) -> Result<()> {
        let key = T::KEY;
        let schema = schema_for!(T);
        self.schemas
            .write()
            .insert(key.to_string(), serde_json::to_value(schema)?);
        Ok(())
    }

    async fn perform_evolution<T: ConfigGroup>(&self) -> Result<()> {
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

    async fn validate_or_default<T: ConfigGroup>(&self) -> Result<()> {
        let key = T::KEY;

        if let Err(e) = self.get_value::<T>() {
            warn!(
                "Config '{key}' data is corrupted or mismatched and will resetting to default: {e:?}"
            );
            self.set(T::default()).await?;
        }

        Ok(())
    }

    async fn save_config(&self) -> Result<()> {
        let _guard = self.write_lock.lock().await;
        let json = serde_json::to_string_pretty(&*self.values.read())?;
        self.persistence.save(json).await?;
        info!("Config saved to {}", self.persistence.source_description());
        Ok(())
    }

    fn validate_key_exists(&self, key: &str) -> Result<()> {
        if self.schemas.read().contains_key(key) {
            Ok(())
        } else {
            Err(anyhow!("Config '{key}' is not registered"))
        }
    }

    fn try_parse_json(&self, content: &str) -> Result<HashMap<String, Value>> {
        serde_json::from_str(content).context("Failed to parse config JSON")
    }
}
