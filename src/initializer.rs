mod app_state;
mod logs;
mod seed;
mod tls;

pub use app_state::AppState;
pub use logs::setup_logs;
pub use tls::build_tls_config;

use anyhow::Result;
use bloom_filter_yss::{BloomFilter, BloomFilterBuilder};
use futures::TryStreamExt;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn build_bloom_filter(pool: &PgPool, n: usize) -> Result<BloomFilter> {
    let path = dotenvy::var("LOCAL_BLOOM_FILTER_PATH")?;
    if let Ok(bloom_filter) = BloomFilterBuilder::load(&path) {
        return Ok(bloom_filter);
    }

    let mut bloom_filter = BloomFilterBuilder::new(n).build();
    let mut rows = sqlx::query("SELECT name FROM names").fetch(pool);

    let mut i = 0;
    while let Some(row) = rows.try_next().await? {
        i += 1;
        let name = row.try_get("name")?;
        bloom_filter.insert(name);
        if i >= n {
            break;
        }
    }

    Ok(bloom_filter)
}

fn prepare_local_disk() -> Result<()> {
    let path = dotenvy::var("LOCAL_BLOOM_FILTER_PATH").unwrap();
    let path = Path::new(&path);

    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
    }

    Ok(())
}

pub async fn initialize() -> Result<AppState> {
    prepare_local_disk()?;
    let database_url = dotenvy::var("DATABASE_URL").unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    sqlx::migrate!().run(&pool).await?;
    seed::seed_data(&pool, 0).await?;
    let bloom_filter = build_bloom_filter(&pool, 10_000_000).await?;
    let app_state = AppState {
        pool,
        bloom_filter: Arc::new(Mutex::new(bloom_filter)),
        api_client: reqwest::Client::new(),
    };

    Ok(app_state)
}
