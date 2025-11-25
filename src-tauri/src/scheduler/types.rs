use std::sync::Arc;
use dashmap::DashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum TaskState {
    Pending,
    Running,
    Finished,
    Failed,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskSnapshot {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub state: TaskState,
    pub progress: f64,
    pub message: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskNode {
    pub id: Uuid,
    pub name: String,
    pub state: TaskState,
    pub progress: f64,
    pub message: Option<String>,
    pub children: Vec<TaskNode>,
}

pub type TaskRegistry = Arc<DashMap<Uuid, TaskSnapshot>>;