use std::{path::PathBuf, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use crate::{
    coordinator, dataflow_builder, dataflows, error::ApiError, models, project_scan,
    state::AppState, validate,
};

async fn dataflows(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<models::DataflowSummary>>, ApiError> {
    let mut dataflows = project_scan::list_all_dataflows().map_err(ApiError::from)?;
    let rt = state.runtime.status().await;
    let coord = coordinator::query_coordinator().await;

    for df in &mut dataflows {
        if rt.status == "running" && rt.dataflow_id.as_deref() == Some(&df.id) {
            df.status = "running".to_string();
        } else if coord.connected {
            for cdf in &coord.dataflows {
                if cdf.status == "running"
                    && (df.name.contains(&cdf.name) || cdf.name.contains(&df.name))
                {
                    df.status = "running".to_string();
                }
            }
        }
    }
    Ok(Json(dataflows))
}

async fn dataflow_definition(
    Path(id): Path<String>,
) -> Result<Json<models::DataflowDefinition>, ApiError> {
    dataflows::load_definition(&id)
        .map(Json)
        .map_err(ApiError::from)
}

async fn dataflow_nodes(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<models::NodeMetrics>>, ApiError> {
    let mut metrics = dataflows::nodes(&id).map_err(ApiError::from)?;
    let state = state.runtime.status().await;
    if state.status == "running" && state.dataflow_id.as_deref() == Some(&id) {
        for node in &mut metrics {
            node.status = "running".to_string();
        }
    }
    Ok(Json(metrics))
}

async fn dataflow_logs(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<Vec<models::LogEntry>> {
    let rt = state.runtime.status().await;
    if rt.status == "running" && rt.dataflow_id.as_deref() == Some(&id) {
        Json(state.runtime.logs().await)
    } else {
        Json(Vec::new())
    }
}

async fn dataflow_graph(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<models::DataflowGraph>, ApiError> {
    let mut graph = dataflows::graph(&id).map_err(ApiError::from)?;
    let state = state.runtime.status().await;
    if state.status == "running" && state.dataflow_id.as_deref() == Some(&id) {
        for node in &mut graph.nodes {
            node.status = "running".to_string();
        }
    }
    Ok(Json(graph))
}

async fn dataflow_start(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<models::RuntimeState>, ApiError> {
    let dataflow = dataflows::resolve_dataflow(&id).map_err(ApiError::from)?;
    Ok(Json(
        state
            .runtime
            .start_dataflow(dataflow.id, dataflow.path, dataflow.relative_path)
            .await,
    ))
}

async fn dataflow_stop(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<models::RuntimeState>, ApiError> {
    dataflows::resolve_dataflow(&id).map_err(ApiError::from)?;
    Ok(Json(state.runtime.stop().await))
}

async fn dataflow_restart(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<models::RuntimeState>, ApiError> {
    let dataflow = dataflows::resolve_dataflow(&id).map_err(ApiError::from)?;
    state.runtime.stop().await;
    Ok(Json(
        state
            .runtime
            .start_dataflow(dataflow.id, dataflow.path, dataflow.relative_path)
            .await,
    ))
}

async fn dataflow_build(
    Json(graph): Json<dataflow_builder::DataflowGraph>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut builder = dataflow_builder::DataflowBuilder::new();
    for node in graph.nodes {
        builder.add_node(node).map_err(|e| ApiError {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: e.to_string(),
        })?;
    }
    for edge in graph.edges {
        builder.connect(edge).map_err(|e| ApiError {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: e.to_string(),
        })?;
    }
    let yaml = builder.to_yaml();
    Ok(Json(serde_json::json!({
        "yaml": yaml,
        "node_count": builder.graph().nodes.len(),
        "edge_count": builder.graph().edges.len(),
    })))
}

async fn dataflow_validate(
    Json(graph): Json<dataflow_builder::DataflowGraph>,
) -> Json<serde_json::Value> {
    let mut builder = dataflow_builder::DataflowBuilder::new();
    let mut errors = Vec::new();
    for node in graph.nodes {
        if let Err(e) = builder.add_node(node) {
            errors.push(e.to_string());
        }
    }
    for edge in graph.edges {
        if let Err(e) = builder.connect(edge) {
            errors.push(e.to_string());
        }
    }
    match builder.validate() {
        Ok(()) if errors.is_empty() => Json(serde_json::json!({ "valid": true, "errors": [] })),
        Ok(()) => Json(serde_json::json!({ "valid": false, "errors": errors })),
        Err(mut validation_errors) => {
            for e in validation_errors {
                errors.push(e.to_string());
            }
            Json(serde_json::json!({ "valid": false, "errors": errors }))
        }
    }
}

async fn dataflow_parse(
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let yaml = body.get("yaml").and_then(|v| v.as_str()).ok_or(ApiError {
        status: StatusCode::BAD_REQUEST,
        message: "missing 'yaml' field".to_string(),
    })?;
    let builder = dataflow_builder::DataflowBuilder::from_yaml(yaml).map_err(|e| ApiError {
        status: StatusCode::UNPROCESSABLE_ENTITY,
        message: e.to_string(),
    })?;
    Ok(Json(serde_json::json!({
        "graph": builder.graph(),
    })))
}

// --- Save API (M18 Task 3.6) ---

/// Copy the current dataflow file into ~/.config/dora-studio/backups
/// before a write-back, so a broken save can be restored manually.
fn backup_file(path: &std::path::Path) -> Result<PathBuf, String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let backups = std::path::Path::new(&home).join(".config/dora-studio/backups");
    std::fs::create_dir_all(&backups).map_err(|error| format!("backup dir: {error}"))?;
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "dataflow.yml".to_string());
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    let backup = backups.join(format!("{name}.{ts}.bak"));
    std::fs::copy(path, &backup).map_err(|error| format!("backup: {error}"))?;
    Ok(backup)
}

/// Validate content against `dora validate`, then write it to `target`.
/// Validation errors return 422 with the full SaveResponse JSON body;
/// warnings (and the "validation skipped" notice on dora 0.x) are
/// non-blocking and returned with the successful save.
async fn finish_save(
    target: &std::path::Path,
    content: &str,
    display_path: &str,
) -> Result<Json<models::SaveResponse>, ApiError> {
    // Stage the temp file in the target's directory so the final rename is
    // a same-filesystem atomic replace (spec §4 step 6: a crash mid-write
    // must not corrupt the dataflow file). Fall back to the system temp dir
    // when the target has no parent directory.
    let tmp_dir = target
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| parent.to_path_buf())
        .unwrap_or_else(std::env::temp_dir);
    let tmp = tmp_dir.join(format!("dora-studio-save-{}.yml", uuid::Uuid::new_v4()));
    std::fs::write(&tmp, content).map_err(|error| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("failed to write temp file: {error}"),
    })?;
    let outcome = match validate::validate_yaml(&tmp).await {
        Ok(outcome) => outcome,
        Err(skipped) => validate::ValidateOutcome {
            errors: Vec::new(),
            warnings: vec![models::SaveIssue {
                node_id: None,
                port_id: None,
                message: skipped,
            }],
        },
    };
    if !outcome.errors.is_empty() {
        let _ = std::fs::remove_file(&tmp);
        return Err(ApiError {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: serde_json::to_string(&models::SaveResponse {
                ok: false,
                path: display_path.to_string(),
                warnings: outcome.warnings,
                errors: outcome.errors,
            })
            .unwrap_or_else(|_| "validate failed".to_string()),
        });
    }
    // The temp file already holds `content`; rename it into place instead
    // of re-writing the target (atomic on the same filesystem).
    std::fs::rename(&tmp, target).map_err(|error| {
        let _ = std::fs::remove_file(&tmp);
        ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("failed to write {display_path}: {error}"),
        }
    })?;
    Ok(Json(models::SaveResponse {
        ok: true,
        path: display_path.to_string(),
        warnings: outcome.warnings,
        errors: Vec::new(),
    }))
}

/// Write-back: patch an existing discovered dataflow in place.
async fn dataflow_save(
    Path(id): Path<String>,
    Json(req): Json<models::SaveRequest>,
) -> Result<Json<models::SaveResponse>, ApiError> {
    let file = dataflows::resolve_dataflow(&id).map_err(ApiError::from)?;
    let original = std::fs::read_to_string(&file.path).map_err(|error| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("Failed to read {}: {error}", file.relative_path),
    })?;
    let patched =
        dataflow_builder::patch_yaml(&original, &req.graph).map_err(|error| ApiError {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: format!("Failed to build YAML: {error}"),
        })?;
    backup_file(&file.path).map_err(|error| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: error,
    })?;
    finish_save(&file.path, &patched, &file.relative_path).await
}

/// Save-as: generate a fresh dora 1.0 dataflow YAML at an arbitrary path.
async fn dataflow_save_as(
    Json(req): Json<models::SaveAsRequest>,
) -> Result<Json<models::SaveResponse>, ApiError> {
    let target = PathBuf::from(req.target_path.trim());
    if target.as_os_str().is_empty() {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            message: "missing 'targetPath' field".to_string(),
        });
    }
    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|error| ApiError {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                message: format!("cannot create target directory: {error}"),
            })?;
        }
    }
    let mut builder = dataflow_builder::DataflowBuilder::new();
    builder.type_rules = req.graph.type_rules.clone();
    for node in req.graph.nodes {
        builder.add_node(node).map_err(|error| ApiError {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: error.to_string(),
        })?;
    }
    for edge in req.graph.edges {
        builder.connect(edge).map_err(|error| ApiError {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: error.to_string(),
        })?;
    }
    let yaml = builder.to_yaml();
    // Back up an existing target before overwriting it, matching the
    // write-back precedent.
    if target.exists() {
        backup_file(&target).map_err(|error| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: error,
        })?;
    }
    let display = target.to_string_lossy().to_string();
    finish_save(&target, &yaml, &display).await
}

async fn dataflow_run(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<models::RuntimeState>, ApiError> {
    let yaml = body.get("yaml").and_then(|v| v.as_str()).ok_or(ApiError {
        status: StatusCode::BAD_REQUEST,
        message: "missing 'yaml' field".to_string(),
    })?;
    let name = body
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("studio-dataflow");

    // Stop any existing run first
    if state.runtime.status().await.status == "running" {
        state.runtime.stop().await;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    let rt_state = state.runtime.run_yaml(yaml, name).await;
    Ok(Json(rt_state))
}

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/dataflows", get(dataflows))
        .route("/api/dataflows/save-as", post(dataflow_save_as))
        .route("/api/dataflows/:id/save", post(dataflow_save))
        .route("/api/dataflows/:id/definition", get(dataflow_definition))
        .route("/api/dataflows/:id/nodes", get(dataflow_nodes))
        .route("/api/dataflows/:id/logs", get(dataflow_logs))
        .route("/api/dataflows/:id/graph", get(dataflow_graph))
        .route("/api/dataflows/:id/start", post(dataflow_start))
        .route("/api/dataflows/:id/stop", post(dataflow_stop))
        .route("/api/dataflows/:id/restart", post(dataflow_restart))
        .route("/api/dataflow/build", post(dataflow_build))
        .route("/api/dataflow/validate", post(dataflow_validate))
        .route("/api/dataflow/parse", post(dataflow_parse))
        .route("/api/dataflow/run", post(dataflow_run))
}
