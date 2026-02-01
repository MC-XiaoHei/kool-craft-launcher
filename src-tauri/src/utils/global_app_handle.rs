use anyhow::{Result, anyhow};
use log::error;
use std::sync::OnceLock;
use tap::Pipe;
use tauri::{App, AppHandle};

static GLOBAL_APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub fn set_global_app_handle(app: &mut App) -> Result<()> {
    GLOBAL_APP_HANDLE
        .set(app.handle().clone())
        .map_err(|e| anyhow!("failed to set global app handle: {e:?}"))?;
    Ok(())
}

pub fn get_global_app_handle() -> AppHandle {
    GLOBAL_APP_HANDLE
        .get()
        .expect("Internal Error: Global app handle not set") // this should never happen
        .clone()
}
