mod html_template;
mod initializer;
mod tls;

use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::Request,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use initializer::{initialize, AppState};
use std::{net::SocketAddr, time::Duration};
use tls::{build_tls_config, redirect_http_to_https, Ports};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    classify::ServerErrorsFailureClass, compression::CompressionLayer,
    decompression::RequestDecompressionLayer, services::ServeDir, timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{debug, error, info, info_span, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            format!("{}=trace,tower_http=trace", env!("CARGO_CRATE_NAME")).into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let ports = Ports {
        http: 7878,
        https: 3000,
    };
    let tls_config = build_tls_config()?;
    let app_state = initialize().await?;
    let app = app(app_state.clone());
    let addr = SocketAddr::from(([0, 0, 0, 0], ports.https));
    let handle = axum_server::Handle::new();
    let shutdown_future = shutdown_signal(handle.clone(), app_state.clone());

    tokio::spawn(redirect_http_to_https(ports, shutdown_future));

    debug!("listening on {addr}");
    axum_server::bind_rustls(addr, tls_config)
        .handle(handle)
        .serve(app.into_make_service())
        .await
        .context("Error while starting server")?;

    Ok(())
}

fn app(app_state: AppState) -> Router {
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|req: &Request<_>| {
            info_span!(
                "request",
                method = ?req.method(),
                uri = ?req.uri(),
                version = ?req.version(),
                status_code = tracing::field::Empty,
            )
        })
        .on_request(|req: &Request<_>, _span: &Span| {
            info!("📥 Request: {} {}", req.method(), req.uri());
        })
        .on_response(|res: &Response<_>, latency: Duration, span: &Span| {
            let status = res.status().as_u16();
            span.record("status_code", status);

            info!("✅ Response: {} | Latency: {:?}", status, latency);
        })
        .on_failure(
            |error: ServerErrorsFailureClass, latency: Duration, _span: &Span| {
                error!("❌ Request failed after {:?}: {:?}", latency, error);
            },
        );
    let timeout_layer = TimeoutLayer::new(Duration::from_secs(10));
    let service_layer = ServiceBuilder::new()
        .layer(trace_layer)
        .layer(timeout_layer)
        .layer(RequestDecompressionLayer::new())
        .layer(CompressionLayer::new());

    Router::new()
        .route("/", get(html_template::lookup::index))
        .route("/_chk", get(health_check))
        .route("/lookup", post(html_template::lookup::add_name))
        .route("/joke", get(html_template::joke::index))
        .route("/joke/renew", get(html_template::joke::renew))
        .nest_service("/static", ServeDir::new("static"))
        .layer(service_layer)
        .with_state(app_state.clone())
        .fallback(handler_404)
}

async fn health_check() -> impl IntoResponse {
    "OK"
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

async fn shutdown_signal(handle: axum_server::Handle, state: AppState) {
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

    info!("Received termination signal shutting down");
    handle.graceful_shutdown(Some(Duration::from_secs(10)));
}
