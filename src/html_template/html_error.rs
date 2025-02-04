use super::HtmlTemplate;
use askama::Template;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt::Display;
use tracing::error;

pub struct HtmlError(StatusCode, String);

impl HtmlError {
    pub fn internal_error(message: &str) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, message.to_owned())
    }

    pub fn bad_request(message: &str) -> Self {
        Self(StatusCode::BAD_REQUEST, message.to_owned())
    }
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate {
    pub status_code: StatusCode,
    pub message: String,
}

impl IntoResponse for HtmlError {
    fn into_response(self) -> Response {
        let template = HtmlTemplate(ErrorTemplate {
            status_code: self.0,
            message: self.1.clone(),
        });
        template.into_response()
    }
}

pub fn internal_error<E>(err: E) -> HtmlError
where
    E: Into<anyhow::Error> + Display,
{
    error!(%err);
    HtmlError::internal_error("Internal server error")
}

pub fn bad_request<E>(err: E) -> HtmlError
where
    E: Into<anyhow::Error> + Display,
{
    error!(%err);
    HtmlError::bad_request("Oops, bad request")
}
