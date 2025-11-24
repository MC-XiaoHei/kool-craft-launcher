use std::future::Future;
use anyhow::Result;
use super::traits::Task;
use super::combinators::{Leaf, Chain, NamedTask, Race, Parallel, GroupBuilder};
use super::core::Context;

pub fn task<F, Fut, In, Out>(name: &str, func: F) -> Leaf<F, Fut, In, Out>
where
    F: Fn(In, Context) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Out>> + Send
{
    Leaf::new(name, 1.0, func)
}

pub fn pipeline(name: &str) -> PipelineStarter {
    PipelineStarter { name: name.to_string() }
}

pub fn race<T: Task>(name: &str) -> GroupBuilder<T, Race<T>> {
    Race::<T>::builder(name)
}

pub fn parallel<T: Task>(name: &str) -> GroupBuilder<T, Parallel<T>> {
    Parallel::<T>::builder(name)
}

pub struct PipelineStarter { name: String }

impl PipelineStarter {
    pub fn first<T: Task>(self, task: T) -> PipelineBuilder<T> {
        PipelineBuilder {
            name: self.name,
            current: task,
        }
    }
}

pub struct PipelineBuilder<T> {
    name: String,
    current: T,
}

impl<T: Task> PipelineBuilder<T> {
    pub fn then<Next>(self, next: Next) -> PipelineBuilder<Chain<T, Next>>
    where Next: Task<Input = T::Output> {
        PipelineBuilder {
            name: self.name,
            current: Chain { head: self.current, tail: next },
        }
    }

    pub fn build(self) -> NamedTask<T> {
        NamedTask {
            name: self.name,
            inner: self.current,
        }
    }
}

pub struct TaskBuilder<T>(T);
impl<T: Task> TaskBuilder<T> {
    pub fn new(task: T) -> Self { Self(task) }
    pub fn then<Next>(self, next: Next) -> TaskBuilder<Chain<T, Next>>
    where Next: Task<Input = T::Output> {
        TaskBuilder(Chain { head: self.0, tail: next })
    }
    pub fn build(self) -> T { self.0 }
}