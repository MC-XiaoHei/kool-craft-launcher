use serde::{Deserialize, Serialize};

pub const CLIENT_ID: &str = "195f260c-d211-4160-99d6-9c18e3a1db73";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OAuthMethod {
    InApp,
    Browser,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MicrosoftToken {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct XboxLiveToken {
    pub token: String,
    pub user_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct XSTSToken {
    pub token: String,
    pub user_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MinecraftToken {
    pub token: String,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameProfile {
    pub id: String,
    pub name: String,
    pub skins: Vec<Skin>,
    #[serde(default)]
    pub capes: Vec<Cape>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Skin {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: SkinVariant,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkinVariant {
    #[serde(rename = "CLASSIC")]
    #[default]
    Classic,
    #[serde(rename = "SLIM")]
    Slim,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Cape {
    pub id: String,
    pub state: String,
    pub url: String,
    pub alias: String,
}

pub struct LoginResponse {
    pub access_token: String,
    pub expires_at: u64,
}
