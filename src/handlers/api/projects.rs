use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};

use crate::db::projects::{self, CreateProject, UpdateProject};
use crate::AppState;

pub async fn list_projects(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match projects::list(&state.db).await {
        Ok(projects) => Ok(Json(json!({ "projects": projects }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match projects::get_by_id(&state.db, id).await {
        Ok(Some(project)) => Ok(Json(json!(project))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Project not found" })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn create_project(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateProject>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match projects::create(&state.db, input).await {
        Ok(project) => Ok((StatusCode::CREATED, Json(json!(project)))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn update_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateProject>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match projects::update(&state.db, id, input).await {
        Ok(Some(project)) => Ok(Json(json!(project))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Project not found" })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

pub async fn delete_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match projects::delete(&state.db, id).await {
        Ok(true) => Ok(Json(json!({ "success": true }))),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Project not found" })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}
