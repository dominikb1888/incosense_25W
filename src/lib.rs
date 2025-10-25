use axum::{Router, routing::get};
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// Build the Axum router
pub fn build_router() -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/healthcheck", get(healthcheck))
    // Add more routes here as needed
}

/// Run the Axum app on the given address
///
/// If `bind_addr` is `None`, it binds to a random local port
pub async fn run(bind_addr: Option<SocketAddr>) -> std::io::Result<()> {
    let app = build_router();

    // Bind listener
    let addr = bind_addr.unwrap_or(([127, 0, 0, 1], 0).into());
    let listener = TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;

    println!("Listening on http://{local_addr}");

    // Run the server
    axum::serve(listener, app).await
}

// Build a function that returns an HTTP Response 200 OK with an empty body
pub async fn healthcheck() -> &'static str {
    ""
}
