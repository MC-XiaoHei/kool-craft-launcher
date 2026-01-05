#![cfg_attr(debug_assertions, allow(unused))]

use tauri::{Builder, Runtime, Wry};

pub fn register_commands(builder: Builder<Wry>) -> Builder<Wry> {
    builder.invoke_handler(tauri::generate_handler![
        // auth commands
        crate::auth::microsoft::commands::microsoft_account_login,
        // config commands
        crate::config::commands::get_schemas,
        crate::config::commands::get_values,
        // theme commands
        crate::theme::commands::get_theme_config,
        crate::theme::commands::set_theme_config,
        crate::theme::commands::get_wallpaper,
    ])
}
