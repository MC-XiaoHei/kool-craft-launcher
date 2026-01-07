#![cfg_attr(coverage_nightly, coverage(off))]

use crate::config::modules::theme::ThemeConfig;
use crate::config::persistence::FilePersistence;
use crate::config::store::ConfigStore;
use crate::constants::file_system::CONFIG_FILE_NAME;
use crate::utils::command::CommandResult;
use crate::utils::dirs::app_dir;
use anyhow::{Result, anyhow};
use futures::future::BoxFuture;
use log::{info, warn};
use macros::inventory;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tap::Pipe;
use tauri::{App, Builder, Manager, Runtime, State, command};

#[inventory]
pub struct ConfigRegisterHook {
    pub handler: fn(Arc<ConfigStore>) -> BoxFuture<'static, Result<()>>,
}

pub async fn setup_config(app: &App) -> Result<()> {
    let store = create_config_store()?;
    init_config_store(store.clone()).await?;
    app.manage(store);
    Ok(())
}

fn create_config_store() -> Result<Arc<ConfigStore>> {
    let persistence = FilePersistence::new(config_path()?);
    let store = ConfigStore::new(Box::new(persistence));
    Ok(Arc::new(store))
}

async fn init_config_store(store: Arc<ConfigStore>) -> Result<()> {
    store.load().await?;
    register_config_modules(store).await?;
    Ok(())
}

async fn register_config_modules(store: Arc<ConfigStore>) -> Result<()> {
    for hook in inventory::iter::<ConfigRegisterHook> {
        let handler = hook.handler;
        handler(store.clone()).await?;
    }
    Ok(())
}

#[command]
pub async fn set_config(
    store: State<'_, Arc<ConfigStore>>,
    key: String,
    value: String,
) -> CommandResult<()> {
    let value = serde_json::from_str(&value).map_err(|e| anyhow!(e))?;
    store.set_by_key(key, value).await?;
    Ok(())
}

#[command]
pub async fn get_config_schemas(
    store: State<'_, Arc<ConfigStore>>,
) -> CommandResult<HashMap<String, Value>> {
    Ok(store.get_schemas())
}

#[command]
pub async fn get_config_values_json(
    store: State<'_, Arc<ConfigStore>>,
) -> CommandResult<HashMap<String, String>> {
    store
        .get_values()
        .iter()
        .map(|(key, value)| {
            let value_json = serde_json::to_string(value)
                .map_err(|e| warn!("Fail to serialize config value with key {key} to json: {e:?}"))
                .unwrap_or("{}".to_string());
            (key.clone(), value_json)
        })
        .collect::<HashMap<_, _>>()
        .pipe(Ok)
}

fn config_path() -> Result<PathBuf> {
    Ok(app_dir()?.join(CONFIG_FILE_NAME))
}
