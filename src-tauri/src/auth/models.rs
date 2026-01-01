use UserType::*;
use std::fmt::{Display, Formatter};
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

impl Display for UserType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Microsoft => "msa",
            AuthLib => "mojang",
            Offline | Demo => "legacy",
        })
    }
}
