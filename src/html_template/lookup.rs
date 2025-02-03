use super::HtmlTemplate;
use crate::{error::internal_error, AppState, State};
use askama::Template;
use axum::{extract::Form, http::StatusCode};
use serde::Deserialize;

#[derive(Template)]
#[template(path = "lookup.html")]
pub struct LookupTemplate;

pub async fn show() -> HtmlTemplate<LookupTemplate> {
    let template = LookupTemplate {};
    HtmlTemplate(template)
}

#[derive(Deserialize, Debug)]
pub struct AddNameRequest {
    name: String,
}

pub async fn add_name(
    State(state): State<AppState>,
    Form(request): Form<AddNameRequest>,
) -> Result<String, (StatusCode, String)> {
    let mut bloom_filter = state.bloom_filter.lock().map_err(internal_error)?;

    if bloom_filter.lookup(&request.name) {
        Ok("Already exists, come up with another name pls...".to_string())
    } else {
        bloom_filter.insert(&request.name);
        Ok("Nice name!".to_string())
    }
}
