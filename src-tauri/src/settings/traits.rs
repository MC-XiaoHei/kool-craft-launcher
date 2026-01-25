use anyhow::Result;
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;

pub trait SettingsGroup:
    Serialize + DeserializeOwned + JsonSchema + Default + Debug + Clone + Send + Sync + 'static
{
    const KEY: &'static str;

    fn evolve(_current: &mut Value, _all_settings: &HashMap<String, Value>) -> Result<()> {
        Ok(())
    }

    fn post_process(&mut self) -> Result<()> {
        Ok(())
    }

    fn on_update(&self, old: Self) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
pub trait SettingsPersistence: Send + Sync {
    async fn load(&self) -> Result<Option<String>>;
    async fn save(&self, content: String) -> Result<()>;
    fn source_description(&self) -> String;
}
