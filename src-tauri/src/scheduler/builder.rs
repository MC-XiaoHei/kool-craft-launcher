use super::combinators::{Chain, FnTask, NamedTask, Parallel, Race};
use super::context::Context;
use super::traits::Task;
use anyhow::Result;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;
use uuid::Uuid;

pub fn task_with_ctx<F, Fut, In, Out>(name: impl Into<String>, func: F) -> FnTask<F, Fut, In, Out>
where
    F: Fn(In, Context) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Out>> + Send,
{
    FnTask::new(name.into(), func)
}

pub fn task<F, Fut, In, Out>(
    name: impl Into<String>,
    func: F,
) -> FnTask<impl Fn(In, Context) -> Fut + Send + Sync, Fut, In, Out>
where
    F: Fn(In) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Out>> + Send + 'static,
    In: Send + 'static,
{
    FnTask::new(name.into(), move |input: In, _: Context| func(input))
}

pub fn pipeline(name: impl Into<String>) -> PipelineStarter {
    PipelineStarter::new(name)
}

pub fn race(name: impl Into<String>) -> RaceStarter {
    RaceStarter::new(name)
}

pub fn parallel(name: impl Into<String>) -> ParallelStarter {
    ParallelStarter::new(name)
}

pub struct PipelineStarter {
    name: String,
}

impl PipelineStarter {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn first<T: Task>(self, task: T) -> PipelineBuilder<T> {
        PipelineBuilder {
            name: self.name,
            current: task,
        }
    }
}

pub struct ParallelStarter {
    name: String,
}

impl ParallelStarter {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn add<T: Task + 'static>(self, task: T) -> GroupBuilder<T, Parallel<T>> {
        GroupBuilder::new(self.name).add(task)
    }
}

pub struct RaceStarter {
    name: String,
}

impl RaceStarter {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn add<T: Task + 'static>(self, task: T) -> GroupBuilder<T, Race<T>> {
        GroupBuilder::new(self.name).add(task)
    }
}

pub struct PipelineBuilder<T> {
    name: String,
    current: T,
}

impl<T: Task> PipelineBuilder<T> {
    pub fn then<Next>(self, next: Next) -> PipelineBuilder<Chain<T, Next>>
    where
        Next: Task<Input = T::Output>,
    {
        PipelineBuilder {
            name: self.name,
            current: Chain::new(self.current, next),
        }
    }

    pub fn build(self) -> NamedTask<T> {
        NamedTask::new(self.name, self.current)
    }
}

pub struct TaskBuilder<T>(T);
impl<T: Task> TaskBuilder<T> {
    pub fn new(task: T) -> Self {
        Self(task)
    }
    pub fn then<Next>(self, next: Next) -> TaskBuilder<Chain<T, Next>>
    where
        Next: Task<Input = T::Output>,
    {
        TaskBuilder(Chain::new(self.0, next))
    }
    pub fn build(self) -> T {
        self.0
    }
}

pub struct GroupBuilder<T: Task, Target> {
    name: String,
    tasks: Vec<Arc<dyn Task<Input = T::Input, Output = T::Output>>>,
    _phantom: PhantomData<Target>,
}

impl<T: Task, Target> GroupBuilder<T, Target> {
    pub fn new(name: impl Into<String>) -> Self {
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

impl<T: Task> GroupBuilder<T, Race<T>> {
    pub fn build(self) -> Race<T> {
        Race {
            id: Uuid::new_v4(),
            name: self.name,
            tasks: self.tasks,
        }
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
