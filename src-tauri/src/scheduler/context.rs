use crate::scheduler::models::TaskRegistry;
use crate::scheduler::sync::RaceContext;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Clone)]
pub struct Context {
    pub(super) race_ctx: Option<RaceContext>,
    pub(super) semaphore: Arc<Semaphore>,
    pub(super) registry: TaskRegistry,
    pub(super) parent_id: Option<Uuid>,
    pub(super) cancel_token: CancellationToken,
}

impl Context {
    pub async fn acquire_permit(&self) -> Option<OwnedSemaphorePermit> {
        self.semaphore.clone().acquire_owned().await.ok()
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }
}
