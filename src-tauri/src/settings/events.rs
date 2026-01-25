use crate::settings::traits::SettingsGroup;
use crate::utils::global_app_handle::get_global_app_handle;
use anyhow::Result;
use macros::event;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use tauri::{Emitter, EventTarget};

#[event]
pub struct SettingsUpdateEvent {
    pub key: String,
    pub value: String,
}

impl SettingsUpdateEvent {
    pub fn new(key: impl Into<String>, value: Value) -> Result<Self> {
        Ok(Self {
            key: key.into(),
            value: serde_json::to_string(&value)?,
        })
    }
}
