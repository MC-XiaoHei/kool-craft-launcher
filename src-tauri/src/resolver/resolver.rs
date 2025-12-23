use crate::resolver::loader::FileSystemVersionLoader;
use crate::resolver::{
    FileSystemScanner, VersionLoadError, VersionLoader, VersionManifest, VersionScanner,
};
use futures::future::join_all;
use std::path::Path;

pub async fn resolve_all_versions_default(
    minecraft_dir: &Path,
) -> Vec<Result<VersionManifest, VersionLoadError>> {
    resolve_all_versions(&FileSystemScanner, &FileSystemVersionLoader, minecraft_dir).await
}

pub async fn resolve_all_versions<S, L>(
    scanner: &S,
    loader: &L,
    minecraft_dir: &Path,
) -> Vec<Result<VersionManifest, VersionLoadError>>
where
    S: VersionScanner + ?Sized,
    L: VersionLoader + ?Sized,
{
    let meta_list = match scanner.scan_versions(minecraft_dir).await {
        Ok(list) => list,
        Err(_) => return vec![],
    };

    let tasks: Vec<_> = meta_list
        .iter()
        .map(|meta| loader.load_and_resolve(meta, minecraft_dir))
        .collect();

    join_all(tasks).await
}
