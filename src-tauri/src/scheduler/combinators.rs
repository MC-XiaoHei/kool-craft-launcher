use super::context::Context;
use super::traits::Task;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::future::select_ok;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::task::JoinSet;
use uuid::Uuid;

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
    Fut: Future<Output=Result<Out>> + Send,
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

pub struct Chain<A, B> {
    id: Uuid,
    pub head: A,
    pub tail: B,
}

impl<A, B> Chain<A, B> {
    pub fn new(head: A, tail: B) -> Self {
        Self {
            id: Uuid::new_v4(),
            head,
            tail,
        }
    }
}

#[async_trait]
impl<A, B> Task for Chain<A, B>
where
    A: Task,
    B: Task<Input=A::Output>,
{
    type Input = A::Input;
    type Output = B::Output;

    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        "Chain"
    }
    fn weight(&self) -> u64 {
        self.head.weight() + self.tail.weight()
    }
    fn is_hidden_in_view(&self) -> bool {
        self.head.is_hidden_in_view() && self.tail.is_hidden_in_view()
    }

    async fn run(&self, input: Self::Input, ctx: Context) -> Result<Self::Output> {
        let head_output = self.head.run(input, ctx.clone()).await?;
        let output = self.tail.run(head_output, ctx.clone()).await?;
        Ok(output)
    }
}

pub struct NamedTask<T> {
    id: Uuid,
    pub name: String,
    pub inner: T,
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

pub struct GroupBuilder<T: Task, Target> {
    name: String,
    tasks: Vec<Arc<dyn Task<Input=T::Input, Output=T::Output>>>,
    _phantom: PhantomData<Target>,
}

impl<T: Task, Target> GroupBuilder<T, Target> {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            tasks: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn add<U>(mut self, task: U) -> Self
    where
        U: Task<Input=T::Input, Output=T::Output> + 'static,
    {
        self.tasks.push(Arc::new(task));
        self
    }

    pub fn extend<I, U>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item=U>,
        U: Task<Input=T::Input, Output=T::Output> + 'static,
    {
        for task in iter {
            self.tasks.push(Arc::new(task));
        }
        self
    }
}

pub struct Race<T: Task> {
    id: Uuid,
    name: String,
    tasks: Vec<Arc<dyn Task<Input=T::Input, Output=T::Output>>>,
}

impl<T: Task> Race<T> {
    pub fn builder(name: &str) -> GroupBuilder<T, Self> {
        GroupBuilder::<T, Self>::new(name)
    }
}

impl<T: Task> GroupBuilder<T, Race<T>> {
    pub fn build(self) -> Race<T> {
        Race {
            id: Uuid::new_v4(),
            name: self.name,
            tasks: self.tasks,
        }
    }
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
    fn name(&self) -> &str {
        &self.name
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
                Err(anyhow!("All race tasks failed: {}", e))
            }
        }
    }
}

pub struct Parallel<T: Task> {
    id: Uuid,
    name: String,
    tasks: Vec<Arc<dyn Task<Input=T::Input, Output=T::Output>>>,
}

impl<T: Task> Parallel<T> {
    pub fn builder(name: &str) -> GroupBuilder<T, Self> {
        GroupBuilder::<T, Self>::new(name)
    }

    fn spawn_subtasks(&self, input: T::Input, ctx: &Context) -> JoinSet<(usize, Result<T::Output>)> {
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

    async fn collect_results(&self, set: &mut JoinSet<(usize, Result<T::Output>)>) -> Result<Vec<T::Output>> {
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

impl<T: Task> GroupBuilder<T, Parallel<T>> {
    pub fn build(self) -> Parallel<T> {
        Parallel {
            id: Uuid::new_v4(),
            name: self.name,
            tasks: self.tasks,
        }
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