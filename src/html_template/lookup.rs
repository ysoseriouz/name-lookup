use super::HtmlTemplate;
use askama::Template;
use axum::extract::Form;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "lookup.html")]
pub struct LookupTemplate;

pub async fn show_form() -> HtmlTemplate<LookupTemplate> {
    let template = LookupTemplate {};
    HtmlTemplate(template)
}

#[derive(Deserialize, Debug)]
pub struct Input {
    name: String,
}

pub async fn accept_form(Form(input): Form<Input>) {
    dbg!(&input);
}
