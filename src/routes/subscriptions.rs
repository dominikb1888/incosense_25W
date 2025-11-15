use axum::Form;
use axum::extract::State;
use axum::response::IntoResponse;
use hyper::StatusCode;

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
    // TODO:
    // - Make Error Message more explicit/transparent -> currently Serde produces a 422 on
    // incomplete form data just like that
    // - Check if data is really arriving as url-encoded and what happens inside serde,
    // currently non-ascii characters are accepted and returned again (probably as UTF-8)

    if !formdata.name.is_ascii() || !formdata.email.is_ascii() {
        return (StatusCode::BAD_REQUEST, format!("{:?}", formdata));
    }

    let result = sqlx::query!(
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
    .await;

    (StatusCode::OK, format!("{:?}", result))
}
