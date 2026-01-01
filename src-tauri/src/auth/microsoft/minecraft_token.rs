use crate::auth::microsoft::model::{MinecraftToken, XSTSToken};
use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn get_minecraft_token(client: Client, xsts_token: XSTSToken) -> Result<MinecraftToken> {
    let token = xsts_token.token;
    let uhs = xsts_token.user_hash;
    let identity_token = format!("XBL3.0 x={};{}", uhs, token);

    let payload = MinecraftLoginRequest { identity_token };

    let response = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Fail to login to Minecraft {}",
            response.text().await?
        ));
    }

    let data: MinecraftLoginResponse = response.json().await?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Ok(MinecraftToken {
        token: data.access_token,
        expires_at: now + data.expires_in,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftLoginRequest {
    pub identity_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MinecraftLoginResponse {
    pub username: String,
    pub access_token: String,
    pub expires_in: u64,
}
