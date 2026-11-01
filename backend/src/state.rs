use std::path::PathBuf;

use crate::{
    coordinator_ws, drec, live, metrics, monitoring, otel, profile, recording, runtime,
    schema_registry, session, urn_catalog,
};

/// Shared handles backing every API handler.
pub(crate) struct AppState {
    pub(crate) runtime: runtime::RuntimeHandle,
    pub(crate) session: session::SessionHandle,
    pub(crate) schemas: schema_registry::SchemaRegistry,
    pub(crate) ws_client: coordinator_ws::CoordinatorWsClient,
    pub(crate) recordings: drec::service::RecordingManager,
    pub(crate) recording: recording::RecordingHandle,
    pub(crate) monitoring: monitoring::MonitoringController,
    pub(crate) profiles: profile::ProfileManager,
    pub(crate) live: live::LiveFeed,
    pub(crate) catalog: urn_catalog::Catalog,
}

impl AppState {
    /// Boots every shared service exactly once, before the listeners bind.
    pub(crate) async fn init() -> std::sync::Arc<Self> {
        let ws_client = coordinator_ws::CoordinatorWsClient::new();

        // Best-effort connect to coordinator WebSocket (non-blocking)
        let ws_connect_handle = {
            let ws = ws_client.clone();
            tokio::spawn(async move {
                if let Err(e) = ws.connect().await {
                    eprintln!("coordinator WebSocket unavailable (CLI fallback active): {e}");
                } else {
                    eprintln!("coordinator WebSocket connected");
                }
            })
        };

        // Monitoring collectors start STOPPED (M11.5 D1): diagnostics are opt-in,
        // enabled through /api/monitoring/toggle. No polling at boot.
        // D2: node metrics poll the coordinator WebSocket first, falling back to
        // `dora node list` per attempt when the WS is unavailable.
        let metrics_collector = metrics::MetricsCollector::new_with_ws(
            std::time::Duration::from_secs(2),
            ws_client.clone(),
        );

        // OTel trace backend (Jaeger-compatible API). Default: local Jaeger.
        let otel_endpoint = std::env::var("DORA_OTEL_QUERY_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:16686".to_string());
        let otel_collector = otel::OtelCollector::new(otel_endpoint);

        let state = std::sync::Arc::new(Self {
            runtime: runtime::RuntimeManager::new(),
            session: session::DoraSessionManager::new(),
            schemas: schema_registry::SchemaRegistry::new(),
            ws_client,
            recordings: drec::service::RecordingManager::new(),
            recording: recording::RecordingController::new(),
            monitoring: monitoring::MonitoringController::new(metrics_collector, otel_collector),
            profiles: profile::ProfileManager::new(
                &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../profiles"),
            ),
            live: live::LiveFeed::new(),
            catalog: urn_catalog::Catalog::new(),
        });

        // Wait for WS connect attempt to settle (up to 3s) before starting server
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), ws_connect_handle).await;

        state
    }
}
