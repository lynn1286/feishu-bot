use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::db::rules::{self, CreateRule, UpdateRule};
use crate::AppState;

pub async fn list_rules(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match rules::list(&state.db).await {
        Ok(rules) => Json(json!({ "data": rules })).into_response(),
        Err(e) => {
            tracing::error!("Failed to list rules: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

pub async fn get_rule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match rules::get_by_id(&state.db, id).await {
        Ok(Some(rule)) => Json(json!({ "data": rule })).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Rule not found" }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to get rule: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

pub async fn create_rule(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateRule>,
) -> impl IntoResponse {
    match rules::create(&state.db, input).await {
        Ok(rule) => (StatusCode::CREATED, Json(json!({ "data": rule }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to create rule: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

pub async fn update_rule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateRule>,
) -> impl IntoResponse {
    match rules::update(&state.db, id, input).await {
        Ok(Some(rule)) => Json(json!({ "data": rule })).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Rule not found" }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to update rule: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

pub async fn delete_rule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match rules::delete(&state.db, id).await {
        Ok(true) => (StatusCode::NO_CONTENT, "").into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(json!({ "error": "Rule not found" }))).into_response(),
        Err(e) => {
            tracing::error!("Failed to delete rule: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}
