use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum TaskState {
    Pending,
    Running,
    Finished,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSnapshot {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub state: TaskState,
    pub progress: f64,
    pub weight: u64,
    pub hidden_in_view: bool,
    pub message: Option<String>,
}

impl TaskSnapshot {
    pub fn calculate_effective_progress(&self, children: &[TaskNode]) -> f64 {
        if children.is_empty() {
            return self.progress;
        }

        let total_weight: f64 = children.iter().map(|n| n.weight as f64).sum();

        if total_weight == 0.0 {
            if self.state == TaskState::Finished || self.state == TaskState::Failed {
                1.0
            } else {
                0.0
            }
        } else {
            let weighted_sum: f64 = children.iter().map(|n| n.progress * n.weight as f64).sum();
            weighted_sum / total_weight
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskNode {
    pub id: Uuid,
    pub name: String,
    pub state: TaskState,
    pub progress: f64,
    pub message: Option<String>,
    pub weight: u64,
    pub children: Vec<TaskNode>,
}

pub type TaskRegistry = Arc<DashMap<Uuid, TaskSnapshot>>;
