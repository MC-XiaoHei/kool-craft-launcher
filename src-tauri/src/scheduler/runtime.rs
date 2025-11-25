use std::collections::HashMap;
use super::core::{Context, ProgressReporter, TaskNode, TaskRegistry, TaskSnapshot};
use super::traits::Task;
use anyhow::Result;
use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::{mpsc, Semaphore};
use tokio::task::JoinHandle;
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
        let (tx, mut rx) = mpsc::unbounded_channel();

        let total_weight = task.weight();
        let reporter = ProgressReporter::new(tx, total_weight);
        let name = task.name().to_string();

        let factor = if total_weight > 0.0 {
            100.0 / total_weight
        } else {
            0.0
        };

        let handle = tokio::spawn(async move {
            while let Some((curr, _total)) = rx.recv().await {
                let pct = curr * factor;
                println!(">> [Progress] {:<15} {:.1}%", name, pct);
            }
        });

        // The guard ensures the background progress reporting task is aborted when the function exits.
        let _guard = AbortGuard::new(handle);

        let ctx = Context {
            reporter,
            race_ctx: None,
            semaphore: self.semaphore.clone(),
            registry: self.registry.clone(),
            parent_id: None,
        };

        task.run((), ctx).await
    }

    pub fn tree(&self) -> Vec<TaskNode> {
        let snapshots: Vec<TaskSnapshot> = self.registry.iter().map(|r| r.value().clone()).collect();

        let mut children_map: HashMap<Option<Uuid>, Vec<TaskSnapshot>> = HashMap::new();
        for snap in snapshots {
            children_map.entry(snap.parent_id).or_default().push(snap);
        }

        fn build_nodes(
            parent_id: Option<Uuid>,
            map: &HashMap<Option<Uuid>, Vec<TaskSnapshot>>
        ) -> Vec<TaskNode> {
            if let Some(children) = map.get(&parent_id) {
                let mut nodes = Vec::new();
                for child in children {
                    nodes.push(TaskNode {
                        id: child.id,
                        name: child.name.clone(),
                        state: child.state,
                        progress: child.progress,
                        message: child.message.clone(),
                        children: build_nodes(Some(child.id), map),
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

struct AbortGuard(JoinHandle<()>);

impl AbortGuard {
    fn new(h: JoinHandle<()>) -> Self {
        Self(h)
    }
}

impl Drop for AbortGuard {
    fn drop(&mut self) {
        self.0.abort();
    }
}
