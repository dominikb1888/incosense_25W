use hyper::StatusCode;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use incosense::configuration::get_configuration;
use incosense::routes::{AppState, build_router};

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
async fn subscribe_returns_200_for_all_valid_form_data() {
    let (base_url, server_handle, connection_pool) = spawn_app().await;
    let client = reqwest::Client::new();

    let b_255 = "b".repeat(255);

    let valid_cases: Vec<(String, &str, String, &str)> = vec![
        // original valid case
        (
            "name=le%20guin&email=ursula_le_guin%40gmail.com".to_string(),
            "simple valid name/email",
            "le guin".to_string(),
            "ursula_le_guin@gmail.com",
        ),
        // unicode valid case
        (
            "name=%C3%89l%C3%A9onore&email=eleonore%40example.com".to_string(),
            "unicode valid name",
            "Éléonore".to_string(),
            "eleonore@example.com",
        ),
        // emoji valid case (short)
        (
            "name=%F0%9F%98%80&email=emoji%40example.com".to_string(),
            "emoji valid name",
            "😀".to_string(),
            "emoji@example.com",
        ),
        // name with spaces
        (
            "name=John%20Doe&email=john%40example.com".to_string(),
            "name with spaces",
            "John Doe".to_string(),
            "john@example.com",
        ),
        // max-length name (255)
        (
            format!("name={}&email=maxlength2%40example.com", b_255.clone()),
            "max-length name",
            b_255,
            "maxlength2@example.com",
        ),
    ];

    for (body, description, expected_name, expected_email) in valid_cases {
        println!("Running valid case: {}", description);

        let response = client
            .post(format!("{}/subscriptions", base_url))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            StatusCode::CREATED,
            response.status(),
            "Expected 201 CREATED for case '{}' but got {}",
            description,
            response.status()
        );

        // Fetch the most recently saved subscription
        let saved = sqlx::query!(
            r#"
              SELECT email, name
              FROM subscriptions
              WHERE email = $1
              ORDER BY id
              DESC LIMIT 1
            "#,
            &expected_email
        )
        .fetch_one(&connection_pool)
        .await
        .expect("Query failed");

        assert_eq!(
            saved.email, expected_email,
            "Email mismatch in '{}'",
            description
        );
        assert_eq!(
            saved.name, expected_name,
            "Name mismatch in '{}'",
            description
        );
    }

    server_handle.abort();
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let (base_url, server_handle, _connection_pool) = spawn_app().await;

    let client = reqwest::Client::new();

    let a_256 = "a".repeat(256);
    let a_65536 = "a".repeat(65536); // Postgres max-length for text field is 65535 bytes
    let e_40000 = "é".repeat(40000);
    let emoji_repeat = "👨‍👩‍👦‍👦".repeat(37);

    let test_cases: Vec<(String, &str)> = vec![
        // original invalid cases
        ("name=le%20guin".to_string(), "missing the email"),
        (
            "email=crsula_le_guin%40gmail.com".to_string(),
            "missing the name",
        ),
        ("".to_string(), "missing both name and email"),
        // NEW invalid cases
        ("name=&email=test%40example.com".to_string(), "name empty"),

        // extremely long input
        (
            format!("name={}&email=test%40example.com", a_65536),
            "extremely long name",
        ),
        // name exceeding ideal max-length by 1
        (
            format!("name={}&email=test%40example.com", a_256),
            "name exceeding max length",
        ),
        // SQL injection attempt
        (
            "name='%3B%20DROP%20TABLE%20subscribers%3B%20--&email=test%40example.com".to_string(),
            "sql injection attempt",
        ),
        // XSS attack payload
        (
            "name=%3Cscript%3Ealert('x')%3C%2Fscript%3E&email=test%40example.com".to_string(),
            "xss attempt in name",
        ),
        // malformed Unicode / combining chars *if you treat them as invalid*
        (
            format!("name={}&email=test%40example.com", e_40000), // over-length due to multibyte
            "unicode multibyte name too long",
        ),
        // emoji-heavy name — also too long after encoding
        (
            format!("name={}&email=test%40example.com", emoji_repeat),
            "emoji multi-codepoint too long",
        ),
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
            StatusCode::BAD_REQUEST,
            response.status(),
            "The API did not fail with 400 (Bad Request) when the payload was {}.",
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
        b"name=hello%00world&email=test%40example.com", // Null Byte in Name
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
            StatusCode::UNPROCESSABLE_ENTITY,
            response.status(),
            "Expected UNPROCESSABLE_ENTITY for bytes: {:x?}",
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
