use crate::config::traits::ConfigGroup;
use crate::theme::effect::apply_effect;
use crate::utils::global_app_handle::get_global_app_handle;
use anyhow::Result;
use log::info;
use macros::{config, config_type};
use tauri::Manager;

#[config(name = "theme", post_process = post_process, update_handler = on_update)]
#[serde(rename_all = "camelCase")]
pub struct ThemeConfig {
    pub effect: ThemeEffect,
    pub mode: ThemeMode,
}

fn post_process(config: &mut ThemeConfig) -> Result<()> {
    let os_info = os_info::get();
    config.sanitize(&os_info);
    Ok(())
}

fn on_update(neo: &ThemeConfig, _old: ThemeConfig) -> Result<()> {
    get_global_app_handle()?
        .webview_windows()
        .values()
        .for_each(|window| apply_effect(window, neo));
    Ok(())
}

#[config_type]
#[derive(Default, PartialEq, Eq)]
pub enum ThemeEffect {
    #[default]
    Auto,
    Mica,
    Vibrancy,
    Wallpaper,
}

#[config_type]
#[derive(Default, PartialEq, Eq)]
pub enum ThemeMode {
    #[default]
    Auto,
    Dark,
    Light,
}
