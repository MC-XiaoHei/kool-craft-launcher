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

use crate::config::commands::setup_config;
use crate::scheduler::commands::setup_scheduler;
use crate::ui_theme::commands::{register_theme_commands, setup_theme};
use anyhow::{Context, Result};
use std::error::Error;
use std::time::SystemTime;
use tap::Pipe;
use tauri::async_runtime::block_on;
use tauri::plugin::TauriPlugin;
use tauri::{App, Builder, Runtime};

pub fn run() -> Result<()> {
    println!("App started at {:?}", SystemTime::now());

    Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(log_plugin())
        .setup(setup_app_handler)
        .pipe(register_commands)
        .run(tauri::generate_context!())
        .context("error while running tauri application")?;

    Ok(())
}

fn set_async_runtime() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());
}

fn log_plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_log::Builder::default()
        .level(log::LevelFilter::Info)
        .max_file_size(5_000_000 /* bytes */)
        .format(|out, message, record| {
            let full_target = record.target();

            let display_target = full_target
                .strip_prefix("kool_craft_launcher_lib::")
                .unwrap_or(full_target);

            out.finish(format_args!(
                "[{}] [{}] {}",
                record.level(),
                display_target,
                message
            ))
        })
        .build()
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
    builder.pipe(register_theme_commands)
}
