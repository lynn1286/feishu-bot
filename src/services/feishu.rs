use anyhow::{Context, Result};
use reqwest::Client;

use crate::db::groups::FeishuGroup;
use crate::models::feishu::FeishuMessage;

/// Send message to Feishu webhook
pub async fn send_message(client: &Client, webhook_url: &str, message: &FeishuMessage) -> Result<()> {
    let json = serde_json::to_string_pretty(message)?;
    tracing::info!("Sending Feishu message: {}", json);

    let response = client
        .post(webhook_url)
        .json(message)
        .send()
        .await
        .context("Failed to send request to Feishu")?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    if !status.is_success() {
        anyhow::bail!("Feishu API returned error: {} - {}", status, body);
    }

    // Parse response to check for Feishu-specific errors
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        if let Some(code) = json.get("code").and_then(|c| c.as_i64()) {
            if code != 0 {
                let msg = json.get("msg").and_then(|m| m.as_str()).unwrap_or("Unknown error");
                anyhow::bail!("Feishu API error: {} - {}", code, msg);
            }
        }
    }

    Ok(())
}

/// Send test message to Feishu webhook using group configuration
pub async fn send_test_message(client: &Client, group: &FeishuGroup) -> Result<()> {
    let message = crate::services::message_builder::build_test_message(group);
    send_message(client, &group.webhook_url, &message).await
}
