use async_trait::async_trait;
use anyhow::Result;
use uuid::Uuid;
use crate::scheduler::status::TaskStatusUpdater;
use super::context::{Context};

#[async_trait]
pub trait Task: Send + Sync {
    type Input: Send + Clone + 'static;
    type Output: Send + 'static;

    fn id(&self) -> Uuid;
    fn name(&self) -> &str;
    fn weight(&self) -> u64;
    fn is_hidden_in_view(&self) -> bool;

    async fn run(&self, input: Self::Input, ctx: Context) -> Result<Self::Output>;

    fn monitor<'a>(&'a self, ctx: &'a Context) -> TaskStatusUpdater<'a> {
        TaskStatusUpdater::new(ctx, self.id(), self.weight(), self.name(), self.is_hidden_in_view())
    }
}