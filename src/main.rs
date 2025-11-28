use sqlx::PgPool;
use sqlx::migrate::Migrator;
use std::net::SocketAddr;
use tracing_subscriber::{EnvFilter, util::SubscriberInitExt};

use incosense::configuration::Settings;
use incosense::startup::run;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .json()
        .flatten_event(true)
        .with_current_span(true)
        .with_span_list(false)
        .finish()
        .init();

    // Settings::from_env() now returns Settings directly
    let configuration = Settings::from_env();

    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    // Run pending migrations automatically
    MIGRATOR
        .run(&connection_pool)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let bind_addr: SocketAddr = ([0, 0, 0, 0], configuration.application_port).into();
    run(Some(bind_addr), connection_pool).await?;
    Ok(())
}
