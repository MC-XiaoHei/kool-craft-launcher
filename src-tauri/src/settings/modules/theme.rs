use crate::settings::components::Password;
use crate::settings::traits::SettingsGroup;
use crate::theme::effect::apply_effect;
use crate::utils::global_app_handle::get_global_app_handle;
use anyhow::Result;
use log::{info, warn};
use macros::{settings, settings_type};
use schemars::Schema;
use tauri::Manager;

#[settings(name = "theme", post_process = post_process, update_handler = on_update, no_default)]
pub struct ThemeSettings {
    pub effect: ThemeEffect,
    pub mode: ThemeMode,
    pub primary_hex: String,
}

const DEFAULT_PRIMARY_HEX: &str = "#01ca8a";

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            effect: ThemeEffect::Auto,
            mode: ThemeMode::Auto,
            primary_hex: DEFAULT_PRIMARY_HEX.into(),
        }
    }
}

fn post_process(settings: &mut ThemeSettings) -> Result<()> {
    let os_info = os_info::get();
    settings.effect.sanitize(&os_info);
    sanitize_primary_color(settings);
    Ok(())
}

fn sanitize_primary_color(settings: &mut ThemeSettings) {
    if settings.primary_hex.is_empty() {
        settings.primary_hex = DEFAULT_PRIMARY_HEX.into();
        warn!("Primary color is missing, resetting to default ${DEFAULT_PRIMARY_HEX}");
    }
}

fn on_update(neo: &ThemeSettings, _old: ThemeSettings) -> Result<()> {
    get_global_app_handle()
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
