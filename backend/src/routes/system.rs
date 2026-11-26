use std::sync::Arc;

use axum::{routing::get, Json, Router};

use crate::{coordinator, models, state::AppState};

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

async fn system_status() -> Json<models::SystemStatus> {
    let coordinator = coordinator::query_coordinator().await;
    if coordinator.connected {
        Json(models::SystemStatus {
            coordinator: "connected".to_string(),
            daemon: "healthy".to_string(),
            version: coordinator.version,
            running_dataflows: coordinator.running_dataflows,
            active_nodes: coordinator.active_nodes,
            error_count: 0,
        })
    } else {
        Json(models::SystemStatus {
            coordinator: "unavailable".to_string(),
            daemon: "unavailable".to_string(),
            version: String::new(),
            running_dataflows: 0,
            active_nodes: 0,
            error_count: 0,
        })
    }
}

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/system/status", get(system_status))
}
