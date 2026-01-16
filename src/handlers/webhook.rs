use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};

use crate::db::{alerts, groups, rules};
use crate::models::sentry::SentryWebhookPayload;
use crate::services::{feishu, message_builder, router, signature};
use crate::AppState;

pub async fn handle_sentry_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // Log incoming webhook
    tracing::info!("Received Sentry webhook");

    // Convert body to string for logging and storage
    let raw_payload = String::from_utf8_lossy(&body).to_string();

    // Verify signature if secret is configured
    if let Some(ref secret) = state.config.sentry_client_secret {
        let signature_header = headers
            .get("sentry-hook-signature")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !signature::verify_sentry_signature(&body, signature_header, secret) {
            tracing::warn!("Invalid webhook signature: verification failed");

            // Record verification failure
            let _ = alerts::create(
                &state.db,
                alerts::CreateAlert {
                    sentry_event_id: None,
                    issue_id: None,
                    project: None,
                    environment: None,
                    platform: None,
                    title: Some("Security: Invalid Signature".to_string()),
                    message: Some("Webhook signature verification failed".to_string()),
                    level: Some("error".to_string()),
                    web_url: None,
                    triggered_rule: None,
                    matched_rule_id: None,
                    target_group_id: None,
                    target_group_name: None,
                    status: "verification_failed".to_string(),
                    error_message: Some("Invalid or missing sentry-hook-signature".to_string()),
                    raw_payload: Some(raw_payload),
                },
            )
            .await;

            return (StatusCode::UNAUTHORIZED, "Invalid signature").into_response();
        }
    }

    // Log raw payload for debugging
    tracing::debug!("Sentry raw payload: {}", raw_payload);

    // Parse payload
    let payload: SentryWebhookPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to parse Sentry payload: {}", e);
            tracing::error!("Raw payload was: {}", raw_payload);
            return (StatusCode::BAD_REQUEST, "Invalid payload").into_response();
        }
    };

    // Check resource type
    let resource = headers
        .get("sentry-hook-resource")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    tracing::info!(
        "Sentry webhook: resource={}, action={}",
        resource,
        payload.action
    );

    // Only process event_alert and metric_alert
    if resource != "event_alert" && resource != "metric_alert" {
        tracing::debug!("Ignoring webhook resource: {}", resource);
        return (StatusCode::OK, "Ignored").into_response();
    }

    // Get enabled routing rules
    let enabled_rules = match rules::get_enabled_rules(&state.db).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to get routing rules: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    // Match rules
    let matched_rule = router::match_rule(&payload, &enabled_rules);

    if matched_rule.is_none() {
        tracing::info!("No matching rule found for payload");

        // Record as unmatched
        let _ = alerts::create(
            &state.db,
            alerts::CreateAlert {
                sentry_event_id: payload.get_event_id(),
                issue_id: payload.get_issue_id(),
                project: payload.get_project(),
                environment: payload.get_environment(),
                platform: payload.get_platform(),
                title: payload.get_title(),
                message: payload.get_message(),
                level: payload.get_level(),
                web_url: payload.get_web_url(),
                triggered_rule: payload.get_triggered_rule(),
                matched_rule_id: None,
                target_group_id: None,
                target_group_name: None,
                status: "no_match".to_string(),
                error_message: Some("No matching routing rule".to_string()),
                raw_payload: Some(raw_payload),
            },
        )
        .await;

        return (StatusCode::OK, "No matching rule").into_response();
    }

    let matched_rule = matched_rule.unwrap();

    // Get target group
    let group = match groups::get_by_id(&state.db, matched_rule.group_id).await {
        Ok(Some(g)) => g,
        Ok(None) => {
            tracing::error!("Target group not found: {}", matched_rule.group_id);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Group not found").into_response();
        }
        Err(e) => {
            tracing::error!("Failed to get group: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    // Build message
    let message = message_builder::build_message(&group, &payload, &state.db).await;

    // Send to Feishu
    let result = feishu::send_message(&state.http_client, &group.webhook_url, &message).await;

    // Record alert history
    let (status, error_message) = match &result {
        Ok(_) => ("success".to_string(), None),
        Err(e) => ("failed".to_string(), Some(e.to_string())),
    };

    let _ = alerts::create(
        &state.db,
        alerts::CreateAlert {
            sentry_event_id: payload.get_event_id(),
            issue_id: payload.get_issue_id(),
            project: payload.get_project(),
            environment: payload.get_environment(),
            platform: payload.get_platform(),
            title: payload.get_title(),
            message: payload.get_message(),
            level: payload.get_level(),
            web_url: payload.get_web_url(),
            triggered_rule: payload.get_triggered_rule(),
            matched_rule_id: Some(matched_rule.id),
            target_group_id: Some(group.id),
            target_group_name: Some(group.name.clone()),
            status,
            error_message,
            raw_payload: Some(raw_payload),
        },
    )
    .await;

    match result {
        Ok(_) => {
            tracing::info!("Alert sent successfully to group: {}", group.name);
            (StatusCode::OK, "OK").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to send alert: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to send alert").into_response()
        }
    }
}
