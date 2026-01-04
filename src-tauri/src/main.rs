// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

fn main() {
    kool_craft_launcher_lib::run().unwrap_or_else(log_error);
}

fn log_error(err: anyhow::Error) {
    eprintln!("Fail to run launcher: {err:?}");
}
