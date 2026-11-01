use std::{path::PathBuf, sync::Arc};

use axum::{routing::get, Json, Router};

use crate::{model_catalog, models, state::AppState};

/// M13 D6: locally available robot models (URDF directories under
/// models/) for the tool panel's model selector.
async fn available_models() -> Json<models::AvailableModelsResponse> {
    let models_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../models");
    Json(models::AvailableModelsResponse {
        models: model_catalog::list_available_models(&models_dir)
            .into_iter()
            .map(|m| models::AvailableModel {
                id: m.id,
                urdf_path: m.urdf_path,
                mesh_base_path: m.mesh_base_path,
            })
            .collect(),
    })
}

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/models", get(available_models))
}
