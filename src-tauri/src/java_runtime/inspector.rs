use crate::java_runtime::models::{JavaArch, JavaInstance};
use crate::java_runtime::vendors::VENDOR_KEYWORDS_MAP;
use crate::utils::executor::Executable;
use anyhow::{Context, Result, anyhow};
use log::warn;
use regex::Regex;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::Duration;
use tap::Pipe;
use tokio::time::timeout;

const JAVA_VERSION_DETECT_TIMEOUT: Duration = Duration::from_secs(3);

pub async fn inspect_java_executable(path: PathBuf) -> Option<JavaInstance> {
    let exec = Executable {
        program: path.to_string_lossy().to_string(),
        args: vec!["-version".to_string()],
        cwd: None,
        kill_on_drop: true,
    };

    let result = timeout(JAVA_VERSION_DETECT_TIMEOUT, exec.run_and_get_output()).await;

    let Ok(Ok(output)) = result else { return None };

    let parse_result = parse_output(path.clone(), output);
    match parse_result {
        Ok(instance) => Some(instance),
        Err(err) => {
            warn!("error while detecting java in {path:?}: {err:?}");
            None
        }
    }
}

fn parse_output(path: PathBuf, output: String) -> Result<JavaInstance> {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"version "([^"]+)""#)
            .expect("Internal Error: Failed to compile java_version_detect regex") // this should never happen
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
            .map_err(|_| anyhow!("Invalid major version: {version_str}"))?
    } else {
        version_str
            .split('.')
            .next()
            .unwrap_or("0")
            .parse()
            .map_err(|_| anyhow!("Invalid major version: {version_str}"))?
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
        .to_string()
        .pipe(detect_vendor);

    Ok(JavaInstance {
        path: path.to_path_buf(),
        version: version_str,
        major_version,
        arch,
        vendor_name,
    })
}

fn detect_vendor(raw: impl Into<String>) -> String {
    let raw = raw.into();

    for (k, v) in VENDOR_KEYWORDS_MAP {
        if raw.to_lowercase().contains(k) {
            return v.to_string();
        }
    }

    raw
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const OUTPUT_JAVA_17_HOTSPOT: &str = r#"java version "17.0.1" 2021-10-19 LTS
Java(TM) SE Runtime Environment (build 17.0.1+12-LTS-39)
Java HotSpot(TM) 64-Bit Server VM (build 17.0.1+12-LTS-39, mixed mode, sharing)"#;

    const OUTPUT_JAVA_8_LEGACY: &str = r#"java version "1.8.0_301"
Java(TM) SE Runtime Environment (build 1.8.0_301-b09)
Java HotSpot(TM) 64-Bit Server VM (build 25.301-b09, mixed mode)"#;

    const OUTPUT_OPENJDK_ARM64: &str = r#"openjdk version "11.0.11" 2021-04-20
OpenJDK Runtime Environment (build 11.0.11+9-post-Ubuntu-0ubuntu2.20.04)
OpenJDK 64-Bit Server VM (build 11.0.11+9-post-Ubuntu-0ubuntu2.20.04, mixed mode, aarch64)"#;

    const OUTPUT_X86_32BIT: &str = r#"java version "1.7.0_80"
Java(TM) SE Runtime Environment (build 1.7.0_80-b15)
Java HotSpot(TM) Client VM (build 24.80-b15, mixed mode)"#;

    fn parse(output: &str) -> Result<JavaInstance> {
        parse_output(PathBuf::from("/mock/java"), output.to_string())
    }

    #[test]
    fn should_parse_modern_java_major_version() {
        let instance = parse(OUTPUT_JAVA_17_HOTSPOT).expect("Should parse valid output");

        assert_eq!(instance.major_version, 17);
        assert_eq!(instance.version, "17.0.1");
    }

    #[test]
    fn should_parse_legacy_java_major_version() {
        let instance = parse(OUTPUT_JAVA_8_LEGACY).expect("Should parse valid output");

        assert_eq!(instance.major_version, 8);
        assert_eq!(instance.version, "1.8.0_301");
    }

    #[test]
    fn should_detect_x64_architecture() {
        let instance = parse(OUTPUT_JAVA_17_HOTSPOT).unwrap();
        assert!(matches!(instance.arch, JavaArch::X64));
    }

    #[test]
    fn should_detect_arm64_architecture() {
        let instance = parse(OUTPUT_OPENJDK_ARM64).unwrap();
        assert!(matches!(instance.arch, JavaArch::Arm64));
    }

    #[test]
    fn should_fallback_to_x86_for_32bit_vm() {
        let instance = parse(OUTPUT_X86_32BIT).unwrap();
        assert!(matches!(instance.arch, JavaArch::X86));
    }

    #[test]
    fn should_extract_vendor_name_from_second_line() {
        let instance = parse(OUTPUT_JAVA_17_HOTSPOT).unwrap();
        assert_eq!(
            instance.vendor_name,
            "Java(TM) SE Runtime Environment (build 17.0.1+12-LTS-39)"
        );
    }

    #[test]
    fn should_handle_single_line_output_for_vendor() {
        let output = r#"version "1.8.0""#;
        let instance = parse(output).unwrap();
        assert_eq!(instance.vendor_name, r#"version "1.8.0""#);
    }

    #[test]
    fn should_handle_malformed_legacy_version_numbers_gracefully() {
        let malformed_output = r#"version "1.Invalid.Number""#;

        let instance = parse(malformed_output);

        assert!(instance.is_err(), "Non-numeric version should return error");
    }

    #[test]
    fn should_handle_malformed_modern_version_numbers_gracefully() {
        let malformed_output = r#"version "Invalid.Number""#;

        let instance = parse(malformed_output);

        assert!(instance.is_err(), "Non-numeric version should return error");
    }

    #[test]
    fn should_fail_when_output_contains_no_version_string() {
        let invalid_output = "Command not found or invalid output";

        let result = parse(invalid_output);

        assert!(
            result.is_err(),
            "Should return error when version string is missing"
        );
    }
}
