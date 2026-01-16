use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::db::groups::{self, CreateGroup, UpdateGroup};
use crate::services::feishu;
use crate::AppState;

pub async fn list_groups(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match groups::list(&state.db).await {
        Ok(groups) => Json(json!({ "data": groups })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list groups: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

pub async fn get_group(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match groups::get_by_id(&state.db, id).await {
        Ok(Some(group)) => Json(json!({ "data": group })).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Group not found" }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to get group: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

pub async fn create_group(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateGroup>,
) -> impl IntoResponse {
    match groups::create(&state.db, input).await {
        Ok(group) => (StatusCode::CREATED, Json(json!({ "data": group }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to create group: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

pub async fn update_group(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateGroup>,
) -> impl IntoResponse {
    match groups::update(&state.db, id, input).await {
        Ok(Some(group)) => Json(json!({ "data": group })).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Group not found" }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to update group: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

pub async fn delete_group(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match groups::delete(&state.db, id).await {
        Ok(true) => (StatusCode::NO_CONTENT, "").into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Group not found" }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to delete group: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

pub async fn test_group(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    // Get group
    let group = match groups::get_by_id(&state.db, id).await {
        Ok(Some(g)) => g,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({ "error": "Group not found" }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to get group: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response();
        }
    };

    // Send test message using group configuration
    match feishu::send_test_message(&state.http_client, &group).await {
        Ok(_) => Json(json!({ "success": true, "message": "Test message sent successfully" })).into_response(),
        Err(e) => {
            tracing::error!("Failed to send test message: {}", e);
            (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": e.to_string() }))).into_response()
        }
    }
}
