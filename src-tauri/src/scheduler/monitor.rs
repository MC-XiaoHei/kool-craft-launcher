use uuid::Uuid;
use crate::scheduler::context::{Context};
use crate::scheduler::types::{TaskSnapshot, TaskState};

pub struct TaskMonitor<'a> {
    ctx: &'a Context,
    id: Uuid,
    name: &'a str,
}

impl<'a> TaskMonitor<'a> {
    pub fn new(ctx: &'a Context, id: Uuid, name: &'a str) -> Self {
        Self { ctx, id, name }
    }

    pub fn pending(&self) {
        self.update(TaskState::Pending, 0.0, None);
    }

    pub fn running(&self, progress: f64) {
        self.update(TaskState::Running, progress, None);
    }

    pub fn finished(&self) {
        self.update(TaskState::Finished, 1.0, None);
    }

    pub fn failed(&self, error: &anyhow::Error) {
        self.update(TaskState::Failed, 1.0, Some(error.to_string()));
    }

    fn update(&self, state: TaskState, progress: f64, message: Option<String>) {
        self.ctx.registry.insert(self.id, TaskSnapshot {
            id: self.id,
            parent_id: self.ctx.parent_id,
            name: self.name.to_string(),
            state,
            progress,
            message,
        });
        self.report_progress(progress);
    }

    fn report_progress(&self, ratio: f64) {
        self.ctx.reporter.update(ratio);
    }
}