use bloom_filter_yss::BloomFilter;
use sqlx::PgPool;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub bloom_filter: Arc<Mutex<BloomFilter>>,
}
