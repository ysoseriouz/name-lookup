use bloom_filter_yss::BloomFilter;
use futures::TryStreamExt;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::error::Error;

fn prepare_test_data(n: usize) -> Vec<String> {
    (1..=n).map(|i| format!("test{}", i)).collect()
}

async fn count_names(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM names")
        .fetch_one(pool)
        .await?;

    Ok(count.0)
}

async fn bulk_insert(pool: &PgPool, names: &[String]) -> Result<u64, sqlx::Error> {
    if names.is_empty() {
        return Ok(0);
    }
    let sql_params = names
        .iter()
        .map(|name| format!("('{}')", name))
        .collect::<Vec<String>>()
        .join(", ");
    let query = format!(
        r#"
        INSERT INTO names (name)
        VALUES {}
        ON CONFLICT (name) DO NOTHING
        "#,
        sql_params
    );
    let rows_affected = sqlx::query(&query).execute(pool).await?.rows_affected();

    Ok(rows_affected)
}

async fn seed_data(pool: &PgPool, n: usize) -> Result<(), sqlx::Error> {
    if n == 0 {
        return Ok(());
    }

    let test_data = prepare_test_data(n);
    let rows_inserted = bulk_insert(pool, &test_data).await?;
    println!("Inserted {}", rows_inserted);

    let number_of_names = count_names(pool).await?;
    println!("Total records: {}", number_of_names);

    Ok(())
}

async fn build_bloom_filter(pool: &PgPool, n: usize) -> Result<BloomFilter, sqlx::Error> {
    let mut bloom_filter = BloomFilter::new(n);
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

fn lookup(bloom_filter: &BloomFilter, key: &str) {
    if bloom_filter.lookup(key) {
        println!("{} may exist", key);
    } else {
        println!("{} not exist", key);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let database_url = dotenvy::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    seed_data(&pool, 0).await?;

    let bloom_filter = build_bloom_filter(&pool, 10_000_000).await?;
    lookup(&bloom_filter, "test");
    lookup(&bloom_filter, "test1");

    Ok(())
}
