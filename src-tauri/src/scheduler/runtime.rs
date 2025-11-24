use super::core::{Context, ProgressReporter};
use super::traits::Task;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use tokio::task::JoinHandle;

pub struct Scheduler {
    semaphore: Arc<Semaphore>,
}

impl Scheduler {
    pub fn new(concurrency_limit: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(concurrency_limit)),
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
            task_name: "RootTask".to_string(),
        };

        task.run((), ctx).await
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
