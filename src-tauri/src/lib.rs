#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(debug_assertions, allow(unused))]

mod auth;
mod commands;
mod config;
mod constants;
mod game_assets;
mod game_launcher;
mod game_resolver;
mod i18n;
mod java_runtime;
mod scheduler;
mod theme;
mod utils;

use crate::config::commands::setup_config;
use crate::constants::file_system::LOG_DIR_NAME;
use crate::scheduler::commands::setup_scheduler;
use crate::theme::commands::setup_theme;
use crate::utils::dirs::app_dir;
use anyhow::{Context, Result};
use chrono::Local;
use commands::register_commands;
use futures::FutureExt;
use specta_typescript::Typescript;
use std::error::Error;
use std::fmt::Arguments;
use tap::Pipe;
use tauri::async_runtime::block_on;
use tauri::plugin::TauriPlugin;
use tauri::{App, Builder, Wry};

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run() -> Result<()> {
    #[cfg(debug_assertions)]
    export_types();

    Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(log_plugin()?)
        .setup(setup_app_handler)
        .pipe(register_commands)
        .run(tauri::generate_context!())
        .context("error while running tauri application")?;

    Ok(())
}

// use .unwrap() here is safe, because this function only calls in debug or generating type bindings
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn export_types() {
    Typescript::default()
        .export_to("../src/bindings/types.ts", &specta::export())
        .unwrap();
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn log_plugin() -> Result<TauriPlugin<Wry>> {
    use log::*;
    use tauri_plugin_log::*;

    let frontend_console_target = Target::new(TargetKind::Webview);

    let app_dir_logs_target = Target::new(TargetKind::Folder {
        path: app_dir()?.join(LOG_DIR_NAME),
        file_name: Some(Local::now().format("%Y-%m-%d").to_string()),
    });

    fn formatter(out: fern::FormatCallback, message: &Arguments, record: &Record) {
        let full_target = record.target();

        let display_target = full_target
            .strip_prefix("kool_craft_launcher_lib::")
            .unwrap_or(full_target);

        let result = format_args!(
            "[{}] [{}] [{}] {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            display_target,
            message
        );

        out.finish(result)
    }

    let plugin = Builder::default()
        .level(LevelFilter::Info)
        .target(frontend_console_target)
        .target(app_dir_logs_target)
        .format(formatter)
        .build();
    Ok(plugin)
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn setup_app_handler(app: &mut App) -> Result<(), Box<dyn Error>> {
    block_on(async { setup_app(app).await }).map_err(|e| e.into())
}

#[cfg_attr(coverage_nightly, coverage(off))]
async fn setup_app(app: &mut App) -> Result<()> {
    setup_config(app).await?;
    setup_theme(app)?;
    setup_scheduler(app);
    Ok(())
}
