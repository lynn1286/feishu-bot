use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Response, Redirect},
    Json,
};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::sync::Arc;

use crate::AppState;
use crate::db::users;

#[derive(RustEmbed)]
#[folder = "web/dist"]
pub struct AdminAssets;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password_hash: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub success: bool,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password_hash: String,
    pub new_password_hash: String,
}

pub fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    let user = users::get_user_by_username(&state.db, &req.username).await;

    if let Ok(Some((_, stored_hash))) = user {
        if req.password_hash == stored_hash {
            return Json(LoginResponse { success: true });
        }
    }

    Json(LoginResponse { success: false })
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    let user = users::get_user_by_username(&state.db, "admin").await;

    if let Ok(Some((_, stored_hash))) = user {
        if req.old_password_hash == stored_hash {
            if users::update_password(&state.db, "admin", &req.new_password_hash).await.is_ok() {
                return Json(LoginResponse { success: true });
            }
        }
    }

    Json(LoginResponse { success: false })
}

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let auth_header = req.headers().get("X-Admin-Token");

    if let Some(token) = auth_header {
        if let Ok(Some((_, stored_hash))) = users::get_user_by_username(&state.db, "admin").await {
            if token.to_str().unwrap_or("") == stored_hash {
                return next.run(req).await;
            }
        }
    }

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Body::from("Unauthorized"))
        .unwrap()
}

pub async fn serve_admin(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // Redirect root to /admin
    if path.is_empty() {
        return Redirect::permanent("/admin").into_response();
    }

    // Handle admin routes - serve index.html
    let path = if path.starts_with("admin") {
        // For SPA routing, serve index.html for all /admin/* routes
        let asset_path = path.strip_prefix("admin").unwrap_or("");
        let asset_path = asset_path.trim_start_matches('/');

        if asset_path.is_empty() || !asset_path.contains('.') {
            "index.html"
        } else {
            asset_path
        }
    } else {
        path
    };

    match <AdminAssets as RustEmbed>::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data.into_owned()))
                .unwrap()
                .into_response()
        }
        None => {
            // For SPA, fallback to index.html for non-asset routes
            if !path.contains('.') {
                if let Some(content) = <AdminAssets as RustEmbed>::get("index.html") {
                    return Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from(content.data.into_owned()))
                        .unwrap()
                        .into_response();
                }
            }

            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap()
                .into_response()
        }
    }
}
