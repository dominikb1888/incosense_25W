use hyper::StatusCode;
use incosense::build_router;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use incosense::AppState;
use incosense::configuration::get_configuration;

#[tokio::test]
async fn healthcheck_works() {
    let (base_url, server_handle, _connection_pool) = spawn_app().await;

    let response = reqwest::get(format!("{base_url}/healthcheck"))
        .await
        .unwrap();

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

    server_handle.abort();
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let (base_url, server_handle, connection_pool) = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=Ursula_le_guin%40gmail.com";

    let response = client
        .post(format!("{}/subscriptions", base_url))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(StatusCode::OK, response.status());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&connection_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "Ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

    server_handle.abort();
}

#[tokio::test]
async fn subscribe_returns_a_422_when_data_is_missing() {
    let (base_url, server_handle, _connection_pool) = spawn_app().await;

    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", base_url))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            StatusCode::UNPROCESSABLE_ENTITY,
            response.status(),
            "The API did not fail with 422 (unprocessable entity) when the payload was {}.",
            error_message
        );
    }

    server_handle.abort();
}

#[tokio::test]
async fn test_non_utf8_form_rejected() {
    let (base_url, server_handle, _connection_pool) = spawn_app().await;

    let invalid_payloads: &[&[u8]] = &[
        b"name=H\xE4llo&email=W\xF6rld", // ISO-8859-1 ä ö
        b"name=\xFFfoo&email=bar",       // lone invalid byte
        b"name=\xC3\x28&email=test",     // invalid UTF-8 sequence
        b"name=%E4llo&email=W%F6rld",    // percent-encoded Latin-1
        b"name=\xFE\xFF&email=baz",      // invalid high bytes
        b"name=foo\x80bar&email=baz",    // stray continuation byte
    ];

    let client = reqwest::Client::new();

    for bytes in invalid_payloads {
        let response = client
            .post(format!("{}/subscriptions", base_url))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(bytes.to_vec())
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            StatusCode::BAD_REQUEST,
            response.status(),
            "Expected BAD_REQUEST for bytes: {:x?}",
            bytes
        );
    }

    server_handle.abort();
}

pub async fn spawn_app() -> (String, JoinHandle<()>, PgPool) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let configuration = get_configuration().expect("Failed to read configuration.");

    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let app = build_router(AppState {
        db: connection_pool.clone(),
    });

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (
        format!("http://127.0.0.1:{port}"),
        server_handle,
        connection_pool,
    )
}
