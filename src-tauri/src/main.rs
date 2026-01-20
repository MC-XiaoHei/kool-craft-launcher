// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(coverage_nightly, coverage(off))]

fn main() {
    #[cfg(target_os = "windows")]
    #[cfg(debug_assertions)]
    configure_dev_webview_runtime();
    kool_craft_launcher_lib::run().unwrap_or_else(log_error);
}

fn log_error(err: anyhow::Error) {
    eprintln!("Fail to run launcher: {err:?}");
}

#[cfg(target_os = "windows")]
#[cfg(debug_assertions)]
fn configure_dev_webview_runtime() {
    let _ = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|p| p.join("runtime")))
        .filter(|path| path.is_dir())
        .map(|path| {
            println!("Using founded local webview runtime: {}", path.display());
            // SAFETY:
            // 1. Concurrency: This runs at the start of main before any threads are spawned, avoiding race conditions.
            // 2. Build Scope: This function is guarded by #[cfg(debug_assertions)], so this unsafe code is completely stripped from release builds and poses no risk to production.
            unsafe {
                std::env::set_var("WEBVIEW2_BROWSER_EXECUTABLE_FOLDER", path);
            }
        });
}
