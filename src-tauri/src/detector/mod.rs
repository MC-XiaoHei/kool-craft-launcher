pub mod loader;
pub mod model;
pub mod scanner;
mod tests;

pub use loader::{VersionLoadError, VersionLoader};
pub use model::VersionManifest;
pub use scanner::{FileSystemScanner, VersionScanner};

use futures::future::join_all;
use std::path::Path;

pub async fn resolve_all_versions_default(
    minecraft_dir: &Path,
) -> Vec<Result<VersionManifest, VersionLoadError>> {
    resolve_all_versions(&FileSystemScanner, &VersionLoader, minecraft_dir).await
}

pub async fn resolve_all_versions<S>(
    scanner: &S,
    loader: &VersionLoader,
    minecraft_dir: &Path,
) -> Vec<Result<VersionManifest, VersionLoadError>>
where
    S: VersionScanner + ?Sized,
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
