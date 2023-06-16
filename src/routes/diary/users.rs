use axum::extract::Path;
use axum::Form;
use axum::response::IntoResponse;
use serde::Serialize;

use crate::{lock_database, ResponseJson};
use crate::routes::diary::{AuthForm, failure_response, hash_password, ResponseStatus};

#[derive(Serialize)]
pub struct UserProfile {
    pub signup_time: u64,
    pub username: String,
    pub email: Option<String>,
    pub name: Option<String>,
}

pub async fn user_info(Path(username): Path<String>) -> impl IntoResponse {
    let database = lock_database!();

    match database.query_user_profile(&username) {
        None => failure_response(ResponseStatus::NoRecord).into_response(),
        Some(x) => ResponseJson::ok(x).into_response(),
    }
}

pub async fn create_user(Form(form): Form<AuthForm>) -> impl IntoResponse {
    let pw_hash = hash_password(&form.password);
    let database = lock_database!();

    if database.check_existence(&form.username, None) {
        // user exists
        return failure_response(ResponseStatus::UserExists).into_response();
    }

    database.add_user(&form.username, &pw_hash);

    ResponseJson::ok(()).into_response()
}
