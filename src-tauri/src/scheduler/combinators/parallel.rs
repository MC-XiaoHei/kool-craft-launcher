use std::sync::Arc;
use async_trait::async_trait;
use anyhow::{anyhow, Result};
use tokio::task::JoinSet;
use uuid::Uuid;
use crate::scheduler::context::Context;
use crate::scheduler::Task;

pub struct Parallel<T: Task> {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) tasks: Vec<Arc<dyn Task<Input = T::Input, Output = T::Output>>>,
}

impl<T: Task> Parallel<T> {
    fn spawn_subtasks(
        &self,
        input: T::Input,
        ctx: &Context,
    ) -> JoinSet<(usize, Result<T::Output>)> {
        let mut set = JoinSet::new();

        for (i, task) in self.tasks.iter().enumerate() {
            let task = task.clone();
            let input = input.clone();

            let sub_ctx = Context {
                parent_id: Some(self.id),
                ..ctx.clone()
            };

            set.spawn(async move {
                let res = task.run(input, sub_ctx).await;
                (i, res)
            });
        }
        set
    }

    async fn collect_results(
        &self,
        set: &mut JoinSet<(usize, Result<T::Output>)>,
    ) -> Result<Vec<T::Output>> {
        let mut indexed_results = Vec::with_capacity(self.tasks.len());
        let mut error = None;

        while let Some(res) = set.join_next().await {
            match res {
                Ok((i, Ok(val))) => indexed_results.push((i, val)),
                Ok((i, Err(e))) => {
                    set.shutdown().await;
                    error = Some(anyhow!("Parallel task {} failed: {}", i, e));
                    break;
                }
                Err(e) => {
                    set.shutdown().await;
                    error = Some(anyhow!("Task panic: {}", e));
                    break;
                }
            }
        }

        if let Some(e) = error {
            return Err(e);
        }

        indexed_results.sort_by_key(|(i, _)| *i);
        Ok(indexed_results.into_iter().map(|(_, val)| val).collect())
    }
}

#[async_trait]
impl<T: Task> Task for Parallel<T> {
    type Input = T::Input;
    type Output = Vec<T::Output>;

    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn weight(&self) -> u64 {
        self.tasks.iter().map(|t| t.weight()).sum()
    }
    fn is_hidden_in_view(&self) -> bool {
        self.tasks.iter().all(|t| t.is_hidden_in_view())
    }

    async fn run(&self, input: Self::Input, ctx: Context) -> Result<Self::Output> {
        let monitor = self.monitor(&ctx);
        monitor.running(0.0);

        let mut subtasks_set = self.spawn_subtasks(input, &ctx);

        let result = self.collect_results(&mut subtasks_set).await;

        match &result {
            Ok(_) => monitor.finished(),
            Err(e) => monitor.failed(e),
        }
        result
    }
}
