use bloom_filter_yss::BloomFilter;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub bloom_filter: BloomFilter,
}
