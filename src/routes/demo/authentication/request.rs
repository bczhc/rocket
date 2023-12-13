use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::CookieJar;
use serde::Serialize;

use crate::routes::demo::authentication::JwtClaims;
use crate::security::resolve_jwt;
use crate::ResponseJson;

#[derive(Serialize)]
pub struct ResponseData {
    username: Option<String>,
}

pub async fn request(cookies: CookieJar) -> impl IntoResponse {
    let forbidden_status = || (StatusCode::FORBIDDEN, Json(ResponseData { username: None }));

    let jwt = resolve_jwt::<JwtClaims>(&cookies);
    let Some(claims) = jwt else {
        return forbidden_status().into_response();
    };

    ResponseJson::ok(ResponseData {
        username: Some(claims.claims.username),
    })
    .into_response()
}
