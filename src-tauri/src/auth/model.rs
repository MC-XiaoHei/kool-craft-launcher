use UserType::*;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PlayerProfile {
    pub name: String,
    pub uuid: Uuid,
    pub xuid: String,
    pub access_token: String,
    pub user_type: UserType,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UserType {
    Microsoft,
    Offline,
    AuthLib,
    Demo,
}

impl UserType {
    pub fn to_string(&self) -> String {
        match self {
            Microsoft => "msa",
            AuthLib => "mojang",
            Offline | Demo => "legacy",
        }
        .into()
    }
}
