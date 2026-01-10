#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(debug_assertions, allow(unused))]

mod auth;
mod config;
mod constants;
mod game_assets;
mod game_launcher;
mod game_resolver;
mod i18n;
mod ipc;
mod java_runtime;
mod scheduler;
mod theme;
pub mod utils;

use crate::config::commands::setup_config;
use crate::constants::file_system::LOG_DIR_NAME;
use crate::ipc::command::command_handler;
use crate::scheduler::commands::setup_scheduler;
use crate::theme::commands::setup_theme;
use crate::utils::dirs::app_dir;
use crate::utils::global_app_handle::set_global_app_handle;
use anyhow::{Context, Result};
use chrono::Local;
use futures::FutureExt;
use std::error::Error;
use std::fmt::Arguments;
use tap::Pipe;
use tauri::async_runtime::block_on;
use tauri::plugin::TauriPlugin;
use tauri::{App, Builder, Wry};
use utils::codegen::do_codegen;

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run() -> Result<()> {
    #[cfg(debug_assertions)]
    do_codegen();

    Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(log_plugin()?)
        .setup(setup_app_handler)
        .invoke_handler(command_handler())
        .run(tauri::generate_context!())
        .context("error while running tauri application")?;

    Ok(())
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

        let current_time = Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_level = record.level();
        let result = format_args!("[{current_time}] [{log_level}] [{display_target}] {message}");

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
    set_global_app_handle(app)?;
    setup_config(app).await?;
    setup_theme(app)?;
    setup_scheduler(app);
    Ok(())
}
