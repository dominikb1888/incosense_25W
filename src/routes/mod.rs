use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;

pub mod health_check;
pub mod subscriptions;

use health_check::healthcheck;
use subscriptions::post_subscriber;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub fn build_router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/healthcheck", get(healthcheck))
        .route("/subscriptions", post(post_subscriber))
        .with_state(app_state)
}
