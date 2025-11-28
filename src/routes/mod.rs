use axum::{
    Router,
    extract::ConnectInfo,
    http::Request,
    routing::{get, post},
};
use sqlx::PgPool;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::{DefaultOnResponse, TraceLayer},
};
use tracing::Span;

use std::net::SocketAddr;

pub mod health_check;
pub mod subscriptions;

use health_check::healthcheck;
use subscriptions::post_subscriber;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

/// Create a span for every request, including method, path, and client IP
fn make_request_span<B>(req: &Request<B>) -> Span {
    let headers = req.headers();

    // Get client IP from X-Forwarded-For or ConnectInfo fallback
    let forwarded_for = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let connect_ip = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.to_string());

    let client_ip = forwarded_for
        .or(connect_ip)
        .unwrap_or_else(|| "unknown".to_string());

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let request_id = req
        .extensions()
        .get::<RequestId>()
        .and_then(|id| id.header_value().to_str().ok())
        .unwrap_or("unknown");

    let query = req.uri().query().unwrap_or("");

    dbg!(req.extensions().get::<tower_http::request_id::RequestId>());

    tracing::info_span!(
        "http_request",
        request_id=%request_id,
        method = %req.method(),
        path   = %req.uri().path(),
        query  = %query,
        user_agent = %user_agent,
        client_ip  = %client_ip,
    )
}

pub fn build_router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/healthcheck", get(healthcheck))
        .route("/subscriptions", post(post_subscriber))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(make_request_span)
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        .with_state(app_state)
}
