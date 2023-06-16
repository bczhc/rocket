use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use jsonwebtoken::TokenData;
use serde::{Deserialize, Serialize};

use crate::routes::diary::{JwtClaims, DATABASE};
use crate::security::resolve_jwt;
use crate::{mutex_lock, ResponseJson};

#[derive(Serialize, Deserialize)]
pub struct Form {
    name: String,
}

// with JWT cookie
pub async fn create(cookies: CookieJar, axum::Form(form): axum::Form<Form>) -> impl IntoResponse {
    let jwt = resolve_jwt::<JwtClaims>(&cookies);
    let Some(TokenData { claims, .. }) = jwt else {
        return StatusCode::FORBIDDEN.into_response()
    };

    let database = mutex_lock!(DATABASE);
    database.create_diary_book(&form.name, claims.user_id);

    ResponseJson::ok(()).into()
}
