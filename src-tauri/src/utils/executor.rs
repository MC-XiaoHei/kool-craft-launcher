use anyhow::{Context, Result};
use std::io::{PipeReader, PipeWriter};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Executable {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub kill_on_drop: bool,
}

pub struct RunningProcess {
    pub child: Child,
    pub log_receiver: mpsc::UnboundedReceiver<String>,
}

impl Executable {
    pub fn spawn(&self) -> Result<RunningProcess> {
        let (pipe_reader, stdout_write, stderr_write) = self.create_merged_pipes()?;

        let child = self.start_process(stdout_write, stderr_write)?;

        let rx = self.spawn_log_pump(pipe_reader);

        Ok(RunningProcess {
            child,
            log_receiver: rx,
        })
    }

    pub async fn run_and_get_output(&self) -> Result<String> {
        let mut process = self
            .spawn()
            .context("Failed to spawn process for output capture")?;

        let mut output_buffer = String::with_capacity(1024);

        while let Some(line) = process.log_receiver.recv().await {
            output_buffer.push_str(&line);
            output_buffer.push('\n');
        }

        process
            .child
            .wait()
            .await
            .context("Failed to wait for process exit")?;

        Ok(output_buffer)
    }

    fn create_merged_pipes(&self) -> Result<(std::fs::File, PipeWriter, PipeWriter)> {
        let (pipe_reader, pipe_writer) = std::io::pipe().context("Failed to create OS pipe")?;

        let pipe_writer_err = pipe_writer
            .try_clone()
            .context("Failed to clone pipe writer for stderr")?;

        let reader_file = into_file(pipe_reader);

        Ok((reader_file, pipe_writer, pipe_writer_err))
    }

    fn start_process(&self, stdout: PipeWriter, stderr: PipeWriter) -> Result<Child> {
        let mut cmd = Command::new(&self.program);

        cmd.args(&self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .kill_on_drop(self.kill_on_drop);

        if let Some(dir) = &self.cwd {
            cmd.current_dir(dir);
        }

        cmd.spawn()
            .context(format!("Failed to spawn executable: {}", self.program))
    }

    fn spawn_log_pump(&self, reader: std::fs::File) -> mpsc::UnboundedReceiver<String> {
        let async_file = tokio::fs::File::from_std(reader);
        let mut line_reader = BufReader::new(async_file).lines();

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            while let Ok(Some(line)) = line_reader.next_line().await {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });

        rx
    }
}

fn into_file(reader: PipeReader) -> std::fs::File {
    #[cfg(unix)]
    use std::os::unix::io::OwnedFd as FileHandle;
    #[cfg(windows)]
    use std::os::windows::io::OwnedHandle as FileHandle;

    let handle: FileHandle = reader.into();
    std::fs::File::from(handle)
}
