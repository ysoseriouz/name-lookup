mod html_template;
mod initializer;
mod router;

use anyhow::{Context, Result};
use axum::extract::State;
use initializer::{build_tls_config, initialize, setup_logs, AppState};
use router::router;
use std::{net::SocketAddr, time::Duration};
use tokio::signal;
use tracing::{debug, error, info};

#[tokio::main]
async fn main() -> Result<()> {
    setup_logs();

    let tls_config = build_tls_config()?;
    let app_state = initialize().await?;
    let app = router(app_state.clone());
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let handle = axum_server::Handle::new();

    tokio::spawn(shutdown_signal(handle.clone(), app_state));

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
