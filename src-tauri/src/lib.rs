#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod auth;
mod constants;
mod game_assets;
mod game_launcher;
mod game_resolver;
mod i18n;
mod java_runtime;
mod scheduler;
mod ui_theme;
mod utils;

use crate::scheduler::commands::setup_scheduler;
use crate::ui_theme::commands::{register_theme_commands, setup_theme};
use log::info;
use std::error::Error;
use tap::Pipe;
use tauri::plugin::TauriPlugin;
use tauri::{App, Runtime};

pub fn run() {
    info!("App started at {:?}", std::time::SystemTime::now());
    tauri::async_runtime::set(tokio::runtime::Handle::current());
    tauri::Builder::default()
        .plugin(log_plugin())
        .setup(setup_app)
        .pipe(register_theme_commands)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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

fn setup_app(app: &mut App) -> Result<(), Box<dyn Error>> {
    setup_scheduler(app);
    setup_theme(app)?;
    Ok(())
}
