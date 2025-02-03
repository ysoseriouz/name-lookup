mod error;
mod html_template;
mod initializer;

use anyhow::{Context, Result};
use axum::{extract::State, http::StatusCode, routing::get, Router};
use error::internal_error;
use initializer::{initialize, AppState};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn root(State(app_state): State<AppState>) -> Result<String, (StatusCode, String)> {
    let row = sqlx::query!("SELECT name FROM names ORDER BY random() LIMIT 1")
        .fetch_one(&app_state.pool)
        .await
        .map_err(internal_error)?;

    Ok(row.name)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=trace", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Initialize database...");
    let app_state = initialize().await?;
    info!("Database initialized!");

    info!("Initialize router...");
    let app = Router::new()
        .route("/", get(root).post(root))
        .route(
            "/lookup",
            get(html_template::lookup::show).post(html_template::lookup::add_name),
        )
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(app_state);

    let port = 3000;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Router initialized, now listening on port {}", port);

    axum::serve(listener, app)
        .await
        .context("Error while starting server")?;

    Ok(())
}
