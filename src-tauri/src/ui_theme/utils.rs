use crate::ui_theme::model::{EffectMode, ThemeConfig, ThemeMode};
use tauri::{Runtime, Theme, WebviewWindow};
#[cfg(target_os = "macos")]
use window_vibrancy::{NSVisualEffectMaterial, apply_vibrancy as apply_vibrancy_internal};
#[cfg(target_os = "windows")]
use window_vibrancy::{apply_mica as apply_mica_internal, clear_mica};

#[cfg(target_os = "macos")]
fn apply_vibrancy<R: Runtime>(window: &WebviewWindow<R>, is_dark: bool) {
    let material = if is_dark {
        NSVisualEffectMaterial::HudWindow
    } else {
        NSVisualEffectMaterial::UnderWindowBackground
    };
    let _ = apply_vibrancy_internal(window, material, None, None);
}

#[cfg(target_os = "windows")]
fn apply_mica<R: Runtime>(window: &WebviewWindow<R>, is_dark: bool) {
    let _ = apply_mica_internal(window, Some(is_dark));
}

pub fn apply_effect<R: Runtime>(window: &WebviewWindow<R>, config: &ThemeConfig) {
    let system_is_dark = window.theme().unwrap_or(Theme::Light) == Theme::Dark;
    let is_dark = match config.theme {
        ThemeMode::Dark => true,
        ThemeMode::Light => false,
        ThemeMode::Auto => system_is_dark,
    };

    #[cfg(target_os = "windows")]
    let _ = clear_mica(window);

    #[cfg(target_os = "macos")]
    if config.mode == "static" {
        let _ = clear_vibrancy(window);
    }

    match config.effect {
        #[cfg(target_os = "windows")]
        EffectMode::Mica => {
            apply_mica(window, is_dark);
        }

        #[cfg(target_os = "macos")]
        EffectMode::Vibrancy => {
            apply_vibrancy(window, is_dark);
        }

        EffectMode::Wallpaper => {}

        _ => {
            #[cfg(target_os = "windows")]
            if is_windows_11() {
                apply_mica(window, is_dark);
            }

            #[cfg(target_os = "macos")]
            {
                apply_vibrancy(window, is_dark);
            }
        }
    }
}

fn is_windows_11() -> bool {
    let info = os_info::get();

    if info.os_type() != os_info::Type::Windows {
        return false;
    }

    match info.version() {
        os_info::Version::Semantic(major, _, build) => *major == 10 && *build >= 22000,
        _ => false,
    }
}
