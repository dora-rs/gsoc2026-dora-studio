use std::sync::Arc;

use axum::{routing::get, Json, Router};

use crate::{coordinator, external, models, state::AppState};

async fn dviz_status() -> Json<models::DvizStatus> {
    Json(external::query_dviz())
}

async fn dviz_topics() -> Json<models::DvizTopicsResponse> {
    Json(external::query_dviz_topics())
}

async fn dviz_displays() -> Json<models::DvizDisplaysResponse> {
    Json(external::query_dviz_displays())
}

async fn dviz_snapshot() -> Json<models::DvizSnapshotResponse> {
    Json(external::query_dviz_snapshot())
}

async fn robot_profile() -> Json<models::RobotProfileResponse> {
    Json(external::query_robot_profile())
}

async fn moveit_status() -> Json<models::MoveitStatus> {
    let mut status = external::query_moveit();

    // Cross-reference with running dataflows: check if moveit nodes are active
    let coordinator = coordinator::query_coordinator().await;
    for df in &coordinator.dataflows {
        if df.name.contains("moveit") || df.name.contains("motion") {
            status.running = df.status == "running";
            if status.running {
                status.message = format!(
                    "dora-moveit2 dataflow '{}' is running with {} nodes.",
                    df.name, df.nodes
                );
            }
        }
    }

    Json(status)
}

async fn moveit_snapshot() -> Json<models::MoveitSnapshotResponse> {
    Json(external::query_moveit_snapshot())
}

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/dviz/status", get(dviz_status))
        .route("/api/dviz/topics", get(dviz_topics))
        .route("/api/dviz/displays", get(dviz_displays))
        .route("/api/dviz/snapshot", get(dviz_snapshot))
        .route("/api/robot/profile", get(robot_profile))
        .route("/api/moveit/status", get(moveit_status))
        .route("/api/moveit/snapshot", get(moveit_snapshot))
}
