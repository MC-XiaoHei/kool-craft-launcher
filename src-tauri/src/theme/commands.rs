#![cfg_attr(coverage_nightly, coverage(off))]

use crate::config::modules::theme::{ThemeConfig, ThemeEffect};
use crate::config::store::ConfigStore;
use crate::theme::effect::apply_effect;
use crate::utils::command::CommandResult;
use anyhow::Result;
use anyhow::{Context, anyhow};
use base64::Engine;
use base64::engine::general_purpose;
use image::{ImageFormat, ImageReader};
use macros::command;
use std::io::Cursor;
use std::sync::Arc;
use tauri::{App, Builder, Manager, Runtime, State, WebviewWindow, Wry};
use tokio::task::spawn_blocking;

pub fn setup_theme(app: &mut App) -> Result<()> {
    let window = app
        .get_webview_window("main")
        .context("Main window not found")?;
    let store = app.state::<Arc<ConfigStore>>();

    let config = store.get::<ThemeConfig>();
    apply_effect(&window, &config);

    if matches!(config.effect, ThemeEffect::Auto | ThemeEffect::Mica) {
        window.show()?;
    }

    Ok(())
}

#[command]
pub async fn refresh_window_theme(
    window: WebviewWindow<Wry>,
    store: State<'_, Arc<ConfigStore>>,
) -> CommandResult<()> {
    let config = store.get::<ThemeConfig>();
    apply_effect(&window, &config);
    Ok(())
}

#[command]
pub async fn get_wallpaper_data_url() -> CommandResult<String> {
    spawn_blocking(get_wallpaper_data_url_sync)
        .await
        .map_err(|e| anyhow!(e))?
        .context("Failed to get wallpaper data")
        .map_err(Into::into)
}

fn get_wallpaper_data_url_sync() -> Result<String> {
    let path_str = wallpaper::get().map_err(|e| anyhow::anyhow!("System wallpaper error: {e}"))?;

    let img = ImageReader::open(&path_str)
        .with_context(|| format!("Failed to open image at {path_str}"))?
        .decode()
        .context("Failed to decode image format")?;

    let mut buffer = Cursor::new(Vec::new());
    img.write_to(&mut buffer, ImageFormat::Jpeg)
        .context("Failed to encode image to JPEG")?;

    let b64 = general_purpose::STANDARD.encode(buffer.get_ref());

    Ok(format!("data:image/jpeg;base64,{b64}"))
}
