use crate::launcher::model::LaunchRequest;
use crate::scheduler::{Task, task};
use anyhow::Result;

pub fn get_launch_task() -> impl Task {
    task("launch_minecraft", launch_minecraft)
}

async fn launch_minecraft(request: LaunchRequest) -> Result<()> {
    // TODO
    Ok(())
}
