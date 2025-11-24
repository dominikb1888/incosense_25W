use axum::response::IntoResponse;
use hyper::StatusCode;
use tracing::info;

// Build a function that returns an HTTP Response 200 OK with an empty body
pub async fn healthcheck() -> impl IntoResponse {
    info!("Handling health_check request");
    StatusCode::OK
}
