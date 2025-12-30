use super::context::Context;
use super::traits::Task;
use crate::scheduler::model::TaskRegistry;
use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{Semaphore, watch};
use tokio_util::sync::CancellationToken;

pub struct Scheduler {
    limit_tx: watch::Sender<usize>,
    pub(super) semaphore: Arc<Semaphore>,
    pub(super) registry: TaskRegistry,
}

impl Scheduler {
    pub fn new(concurrency_limit: usize) -> Self {
        let semaphore = Arc::new(Semaphore::new(concurrency_limit));
        let (tx, rx) = watch::channel(concurrency_limit);
        let scheduler = Self {
            semaphore: semaphore.clone(),
            registry: Arc::new(DashMap::new()),
            limit_tx: tx,
        };

        scheduler.spawn_limit_maintainer(semaphore, rx);

        scheduler
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
            cancel_token: CancellationToken::new(),
        };

        task.run((), ctx).await
    }

    fn spawn_limit_maintainer(&self, sem: Arc<Semaphore>, mut rx: watch::Receiver<usize>) {
        tokio::spawn(async move {
            let mut current_limit = { *rx.borrow() };

            loop {
                let target = { *rx.borrow() };

                if target > current_limit {
                    let diff = target - current_limit;
                    sem.add_permits(diff);
                    current_limit = target;
                } else if target < current_limit {
                    tokio::select! {
                        permit = sem.clone().acquire_owned() => {
                            if let Ok(p) = permit {
                                p.forget();
                                current_limit -= 1;
                            } else {
                                break;
                            }
                        }
                        _ = rx.changed() => {
                            continue;
                        }
                    }
                } else if rx.changed().await.is_err() {
                    break;
                }
            }
        });
    }

    pub fn set_concurrency_limit(&self, new_limit: usize) {
        let _ = self.limit_tx.send(new_limit);
    }

    pub fn get_concurrency_limit(&self) -> usize {
        *self.limit_tx.borrow()
    }
}
