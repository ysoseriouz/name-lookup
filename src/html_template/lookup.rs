use super::{internal_error, HtmlTemplate, ResultHtml};
use crate::{AppState, State};
use askama::Template;
use axum::extract::Form;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Template)]
#[template(path = "lookup/index.html")]
pub struct IndexTemplate;

#[derive(Template)]
#[template(path = "lookup/response.html")]
pub struct ResponseTemplate<'a> {
    message: &'a str,
    text_color: &'a str,
}

#[derive(Deserialize, Debug)]
pub struct Request {
    name: String,
}

pub async fn index() -> ResultHtml<IndexTemplate> {
    let template = IndexTemplate {};
    Ok(HtmlTemplate(template))
}

pub async fn add_name<'a>(
    State(state): State<AppState>,
    Form(request): Form<Request>,
) -> ResultHtml<ResponseTemplate<'a>> {
    let mut bloom_filter = state.bloom_filter.lock().await;
    let template = if bloom_filter.lookup(&request.name) {
        ResponseTemplate {
            message: "Another name plz!",
            text_color: "text-red-800",
        }
    } else {
        bloom_filter.insert(&request.name);
        insert_name_db(&state.pool, &request.name)
            .await
            .map_err(internal_error)?;
        ResponseTemplate {
            message: "Noice!",
            text_color: "text-blue-800",
        }
    };

    Ok(HtmlTemplate(template))
}

async fn insert_name_db(pool: &PgPool, name: &str) -> anyhow::Result<()> {
    let query = format!(
        r#"
            INSERT INTO names (name)
            VALUES ('{}')
            ON CONFLICT (name) DO NOTHING
        "#,
        name,
    );
    sqlx::query(&query).execute(pool).await?;
    Ok(())
}
