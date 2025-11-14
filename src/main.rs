use incosense::run;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let bind_addr: Option<SocketAddr> = Some(([127, 0, 0, 1], 3000).into());
    run(bind_addr).await
}
