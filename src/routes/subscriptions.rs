use axum::Form;
use axum::extract::State;
use axum::response::IntoResponse;
use hyper::StatusCode;
use sqlx::postgres::PgDatabaseError;

use crate::routes::AppState;

#[derive(serde::Deserialize, Debug)]
pub struct Subscriber {
    pub name: String,
    pub email: String,
}

pub async fn post_subscriber(
    State(state): State<AppState>,
    Form(formdata): Form<Subscriber>,
) -> impl IntoResponse {
    if formdata.name.len() == 0 || formdata.email.len() == 0 {
        return (StatusCode::UNPROCESSABLE_ENTITY, format!("{:?}", formdata));
    }

    let code = match sqlx::query!(
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
    .await
    {
        Ok(_) => StatusCode::CREATED,

        Err(sqlx::Error::Database(db_err)) => {
            // Downcast to Postgres error
            if let Some(pg_err) = db_err.try_downcast_ref::<PgDatabaseError>() {
                match pg_err.code() {
                    "23505" => StatusCode::CONFLICT,    // unique violation
                    "23503" => StatusCode::BAD_REQUEST, // foreign key violation
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                }
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }

        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (code, "".to_string())
}
