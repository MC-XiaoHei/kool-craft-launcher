use crate::scheduler::Context;
use crate::scheduler::Task;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use futures::future::select_ok;
use std::sync::Arc;
use uuid::Uuid;

pub struct Race<T: Task> {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) tasks: Vec<Arc<dyn Task<Input = T::Input, Output = T::Output>>>,
}

#[async_trait]
impl<T: Task> Task for Race<T>
where
    T::Output: Clone,
{
    type Input = T::Input;
    type Output = T::Output;

    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> String {
        self.name.clone()
    }
    fn weight(&self) -> u64 {
        self.tasks.iter().map(|t| t.weight()).fold(0, u64::max)
    }
    fn is_hidden_in_view(&self) -> bool {
        self.tasks.iter().all(|t| t.is_hidden_in_view())
    }

    async fn run(&self, input: Self::Input, ctx: Context) -> Result<Self::Output> {
        let monitor = self.monitor(&ctx);
        monitor.running(0.0);

        let race_ctx = ctx.race_ctx.clone().unwrap_or_default();
        let mut futures = Vec::new();

        for task in &self.tasks {
            let task = task.clone();
            let input = input.clone();
            let ctx = Context {
                race_ctx: Some(race_ctx.clone()),
                parent_id: Some(self.id),
                ..ctx.clone()
            };
            futures.push(Box::pin(async move { task.run(input, ctx).await }));
        }

        let res = select_ok(futures).await;

        match res {
            Ok((val, _)) => {
                monitor.finished();
                Ok(val)
            }
            Err(e) => {
                monitor.failed(&e);
                Err(anyhow!("All race tasks failed: {e}"))
            }
        }
    }
}
