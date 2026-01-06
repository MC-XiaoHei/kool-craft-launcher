use crate::config::modules::theme::{ThemeEffect, ThemeConfig, ThemeMode};
use crate::utils::os_info::{is_macos, is_windows, is_windows_11};
use log::{info, warn};
use os_info::Info;
use tauri::{Runtime, Theme, WebviewWindow};
use window_vibrancy::{
    NSVisualEffectMaterial, apply_vibrancy as apply_vibrancy_internal, clear_vibrancy,
};
use window_vibrancy::{apply_mica as apply_mica_internal, clear_mica};

pub fn apply_effect<R: Runtime>(window: &WebviewWindow<R>, config: &ThemeConfig) {
    let info = os_info::get();
    clear_previous_effect(window, &info);
    let is_dark = get_is_dark(window, config);
    apply_config_effect(window, &config.effect, &info, is_dark);
}

fn get_is_dark<R: Runtime>(window: &WebviewWindow<R>, config: &ThemeConfig) -> bool {
    let system_is_dark = window.theme().unwrap_or(Theme::Light) == Theme::Dark;
    match config.mode {
        ThemeMode::Dark => true,
        ThemeMode::Light => false,
        ThemeMode::Auto => system_is_dark,
    }
}

fn clear_previous_effect<R: Runtime>(window: &WebviewWindow<R>, info: &Info) {
    if is_windows(info) {
        let _ = clear_mica(window).map_err(|err| {
            warn!("Failed to clear mica effect: {}", err);
        });
    }

    if is_macos(info) {
        let _ = clear_vibrancy(window).map_err(|err| {
            warn!("Failed to clear vibrancy: {}", err);
        });
    }
}

fn apply_config_effect<R: Runtime>(
    window: &WebviewWindow<R>,
    effect: &ThemeEffect,
    info: &Info,
    is_dark: bool,
) {
    match effect {
        ThemeEffect::Mica if is_windows(&info) => {
            apply_mica(window, is_dark);
        }

        ThemeEffect::Vibrancy if is_macos(&info) => {
            apply_vibrancy(window, is_dark);
        }

        ThemeEffect::Wallpaper => {}

        _ if is_windows_11(&info) => {
            apply_mica(window, is_dark);
        }

        _ if is_macos(&info) => {
            apply_vibrancy(window, is_dark);
        }

        _ => {}
    }
}

fn apply_vibrancy<R: Runtime>(window: &WebviewWindow<R>, is_dark: bool) {
    let material = if is_dark {
        NSVisualEffectMaterial::HudWindow
    } else {
        NSVisualEffectMaterial::UnderWindowBackground
    };
    let _ = apply_vibrancy_internal(window, material, None, None);
}

fn apply_mica<R: Runtime>(window: &WebviewWindow<R>, is_dark: bool) {
    let _ = apply_mica_internal(window, Some(is_dark));
}
