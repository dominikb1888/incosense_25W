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

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let (base_url, server_handle) = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let body = "name=le%20guin&email=Ürsula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &base_url))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    server_handle.abort();
}

// #[tokio::test]
// async fn subscribe_returns_a_400_when_data_is_missing() {
//
//     let (base_url, server_handle) = spawn_app().await;
//
//     // Arrange
//     let client = reqwest::Client::new();
//     let test_cases=vec![
//         ("name=le%20guin","missing the email"),
//         ("email=ursula_le_guin%40gmail.com","missing the name"),
//         ("","missing both name and email")
//     ];
//
//     for(invalid_body,error_message) in test_cases {
//         // Act
//         let response = client
//             .post(&format!("{}/subscriptions",&base_url))
//             .header("Content-Type","application/x-www-form-urlencoded")
//             .body(invalid_body)
//             .send()
//             .await
//             .expect("Failed to execute request.");
//
//         // Assert
//         assert_eq!(400,response.status().as_u16(),
//             // Additional customised error message on test failure
//             "The API did not fail with 400 Bad Request when the payload was {}.",error_message);
//     }
//
//     server_handle.abort();
// }

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
