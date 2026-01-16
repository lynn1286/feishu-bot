use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::db::{
    alerts::{self, ListAlertsQuery},
    groups,
};
use crate::models::sentry::SentryWebhookPayload;
use crate::services::{feishu, message_builder};
use crate::AppState;

pub async fn list_alerts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListAlertsQuery>,
) -> impl IntoResponse {
    match alerts::list(&state.db, query).await {
        Ok(result) => Json(result).into_response(),
        Err(e) => {
            tracing::error!("Failed to list alerts: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

pub async fn get_alert(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match alerts::get_by_id(&state.db, id).await {
        Ok(Some(alert)) => Json(json!({ "data": alert })).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Alert not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to get alert: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

pub async fn retry_alert(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    // Get alert
    let alert = match alerts::get_by_id(&state.db, id).await {
        Ok(Some(a)) => a,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Alert not found" })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get alert: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    // Check if we have raw payload and target group
    let raw_payload = match &alert.raw_payload {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "No raw payload available" })),
            )
                .into_response()
        }
    };

    let group_id = match alert.target_group_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "No target group" })),
            )
                .into_response()
        }
    };

    // Parse payload
    let payload: SentryWebhookPayload = match serde_json::from_str(raw_payload) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("Invalid payload: {}", e) })),
            )
                .into_response()
        }
    };

    // Get group
    let group = match groups::get_by_id(&state.db, group_id).await {
        Ok(Some(g)) => g,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Target group not found" })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get group: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    // Build and send message
    let message = message_builder::build_message(&group, &payload, &state.db).await;

    match feishu::send_message(&state.http_client, &group.webhook_url, &message).await {
        Ok(_) => {
            // Update alert status
            let _ = alerts::update_status(&state.db, id, "success", None).await;
            Json(json!({ "success": true, "message": "Alert resent successfully" })).into_response()
        }
        Err(e) => {
            let _ = alerts::update_status(&state.db, id, "failed", Some(&e.to_string())).await;
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": e.to_string() })),
            )
                .into_response()
        }
    }
}
