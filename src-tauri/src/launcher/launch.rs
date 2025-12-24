use crate::launcher::model::LaunchRequest;
use crate::scheduler::{Task, task};
use anyhow::Result;

pub fn get_launch_task() -> impl Task {
    task("launch_minecraft", launch_minecraft)
}

async fn launch_minecraft(request: LaunchRequest) -> Result<()> {
    let command = get_launch_command(request).await?;
    // TODO
    Ok(())
}

async fn get_launch_command(request: LaunchRequest) -> Result<Vec<String>> {
    // TODO
    Ok(vec![])
}
