use std::marker::PhantomData;
use async_trait::async_trait;
use anyhow::Result;
use uuid::Uuid;
use crate::scheduler::context::Context;
use crate::scheduler::Task;

pub struct FnTask<F, Fut, In, Out> {
    id: Uuid,
    name: String,
    weight: u64,
    hidden_in_view: bool,
    func: F,
    _p: PhantomData<fn(In) -> (Out, Fut)>,
}

impl<F, Fut, In, Out> FnTask<F, Fut, In, Out> {
    pub fn new(name: &str, func: F) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            weight: 1,
            hidden_in_view: false,
            func,
            _p: PhantomData,
        }
    }

    pub fn with_weight(mut self, weight: u64) -> Self {
        self.weight = weight;
        self
    }

    pub fn hidden_in_view(mut self) -> Self {
        self.hidden_in_view = true;
        self
    }
}

#[async_trait]
impl<F, Fut, In, Out> Task for FnTask<F, Fut, In, Out>
where
    In: Send + Clone + 'static,
    Out: Send + 'static,
    F: Fn(In, Context) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Out>> + Send,
{
    type Input = In;
    type Output = Out;

    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn weight(&self) -> u64 {
        self.weight
    }
    fn is_hidden_in_view(&self) -> bool {
        self.hidden_in_view
    }

    async fn run(&self, input: In, ctx: Context) -> Result<Out> {
        let monitor = self.monitor(&ctx);
        monitor.pending();

        // Hold the acquired permit for the lifetime of this task to enforce the semaphore-based concurrency limit.
        let _permit = ctx.acquire_permit().await;

        monitor.running(0.0);
        let res = (self.func)(input, ctx.clone()).await;

        match &res {
            Ok(_) => monitor.finished(),
            Err(e) => monitor.failed(e),
        }
        res
    }
}