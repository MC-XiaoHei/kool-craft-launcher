use crate::java_runtime::inspector::inspect_java_executable;
use crate::java_runtime::model::JavaInstance;
use futures::StreamExt;
use futures::stream;
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use tokio::fs::read_dir;

pub async fn scan_all() -> Vec<JavaInstance> {
    let candidates = find_candidate_paths().await;
    validate_concurrently(candidates).await
}

async fn find_candidate_paths() -> HashSet<PathBuf> {
    let mut candidates = HashSet::new();

    candidates.extend(scan_java_home().await);
    candidates.extend(scan_path_environment().await);
    candidates.extend(scan_current_directory().await);

    candidates
}

async fn validate_concurrently(paths: HashSet<PathBuf>) -> Vec<JavaInstance> {
    let mut tasks = Vec::new();

    let paths = stream::iter(paths)
        .filter(|path| is_executable(path.clone()))
        .collect::<Vec<PathBuf>>()
        .await;

    for path in paths {
        tasks.push(tokio::spawn(async move {
            if is_executable(path.clone()).await {
                inspect_java_executable(path).await
            } else {
                None
            }
        }));
    }

    let mut valid_installations = Vec::new();
    let results = futures::future::join_all(tasks).await;

    for result in results {
        if let Ok(Some(java_install)) = result {
            valid_installations.push(java_install);
        }
    }

    valid_installations
}

async fn scan_java_home() -> Vec<PathBuf> {
    if let Some(home_var) = env::var_os("JAVA_HOME") {
        let bin_path = PathBuf::from(home_var).join("bin").join(binary_name());
        vec![bin_path]
    } else {
        vec![]
    }
}

async fn scan_path_environment() -> Vec<PathBuf> {
    let Some(path_var) = env::var_os("PATH") else {
        return vec![];
    };

    stream::iter(env::split_paths(&path_var))
        .map(|p| p.join(binary_name()))
        .collect()
        .await
}

async fn scan_current_directory() -> Vec<PathBuf> {
    let Ok(cwd) = env::current_dir() else {
        return vec![];
    };
    let binary = binary_name();

    let mut paths = vec![
        cwd.join(binary),
        cwd.join("bin").join(binary),
        cwd.join("java").join("bin").join(binary),
        cwd.join("jre").join("bin").join(binary),
    ];

    let runtime_dir = cwd.join("runtime");
    if let Ok(mut entries) = read_dir(&runtime_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let Ok(file_type) = entry.file_type().await else {
                continue;
            };

            if file_type.is_dir() {
                paths.push(entry.path().join("bin").join(binary));
            }
        }
    }

    paths
}

fn binary_name() -> &'static str {
    if cfg!(windows) { "java.exe" } else { "java" }
}

async fn is_executable(path: PathBuf) -> bool {
    tokio::fs::metadata(path).await.is_ok()
}
