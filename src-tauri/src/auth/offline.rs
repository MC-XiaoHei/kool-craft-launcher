use uuid::Uuid;
use crate::auth::model::PlayerProfile;
use crate::auth::model::UserType::Offline;

pub fn generate_offline_uuid(username: impl Into<String>) -> Uuid {
    // as same as java: UUID.nameUUIDFromBytes()
    let input = format!("OfflinePlayer:{}", username.into());

    let digest = md5::compute(input.as_bytes());
    let mut bytes = digest.0;

    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    Uuid::from_bytes(bytes)
}

impl PlayerProfile {
    pub fn of_offline(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            uuid: generate_offline_uuid(name),
            xuid: "".into(),
            access_token: "".into(),
            user_type: Offline,
        }
    }
}