use std::sync::Arc;

use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use crate::{coordinator, dora_env, error::ApiError, models, state::AppState};

async fn coordinator_status() -> Json<models::CoordinatorStatus> {
    Json(coordinator::query_coordinator().await)
}

// --- dora version manager (M17) ---

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DoraVersionsResponse {
    active: String,
    overridden_by_env: bool,
    items: Vec<dora_env::DoraVersionItem>,
}

async fn dora_versions() -> Json<DoraVersionsResponse> {
    Json(DoraVersionsResponse {
        active: dora_env::resolve_dora_bin(),
        overridden_by_env: dora_env::env_bin_overrides(),
        items: dora_env::detect_versions().await,
    })
}

#[derive(serde::Deserialize)]
struct DoraPathRequest {
    path: String,
}

fn dora_path_error(message: String) -> ApiError {
    ApiError {
        status: StatusCode::BAD_REQUEST,
        message,
    }
}

async fn dora_switch(
    Json(req): Json<DoraPathRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    dora_env::switch_dora_bin(req.path)
        .await
        .map_err(dora_path_error)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn dora_candidates_add(
    Json(req): Json<DoraPathRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    dora_env::add_candidate(req.path).map_err(dora_path_error)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn dora_candidates_delete(
    Json(req): Json<DoraPathRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    dora_env::delete_candidate(req.path).map_err(dora_path_error)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/coordinator/status", get(coordinator_status))
        .route("/api/dora/versions", get(dora_versions))
        .route("/api/dora/switch", post(dora_switch))
        .route("/api/dora/candidates/add", post(dora_candidates_add))
        .route("/api/dora/candidates/delete", post(dora_candidates_delete))
}
