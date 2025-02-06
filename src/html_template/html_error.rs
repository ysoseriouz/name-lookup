use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
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

    pub fn not_found(message: &str) -> Self {
        Self(StatusCode::NOT_FOUND, message.to_owned())
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
        let template = ErrorTemplate {
            status_code: self.0,
            message: self.1.clone(),
        };
        match template.render() {
            Ok(html) => (self.0, Html(html)).into_response(),
            Err(err) => internal_error(err).into_response(),
        }
    }
}

pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    tracing::error!(%err);

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Something went wrong".to_owned(),
    )
}

pub fn bad_request<E>(err: E) -> HtmlError
where
    E: Into<anyhow::Error> + Display,
{
    error!(%err);
    HtmlError::bad_request("Oops, bad request")
}
