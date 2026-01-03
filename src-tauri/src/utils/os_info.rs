use os_info::Version::Semantic;
use os_info::{Info, Type};
use serde_json::json;

pub fn is_windows_11(info: &Info) -> bool {
    if !is_windows(info) {
        return false;
    }

    match info.version() {
        Semantic(major, _, build) => *major == 10 && *build >= 22000,
        _ => false,
    }
}

pub fn is_windows(info: &Info) -> bool {
    info.os_type() == Type::Windows
}

pub fn is_macos(info: &Info) -> bool {
    info.os_type() == Type::Macos
}

pub fn mock_info(os_type: Type, version: impl Into<String>, arch: impl Into<String>) -> Info {
    let v = json!({
        "os_type": os_type,
        "version": {
            "Custom": version.into(),
        },
        "bitness": "Unknown",
        "architecture": arch.into(),
    });
    serde_json::from_value(v).expect("Failed to deserialize Info mock")
}
