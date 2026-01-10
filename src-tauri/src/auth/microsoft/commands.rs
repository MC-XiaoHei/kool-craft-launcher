use crate::auth::microsoft::game_profile::get_game_profile;
use crate::auth::microsoft::microsoft_login::get_microsoft_token;
use crate::auth::microsoft::minecraft_token::get_minecraft_token;
use crate::auth::microsoft::xbox_live_token::get_xbox_live_token;
use crate::auth::microsoft::xsts_token::get_xsts_token;
use crate::utils::command::CommandResult;
use anyhow::Result;
use macros::command;
use reqwest::Client;
use tauri::{AppHandle, Builder, Runtime};

#[command]
pub async fn microsoft_account_login(app_handle: AppHandle) -> CommandResult<()> {
    let client = Client::new();
    let ms_token = get_microsoft_token(&app_handle, client.clone()).await?;
    let xbl_token = get_xbox_live_token(client.clone(), ms_token.clone()).await?;
    let xsts_token = get_xsts_token(client.clone(), xbl_token).await?;
    let mc_token = get_minecraft_token(client.clone(), xsts_token).await?;
    let profile = get_game_profile(client, mc_token.clone()).await?;
    println!("mc profile: {:?}", profile);

    Ok(())
}
