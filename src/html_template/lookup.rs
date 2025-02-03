use super::HtmlTemplate;
use crate::{AppState, State};
use askama::Template;
use axum::extract::Form;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "lookup/index.html")]
struct IndexTemplate;

#[derive(Template)]
#[template(path = "lookup/response.html")]
struct ResponseTemplate<'a> {
    message: &'a str,
    text_color: &'a str,
}

#[derive(Deserialize, Debug)]
pub struct Request {
    name: String,
}

pub async fn index() -> impl super::IntoResponse {
    let template = IndexTemplate {};
    HtmlTemplate(template)
}

pub async fn add_name(
    State(state): State<AppState>,
    Form(request): Form<Request>,
) -> impl super::IntoResponse {
    let mut bloom_filter = state.bloom_filter.lock().unwrap();
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

    HtmlTemplate(template)
}
