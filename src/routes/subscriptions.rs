use axum::{extract::State, response::IntoResponse};
use hyper::StatusCode;
use sqlx::postgres::PgDatabaseError;
use unicode_segmentation::UnicodeSegmentation;

use crate::routes::AppState;
use crate::strict_form::StrictForm;

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

// ===============================
// Subscriber main struct
// ===============================

#[derive(Debug, Deserialize)]
pub struct Subscriber {
    pub name: SubscriberName,
    pub email: SubscriberEmail,
}

// ===============================
// SubscriberName
// ===============================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct SubscriberName {
    name: String,
}

impl SubscriberName {
    pub fn as_str(&self) -> &str {
        &self.name
    }

    pub fn len(&self) -> usize {
        self.name.graphemes(true).count()
    }

    pub fn is_empty(&self) -> bool {
        self.name.trim().is_empty()
    }
}

impl TryFrom<String> for SubscriberName {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let length = value.chars().count();

        if length == 0 {
            return Err("Name cannot be empty".into());
        }

        if length > 255 {
            return Err("Name is too long (maximum 255 characters)".into());
        }

        if value.contains('<') || value.contains('>') {
            return Err("Name contains markup: potential XSS attack".into());
        }

        if value.contains(';') || value.contains("--") || value.contains("/*") {
            return Err("Name contains forbidden characters".into());
        }

        Ok(Self { name: value })
    }
}

// ===============================
// SubscriberEmail
// ===============================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct SubscriberEmail {
    pub email: String,
}

impl SubscriberEmail {
    pub fn as_str(&self) -> &str {
        &self.email
    }

    pub fn domain(&self) -> Option<&str> {
        self.email.split('@').nth(1)
    }
}

impl TryFrom<String> for SubscriberEmail {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() == 0 {
            return Err("Email cannot be empty".into());
        }

        if value.len() > 255 {
            return Err("Email is too long (maximum 255 characters)".into());
        }

        // Minimal but effective validation
        if !value.contains('@') {
            return Err("Email must contain '@'".into());
        }

        let parts: Vec<&str> = value.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err("Invalid email format".into());
        }

        Ok(Self { email: value })
    }
}

pub async fn post_subscriber(
    State(state): State<AppState>,
    StrictForm(formdata): StrictForm<Subscriber>,
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
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                }
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }

        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (status, "".to_string())
}
