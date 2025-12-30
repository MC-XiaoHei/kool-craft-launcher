use crate::game_launcher::model::LaunchRequest;
use crate::scheduler::{Task, task};
use crate::utils::executor::Executable;
use anyhow::Result;

pub fn get_launch_task() -> impl Task {
    task("launch_minecraft", launch_minecraft)
}

async fn launch_minecraft(request: LaunchRequest) -> Result<()> {
    let executable = get_launch_executable(request).await?;
    let process = executable.spawn()?;
    // TODO
    Ok(())
}

async fn get_launch_executable(request: LaunchRequest) -> Result<Executable> {
    let java = request.java_profile.get_java_executable_path_str()?;

    let rule_context = request.get_rule_context();
    let arguments_context = request.get_arguments_context()?;
    let mut args = vec![];

    args.append(&mut request.manifest.arguments.get_jvm_arguments(
        rule_context.clone(),
        arguments_context.clone(),
        request.custom_info.custom_jvm_args.clone(),
    ));

    args.push(request.manifest.main_class);

    args.append(&mut request.manifest.arguments.get_game_arguments(
        rule_context,
        arguments_context,
        request.custom_info.custom_game_args.clone(),
    ));

    Ok(Executable {
        program: java,
        args,
        cwd: None,
        kill_on_drop: false,
    })
}
