use crate::config::store::ConfigStore;
use crate::scheduler::Scheduler;
use tauri::{App, Manager};

pub fn setup_scheduler(app: &mut App) {
    let store = app.state::<ConfigStore>();
    let scheduler = Scheduler::new(32); // TODO: make concurrency limit configurable
    app.manage(scheduler);
}
