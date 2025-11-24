use anyhow::{Result};
use log::{error, warn};
use std::future::Future;
use std::marker::PhantomData;
use std::sync::{
    atomic::{AtomicBool, Ordering}, Arc,
    Mutex,
};
use tokio::sync::{mpsc, OwnedSemaphorePermit, Semaphore};

#[derive(Clone)]
pub struct Context {
    pub reporter: ProgressReporter,
    pub race_ctx: Option<RaceContext>,
    pub semaphore: Arc<Semaphore>,
    pub task_name: String,
}

impl Context {
    pub async fn acquire_permit(&self) -> Option<OwnedSemaphorePermit> {
        self.semaphore.clone().acquire_owned().await.ok()
    }

    pub fn name(&self) -> &str {
        &self.task_name
    }
}

#[derive(Clone)]
pub struct RaceContext {
    winner_flag: Arc<AtomicBool>,
}

impl Default for RaceContext {
    fn default() -> Self {
        Self { winner_flag: Arc::new(AtomicBool::new(false)) }
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
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        RollbackGuard::new(func)
    }
}

pub struct RollbackGuard<F, Fut>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    rollback_fn: Option<F>,
    _phantom: PhantomData<Fut>,
}

impl<F, Fut> RollbackGuard<F, Fut>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
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
    Fut: Future<Output = Result<()>> + Send + 'static,
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

#[derive(Clone)]
pub struct ProgressReporter {
    tx: mpsc::UnboundedSender<(f64, f64)>,
    base_offset: f64,
    scope_weight: f64,
    global_total: f64,
    max_reported: Arc<Mutex<f64>>,
}

impl ProgressReporter {
    pub fn new(tx: mpsc::UnboundedSender<(f64, f64)>, total: f64) -> Self {
        Self {
            tx,
            base_offset: 0.0,
            scope_weight: total,
            global_total: total,
            max_reported: Arc::new(Mutex::new(0.0)),
        }
    }

    pub fn update(&self, ratio: f64) {
        let current_abs = self.base_offset + (ratio * self.scope_weight);
        let mut send_value = None;
        {
            if let Ok(mut guard) = self.max_reported.lock()
                && current_abs > *guard
            {
                *guard = current_abs;
                send_value = Some((current_abs, self.global_total));
            }
        }

        if let Some(value) = send_value {
            self.tx.send(value)
                .unwrap_or_else(|e| warn!("Failed to send progress update: {:?}", e));
        }
    }

    pub fn sub_scope(&self, weight_offset: f64, weight: f64) -> Self {
        Self {
            tx: self.tx.clone(),
            base_offset: self.base_offset + weight_offset,
            scope_weight: weight,
            global_total: self.global_total,
            max_reported: self.max_reported.clone(),
        }
    }
}
