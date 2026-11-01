use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use crate::{error::ApiError, live, state::AppState};

#[derive(serde::Deserialize)]
struct LiveRecentQuery {
    stream: Option<String>,
    since_ts: Option<u64>,
    limit: Option<usize>,
}

async fn live_ingest(
    State(state): State<Arc<AppState>>,
    Json(req): Json<live::IngestRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state
        .live
        .ingest(live::LiveFrame {
            node_id: req.node_id,
            output_id: req.output_id,
            timestamp: req.timestamp,
            payload: req.payload,
        })
        .map_err(|e| ApiError {
            status: StatusCode::BAD_REQUEST,
            message: e.0,
        })?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn live_recent(
    State(state): State<Arc<AppState>>,
    Query(q): Query<LiveRecentQuery>,
) -> Json<live::RecentResponse> {
    let limit = q
        .limit
        .unwrap_or(live::DEFAULT_FRAME_LIMIT)
        .min(live::MAX_FRAME_LIMIT);
    let frames = state.live.recent(q.stream.as_deref(), q.since_ts, limit);
    Json(live::RecentResponse { frames })
}

#[derive(serde::Deserialize)]
struct LiveCommandRequest {
    kind: String,
    planner: Option<String>,
    target: Option<Vec<f64>>,
    action: Option<String>,
    object: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
struct LiveCommandQueueQuery {
    since_seq: Option<u64>,
}

async fn live_command(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LiveCommandRequest>,
) -> Result<Json<live::LiveCommand>, ApiError> {
    state
        .live
        .push_command(&req.kind, req.planner, req.target, req.action, req.object)
        .map(Json)
        .map_err(|e| ApiError {
            status: StatusCode::BAD_REQUEST,
            message: e.0,
        })
}

async fn live_command_queue(
    State(state): State<Arc<AppState>>,
    Query(q): Query<LiveCommandQueueQuery>,
) -> Json<serde_json::Value> {
    let commands = state.live.take_commands(q.since_seq.unwrap_or(0));
    let next_seq = state.live.next_command_seq();
    Json(serde_json::json!({ "commands": commands, "next_seq": next_seq }))
}

// --- Metrics API (M07) ---

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/ingest", post(live_ingest))
        .route("/recent", get(live_recent))
        .route("/command", post(live_command))
        .route("/command", get(live_command_queue))
}
