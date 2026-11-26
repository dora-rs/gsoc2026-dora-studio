use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use crate::{attribution, drec, error::ApiError, models, recording, state::AppState};

#[derive(serde::Deserialize)]
struct RecordingCaptureRequest {
    #[serde(rename = "dataflowPath")]
    dataflow_path: String,
}

async fn recording_capture(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RecordingCaptureRequest>,
) -> Json<recording::RecordingStatus> {
    let Some(dataflow_name) = state.runtime.active_dataflow_name().await else {
        return Json(recording::RecordingStatus {
            status: "unavailable".to_string(),
            output_path: None,
            dataflow_path: Some(req.dataflow_path),
            started_at_millis: None,
            frame_count: None,
            message: "Start the selected dataflow from Studio before recording.".to_string(),
        });
    };
    Json(
        state
            .recording
            .capture(req.dataflow_path, dataflow_name)
            .await,
    )
}

async fn recording_stop(State(state): State<Arc<AppState>>) -> Json<recording::RecordingStatus> {
    Json(state.recording.stop().await)
}

async fn recording_list(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<recording::RecordingEntry>> {
    Json(state.recording.list().await)
}

async fn recording_open(
    State(state): State<Arc<AppState>>,
    Json(body): Json<models::OpenRecordingRequest>,
) -> Result<Json<models::RecordingOpened>, ApiError> {
    let path = std::path::PathBuf::from(&body.path);
    let handle = state.recordings.open(&path).await.map_err(|e| ApiError {
        status: StatusCode::BAD_REQUEST,
        message: e,
    })?;

    Ok(Json(models::RecordingOpened {
        id: handle.id.to_string(),
        dataflow_id: handle.header.dataflow_id.to_string(),
        version: handle.header.version,
        start_nanos: handle.header.start_nanos,
        message_count: handle.message_count(),
        duration_nanos: handle.duration_nanos(),
        stream_count: handle.streams().len(),
    }))
}

async fn recording_streams(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let uid: uuid::Uuid = id.parse().map_err(|_| ApiError {
        status: StatusCode::BAD_REQUEST,
        message: format!("invalid recording id: {id}"),
    })?;
    let handle = state.recordings.get(&uid).await.ok_or(ApiError {
        status: StatusCode::NOT_FOUND,
        message: "recording not found".to_string(),
    })?;

    let streams: Vec<serde_json::Value> = handle
        .streams()
        .iter()
        .map(|s| {
            serde_json::json!({
                "nodeId": s.node_id,
                "outputId": s.output_id,
                "entryCount": s.entry_count,
                "timeRange": [s.time_range.0, s.time_range.1],
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "streams": streams })))
}

async fn recording_seek(
    Path(id): Path<String>,
    Query(q): Query<models::SeekQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let uid: uuid::Uuid = id.parse().map_err(|_| ApiError {
        status: StatusCode::BAD_REQUEST,
        message: format!("invalid recording id: {id}"),
    })?;
    let handle = state.recordings.get(&uid).await.ok_or(ApiError {
        status: StatusCode::NOT_FOUND,
        message: "recording not found".to_string(),
    })?;

    match handle.seek(q.timestamp) {
        Some(entry) => Ok(Json(serde_json::json!({
            "byteOffset": entry.byte_offset,
            "timestampNanos": entry.timestamp_absolute_nanos,
            "nodeId": entry.node_id,
            "outputId": entry.output_id,
        }))),
        None => Err(ApiError {
            status: StatusCode::NOT_FOUND,
            message: "no entry at or before timestamp".to_string(),
        }),
    }
}

async fn recording_entries(
    Path(id): Path<String>,
    Query(q): Query<models::EntriesQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let uid: uuid::Uuid = id.parse().map_err(|_| ApiError {
        status: StatusCode::BAD_REQUEST,
        message: format!("invalid recording id: {id}"),
    })?;
    let handle = state.recordings.get(&uid).await.ok_or(ApiError {
        status: StatusCode::NOT_FOUND,
        message: "recording not found".to_string(),
    })?;

    let entries = if let Some(timestamp) = q.timestamp {
        let window = q.window_nanos.unwrap_or(50_000_000);
        let start = timestamp.saturating_sub(window);
        let end = timestamp.saturating_add(window);
        let mut matching = handle.entries_in_range(start, end);
        matching.truncate(q.limit);
        matching
    } else if let (Some(node), Some(output)) = (&q.node, &q.output) {
        handle.stream_entries(node, output, q.offset, q.limit)
    } else {
        let all = handle.index.all_entries();
        let start = q.offset.min(all.len());
        let end = (start + q.limit).min(all.len());
        all[start..end].iter().collect()
    };

    let items: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            let mut obj = serde_json::json!({
                "byteOffset": e.byte_offset,
                "timestampNanos": e.timestamp_absolute_nanos,
                "nodeId": e.node_id,
                "outputId": e.output_id,
            });
            if q.include_data {
                match handle.read_event_bytes(e.byte_offset) {
                    Ok(bytes) => {
                        obj["eventBytes"] = serde_json::json!(bytes);
                    }
                    Err(err) => {
                        eprintln!("read_event_bytes failed at offset {}: {err}", e.byte_offset);
                    }
                }
            }
            obj
        })
        .collect();

    Ok(Json(serde_json::json!({
        "entries": items,
        "offset": q.offset,
        "limit": q.limit,
        "total": handle.message_count(),
    })))
}

async fn recording_close(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let uid: uuid::Uuid = match id.parse() {
        Ok(u) => u,
        Err(_) => return Json(serde_json::json!({ "ok": false, "error": "invalid id" })),
    };
    state.recordings.close(&uid).await;
    Json(serde_json::json!({ "ok": true }))
}

// --- Attribution API (M09) ---

async fn resolve_recording(
    state: &AppState,
    id: &str,
) -> Result<Arc<drec::service::RecordingHandle>, ApiError> {
    let uid: uuid::Uuid = id.parse().map_err(|_| ApiError {
        status: StatusCode::BAD_REQUEST,
        message: format!("invalid recording id: {id}"),
    })?;
    state.recordings.get(&uid).await.ok_or(ApiError {
        status: StatusCode::NOT_FOUND,
        message: "recording not found".to_string(),
    })
}

async fn extract_attribution(
    handle: Arc<drec::service::RecordingHandle>,
) -> Result<attribution::AttributionExtractor, ApiError> {
    tokio::task::spawn_blocking(move || attribution::AttributionExtractor::from_recording(&handle))
        .await
        .map_err(|e| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("attribution extraction panicked: {e}"),
        })?
        .map_err(|e| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: e,
        })
}

async fn recording_attribution(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<attribution::AttributionSummary>, ApiError> {
    let handle = resolve_recording(&state, &id).await?;
    let extractor = extract_attribution(handle).await?;
    Ok(Json(attribution::AttributionSummary::from(&extractor)))
}

#[derive(serde::Deserialize)]
struct AttributionChainQuery {
    timestamp: u64,
}

async fn recording_attribution_chain(
    Path(id): Path<String>,
    Query(q): Query<AttributionChainQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<attribution::AttributionChain>, ApiError> {
    let handle = resolve_recording(&state, &id).await?;
    let extractor = extract_attribution(handle).await?;
    extractor
        .chain_at(q.timestamp)
        .cloned()
        .map(Json)
        .ok_or(ApiError {
            status: StatusCode::NOT_FOUND,
            message: format!("no attribution chain at timestamp {}", q.timestamp),
        })
}

// --- LeRobot API (M10) ---

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/capture", post(recording_capture))
        .route("/stop", post(recording_stop))
        .route("/list", get(recording_list))
        .route("/open", post(recording_open))
        .route("/:id/streams", get(recording_streams))
        .route("/:id/seek", get(recording_seek))
        .route("/:id/entries", get(recording_entries))
        .route("/:id/close", post(recording_close))
        .route("/:id/attribution", get(recording_attribution))
        .route("/:id/attribution/chain", get(recording_attribution_chain))
}
