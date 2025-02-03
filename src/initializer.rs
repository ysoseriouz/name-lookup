mod app_state;
mod seed;

pub use app_state::AppState;

use anyhow::Result;
use bloom_filter_yss::{BloomFilter, BloomFilterBuilder};
use futures::TryStreamExt;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};

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

pub async fn initialize() -> Result<AppState> {
    let database_url = dotenvy::var("DATABASE_URL").unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    seed::seed_data(&pool, 0).await?;
    let bloom_filter = build_bloom_filter(&pool, 10_000_000).await?;
    let app_state = AppState { pool, bloom_filter };

    Ok(app_state)
}
