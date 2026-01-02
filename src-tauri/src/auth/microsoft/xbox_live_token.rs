use crate::auth::microsoft::model::{MicrosoftToken, XboxLiveToken};
use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub async fn get_xbox_live_token(
    client: Client,
    ms_token: MicrosoftToken,
) -> Result<XboxLiveToken> {
    let payload = XboxAuthRequest {
        relying_party: "http://auth.xboxlive.com".to_string(),
        token_type: "JWT".to_string(),
        properties: XboxAuthProperties {
            auth_method: "RPS".to_string(),
            site_name: "user.auth.xboxlive.com".to_string(),
            rps_ticket: format!("d={}", ms_token.access_token),
        },
    };

    let response = client
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Fail to pass Xbox Live auth {}",
            response.text().await?
        ));
    }

    let data: XboxAuthResponse = response.json().await?;

    let token = data.token.clone();
    let uhs = data
        .display_claims
        .xui
        .first()
        .ok_or_else(|| anyhow!("No uhs in xbox response: {data:?}"))?
        .uhs
        .clone();

    Ok(XboxLiveToken {
        token,
        user_hash: uhs,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
struct XboxAuthRequest {
    relying_party: String,
    token_type: String,
    properties: XboxAuthProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
struct XboxAuthProperties {
    auth_method: String,
    site_name: String,
    rps_ticket: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
struct XboxAuthResponse {
    issue_instant: String,
    not_after: String,
    token: String,
    display_claims: DisplayClaims,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct DisplayClaims {
    xui: Vec<Xui>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Xui {
    uhs: String,
}
