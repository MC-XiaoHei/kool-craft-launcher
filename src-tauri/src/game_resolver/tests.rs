#![cfg_attr(coverage_nightly, coverage(off))]
#![cfg(test)]

use crate::constants::minecraft_dir::VERSIONS_DIR_NAME;
use crate::game_resolver::model::VersionData::{Broken, Normal};
use crate::game_resolver::resolve::resolve_all_versions_default;
use crate::game_resolver::{FileSystemScanner, VersionScanner};
use crate::utils::abs_path_buf::AbsPathBuf;
use tempfile::TempDir;
use tokio::fs;
use tokio::io::AsyncWriteExt;

struct TestEnvironment {
    _root: TempDir,
    pub path: AbsPathBuf,
}

impl TestEnvironment {
    async fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path_buf = temp_dir.path().to_path_buf().try_into().unwrap();
        Self {
            _root: temp_dir,
            path: path_buf,
        }
    }

    async fn add_version(&self, id: impl Into<String>, json_content: impl Into<String>) {
        let id = id.into();
        let version_dir = self.path.join(VERSIONS_DIR_NAME).join(id.clone());
        fs::create_dir_all(&version_dir)
            .await
            .expect("Failed to create version dir");

        let file_path = version_dir.join(format!("{id}.json"));
        let mut file = fs::File::create(file_path)
            .await
            .expect("Failed to create json file");
        file.write_all(json_content.into().as_bytes())
            .await
            .expect("Failed to write json");
    }
}

#[tokio::test]
async fn test_version_without_json() {
    let env = TestEnvironment::new().await;
    let version_id = "incomplete";
    env.add_version(version_id, "").await;
    let incomplete_version_json = env
        .path
        .join(VERSIONS_DIR_NAME)
        .join(version_id)
        .join(version_id.to_owned() + ".json");
    fs::remove_file(incomplete_version_json)
        .await
        .expect("Failed to remove json file");
    let scanner = FileSystemScanner;
    let result = scanner.scan_versions(env.path).await;
    assert_eq!(
        result.unwrap(),
        vec![],
        "Should not detect version without JSON"
    );
}

#[tokio::test]
async fn test_version_with_same_name_dir_instead_of_json() {
    let env = TestEnvironment::new().await;
    let version_id = "incomplete";
    env.add_version(version_id, "").await;
    let incomplete_version_json = env
        .path
        .join(VERSIONS_DIR_NAME)
        .join(version_id)
        .join(version_id.to_owned() + ".json");
    fs::remove_file(incomplete_version_json.clone())
        .await
        .expect("Failed to remove json file");
    fs::create_dir(incomplete_version_json)
        .await
        .expect("Failed to create json dir");
    let scanner = FileSystemScanner;
    let result = scanner.scan_versions(env.path).await;
    assert_eq!(
        result.unwrap(),
        vec![],
        "Should not detect version with dir instead of JSON"
    );
}

#[tokio::test]
async fn test_resilience_to_broken_json() {
    let env = TestEnvironment::new().await;

    env.add_version(
        "good",
        r#"{
            "id": "good",
            "time": "",
            "releaseTime": "",
            "type": "release",
            "mainClass": "Main"
        }"#,
    )
    .await;
    env.add_version(
        "bad",
        r#"{
            "id": "bad",
            "missing_bracket": "#,
    )
    .await;

    let results = resolve_all_versions_default(env.path).await;

    assert_eq!(results.len(), 2, "Should detect both folders");

    let good_res = results.iter().find(|r| matches!(r, Normal(_)));
    let bad_res = results.iter().find(|r| matches!(r, Broken(_)));

    assert!(good_res.is_some(), "Good version should load successfully");
    assert!(
        bad_res.is_some(),
        "Bad version should exist as an broken result"
    );
}
