use sqlx::PgPool;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::routes::AppState;
use crate::routes::build_router;

/// Run the Axum app on the given address
/// If `bind_addr` is `None`, it binds to a random local port
pub async fn run(bind_addr: Option<SocketAddr>, connection_pool: PgPool) -> std::io::Result<()> {
    let app_state = AppState {
        db: connection_pool,
    };
    let app = build_router(app_state);

    // Bind listener
    let addr = bind_addr.unwrap_or(([127, 0, 0, 1], 0).into());
    let listener = TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;

    println!("Listening on http://{local_addr}");

    // Run the server
    axum::serve(listener, app).await
}
