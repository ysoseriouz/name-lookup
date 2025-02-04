use super::{HtmlTemplate, ResultHtml};
use crate::{AppState, State};
use askama::Template;
use axum::extract::Form;
use serde::Deserialize;

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
        ResponseTemplate {
            message: "Noice!",
            text_color: "text-blue-800",
        }
    };

    Ok(HtmlTemplate(template))
}
