use async_trait::async_trait;
use anyhow::Result;
use uuid::Uuid;
use crate::scheduler::context::Context;
use crate::scheduler::Task;

pub struct NamedTask<T> {
    id: Uuid,
    name: String,
    pub(super) inner: T,
}

impl<T> NamedTask<T> {
    pub fn new(name: String, inner: T) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            inner,
        }
    }
}

#[async_trait]
impl<T: Task> Task for NamedTask<T> {
    type Input = T::Input;
    type Output = T::Output;

    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn weight(&self) -> u64 {
        self.inner.weight()
    }
    fn is_hidden_in_view(&self) -> bool {
        self.inner.is_hidden_in_view()
    }

    async fn run(&self, input: Self::Input, ctx: Context) -> Result<Self::Output> {
        let monitor = self.monitor(&ctx);
        monitor.running(0.0);

        let child_ctx = Context {
            parent_id: Some(self.id),
            ..ctx.clone()
        };

        let res = self.inner.run(input, child_ctx).await;

        match &res {
            Ok(_) => monitor.finished(),
            Err(e) => monitor.failed(e),
        }
        res
    }
}