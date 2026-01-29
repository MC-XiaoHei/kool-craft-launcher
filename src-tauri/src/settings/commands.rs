use crate::settings::modules::theme::ThemeSettings;
use crate::settings::persistence::FilePersistence;
use crate::settings::store::SettingsStore;
use crate::constants::file_system::SETTINGS_FILE_NAME;
use crate::utils::command::CommandResult;
use crate::utils::dirs::app_dir;
use anyhow::{Result, anyhow};
use futures::future::BoxFuture;
use log::{info, warn};
use macros::{command, inventory};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tap::Pipe;
use tauri::{App, Builder, Manager, Runtime, State};

#[inventory]
pub struct SettingsRegisterHook {
    pub handler: fn(Arc<SettingsStore>) -> BoxFuture<'static, Result<()>>,
}

pub async fn setup_settings(app: &App) -> Result<()> {
    let store = create_settings_store()?;
    init_settings_store(store.clone()).await?;
    app.manage(store);
    Ok(())
}

fn create_settings_store() -> Result<Arc<SettingsStore>> {
    let persistence = FilePersistence::new(settings_path()?);
    let store = SettingsStore::new(Box::new(persistence));
    Ok(Arc::new(store))
}

async fn init_settings_store(store: Arc<SettingsStore>) -> Result<()> {
    store.load().await?;
    register_settings_modules(store.clone()).await?;
    store.save().await?;
    Ok(())
}

async fn register_settings_modules(store: Arc<SettingsStore>) -> Result<()> {
    for hook in inventory::iter::<SettingsRegisterHook> {
        let handler = hook.handler;
        handler(store.clone()).await?;
    }
    Ok(())
}

#[command]
pub async fn set_settings(
    store: State<'_, Arc<SettingsStore>>,
    key: String,
    value: String,
) -> CommandResult<()> {
    let value = serde_json::from_str(&value).map_err(|e| anyhow!(e))?;
    store.set_by_key(key, value).await?;
    Ok(())
}

#[command]
pub async fn get_settings_schemas(
    store: State<'_, Arc<SettingsStore>>,
) -> CommandResult<HashMap<String, Value>> {
    Ok(store.get_schemas())
}

#[command]
pub async fn get_settings_values_json(
    store: State<'_, Arc<SettingsStore>>,
) -> CommandResult<HashMap<String, String>> {
    store
        .get_values()
        .iter()
        .map(|(key, value)| {
            let value_json = serde_json::to_string(value)
                .map_err(|e| warn!("Fail to serialize settings value with key {key} to json: {e:?}"))
                .unwrap_or("{}".to_string());
            (key.clone(), value_json)
        })
        .collect::<HashMap<_, _>>()
        .pipe(Ok)
}

fn settings_path() -> Result<PathBuf> {
    Ok(app_dir()?.join(SETTINGS_FILE_NAME))
}
