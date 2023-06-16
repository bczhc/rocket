use axum::response::IntoResponse;
use axum::Form;
use serde::Serialize;

use crate::routes::diary::{failure_response, hash_password, AuthForm, ResponseStatus, DATABASE};
use crate::{mutex_lock, ResponseJson};

#[derive(Serialize)]
struct ResponseData {}

pub async fn register(Form(form): Form<AuthForm>) -> impl IntoResponse {
    let pw_hash = hash_password(&form.password);
    let database = mutex_lock!(DATABASE);

    if database.check_existence(&form.username, None) {
        // user exists
        return failure_response(ResponseStatus::UserExists).into_response();
    }

    database.add_user(&form.username, &pw_hash);

    ResponseJson::ok(()).into_response()
}
