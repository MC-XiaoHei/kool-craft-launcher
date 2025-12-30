use crate::java_runtime::model::{JavaArch, JavaInstance};
use crate::utils::executor::Executable;
use anyhow::{Context, Result};
use regex::Regex;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::time::timeout;

const JAVA_VERSION_DETECT_TIMEOUT: Duration = Duration::from_secs(3);

pub async fn inspect(path: PathBuf) -> Result<JavaInstance> {
    let exec = Executable {
        program: path.to_string_lossy().to_string(),
        args: vec!["-version".to_string()],
        cwd: None,
        kill_on_drop: true,
    };

    let output = timeout(JAVA_VERSION_DETECT_TIMEOUT, exec.run_and_get_output())
        .await
        .context("Java version check timed out")??;

    parse_output(path, output)
}

fn parse_output(path: PathBuf, output: String) -> Result<JavaInstance> {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"version "([^"]+)""#)
            .expect("Internal error: Failed to compile java_version_detect regex") // this should never happen...
    });

    let version_str = RE
        .captures(&output)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .context("No version string found in output")?;

    let major_version = if version_str.starts_with("1.") {
        version_str
            .split('.')
            .nth(1)
            .unwrap_or("0")
            .parse()
            .unwrap_or(0)
    } else {
        version_str
            .split('.')
            .next()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0)
    };

    let out_lower = output.to_lowercase();
    let arch = if out_lower.contains("aarch64") || out_lower.contains("arm64") {
        JavaArch::Arm64
    } else if out_lower.contains("64-bit")
        || out_lower.contains("x86_64")
        || out_lower.contains("amd64")
    {
        JavaArch::X64
    } else {
        JavaArch::X86
    };

    let vendor_name = output
        .lines()
        .nth(1)
        .unwrap_or_else(|| output.lines().next().unwrap_or("Unknown"))
        .trim()
        .to_string();

    Ok(JavaInstance {
        path: path.to_path_buf(),
        version: version_str,
        major_version,
        arch,
        vendor_name,
    })
}
