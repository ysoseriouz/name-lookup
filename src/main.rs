use bloom_filter_yss::BloomFilter;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::error::Error;

fn prepare_test_data() -> Vec<String> {
    (1..1_000_000).map(|i| format!("test{}", i)).collect()
}

async fn count_names(pool: &PgPool) -> Result<(), sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM names")
        .fetch_one(pool)
        .await?;

    println!("Record: {}", count.0);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let database_url = dotenvy::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    count_names(&pool).await?;

    Ok(())
}
