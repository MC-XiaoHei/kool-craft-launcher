use super::context::Context;
use super::traits::Task;
use crate::scheduler::types::{TaskNode, TaskRegistry, TaskSnapshot};
use crate::scheduler::TaskState;
use anyhow::Result;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;

pub struct Scheduler {
    semaphore: Arc<Semaphore>,
    registry: TaskRegistry,
}

impl Scheduler {
    pub fn new(concurrency_limit: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(concurrency_limit)),
            registry: Arc::new(DashMap::new()),
        }
    }

    pub async fn run<T>(&self, task: T) -> Result<T::Output>
    where
        T: Task<Input = ()>,
    {
        let ctx = Context {
            race_ctx: None,
            semaphore: self.semaphore.clone(),
            registry: self.registry.clone(),
            parent_id: None,
        };

        task.run((), ctx).await
    }

    pub fn tree(&self) -> Vec<TaskNode> {
        let snapshots: Vec<TaskSnapshot> =
            self.registry.iter().map(|r| r.value().clone()).collect();

        let mut children_map: HashMap<Option<Uuid>, Vec<TaskSnapshot>> = HashMap::new();
        for snap in snapshots {
            children_map.entry(snap.parent_id).or_default().push(snap);
        }

        fn build_nodes(
            parent_id: Option<Uuid>,
            map: &HashMap<Option<Uuid>, Vec<TaskSnapshot>>,
        ) -> Vec<TaskNode> {
            if let Some(children) = map.get(&parent_id) {
                let mut nodes = Vec::new();

                for child_snap in children {
                    let child_nodes = build_nodes(Some(child_snap.id), map);

                    let child_progress = if child_nodes.is_empty() {
                        child_snap.progress
                    } else {
                        let total_weight_f64: f64 =
                            child_nodes.iter().map(|n| n.weight as f64).sum();

                        if total_weight_f64 == 0.0 {
                            if child_snap.state == TaskState::Finished
                                || child_snap.state == TaskState::Failed
                            {
                                1.0
                            } else {
                                0.0
                            }
                        } else {
                            let weighted_progress: f64 = child_nodes
                                .iter()
                                .map(|n| n.progress * n.weight as f64)
                                .sum();
                            weighted_progress / total_weight_f64
                        }
                    };

                    nodes.push(TaskNode {
                        id: child_snap.id,
                        name: child_snap.name.clone(),
                        state: child_snap.state,
                        progress: child_progress,
                        message: child_snap.message.clone(),
                        children: child_nodes,
                        weight: child_snap.weight,
                    });
                }

                nodes
            } else {
                Vec::new()
            }
        }

        build_nodes(None, &children_map)
    }
}