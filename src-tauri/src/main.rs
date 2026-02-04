// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(coverage_nightly, coverage(off))]

use std::env;

fn main() {
    #[cfg(target_os = "windows")]
    #[cfg(debug_assertions)]
    try_configure_dev_webview_runtime();
    #[cfg(target_os = "linux")]
    try_resolve_nv_wayland_issue();
    kool_craft_launcher_lib::run().unwrap_or_else(log_error);
}

fn log_error(err: anyhow::Error) {
    eprintln!("Fail to run launcher: {err:?}");
}

#[cfg(target_os = "windows")]
#[cfg(debug_assertions)]
fn try_configure_dev_webview_runtime() {
    let _ = env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|p| p.join("runtime")))
        .filter(|path| path.is_dir())
        .map(|path| {
            println!("Using founded local webview runtime: {}", path.display());
            // SAFETY:
            // 1. Concurrency: This runs at the start of main before any threads are spawned, avoiding race conditions.
            // 2. Build Scope: This function is guarded by #[cfg(debug_assertions)], so this unsafe code is completely stripped from release builds and poses no risk to production.
            unsafe {
                env::set_var("WEBVIEW2_BROWSER_EXECUTABLE_FOLDER", path);
            }
        });
}

// Related issue: https://github.com/tauri-apps/tauri/issues/10702
#[cfg(target_os = "linux")]
fn try_resolve_nv_wayland_issue() {
    let is_wayland = env::var("XDG_SESSION_TYPE")
        .map(|v| v.to_lowercase().contains("wayland"))
        .unwrap_or(false);

    let has_nvidia = std::path::Path::new("/proc/driver/nvidia").exists();

    if is_wayland && has_nvidia {
        println!("Nvidia & Wayland detected, disabling Explicit Sync & Threaded Optimizations");
        // SAFETY: This runs at the start of main before any threads are spawned, avoiding race conditions.
        unsafe {
            env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1");
            env::set_var("__GL_THREADED_OPTIMIZATIONS", "0");
        }
    }
}
