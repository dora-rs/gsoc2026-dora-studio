use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use crate::{error::ApiError, metrics, models, monitoring, otel, state::AppState};

async fn metrics_nodes(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<metrics::NodeMetricSummary>> {
    Json(state.monitoring.metrics.nodes_summary().await)
}

#[derive(serde::Deserialize)]
struct NodeHistoryQuery {
    #[serde(default)]
    window: Option<u64>,
}

async fn metrics_node_history(
    State(state): State<Arc<AppState>>,
    Path(node_id): Path<String>,
    Query(q): Query<NodeHistoryQuery>,
) -> Result<Json<Vec<metrics::NodeMetricSample>>, ApiError> {
    match state
        .monitoring
        .metrics
        .node_history(&node_id, q.window)
        .await
    {
        Some(history) => Ok(Json(history)),
        None => Err(ApiError {
            status: StatusCode::NOT_FOUND,
            message: format!("Node '{}' not found in metrics", node_id),
        }),
    }
}

// --- OTel API (M08) ---

async fn otel_status(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(state.monitoring.otel.status().await)
}

#[derive(serde::Deserialize)]
struct OtelSpansQuery {
    node: Option<String>,
    #[serde(default = "default_otel_limit")]
    limit: usize,
}

fn default_otel_limit() -> usize {
    200
}

async fn otel_spans(
    State(state): State<Arc<AppState>>,
    Query(q): Query<OtelSpansQuery>,
) -> Json<Vec<otel::OtelSpan>> {
    Json(
        state
            .monitoring
            .otel
            .spans_for_node(q.node.as_deref(), q.limit)
            .await,
    )
}

async fn otel_trace(
    State(state): State<Arc<AppState>>,
    Path(trace_id): Path<String>,
) -> Result<Json<Vec<otel::SpanNode>>, ApiError> {
    match state.monitoring.otel.trace_tree(&trace_id).await {
        Some(tree) => Ok(Json(tree)),
        None => Err(ApiError {
            status: StatusCode::NOT_FOUND,
            message: format!("Trace '{}' not found", trace_id),
        }),
    }
}

// --- Monitoring control API (M11.5) ---

async fn monitoring_status(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(state.monitoring.status().await)
}

async fn monitoring_toggle(
    State(state): State<Arc<AppState>>,
    Json(req): Json<models::MonitoringToggleRequest>,
) -> Json<serde_json::Value> {
    if let Some(enabled) = req.node_metrics {
        state
            .monitoring
            .set_enabled(monitoring::MonitorTarget::NodeMetrics, enabled);
    }
    if let Some(enabled) = req.otel_spans {
        state
            .monitoring
            .set_enabled(monitoring::MonitorTarget::OtelSpans, enabled);
    }
    Json(state.monitoring.status().await)
}

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/metrics/nodes", get(metrics_nodes))
        .route("/api/metrics/nodes/:id/history", get(metrics_node_history))
        .route("/api/otel/status", get(otel_status))
        .route("/api/otel/spans", get(otel_spans))
        .route("/api/otel/trace/:trace_id", get(otel_trace))
        .route("/api/monitoring/status", get(monitoring_status))
        .route("/api/monitoring/toggle", post(monitoring_toggle))
}
