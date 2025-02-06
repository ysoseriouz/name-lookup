mod html_error;
pub mod joke;
pub mod lookup;

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
pub use html_error::*;

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        let fallback_render = (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Something went wrong".to_owned(),
        );

        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => {
                tracing::error!(%err);
                fallback_render.into_response()
            }
        }
    }
}

pub type ResultHtml<T> = Result<HtmlTemplate<T>, HtmlError>;
