mod error;
mod html_template;
mod initializer;

use anyhow::{Context, Result};
use axum::{
    extract::{MatchedPath, Request, State},
    routing::{get, post},
    Router,
};
use initializer::{initialize, AppState};
use std::time::Duration;
use tokio::signal;
use tower_http::services::ServeDir;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::{debug_span, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Initializing...");
    let app_state = initialize().await?;
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|req: &Request| {
            let method = req.method();
            let uri = req.uri();
            let matched_path = req
                .extensions()
                .get::<MatchedPath>()
                .map(|matched_path| matched_path.as_str());

            debug_span!("request", %method, %uri, matched_path)
        })
        .on_failure(());
    let timeout_layer = TimeoutLayer::new(Duration::from_secs(10));
    let app = Router::new()
        .route("/", get(html_template::lookup::index))
        .route("/lookup", post(html_template::lookup::add_name))
        .route("/joke", get(html_template::joke::index))
        .route("/joke/renew", get(html_template::joke::renew))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(app_state.clone())
        .layer((trace_layer, timeout_layer));
    info!("Initialized!");

    let port = 3000;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Listening on port {}", port);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(app_state))
        .await
        .context("Error while starting server")?;

    Ok(())
}

async fn save_state(state: AppState) {
    match state.save().await {
        Ok(_) => info!("Save app state successfully!"),
        Err(err) => error!(%err),
    }
}

async fn shutdown_signal(state: AppState) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => save_state(state).await,
        _ = terminate => save_state(state).await,
    }
}
