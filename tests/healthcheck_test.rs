use incosense::build_router;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

#[tokio::test]
async fn healthcheck_works() {
    // Arrange
    let (base_url, server_handle) = spawn_app().await;
    // We need to bring in `reqwest`
    // to perform HTTP requests against our application.
    let response = reqwest::get(format!("{base_url}/healthcheck"))
        .await
        .unwrap();

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

    // Abort
    server_handle.abort();
}

pub async fn spawn_app() -> (String, JoinHandle<()>) {
    // Bind to a random free port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    // Build the router
    let app = build_router();

    // Spawn the server
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://127.0.0.1:{port}"), server_handle)
}
