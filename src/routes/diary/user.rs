use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Form;
use serde::Serialize;

use crate::routes::diary::{failure_response, hash_password, AuthForm, ResponseStatus};
use crate::{lock_database, ResponseJson};

#[derive(Serialize)]
pub struct UserProfile {
    pub signup_time: u64,
    pub username: String,
    pub email: Option<String>,
    pub name: Option<String>,
}

pub async fn user_info(Path(username): Path<String>) -> impl IntoResponse {
    let database = lock_database!();

    let result: Option<UserProfile> = try {
        let user_id = database.query_user_id(&username)?;
        database.query_user_profile(user_id)?
    };
    match result {
        Some(a) => ResponseJson::ok(a).into_response(),
        None => failure_response(ResponseStatus::NoRecord).into_response(),
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

pub async fn update_user() -> impl IntoResponse {
    todo!()
}
