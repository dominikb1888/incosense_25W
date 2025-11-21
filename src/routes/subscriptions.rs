use axum::{extract::Form, extract::State, response::IntoResponse};
use hyper::StatusCode;
use sqlx::postgres::PgDatabaseError;
use unicode_segmentation::UnicodeSegmentation;

use crate::routes::AppState;


#[derive(serde::Deserialize, Debug)]
pub struct Subscriber {
    pub name: SubscriberName, // TODO: How do we make Postgres understand our types?
    pub email: SubscriberEmail,
}

#[derive(serde::Deserialize, Debug)]
struct SubscriberName {
    name: String
}

impl SubscriberName {
    fn new(data: String) -> Self {
        if data.graphemes(true).count() < 256 {
            SubscriberName { name: data }
        } else {
         // TODO: ???
        }
    }
}

#[derive(serde::Deserialize, Debug)]
struct SubscriberEmail {
    email: String
}

impl SubscriberEmail {
    fn new(data: String) -> Self {
        //TODO: Check for valid email to be added
        if data.graphemes(true).count() < 256 {
            SubscriberEmail { email: data }
        } else {
        //TODO: ???
        }
    }
}

pub async fn post_subscriber(
    State(state): State<AppState>,
    Form(formdata): Form<Subscriber>,
) -> impl IntoResponse {

    let status = match sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
        "#,
        uuid::Uuid::new_v4(),
        formdata.email.email,
        formdata.name.name,
        chrono::Utc::now()
    )
    .execute(&state.db)
    .await
    {
        Ok(_) => StatusCode::CREATED,

        Err(sqlx::Error::Database(db_err)) => {
            if let Some(pg_err) = db_err.try_downcast_ref::<PgDatabaseError>() {
                match pg_err.code() {
                    "23505" => StatusCode::CONFLICT,    // unique violation
                    "23503" => StatusCode::BAD_REQUEST, // FK violation
                    _       => StatusCode::INTERNAL_SERVER_ERROR,
                }
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }

        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (status, "".to_string())
}

