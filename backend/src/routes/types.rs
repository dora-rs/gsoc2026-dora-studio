use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};

use crate::{error::ApiError, state::AppState, urn_catalog};

async fn types_catalog(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "types": state.catalog.entries() }))
}

async fn types_get(
    State(state): State<Arc<AppState>>,
    Path(urn): Path<String>,
) -> Result<Json<urn_catalog::TypeDef>, ApiError> {
    state.catalog.resolve(&urn).map(Json).ok_or(ApiError {
        status: StatusCode::NOT_FOUND,
        message: format!("Unknown type URN: {urn}"),
    })
}

pub(crate) fn router() -> Router<Arc<AppState>> {
    // Wildcard capture so multi-segment URNs (e.g. std/media/v1/Image)
    // resolve as a single parameter; axum 0.6 `:param` only matches one
    // segment and would reject URNs containing `/`.
    Router::new()
        .route("/api/types/catalog", get(types_catalog))
        .route("/api/types/*urn", get(types_get))
}
