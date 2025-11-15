use axum::response::IntoResponse;
use hyper::StatusCode;

// Build a function that returns an HTTP Response 200 OK with an empty body
pub async fn healthcheck() -> impl IntoResponse {
    StatusCode::OK
}
