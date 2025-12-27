pub mod launcher {
    pub const LAUNCHER_NAME: &str = "Kool Craft Launcher";
    pub const SHORT_LAUNCHER_NAME: &str = "KCl";
    pub const LAUNCHER_VERSION: &str = env!("CARGO_PKG_VERSION");
    pub const LAUNCHER_VERSION_MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");
    pub const LAUNCHER_VERSION_MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");
    pub const LAUNCHER_VERSION_PATCH: &str = env!("CARGO_PKG_VERSION_PATCH");
}

pub mod minecraft_dir {
    pub const VERSIONS_DIR_NAME: &str = "versions";
    pub const ASSETS_DIR_NAME: &str = "assets";
    pub const NATIVES_DIR_NAME: &str = "natives";
    pub const LIBRARIES_DIR_NAME: &str = "libraries";
}

pub mod minecraft_behavior {
    pub const DEFAULT_VERSION_INDEPENDENT: bool = true;
}
