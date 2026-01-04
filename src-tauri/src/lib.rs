#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod auth;
mod config;
mod constants;
mod game_assets;
mod game_launcher;
mod game_resolver;
mod i18n;
mod java_runtime;
mod scheduler;
mod ui_theme;
mod utils;

use crate::config::commands::{register_config_commands, setup_config};
use crate::constants::file_system::LOG_DIR_NAME;
use crate::scheduler::commands::setup_scheduler;
use crate::ui_theme::commands::{register_theme_commands, setup_theme};
use crate::utils::dirs::app_dir;
use anyhow::{Context, Result};
use chrono::Local;
use std::error::Error;
use tap::Pipe;
use tauri::async_runtime::block_on;
use tauri::plugin::TauriPlugin;
use tauri::{App, Builder, Runtime};

pub fn run() -> Result<()> {
    Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(log_plugin()?)
        .setup(setup_app_handler)
        .pipe(register_commands)
        .run(tauri::generate_context!())
        .context("error while running tauri application")?;

    Ok(())
}

fn log_plugin<R: Runtime>() -> Result<TauriPlugin<R>> {
    use tauri_plugin_log::*;
    let plugin = Builder::default()
        .level(log::LevelFilter::Info)
        .target(Target::new(TargetKind::Webview))
        .target(Target::new(TargetKind::Folder {
            path: app_dir()?.join(LOG_DIR_NAME),
            file_name: Some(Local::now().format("%Y-%m-%d").to_string()),
        }))
        .format(|out, message, record| {
            let full_target = record.target();

            let display_target = full_target
                .strip_prefix("kool_craft_launcher_lib::")
                .unwrap_or(full_target);

            out.finish(format_args!(
                "[{}] [{}] [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                display_target,
                message
            ))
        })
        .build();
    Ok(plugin)
}

fn setup_app_handler(app: &mut App) -> Result<(), Box<dyn Error>> {
    block_on(async { setup_app(app).await }).map_err(|e| e.into())
}

async fn setup_app(app: &mut App) -> Result<()> {
    setup_config(app).await?;
    setup_theme(app)?;
    setup_scheduler(app);
    Ok(())
}

pub fn register_commands<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    builder
        .pipe(register_config_commands)
        .pipe(register_theme_commands)
}
