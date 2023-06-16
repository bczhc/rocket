use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{get_session, lock_database, ResponseJson};

#[derive(Serialize, Deserialize)]
pub struct Form {
    name: String,
}

// with JWT cookie
pub async fn create_diary_book(
    cookies: CookieJar,
    axum::Form(form): axum::Form<Form>,
) -> impl IntoResponse {
    let claims = get_session!(&cookies);

    let database = lock_database!();
    database.create_diary_book(&form.name, claims.user_id);

    ResponseJson::ok(()).into()
}
