use axum::Form;
use axum::extract::State;
use axum::{Router, routing::get, routing::post};
use axum::{http::StatusCode, response::IntoResponse};
use serde;
use sqlx::PgPool;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub mod configuration;

#[derive(serde::Deserialize, Debug)]
pub struct Subscriber {
    pub name: String,
    pub email: String,
}

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

// Build a function that returns an HTTP Response 200 OK with an empty body
pub async fn healthcheck() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn post_subscriber(
    State(state): State<AppState>,
    Form(formdata): Form<Subscriber>,
) -> impl IntoResponse {
    // TODO:
    // - Make Error Message more explicit/transparent -> currently Serde produces a 422 on
    // incomplete form data just like that
    // - Check if data is really arriving as url-encoded and what happens inside serde,
    // currently non-ascii characters are accepted and returned again (probably as UTF-8)

    if !formdata.name.is_ascii() || !formdata.email.is_ascii() {
        return (StatusCode::BAD_REQUEST, format!("{:?}", formdata));
    }

    let result = sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
            "#,
        uuid::Uuid::new_v4(),
        formdata.email,
        formdata.name,
        chrono::Utc::now()
    )
    .execute(&state.db)
    .await;

    (StatusCode::OK, format!("{:?}", result))
}
