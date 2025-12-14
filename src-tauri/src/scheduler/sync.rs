use log::error;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone)]
pub struct RaceContext {
    winner_flag: Arc<AtomicBool>,
}

impl Default for RaceContext {
    fn default() -> Self {
        Self {
            winner_flag: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl RaceContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn try_win(&self) -> bool {
        self.winner_flag
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    pub fn defer<F, Fut>(&self, func: F) -> RollbackGuard<F, Fut>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        RollbackGuard::new(func)
    }
}

pub struct RollbackGuard<F, Fut>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    rollback_fn: Option<F>,
    _phantom: PhantomData<Fut>,
}

impl<F, Fut> RollbackGuard<F, Fut>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    pub fn new(func: F) -> Self {
        Self {
            rollback_fn: Some(func),
            _phantom: PhantomData,
        }
    }

    pub fn commit(&mut self) {
        self.rollback_fn = None;
    }
}

impl<F, Fut> Drop for RollbackGuard<F, Fut>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    fn drop(&mut self) {
        if let Some(f) = self.rollback_fn.take() {
            tokio::spawn(async move {
                if let Err(e) = f().await {
                    error!("Rollback failed: {:?}", e);
                }
            });
        }
    }
}
