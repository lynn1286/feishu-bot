use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

use crate::db::{alerts, groups, rules};
use crate::AppState;

pub async fn get_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Get alert stats
    let alert_stats = match alerts::get_stats(&state.db).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to get alert stats: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    // Get group count
    let groups = match groups::list(&state.db).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to list groups: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    // Get rule count
    let rules_list = match rules::list(&state.db).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to list rules: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    let enabled_rules = rules_list.iter().filter(|r| r.rule.enabled == 1).count();

    Json(json!({
        "alerts": {
            "total": alert_stats.total,
            "success": alert_stats.success,
            "failed": alert_stats.failed,
            "today": alert_stats.today,
        },
        "groups": {
            "total": groups.len(),
        },
        "rules": {
            "total": rules_list.len(),
            "enabled": enabled_rules,
        },
        "security": {
            "sentry_client_secret_set": state.config.sentry_client_secret.is_some(),
        }
    }))
    .into_response()
}
