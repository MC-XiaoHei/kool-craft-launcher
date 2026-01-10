use crate::config::traits::ConfigGroup;
use crate::utils::global_app_handle::get_global_app_handle;
use anyhow::Result;
use macros::event;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use tauri::{Emitter, EventTarget};

#[event]
pub struct ConfigUpdateEvent {
    pub key: String,
    pub value: String,
}

impl ConfigUpdateEvent {
    pub fn new(key: impl Into<String>, value: Value) -> Result<Self> {
        Ok(Self {
            key: key.into(),
            value: serde_json::to_string(&value)?,
        })
    }
}
