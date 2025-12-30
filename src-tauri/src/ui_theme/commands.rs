use crate::ui_theme::model::ThemeConfig;
use crate::ui_theme::utils::apply_effect;
use anyhow::Context;
use base64::Engine;
use base64::engine::general_purpose;
use image::{ImageFormat, ImageReader};
use log::{error, info};
use std::io::Cursor;
use std::time::Instant;
use tauri::{Builder, Runtime, WebviewWindow, command};

pub fn register_theme_commands<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    builder.invoke_handler(tauri::generate_handler![
        load_theme_config,
        set_theme_config,
        get_wallpaper
    ])
}

#[command]
fn load_theme_config() -> ThemeConfig {
    ThemeConfig::load()
}

#[command]
async fn set_theme_config<R: Runtime>(
    window: WebviewWindow<R>,
    config: ThemeConfig,
) -> Result<(), String> {
    info!("Setting ui_theme config: {:?}", config);

    apply_effect(&window, &config);
    config.save();

    Ok(())
}

#[command]
async fn get_wallpaper() -> Result<String, String> {
    let start = Instant::now();

    let result = tauri::async_runtime::spawn_blocking(get_wallpaper_data_url)
        .await
        .map_err(|e| format!("Task join error: {e}"))?;

    match &result {
        Ok(_) => info!("Wallpaper processed successfully in {:?}", start.elapsed()),
        Err(e) => error!("Failed to process wallpaper: {e:}"),
    }

    result.map_err(|e| e.to_string())
}

fn get_wallpaper_data_url() -> anyhow::Result<String> {
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
