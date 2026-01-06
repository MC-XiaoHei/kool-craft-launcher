#![cfg_attr(coverage_nightly, coverage(off))]

use tauri::{Builder, Runtime, Wry};

pub fn register_commands(builder: Builder<Wry>) -> Builder<Wry> {
    builder.invoke_handler(tauri::generate_handler![
        // auth commands
        crate::auth::microsoft::commands::microsoft_account_login,
        // config commands
        crate::config::commands::set_config,
        crate::config::commands::get_config_schemas,
        crate::config::commands::get_config_values_json,
        // theme commands
        crate::theme::commands::refresh_window_theme,
        crate::theme::commands::get_wallpaper,
    ])
}
