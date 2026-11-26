use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use crate::{
    compat_engine, error::ApiError, models, schema_registry, state::AppState, urn_catalog,
};

async fn schema_check(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<models::SchemaCheckResponse> {
    // New shape: {source_urn, sink_urn, type_rules?}
    let source_urn = body
        .get("source_urn")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    let sink_urn = body
        .get("sink_urn")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    if source_urn.is_some() || sink_urn.is_some() {
        let user_rules: Vec<(String, String)> = body
            .get("type_rules")
            .and_then(|v| v.as_array())
            .map(|rules| {
                rules
                    .iter()
                    .filter_map(|rule| {
                        Some((
                            rule.get("from")?.as_str()?.to_string(),
                            rule.get("to")?.as_str()?.to_string(),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let result = compat_engine::check(source_urn, sink_urn, &user_rules);
        // enrich struct-mismatch reason via catalog (informational only, appended on mismatches)
        let detail = match (&source_urn, &sink_urn) {
            (Some(from), Some(to)) if !result.compatible => {
                enrich_struct_detail(from, to, &state.catalog)
                    .map(|detail| format!("{} ({detail})", result.reason))
                    .unwrap_or_else(|| result.reason.clone())
            }
            _ => result.reason.clone(),
        };
        return Json(models::SchemaCheckResponse {
            compatible: result.compatible,
            level: result.level,
            detail,
            urn: sink_urn.map(str::to_string),
            rule: result
                .rule
                .map(|(from, to)| models::TypeRuleDef { from, to }),
            suggestion: result.suggestion,
        });
    }

    // Old shape: {source_operator, source_port, sink_operator, sink_port}
    let request = schema_registry::CheckRequest {
        source_operator: body
            .get("source_operator")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        source_port: body
            .get("source_port")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        sink_operator: body
            .get("sink_operator")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        sink_port: body
            .get("sink_port")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
    };
    let response = state.schemas.check(&request);
    Json(models::SchemaCheckResponse {
        compatible: response.compatible,
        level: response.level,
        detail: response.detail,
        urn: None,
        rule: None,
        suggestion: None,
    })
}

fn enrich_struct_detail(from: &str, to: &str, catalog: &urn_catalog::Catalog) -> Option<String> {
    let from_def = catalog.resolve(from)?;
    let to_def = catalog.resolve(to)?;
    if from_def.fields.is_empty() || to_def.fields.is_empty() {
        return None;
    }
    let expected: Vec<compat_engine::TypeField> = to_def
        .fields
        .iter()
        .map(|field| compat_engine::TypeField {
            name: field.name.clone(),
            field_type: field.field_type.clone(),
        })
        .collect();
    let actual: Vec<compat_engine::TypeField> = from_def
        .fields
        .iter()
        .map(|field| compat_engine::TypeField {
            name: field.name.clone(),
            field_type: field.field_type.clone(),
        })
        .collect();
    match compat_engine::schema_compatible(&expected, &actual) {
        Ok(()) => None,
        Err(error) => Some(error.to_string()),
    }
}

async fn schema_operator(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<schema_registry::OperatorSchemas>, ApiError> {
    state
        .schemas
        .operator_schemas(&name)
        .map(Json)
        .ok_or(ApiError {
            status: StatusCode::NOT_FOUND,
            message: format!("Operator '{}' not found in schema registry", name),
        })
}

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/check", post(schema_check))
        .route("/operator/:name", get(schema_operator))
}
