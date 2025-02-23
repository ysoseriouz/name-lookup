use std::time::Duration;

use axum::{
    http::{Request, Response},
    response::IntoResponse,
    routing::{get, get_service, post},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    classify::ServerErrorsFailureClass, compression::CompressionLayer,
    decompression::RequestDecompressionLayer, request_id::MakeRequestUuid, services::ServeDir,
    timeout::TimeoutLayer, trace::TraceLayer, ServiceBuilderExt,
};
use tracing::{error, info, info_span, Span};

use crate::html_template;

use super::AppState;

pub fn router(app_state: AppState) -> Router {
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|req: &Request<_>| {
            let request_id = req
                .headers()
                .get("X-Request-Id")
                .map_or("N/A", |v| v.to_str().unwrap_or("invalid"));
            info_span!(
                "request",
                request_id = request_id,
                method = ?req.method(),
                uri = ?req.uri(),
                version = ?req.version(),
                status_code = tracing::field::Empty,
            )
        })
        .on_request(|req: &Request<_>, _span: &Span| {
            info!("📥 Request: {} {}", req.method(), req.uri());
        })
        .on_response(|res: &Response<_>, latency: Duration, span: &Span| {
            let status = res.status().as_u16();
            span.record("status_code", status);

            info!("✅ Response: {} | Latency: {:?}", status, latency);
        })
        .on_failure(
            |error: ServerErrorsFailureClass, latency: Duration, _span: &Span| {
                error!("❌ Request failed after {:?}: {:?}", latency, error);
            },
        );
    let timeout_layer = TimeoutLayer::new(Duration::from_secs(10));
    let service_layer = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid)
        .layer(trace_layer)
        .propagate_x_request_id()
        .layer(timeout_layer)
        .layer(RequestDecompressionLayer::new())
        .layer(CompressionLayer::new());

    Router::new()
        .route("/", get(html_template::lookup::index))
        .route("/_chk", get(health_check))
        .route("/lookup", post(html_template::lookup::add_name))
        .route("/joke", get(html_template::joke::index))
        .route("/joke/renew", get(html_template::joke::renew))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/public", get_service(ServeDir::new("public")))
        .layer(service_layer)
        .with_state(app_state.clone())
        .fallback(handler_404)
}

async fn health_check() -> impl IntoResponse {
    "OK"
}

async fn handler_404() -> html_template::HtmlError {
    html_template::HtmlError::not_found("Nothing here")
}
