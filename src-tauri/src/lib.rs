#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod assets;
mod auth;
mod i18n;
mod java_runtime;
mod launcher;
mod resolver;
mod scheduler;
mod theme;

use crate::scheduler::Scheduler;
use crate::theme::commands::register_theme_commands;
use crate::theme::utils::apply_effect;
use log::info;
use std::error::Error;
use tap::Pipe;
use tauri::plugin::TauriPlugin;
use tauri::{App, Manager, Runtime};
use theme::model::{EffectMode, ThemeConfig};

pub fn run() {
    info!("App started at {:?}", std::time::SystemTime::now());
    tauri::Builder::default()
        .plugin(log_plugin())
        .manage(Scheduler::new(32)) // TODO: make concurrency limit configurable
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
    let window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;

    let config = ThemeConfig::load();
    apply_effect(&window, &config);
    config.save();

    if matches!(config.effect, EffectMode::Auto | EffectMode::Mica) {
        window.show()?;
    }

    Ok(())
}
