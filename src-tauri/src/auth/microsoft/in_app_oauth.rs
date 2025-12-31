use crate::auth::microsoft::model::CLIENT_ID;
use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindowBuilder};

const LOGIN_WINDOW_ID: &str = "in_app_microsoft_login_window";

fn open_login_window(app: &AppHandle) {
    ensure_previous_window_closed(app);
}

fn ensure_previous_window_closed(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(LOGIN_WINDOW_ID) {
        let _ = window.close();
    }
}

fn create_login_window(app: &AppHandle) {
    WebviewWindowBuilder::new(app, LOGIN_WINDOW_ID, WebviewUrl::External(get_oauth_url()));
}

fn get_oauth_url() -> Url {
    get_oauth_url_str()
        .parse()
        .expect("Internal Error: Fail to parse in_app_microsoft_login url") // this should never happen
}

fn get_oauth_url_str() -> String {
    [
        "https://login.live.com/oauth20_desktop.srf",
        "?client_id=",
        CLIENT_ID,
        "&response_type=code",
        "&redirect_uri=https%3A%2F%2Flogin.live.com%2Foauth20_desktop.srf",
        "&scope=XboxLive.Signin%20offline_access",
        "&prompt=select_account",
    ]
    .join("")
}
