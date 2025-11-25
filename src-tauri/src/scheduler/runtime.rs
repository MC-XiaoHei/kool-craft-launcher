use super::context::Context;
use super::traits::Task;
use crate::scheduler::model::{TaskNode, TaskRegistry, TaskSnapshot};
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
        T: Task<Input=()>,
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
                    let child_progress = child_snap.calculate_effective_progress(&child_nodes);

                    nodes.push(TaskNode {
                        id: child_snap.id,
                        name: child_snap.name.clone(),
                        state: child_snap.state,
                        progress: child_progress,
                        message: child_snap.message.clone(),
                        weight: child_snap.weight,
                        children: child_nodes,
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