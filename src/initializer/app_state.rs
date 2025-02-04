use bloom_filter_yss::BloomFilter;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub bloom_filter: Arc<Mutex<BloomFilter>>,
    pub api_client: reqwest::Client,
}

impl AppState {
    pub async fn save(&self) -> anyhow::Result<()> {
        let path = dotenvy::var("LOCAL_BLOOM_FILTER_PATH")?;
        let bloom_filter = self.bloom_filter.lock().await;
        bloom_filter.to_file(&path);
        Ok(())
    }
}
