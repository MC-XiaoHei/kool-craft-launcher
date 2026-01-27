use crate::settings::traits::SettingsGroup;
use crate::theme::effect::apply_effect;
use crate::utils::global_app_handle::get_global_app_handle;
use anyhow::Result;
use log::info;
use macros::{settings, settings_type};
use tauri::Manager;
use crate::settings::components::Password;

#[settings(name = "theme", post_process = post_process, update_handler = on_update)]
pub struct ThemeSettings {
    pub effect: ThemeEffect,
    pub mode: ThemeMode,
    pub test: Password,
    pub test_group: TestGroup,
}

#[settings_type]
pub struct TestGroup {
    pub password: Password,
}

fn post_process(settings: &mut ThemeSettings) -> Result<()> {
    let os_info = os_info::get();
    settings.sanitize(&os_info);
    Ok(())
}

fn on_update(neo: &ThemeSettings, _old: ThemeSettings) -> Result<()> {
    get_global_app_handle()?
        .webview_windows()
        .values()
        .for_each(|window| apply_effect(window, neo));
    Ok(())
}

#[settings_type]
#[derive(PartialEq, Eq)]
pub enum ThemeEffect {
    #[default]
    Auto,
    Mica,
    Vibrancy,
    Wallpaper,
}

#[settings_type]
#[derive(PartialEq, Eq)]
pub enum ThemeMode {
    #[default]
    Auto,
    Dark,
    Light,
}
