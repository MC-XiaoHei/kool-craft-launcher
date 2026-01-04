use crate::constants::file_system::LAUNCHER_DIR_NAME;
use anyhow::Context;
pub use anyhow::Result;
use log::warn;
use std::env;
use std::path::PathBuf;

pub fn app_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir();
    if let Some(home_dir) = home_dir {
        return Ok(home_dir.join(LAUNCHER_DIR_NAME));
    }
    warn!("No HOME directory found, using current directory");
    let current_dir = env::current_dir().context("Could not get current directory")?;
    Ok(current_dir.join(LAUNCHER_DIR_NAME))
}
