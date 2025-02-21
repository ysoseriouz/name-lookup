mod html_template;
mod initializer;
mod router;
mod tls;

use anyhow::{Context, Result};
use axum::extract::State;
use initializer::{initialize, AppState};
use router::router;
use std::{net::SocketAddr, time::Duration};
use tls::{build_tls_config, redirect_http_to_https, Ports};
use tokio::signal;
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or("TRACE".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let ports = Ports {
        http: 7878,
        https: 3000,
    };
    let tls_config = build_tls_config()?;
    let app_state = initialize().await?;
    let app = router(app_state.clone());
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
