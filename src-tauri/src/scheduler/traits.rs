use async_trait::async_trait;
use anyhow::Result;
use uuid::Uuid;
use super::core::Context;

#[async_trait]
pub trait Task: Send + Sync {
    type Input: Send + Clone + 'static;
    type Output: Send + 'static;

    fn id(&self) -> Uuid;
    fn name(&self) -> &str;
    fn weight(&self) -> f64;

    async fn run(&self, input: Self::Input, ctx: Context) -> Result<Self::Output>;
}