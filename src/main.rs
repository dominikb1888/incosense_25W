use incosense::configuration::get_configuration;
use incosense::startup::run;
use sqlx::PgPool;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let bind_addr: Option<SocketAddr> =
        Some(([127, 0, 0, 1], configuration.application_port).into());
    run(bind_addr, connection_pool).await?;
    Ok(())
}
