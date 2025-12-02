use super::context::Context;
use super::traits::Task;
use crate::scheduler::model::TaskRegistry;
use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

pub struct Scheduler {
    pub(super) semaphore: Arc<Semaphore>,
    pub(super) registry: TaskRegistry,
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
            cancel_token: CancellationToken::new(),
        };

        task.run((), ctx).await
    }
}