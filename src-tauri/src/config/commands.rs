use crate::config::modules::theme::ThemeConfig;
use crate::config::persistence::FilePersistence;
use crate::config::store::ConfigStore;
use crate::constants::file_system::CONFIG_FILE_NAME;
use crate::utils::dirs::app_dir;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{App, Builder, Manager, Runtime, State, command};

pub async fn setup_config(app: &App) -> Result<()> {
    let persistence = FilePersistence::new(config_path()?);
    let store = ConfigStore::new(Box::new(persistence));
    store.load().await?;
    register_config_modules(&store).await?;
    app.manage(store);
    Ok(())
}

pub fn register_config_commands<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    builder.invoke_handler(tauri::generate_handler![get_schemas, get_values])
}

async fn register_config_modules(store: &ConfigStore) -> Result<()> {
    store.register::<ThemeConfig>().await?;
    Ok(())
}

#[command]
async fn get_schemas(store: State<'_, ConfigStore>) -> Result<HashMap<String, Value>, ()> {
    Ok(store.export_schemas())
}

#[command]
async fn get_values(store: State<'_, ConfigStore>) -> Result<HashMap<String, Value>, ()> {
    Ok(store.export_values())
}

fn config_path() -> Result<PathBuf> {
    Ok(app_dir()?.join(CONFIG_FILE_NAME))
}
