use crate::auth::microsoft::in_app_oauth::get_microsoft_token_in_app;
use crate::auth::microsoft::model::{MicrosoftToken, OAuthMethod};
use crate::auth::microsoft::xbox_live_token::get_xbox_live_token;
use crate::auth::microsoft::xsts_token::get_xsts_token;
use anyhow::Result;
use reqwest::Client;
use tauri::AppHandle;
use crate::auth::microsoft::game_profile::get_game_profile;
use crate::auth::microsoft::minecraft_token::get_minecraft_token;

pub async fn microsoft_account_login(app_handle: &AppHandle) -> Result<()> {
    let method = OAuthMethod::InApp;
    let client = Client::new();
    let ms_token = get_microsoft_token(app_handle, client.clone(), method).await?;
    let xbl_token = get_xbox_live_token(client.clone(), ms_token.clone()).await?;
    let xsts_token = get_xsts_token(client.clone(), xbl_token).await?;
    let mc_token = get_minecraft_token(client.clone(), xsts_token).await?;
    let profile = get_game_profile(client, mc_token.clone()).await?;
    println!("mc profile: {:?}", profile);

    Ok(())
}

async fn get_microsoft_token(
    app_handle: &AppHandle,
    client: Client,
    oauth_method: OAuthMethod,
) -> Result<MicrosoftToken> {
    match oauth_method {
        OAuthMethod::InApp => get_microsoft_token_in_app(app_handle, client).await,
        OAuthMethod::Browser => todo!(),
    }
}
