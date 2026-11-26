use std::{path::PathBuf, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use crate::{coordinator, error::ApiError, models, state::AppState};

async fn runtime_status(State(state): State<Arc<AppState>>) -> Json<models::RuntimeState> {
    Json(state.runtime.status().await)
}

async fn runtime_logs(State(state): State<Arc<AppState>>) -> Json<Vec<models::LogEntry>> {
    Json(state.runtime.logs().await)
}

async fn runtime_start(State(state): State<Arc<AppState>>) -> Json<models::RuntimeState> {
    Json(state.runtime.start().await)
}

#[derive(serde::Deserialize)]
struct RuntimeStartPathRequest {
    path: String,
}

/// Start a dataflow by arbitrary YAML path (not necessarily one of the
/// discovered example dataflows).
async fn runtime_start_path(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RuntimeStartPathRequest>,
) -> Result<Json<models::RuntimeState>, ApiError> {
    let path = req.path.trim();
    if path.is_empty() {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            message: "missing 'path' field".to_string(),
        });
    }
    let source = std::path::Path::new(path);
    let stem = source
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| "custom-dataflow".to_string());
    let parent = source
        .parent()
        .and_then(|parent| parent.file_name())
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();
    // Parent dir + file stem keeps names unique across folders full of
    // dataflow.yml files.
    let id = if parent.is_empty() {
        stem
    } else {
        format!("{parent}-{stem}")
    };
    Ok(Json(
        state
            .runtime
            .start_dataflow(id, PathBuf::from(path), path.to_string())
            .await,
    ))
}

async fn runtime_stop(State(state): State<Arc<AppState>>) -> Json<models::RuntimeState> {
    Json(state.runtime.stop().await)
}

/// GET /api/runtime/nodes/:dataflow_id — per-node runtime status.
async fn runtime_nodes(
    Path(dataflow_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<Vec<models::NodeRuntimeStatus>> {
    // Try WebSocket first
    if state.ws_client.is_connected().await {
        match state.ws_client.node_statuses(&dataflow_id).await {
            Ok(statuses) => return Json(statuses),
            Err(e) => eprintln!("WS node_statuses failed: {e}"),
        }
    }

    // CLI fallback: query dora list for dataflow-level status
    let coord = coordinator::query_coordinator().await;
    let rt = state.runtime.status().await;

    let mut statuses = Vec::new();

    // If we have graph data, use it to produce per-node status
    if coord.connected {
        for df in &coord.dataflows {
            if df.id == dataflow_id || df.name.contains(&dataflow_id) {
                // We can't get per-node info from CLI, so produce a summary entry
                let status = if df.status == "running" {
                    "running".to_string()
                } else {
                    "exited".to_string()
                };
                // For CLI fallback, create one entry per dataflow (not per-node)
                statuses.push(models::NodeRuntimeStatus {
                    node_id: df.id.clone(),
                    status,
                    uptime_secs: None,
                    restart_count: 0,
                    cpu_usage: None,
                    memory_mb: None,
                    pending_messages: None,
                });
            }
        }
    }

    // If runtime has this dataflow running, include that info
    if rt.status == "running" && rt.dataflow_id.as_deref() == Some(&dataflow_id) {
        if statuses.is_empty() {
            statuses.push(models::NodeRuntimeStatus {
                node_id: dataflow_id.clone(),
                status: "running".to_string(),
                uptime_secs: None,
                restart_count: 0,
                cpu_usage: None,
                memory_mb: None,
                pending_messages: None,
            });
        }
    }

    if statuses.is_empty() {
        statuses.push(models::NodeRuntimeStatus {
            node_id: dataflow_id,
            status: "unknown".to_string(),
            uptime_secs: None,
            restart_count: 0,
            cpu_usage: None,
            memory_mb: None,
            pending_messages: None,
        });
    }

    Json(statuses)
}

/// POST /api/runtime/nodes/:dataflow_id/reload — hot reload a node.
async fn runtime_reload(
    Path(dataflow_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<models::ReloadRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !state.ws_client.is_connected().await {
        return Err(ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            message: "Coordinator WebSocket not connected. Hot reload requires the WebSocket API."
                .to_string(),
        });
    }

    state
        .ws_client
        .reload_node(&dataflow_id, &body.node_id)
        .await
        .map_err(|e| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: e,
        })?;

    Ok(Json(serde_json::json!({
        "ok": true,
        "nodeId": body.node_id,
        "message": "Reload signal sent. Node will restart with updated code."
    })))
}

// --- Recording API (M04) ---

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/runtime/status", get(runtime_status))
        .route("/api/runtime/logs", get(runtime_logs))
        .route("/api/runtime/start", post(runtime_start))
        .route("/api/runtime/start-path", post(runtime_start_path))
        .route("/api/runtime/stop", post(runtime_stop))
        .route("/api/runtime/nodes/:dataflow_id", get(runtime_nodes))
        .route(
            "/api/runtime/nodes/:dataflow_id/reload",
            post(runtime_reload),
        )
}
