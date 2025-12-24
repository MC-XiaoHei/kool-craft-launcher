use crate::resolver::loader::FileSystemVersionLoader;
use crate::resolver::{
    FileSystemScanner, VersionLoadError, VersionLoader, VersionManifest, VersionScanner,
};
use futures::StreamExt;
use futures::stream;
use std::path::Path;

pub async fn resolve_all_versions_default<P: AsRef<Path>>(
    minecraft_folder: P,
) -> Vec<Result<VersionManifest, VersionLoadError>> {
    resolve_all_versions(
        &FileSystemScanner,
        &FileSystemVersionLoader,
        minecraft_folder,
    )
    .await
}

pub async fn resolve_all_versions<S, L, P>(
    scanner: &S,
    loader: &L,
    minecraft_folder: P,
) -> Vec<Result<VersionManifest, VersionLoadError>>
where
    S: VersionScanner + ?Sized,
    L: VersionLoader + ?Sized,
    P: AsRef<Path>,
{
    let minecraft_folder = minecraft_folder.as_ref().to_path_buf();
    let versions = scanner
        .scan_versions(minecraft_folder.clone())
        .await
        .unwrap_or_default();

    const MAX_CONCURRENT_LOADS: usize = 100;

    stream::iter(versions)
        .map(|meta| loader.load_and_resolve(minecraft_folder.clone(), meta))
        .buffer_unordered(MAX_CONCURRENT_LOADS)
        .collect::<Vec<_>>()
        .await
}
