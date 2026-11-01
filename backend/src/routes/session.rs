use std::sync::Arc;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};

use crate::{session, state::AppState};

async fn session_status(State(state): State<Arc<AppState>>) -> Json<session::SessionStatus> {
    Json(state.session.status().await)
}

async fn session_start(State(state): State<Arc<AppState>>) -> Json<session::SessionStatus> {
    Json(state.session.start().await)
}

async fn session_stop(State(state): State<Arc<AppState>>) -> Json<session::SessionStatus> {
    Json(state.session.stop().await)
}

async fn daemon_status(State(state): State<Arc<AppState>>) -> Json<session::LegacyDaemonStatus> {
    Json(session::legacy_daemon_status(&state.session.status().await))
}

async fn daemon_start(State(state): State<Arc<AppState>>) -> Json<session::LegacyDaemonStatus> {
    Json(session::legacy_daemon_status(&state.session.start().await))
}

async fn daemon_stop(State(state): State<Arc<AppState>>) -> Json<session::LegacyDaemonStatus> {
    Json(session::legacy_daemon_status(&state.session.stop().await))
}

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/session/status", get(session_status))
        .route("/api/session/start", post(session_start))
        .route("/api/session/stop", post(session_stop))
        .route("/api/daemon/status", get(daemon_status))
        .route("/api/daemon/start", post(daemon_start))
        .route("/api/daemon/stop", post(daemon_stop))
}
