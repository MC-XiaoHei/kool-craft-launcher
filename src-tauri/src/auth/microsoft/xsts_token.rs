use crate::auth::microsoft::model::{XSTSToken, XboxLiveToken};
use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub async fn get_xsts_token(client: Client, xbl_token: XboxLiveToken) -> Result<XSTSToken> {
    let payload = XSTSAuthRequest {
        relying_party: "rp://api.minecraftservices.com/".to_string(),
        token_type: "JWT".to_string(),
        properties: XSTSAuthProperties {
            user_tokens: vec![xbl_token.token.to_string()],
            sandbox_id: "RETAIL".to_string(),
        },
    };

    let response = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .json(&payload)
        .send()
        .await?;

    if response.status().as_u16() == 401 {
        return Err(anyhow!("XSTS refused to auth: {}", response.text().await?));
    }

    let data: XSTSAuthResponse = response.json().await?;

    let token = data.token.clone();
    let uhs = data
        .display_claims
        .xui
        .first()
        .ok_or_else(|| anyhow!("No uhs in xsts response: {data:?}"))?
        .uhs
        .clone();

    if uhs != xbl_token.user_hash {
        return Err(anyhow!(
            "XSTS uhs {uhs} not equals to Xbox Live uhs: {}",
            xbl_token.user_hash
        ));
    }

    Ok(XSTSToken {
        token,
        user_hash: uhs,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
struct XSTSAuthRequest {
    relying_party: String,
    token_type: String,
    properties: XSTSAuthProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
struct XSTSAuthProperties {
    user_tokens: Vec<String>,
    sandbox_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
struct XSTSAuthResponse {
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
