use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use uuid::Uuid;
use crate::scheduler::progress::ProgressReporter;
use crate::scheduler::sync::RaceContext;
use crate::scheduler::types::TaskRegistry;

#[derive(Clone)]
pub struct Context {
    pub(crate) reporter: ProgressReporter,
    pub(crate) race_ctx: Option<RaceContext>,
    pub(crate) semaphore: Arc<Semaphore>,
    pub(crate) registry: TaskRegistry,
    pub(crate) parent_id: Option<Uuid>,
}

impl Context {
    pub async fn acquire_permit(&self) -> Option<OwnedSemaphorePermit> {
        self.semaphore.clone().acquire_owned().await.ok()
    }
}