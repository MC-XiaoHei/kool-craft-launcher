use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use log::warn;

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
            self.tx
                .send(value)
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