mod html_template;
mod initializer;

use anyhow::{Context, Result};
use axum::{
    extract::{MatchedPath, Request, State},
    routing::{get, post},
    Router,
};
use initializer::{initialize, AppState};
use tokio::{net::TcpListener, signal};
use tower_http::{
    compression::CompressionLayer, decompression::RequestDecompressionLayer, services::ServeDir,
    timeout::TimeoutLayer, trace::TraceLayer,
};
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
    let timeout_layer = TimeoutLayer::new(std::time::Duration::from_secs(10));
    let app = Router::new()
        .route("/", get(html_template::lookup::index))
        .route("/lookup", post(html_template::lookup::add_name))
        .route("/joke", get(html_template::joke::index))
        .route("/joke/renew", get(html_template::joke::renew))
        .nest_service("/static", ServeDir::new("static"))
        .layer(trace_layer)
        .layer(timeout_layer)
        .layer(RequestDecompressionLayer::new())
        .layer(CompressionLayer::new())
        .with_state(app_state.clone())
        .fallback(handler_404);
    info!("Initialized!");

    let mut listenfd = listenfd::ListenFd::from_env();
    let listener = match listenfd.take_tcp_listener(0).unwrap() {
        Some(listener) => {
            listener.set_nonblocking(true)?;
            TcpListener::from_std(listener)?
        }
        None => TcpListener::bind("127.0.0.1:3000").await?,
    };
    info!("Listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(app_state))
        .await
        .context("Error while starting server")?;

    Ok(())
}

async fn handler_404() -> html_template::HtmlError {
    html_template::HtmlError::not_found("Nothing here")
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
