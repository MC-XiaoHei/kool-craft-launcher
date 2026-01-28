use crate::auth::microsoft::model::{CLIENT_ID, MicrosoftToken};
use crate::constants::launcher::LAUNCHER_NAME;
use anyhow::{Result, anyhow};
use log::warn;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tokio::sync::oneshot;
use url::Url;

const LOGIN_WINDOW_ID: &str = "microsoft_login_window";
const LOGIN_URL: &str = "https://login.live.com/oauth20_authorize.srf";
const LOGIN_REDIRECT_URL: &str = "https://login.live.com/oauth20_desktop.srf";
const TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";

pub async fn get_microsoft_token(app: &AppHandle, client: Client) -> Result<MicrosoftToken> {
    let code_rx = open_login_window(app).await?;
    let code = code_rx.await??;
    let ms_token = get_microsoft_token_by_code(client, code).await?;
    Ok(ms_token)
}

async fn open_login_window(app: &AppHandle) -> Result<oneshot::Receiver<Result<String>>> {
    ensure_previous_window_closed(app);
    let (window, rx) = create_login_window(app)?;
    window.show()?;
    window.center()?;
    Ok(rx)
}

fn ensure_previous_window_closed(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(LOGIN_WINDOW_ID) {
        let _ = window.close();
    }
}

fn close_auth_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(LOGIN_WINDOW_ID) {
        if let Err(e) = window.close() {
            warn!("Failed to close auth window: {e:?}");
        }
    } else {
        warn!("Fail to close auth window because window not found");
    }
}

fn create_login_window(
    app: &AppHandle,
) -> Result<(WebviewWindow, oneshot::Receiver<Result<String>>)> {
    let (tx, rx) = oneshot::channel::<Result<String>>();

    let tx_holder = Arc::new(Mutex::new(Some(tx)));
    let handle = app.clone();
    let window =
        WebviewWindowBuilder::new(app, LOGIN_WINDOW_ID, WebviewUrl::External(get_oauth_url()))
            .title(LAUNCHER_NAME) // TODO
            .inner_size(500.0, 650.0)
            .on_navigation(move |url| navigation_handler(url, &handle, tx_holder.clone()))
            .build()?;

    Ok((window, rx))
}

fn get_oauth_url() -> Url {
    let mut url = Url::parse(LOGIN_URL).expect("Internal Error: Fail to parse microsoft login url"); // this should never happen

    url.query_pairs_mut()
        .append_pair("client_id", CLIENT_ID)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", LOGIN_REDIRECT_URL)
        .append_pair("scope", "XboxLive.Signin offline_access")
        .append_pair("prompt", "select_account");

    url
}

fn navigation_handler(
    url: &Url,
    app: &AppHandle,
    tx: Arc<Mutex<Option<oneshot::Sender<Result<String>>>>>,
) -> bool {
    if url.as_str().starts_with(LOGIN_REDIRECT_URL) {
        close_auth_window(app);
        let code = extract_code_from_url(url.clone());
        send_code_to_channel(tx, code);
        false
    } else {
        true
    }
}

fn send_code_to_channel(
    tx: Arc<Mutex<Option<oneshot::Sender<Result<String>>>>>,
    code: Result<String>,
) {
    if let Ok(mut guard) = tx.lock()
        && let Some(sender) = guard.take()
    {
        let _ = sender.send(code);
    }
}

fn extract_code_from_url(url: Url) -> Result<String> {
    let params = url.query_pairs().into_owned().collect::<HashMap<_, _>>();
    let code = params.get("code");
    if let Some(code) = code {
        Ok(code.to_string())
    } else if let Some(err_msg) = params.get("error_description") {
        Err(anyhow!(err_msg.to_string()))
    } else {
        Err(anyhow!("unknown error: {url:?}"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct OAuthTokenResponse {
    access_token: String,
    refresh_token: String,
    user_id: String,
    expires_in: u64,
}

async fn get_microsoft_token_by_code(
    client: Client,
    code: String,
) -> Result<MicrosoftToken> {
    let params = [
        ("client_id", CLIENT_ID),
        ("code", code.as_str()),
        ("grant_type", "authorization_code"),
        ("redirect_uri", LOGIN_REDIRECT_URL),
    ];

    let response = client.post(TOKEN_URL).form(&params).send().await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow::anyhow!("Fail to get token from code: {error_text}"));
    }

    let data: OAuthTokenResponse = response.json().await?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let token = MicrosoftToken {
        access_token: data.access_token,
        refresh_token: data.refresh_token,
        user_id: data.user_id,
        expires_at: now + data.expires_in,
    };

    Ok(token)
}
