use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use crate::{error::ApiError, lerobot, state::AppState};

#[derive(serde::Deserialize)]
struct LerobotScanRequest {
    path: String,
}

#[derive(serde::Deserialize)]
struct LerobotFramesRequest {
    path: String,
    episode: u32,
    #[serde(default = "default_frame_offset")]
    offset: usize,
    #[serde(default = "default_frame_limit")]
    limit: usize,
}

fn default_frame_offset() -> usize {
    0
}

fn default_frame_limit() -> usize {
    200
}

#[derive(serde::Deserialize)]
struct LerobotAttributionRequest {
    path: String,
    profile: Option<String>,
    episode: u32,
    #[serde(default = "default_frame_offset")]
    offset: usize,
    #[serde(default = "default_frame_limit")]
    limit: usize,
}

fn lerobot_api_error(e: String) -> ApiError {
    ApiError {
        status: StatusCode::UNPROCESSABLE_ENTITY,
        message: e,
    }
}

async fn lerobot_status() -> Json<lerobot::LerobotStatus> {
    Json(lerobot::check_status().await)
}

async fn lerobot_scan(
    Json(req): Json<LerobotScanRequest>,
) -> Result<Json<lerobot::DatasetInfo>, ApiError> {
    lerobot::scan_dataset(std::path::Path::new(&req.path))
        .await
        .map(Json)
        .map_err(lerobot_api_error)
}

async fn lerobot_frames(
    Json(req): Json<LerobotFramesRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (frames, total) = lerobot::read_frames(
        std::path::Path::new(&req.path),
        req.episode,
        req.offset,
        req.limit,
    )
    .await
    .map_err(lerobot_api_error)?;
    Ok(Json(
        serde_json::json!({ "frames": frames, "total": total }),
    ))
}

async fn lerobot_profiles(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let names = state.profiles.list().unwrap_or_default();
    let profiles: Vec<serde_json::Value> = names
        .iter()
        .filter_map(|n| {
            state
                .profiles
                .load(n)
                .ok()
                .map(|p| serde_json::json!({ "name": n, "robot": p.robot_name }))
        })
        .collect();
    Json(serde_json::json!({ "profiles": profiles }))
}

async fn lerobot_autodetect(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LerobotScanRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let info = lerobot::scan_dataset(std::path::Path::new(&req.path))
        .await
        .map_err(lerobot_api_error)?;
    let suggestion = state
        .profiles
        .autodetect(&info.columns)
        .map_err(|e| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: e.to_string(),
        })?;
    Ok(Json(serde_json::json!({
        "columns": info.columns,
        "suggestedProfile": suggestion.as_ref().map(|(n, _)| n),
        "score": suggestion.as_ref().map(|(_, s)| s),
    })))
}

async fn lerobot_attribution(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LerobotAttributionRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let info = lerobot::scan_dataset(std::path::Path::new(&req.path))
        .await
        .map_err(lerobot_api_error)?;
    let profile_name = match req.profile {
        Some(n) => n,
        None => state
            .profiles
            .autodetect(&info.columns)
            .map_err(|e| ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: e.to_string(),
            })?
            .map(|(n, _)| n)
            .ok_or(ApiError {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                message: "no matching robot profile; add one under profiles/".to_string(),
            })?,
    };
    let profile = state
        .profiles
        .load(&profile_name)
        .map_err(|e| lerobot_api_error(e.to_string()))?;
    let (frames, total) = lerobot::read_frames(
        std::path::Path::new(&req.path),
        req.episode,
        req.offset,
        req.limit,
    )
    .await
    .map_err(lerobot_api_error)?;
    let chains = lerobot::chains_from_frames(&frames, &info.tasks);
    let summaries: Vec<serde_json::Value> = chains
        .iter()
        .map(|c| {
            serde_json::json!({
                "timestampNanos": c.timestamp_nanos,
                "success": c.success(),
                "stepCount": c.steps.len(),
            })
        })
        .collect();
    Ok(Json(serde_json::json!({
        "chains": chains,
        "summaries": summaries,
        "total": total,
        "profile": profile_name,
        "tasks": info.tasks,
        "angleUnit": profile.angle_unit.as_str(),
    })))
}

// --- Live API (M15 B3) ---

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/status", get(lerobot_status))
        .route("/scan", post(lerobot_scan))
        .route("/frames", post(lerobot_frames))
        .route("/profiles", get(lerobot_profiles))
        .route("/autodetect", post(lerobot_autodetect))
        .route("/attribution", post(lerobot_attribution))
}
