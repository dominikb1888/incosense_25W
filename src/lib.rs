use std::net::SocketAddr;

use axum::Form;
use axum::{Router, routing::get, routing::post};
use axum::{http::StatusCode, response::IntoResponse};
use serde;
use sqlx::PgPool;
use tokio::net::TcpListener;

pub mod configuration;

#[derive(serde::Deserialize, Debug)]
struct Subscriber {
    name: String,
    email: String,
}

pub fn build_router(connection_pool: PgPool) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/healthcheck", get(healthcheck))
        .route("/subscriptions", post(post_subscriber))
        .with_state(connection_pool)
}

/// Run the Axum app on the given address
/// If `bind_addr` is `None`, it binds to a random local port
pub async fn run(bind_addr: Option<SocketAddr>, connection_pool: PgPool) -> std::io::Result<()> {
    let app = build_router(connection_pool);

    // Bind listener
    let addr = bind_addr.unwrap_or(([127, 0, 0, 1], 0).into());
    let listener = TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;

    println!("Listening on http://{local_addr}");

    // Run the server
    axum::serve(listener, app).await
}

// Build a function that returns an HTTP Response 200 OK with an empty body
pub async fn healthcheck() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn post_subscriber(Form(formdata): Form<Subscriber>) -> impl IntoResponse {
    // TODO:
    // - Make Error Message more explicit/transparent -> currently Serde produces a 422 on
    // incomplete form data just like that
    // - Check if data is really arriving as url-encoded and what happens inside serde,
    // currently non-ascii characters are accepted and returned again (probably as UTF-8)

    if !formdata.name.is_ascii() || !formdata.email.is_ascii() {
        return (StatusCode::BAD_REQUEST, format!("{:?}", formdata));
    }

    (StatusCode::OK, format!("{:?}", formdata))
}
