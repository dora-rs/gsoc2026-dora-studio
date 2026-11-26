//! HTTP surface, split by domain.
//!
//! Every module owns the routes, handlers and request/response DTOs for one
//! feature area. [`api_router`] only composes them; [`root_router`] adds the
//! static asset mount, CORS and the shared state.

mod dataflows;
mod dora;
mod lerobot;
mod live;
mod models;
mod monitoring;
mod projects;
mod recording;
mod robot;
mod runtime;
mod schema;
mod session;
mod system;
mod types;

use std::{path::PathBuf, sync::Arc};

use axum::Router;
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::state::AppState;

/// Domain routers keyed by the path prefix they own.
///
/// Modules whose routes span more than one prefix (`/api/dataflows` plus
/// `/api/dataflow`, `/api/projects` plus `/api/palette`, and so on) register
/// absolute paths and are merged instead of nested.
pub(crate) fn api_router() -> Router<Arc<AppState>> {
    Router::new()
        .merge(system::router())
        .merge(models::router())
        .merge(types::router())
        .merge(dataflows::router())
        .merge(projects::router())
        .merge(runtime::router())
        .merge(session::router())
        .merge(dora::router())
        .merge(robot::router())
        .merge(monitoring::router())
        .nest("/api/schema", schema::router())
        .nest("/api/recording", recording::router())
        .nest("/api/lerobot", lerobot::router())
        .nest("/api/live", live::router())
}

/// Full router: static assets, the API surface, CORS and shared state.
pub(crate) fn root_router(state: Arc<AppState>) -> Router {
    let models_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../models");
    Router::new()
        .nest_service("/models", ServeDir::new(models_dir))
        .merge(api_router())
        .with_state(state)
        .layer(CorsLayer::permissive())
}
