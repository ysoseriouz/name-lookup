use anyhow::Result;
use sqlx::PgPool;

fn prepare_test_data(n: usize) -> Vec<String> {
    (1..=n).map(|i| format!("test{}", i)).collect()
}

async fn count_names(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM names")
        .fetch_one(pool)
        .await?;

    Ok(count.0)
}

async fn bulk_insert(pool: &PgPool, names: &[String]) -> Result<u64> {
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

pub async fn seed_data(pool: &PgPool, n: usize) -> Result<()> {
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
