use crate::scheduler::context::Context;
use crate::scheduler::model::{TaskSnapshot, TaskState};
use uuid::Uuid;

pub struct TaskStatusUpdater<'a> {
    ctx: &'a Context,
    id: Uuid,
    weight: u64,
    name: &'a str,
    hidden_in_view: bool,
}

impl<'a> TaskStatusUpdater<'a> {
    pub fn new(
        ctx: &'a Context,
        id: Uuid,
        weight: u64,
        name: &'a str,
        hidden_in_view: bool,
    ) -> Self {
        Self {
            ctx,
            id,
            weight,
            name,
            hidden_in_view,
        }
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
        self.ctx.registry.insert(
            self.id,
            TaskSnapshot {
                id: self.id,
                parent_id: self.ctx.parent_id,
                name: self.name.to_string(),
                weight: self.weight,
                hidden_in_view: self.hidden_in_view,
                state,
                progress,
                message,
            },
        );
    }
}
