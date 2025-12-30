use crate::game_resolver::loader::FileSystemVersionLoader;
use crate::game_resolver::model::{MinecraftFolderInfo, MinecraftFolderSettings, VersionData};
use crate::game_resolver::{FileSystemScanner, VersionLoader, VersionScanner};
use crate::utils::abs_path_buf::AbsPathBuf;
use futures::StreamExt;
use futures::stream;

pub async fn resolve_minecraft_folder(minecraft_folder: AbsPathBuf) -> MinecraftFolderInfo {
    let version_info = resolve_all_versions_default(minecraft_folder.clone()).await;

    MinecraftFolderInfo {
        path: minecraft_folder,
        settings: MinecraftFolderSettings::default(),
        version_info,
    }
}

pub async fn resolve_all_versions_default(minecraft_folder: AbsPathBuf) -> Vec<VersionData> {
    resolve_all_versions(
        &FileSystemScanner,
        &FileSystemVersionLoader,
        minecraft_folder,
    )
    .await
}

pub async fn resolve_all_versions<S, L>(
    scanner: &S,
    loader: &L,
    minecraft_folder: AbsPathBuf,
) -> Vec<VersionData>
where
    S: VersionScanner + ?Sized,
    L: VersionLoader + ?Sized,
{
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
