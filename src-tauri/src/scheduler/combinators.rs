use super::core::Context;
use super::traits::Task;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::future::select_ok;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::task::JoinSet;

pub struct Leaf<F, Fut, In, Out> {
    name: String,
    weight: f64,
    func: F,
    _p: PhantomData<fn(In) -> (Out, Fut)>,
}

impl<F, Fut, In, Out> Leaf<F, Fut, In, Out> {
    pub fn new(name: &str, weight: f64, func: F) -> Self {
        Self {
            name: name.into(),
            weight,
            func,
            _p: PhantomData,
        }
    }
}

#[async_trait]
impl<F, Fut, In, Out> Task for Leaf<F, Fut, In, Out>
where
    In: Send + Clone + 'static,
    Out: Send + 'static,
    F: Fn(In, Context) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Out>> + Send,
{
    type Input = In;
    type Output = Out;
    fn name(&self) -> &str {
        &self.name
    }
    fn weight(&self) -> f64 {
        self.weight
    }

    async fn run(&self, input: In, ctx: Context) -> Result<Out> {
        let task_ctx = Context {
            task_name: self.name.clone(),
            ..ctx
        };
        // Hold the acquired permit for the lifetime of this task to enforce the semaphore-based concurrency limit.
        let _permit = task_ctx.acquire_permit().await;
        task_ctx.reporter.update(0.0);
        let res = (self.func)(input, task_ctx.clone()).await;
        if res.is_ok() {
            task_ctx.reporter.update(1.0);
        }
        res
    }
}

pub struct Chain<A, B> {
    pub head: A,
    pub tail: B,
}

#[async_trait]
impl<A, B> Task for Chain<A, B>
where
    A: Task,
    B: Task<Input = A::Output>,
{
    type Input = A::Input;
    type Output = B::Output;
    fn name(&self) -> &str {
        "Chain"
    }
    fn weight(&self) -> f64 {
        self.head.weight() + self.tail.weight()
    }

    async fn run(&self, input: Self::Input, ctx: Context) -> Result<Self::Output> {
        let head_w = self.head.weight();
        let tail_w = self.tail.weight();

        let head_rep = ctx.reporter.sub_scope(0.0, head_w);
        let head_ctx = Context {
            reporter: head_rep,
            ..ctx.clone()
        };
        let mid = self.head.run(input, head_ctx).await?;

        let tail_rep = ctx.reporter.sub_scope(head_w, tail_w);
        let tail_ctx = Context {
            reporter: tail_rep,
            ..ctx.clone()
        };
        let out = self.tail.run(mid, tail_ctx).await?;

        Ok(out)
    }
}

pub struct NamedTask<T> {
    pub name: String,
    pub inner: T,
}

#[async_trait]
impl<T: Task> Task for NamedTask<T> {
    type Input = T::Input;
    type Output = T::Output;
    fn name(&self) -> &str {
        &self.name
    }
    fn weight(&self) -> f64 {
        self.inner.weight()
    }
    async fn run(&self, input: Self::Input, ctx: Context) -> Result<Self::Output> {
        self.inner.run(input, ctx).await
    }
}

pub struct GroupBuilder<T: Task, Target> {
    name: String,
    tasks: Vec<Arc<dyn Task<Input = T::Input, Output = T::Output>>>,
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
        U: Task<Input = T::Input, Output = T::Output> + 'static,
    {
        self.tasks.push(Arc::new(task));
        self
    }

    pub fn extend<I, U>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = U>,
        U: Task<Input = T::Input, Output = T::Output> + 'static,
    {
        for task in iter {
            self.tasks.push(Arc::new(task));
        }
        self
    }
}

pub struct Race<T: Task> {
    name: String,
    tasks: Vec<Arc<dyn Task<Input = T::Input, Output = T::Output>>>,
}

impl<T: Task> Race<T> {
    pub fn builder(name: &str) -> GroupBuilder<T, Self> {
        GroupBuilder::<T, Self>::new(name)
    }
}

impl<T: Task> GroupBuilder<T, Race<T>> {
    pub fn build(self) -> Race<T> {
        Race {
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
    fn name(&self) -> &str {
        &self.name
    }
    fn weight(&self) -> f64 {
        self.tasks.iter().map(|t| t.weight()).fold(0.0, f64::max)
    }

    async fn run(&self, input: Self::Input, ctx: Context) -> Result<Self::Output> {
        let race_ctx = ctx.race_ctx.clone().unwrap_or_default();
        let mut futures = Vec::new();

        for task in &self.tasks {
            let task = task.clone();
            let input = input.clone();
            let ctx = Context {
                race_ctx: Some(race_ctx.clone()),
                ..ctx.clone()
            };
            futures.push(Box::pin(async move { task.run(input, ctx).await }));
        }

        match select_ok(futures).await {
            Ok((val, _)) => Ok(val),
            Err(e) => Err(anyhow!("All race tasks failed: {}", e)),
        }
    }
}

pub struct Parallel<T: Task> {
    name: String,
    tasks: Vec<Arc<dyn Task<Input = T::Input, Output = T::Output>>>,
}

impl<T: Task> Parallel<T> {
    pub fn builder(name: &str) -> GroupBuilder<T, Self> {
        GroupBuilder::<T, Self>::new(name)
    }
}

impl<T: Task> GroupBuilder<T, Parallel<T>> {
    pub fn build(self) -> Parallel<T> {
        Parallel {
            name: self.name,
            tasks: self.tasks,
        }
    }
}

#[async_trait]
impl<T: Task> Task for Parallel<T> {
    type Input = T::Input;
    type Output = Vec<T::Output>;
    fn name(&self) -> &str {
        &self.name
    }
    fn weight(&self) -> f64 {
        self.tasks.iter().map(|t| t.weight()).sum()
    }

    async fn run(&self, input: Self::Input, ctx: Context) -> Result<Self::Output> {
        let mut set = JoinSet::new();
        let mut offset = 0.0;

        for (i, task) in self.tasks.iter().enumerate() {
            let task = task.clone();
            let input = input.clone();
            let w = task.weight();

            let sub_rep = ctx.reporter.sub_scope(offset, w);
            let sub_ctx = Context {
                reporter: sub_rep,
                ..ctx.clone()
            };

            set.spawn(async move {
                let res = task.run(input, sub_ctx).await;
                (i, res)
            });
            offset += w;
        }

        let mut indexed_results = Vec::with_capacity(self.tasks.len());
        while let Some(res) = set.join_next().await {
            match res {
                Ok((i, Ok(val))) => indexed_results.push((i, val)),
                Ok((i, Err(e))) => {
                    set.shutdown().await;
                    return Err(anyhow!("Parallel task {} failed: {}", i, e));
                }
                Err(e) => {
                    set.shutdown().await;
                    return Err(anyhow!("Task panic: {}", e));
                }
            }
        }

        indexed_results.sort_by_key(|(i, _)| *i);
        Ok(indexed_results.into_iter().map(|(_, val)| val).collect())
    }
}
