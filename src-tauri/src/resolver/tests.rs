#![cfg_attr(coverage_nightly, coverage(off))]
#![cfg(test)]

use crate::resolver::{FileSystemScanner, VersionLoadError, VersionScanner};
use crate::resolver::model::ArgumentValue;
use crate::resolver::resolve::resolve_all_versions_default;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;
use tokio::io::AsyncWriteExt;

struct TestEnvironment {
    _root: TempDir,
    pub path: PathBuf,
}

impl TestEnvironment {
    async fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path_buf = temp_dir.path().to_path_buf();
        Self {
            _root: temp_dir,
            path: path_buf,
        }
    }

    async fn add_version(&self, id: impl Into<String>, json_content: impl Into<String>) {
        let id = id.into();
        let version_dir = self.path.join("versions").join(id.clone());
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
    let incomplete_version_json = env.path.join("versions").join(version_id).join(version_id.to_owned() + ".json");
    fs::remove_file(incomplete_version_json).await.expect("Failed to remove json file");
    let scanner = FileSystemScanner;
    let result = scanner.scan_versions(env.path).await;
    assert_eq!(result.unwrap(), vec![], "Should not detect version without JSON");
}

#[tokio::test]
async fn test_version_with_same_name_dir_instead_of_json() {
    let env = TestEnvironment::new().await;
    let version_id = "incomplete";
    env.add_version(version_id, "").await;
    let incomplete_version_json = env.path.join("versions").join(version_id).join(version_id.to_owned() + ".json");
    fs::remove_file(incomplete_version_json.clone()).await.expect("Failed to remove json file");
    fs::create_dir(incomplete_version_json).await.expect("Failed to create json dir");
    let scanner = FileSystemScanner;
    let result = scanner.scan_versions(env.path).await;
    assert_eq!(result.unwrap(), vec![], "Should not detect version with dir instead of JSON");
}

#[tokio::test]
async fn test_resolve_vanilla_and_fabric_inheritance() {
    let env = TestEnvironment::new().await;

    env.add_version(
        "1.14",
        r#"{
            "id": "1.14",
            "time": "2019-01-01",
            "releaseTime": "2019-01-01",
            "type": "release",
            "mainClass": "net.minecraft.client.main.Main",
            "libraries": [
                { "name": "com.mojang:patchy:1.1" }
            ]
        }"#,
    )
    .await;

    env.add_version(
        "1.14-fabric",
        r#"{
            "id": "1.14-fabric",
            "inheritsFrom": "1.14",
            "time": "2023-01-01",
            "releaseTime": "2023-01-01",
            "type": "release",
            "mainClass": "net.fabricmc.loader.launch.knot.KnotClient",
            "libraries": [
                { "name": "net.fabricmc:fabric-loader:0.14" }
            ]
        }"#,
    )
    .await;

    let results = resolve_all_versions_default(&env.path).await;

    assert_eq!(results.len(), 2, "Should resolve two versions");

    let fabric_res = results
        .iter()
        .find(|r| r.as_ref().unwrap().id == "1.14-fabric")
        .expect("Fabric version not found");

    let fabric_manifest = fabric_res.as_ref().unwrap();

    assert_eq!(
        fabric_manifest.main_class,
        "net.fabricmc.loader.launch.knot.KnotClient"
    );

    let lib_names: Vec<String> = fabric_manifest
        .libraries
        .iter()
        .map(|l| l.name.clone())
        .collect();

    assert!(
        lib_names.contains(&"com.mojang:patchy:1.1".into()),
        "Should contain parent lib"
    );
    assert!(
        lib_names.contains(&"net.fabricmc:fabric-loader:0.14".into()),
        "Should contain child lib"
    );
}

#[tokio::test]
async fn test_arguments_appending_logic() {
    let env = TestEnvironment::new().await;

    env.add_version(
        "base",
        r#"{
            "id": "base",
            "time": "",
            "releaseTime": "",
            "type": "release",
            "mainClass": "Main",
            "arguments": {
                "game": ["--baseArg"]
            }
        }"#,
    )
    .await;

    env.add_version(
        "child",
        r#"{
            "id": "child",
            "inheritsFrom": "base",
            "time": "",
            "releaseTime": "",
            "type": "release",
            "mainClass": "Main",
            "arguments": {
                "game": ["--childArg"]
            }
        }"#,
    )
    .await;

    let results = resolve_all_versions_default(&env.path).await;

    let child_res = results
        .iter()
        .find(|r| r.as_ref().unwrap().id == "child")
        .unwrap()
        .as_ref()
        .unwrap();

    let game_args = &child_res.arguments.as_ref().unwrap().game;

    let simple_args: Vec<String> = game_args
        .iter()
        .filter_map(|arg| match arg {
            ArgumentValue::Simple(s) => Some(s.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(simple_args, vec!["--baseArg", "--childArg"]);
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

    let results = resolve_all_versions_default(&env.path).await;

    assert_eq!(results.len(), 2, "Should detect both folders");

    let good_res = results.iter().find(|r| r.is_ok());
    let bad_res = results.iter().find(|r| r.is_err());

    assert!(good_res.is_some(), "Good version should load successfully");
    assert!(
        bad_res.is_some(),
        "Bad version should exist as an Error result"
    );

    match bad_res.unwrap() {
        Err(VersionLoadError::Parse { .. }) => (),
        _ => panic!("Expected Parse error for bad JSON"),
    }
}

#[tokio::test]
async fn test_circular_dependency_detection() {
    let env = TestEnvironment::new().await;

    env.add_version(
        "A",
        r#"{
            "id": "A",
            "inheritsFrom": "B",
            "time": "",
            "releaseTime": "", "type": "", "mainClass": ""
        }"#,
    )
    .await;
    env.add_version(
        "B",
        r#"{
            "id": "B",
            "inheritsFrom": "A",
            "time": "",
            "releaseTime": "",
            "type": "",
            "mainClass": ""
        }"#,
    )
    .await;

    let results = resolve_all_versions_default(&env.path).await;

    for res in results {
        match res {
            Err(VersionLoadError::CircularDependency(_)) => (),
            Err(e) => panic!("Wrong error type: {:?}", e),
            Ok(v) => panic!("Should not succeed loading circular dependency: {}", v.id),
        }
    }
}
