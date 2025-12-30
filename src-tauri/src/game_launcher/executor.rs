use anyhow::Result;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Executable {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
}

pub struct RunningProcess {
    pub child: Child,
    pub log_receiver: mpsc::UnboundedReceiver<String>,
}

impl Executable {
    pub fn spawn(&self) -> Result<RunningProcess> {
        let (pipe_reader, pipe_writer) = std::io::pipe()?;

        let pipe_writer_err = pipe_writer.try_clone()?;

        let mut cmd = Command::new(self.program.clone());
        cmd.args(self.args.clone())
            .stdout(Stdio::from(pipe_writer))
            .stderr(Stdio::from(pipe_writer_err))
            .stdin(Stdio::null())
            .kill_on_drop(false);

        if let Some(dir) = self.cwd.as_ref() {
            cmd.current_dir(dir);
        }

        let child = cmd.spawn()?;

        let std_file = into_file(pipe_reader);

        let async_file = tokio::fs::File::from_std(std_file);
        let mut line_reader = BufReader::new(async_file).lines();

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            while let Ok(Some(line)) = line_reader.next_line().await {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });

        Ok(RunningProcess {
            child,
            log_receiver: rx,
        })
    }
}

fn into_file(reader: std::io::PipeReader) -> std::fs::File {
    #[cfg(unix)]
    use std::os::unix::io::OwnedFd as FileHandle;
    #[cfg(windows)]
    use std::os::windows::io::OwnedHandle as FileHandle;

    let handle: FileHandle = reader.into();
    std::fs::File::from(handle)
}
