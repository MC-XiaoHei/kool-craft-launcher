#![cfg_attr(coverage_nightly, coverage(off))]

use crate::config::store::ConfigStore;
use crate::scheduler::Scheduler;
use std::sync::Arc;
use tauri::{App, Manager};

pub fn setup_scheduler(app: &mut App) {
    let store = app.state::<Arc<ConfigStore>>();
    let scheduler = Scheduler::new(32); // TODO: make concurrency limit configurable
    app.manage(scheduler);
}
