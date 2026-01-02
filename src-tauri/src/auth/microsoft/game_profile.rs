use crate::auth::microsoft::model::{GameProfile, MinecraftToken};
use anyhow::{Result, anyhow};
use reqwest::Client;

pub async fn get_game_profile(client: Client, mc_token: MinecraftToken) -> Result<GameProfile> {
    let response = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(mc_token.token)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Fail to fetch game profile: {}",
            response.text().await?
        ));
    }

    Ok(response.json().await?)
}
