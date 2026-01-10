use anyhow::{Result, anyhow};
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

pub fn get_global_app_handle() -> Result<AppHandle> {
    GLOBAL_APP_HANDLE
        .get()
        .ok_or_else(|| anyhow!("no global app handle"))?
        .clone()
        .pipe(Ok)
}
