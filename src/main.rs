mod html_template;
mod initializer;

use anyhow::{Context, Result};
use axum::{
    body::Bytes,
    extract::{MatchedPath, State},
    http::{HeaderMap, Request},
    response::Response,
    routing::{get, post},
    Router,
};
use initializer::{initialize, AppState};
use std::time::Duration;
use tokio::{net::TcpListener, signal};
use tower_http::{
    classify::ServerErrorsFailureClass, compression::CompressionLayer,
    decompression::RequestDecompressionLayer, services::ServeDir, timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{error, info, info_span, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
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
        .make_span_with(|req: &Request<_>| {
            let matched_path = req
                .extensions()
                .get::<MatchedPath>()
                .map(MatchedPath::as_str);

            info_span!(
                "http_request",
                method = ?req.method(),
                matched_path,
                some_other_field = tracing::field::Empty
            )
        })
        .on_request(|_req: &Request<_>, _span: &Span| {})
        .on_response(|_res: &Response, _latency: Duration, _span: &Span| {})
        .on_body_chunk(|_chunk: &Bytes, _latency: Duration, _span: &Span| {})
        .on_eos(|_trailers: Option<&HeaderMap>, _stream_duration: Duration, _span: &Span| {})
        .on_failure(|_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {});
    let timeout_layer = TimeoutLayer::new(Duration::from_secs(10));
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
