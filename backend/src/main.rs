mod attribution;
mod compat_engine;
mod coordinator;
mod coordinator_ws;
mod dataflow_builder;
mod dataflows;
mod dora_env;
mod drec;
mod error;
mod external;
mod lerobot;
mod live;
mod metrics;
mod model_catalog;
mod models;
mod monitoring;
mod otel;
mod otlp;
mod otlp_grpc;
mod profile;
mod project_scan;
mod protocol;
mod recording;
mod routes;
mod runtime;
mod schema_registry;
mod session;
mod state;
mod urn_catalog;
mod validate;

use crate::state::AppState;

#[tokio::main]
async fn main() {
    // 1. Shared application state & background services.
    let state = AppState::init().await;

    // 2. Root router: static assets, API surface, CORS.
    let app = routes::root_router(state.clone());

    // 3. Bind the HTTP listener.
    let bind_addr =
        std::env::var("DORA_STUDIO_BACKEND_ADDR").unwrap_or_else(|_| "127.0.0.1:3001".to_string());
    let addr = bind_addr.parse().expect("valid bind address");
    println!("dora-studio backend listening on http://{addr}");

    // 4. Passive OTLP receivers on 4318 (HTTP) and 4317 (gRPC). Neither is
    //    fatal if the port is taken — the main app keeps serving.
    let otlp_task = otlp::spawn(state.monitoring.otel.clone());
    let grpc_task = otlp_grpc::spawn(state.monitoring.otel.clone());

    // 5. Serve until Ctrl-C.
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server failed");
    otlp_task.abort();
    grpc_task.abort();
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
