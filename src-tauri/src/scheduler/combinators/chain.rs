use async_trait::async_trait;
use anyhow::Result;
use uuid::Uuid;
use crate::scheduler::context::Context;
use crate::scheduler::Task;

pub struct Chain<A, B> {
    id: Uuid,
    pub(super) head: A,
    pub(super) tail: B,
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
    B: Task<Input = A::Output>,
{
    type Input = A::Input;
    type Output = B::Output;

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn id(&self) -> Uuid {
        self.id
    }
    #[cfg_attr(coverage_nightly, coverage(off))]
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