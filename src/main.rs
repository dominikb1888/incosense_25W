use axum::{Router, routing::get};
use tokio;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/healthcheck", get(healthcheck));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Build a function that returns an HTTP Response 200 OK with an empty body
async fn healthcheck() -> &'static str {
    ""
}
