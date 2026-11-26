use std::sync::Arc;

use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use crate::{dora_env, error::ApiError, models, project_scan, state::AppState};

/// GET /api/projects/list — builtin + configured project dirs.
async fn projects_list() -> Result<Json<models::ProjectListResponse>, ApiError> {
    project_scan::list_projects()
        .map(|projects| Json(models::ProjectListResponse { projects }))
        .map_err(ApiError::from)
}

/// POST /api/projects/add — persist a user project dir (canonical, deduped).
async fn projects_add(
    Json(req): Json<models::AddProjectRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    dora_env::add_project_dir(&req.path).map_err(|error| ApiError {
        status: StatusCode::UNPROCESSABLE_ENTITY,
        message: error,
    })?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// POST /api/projects/delete — remove a user project dir from settings.
async fn projects_delete(Json(req): Json<models::AddProjectRequest>) -> Json<serde_json::Value> {
    let _ = dora_env::remove_project_dir(&req.path);
    Json(serde_json::json!({ "ok": true }))
}

/// POST /api/projects/nodes — store a manually defined node for the palette.
async fn projects_nodes(
    Json(req): Json<models::ManualNodeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let node = dora_env::ManualNode {
        id: req.id,
        path: req.path,
        description: req.description,
        inputs: req
            .inputs
            .into_iter()
            .map(|port| dora_env::ManualPort {
                name: port.name,
                urn: port.urn,
            })
            .collect(),
        outputs: req
            .outputs
            .into_iter()
            .map(|port| dora_env::ManualPort {
                name: port.name,
                urn: port.urn,
            })
            .collect(),
    };
    dora_env::add_manual_node(node).map_err(|error| ApiError {
        status: StatusCode::UNPROCESSABLE_ENTITY,
        message: error,
    })?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// GET /api/palette — aggregated cross-project node palette (scan of
/// builtin examples + configured project dirs, merged with manual nodes).
async fn palette() -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(
        serde_json::json!({ "entries": project_scan::palette() }),
    ))
}

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/projects/list", get(projects_list))
        .route("/api/projects/add", post(projects_add))
        .route("/api/projects/delete", post(projects_delete))
        .route("/api/projects/nodes", post(projects_nodes))
        .route("/api/palette", get(palette))
}
