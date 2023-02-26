use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::CookieJar;
use jsonwebtoken::{DecodingKey, Validation};
use serde::Serialize;

use crate::routes::authentication_demo::{jwt_secret, JwtClaims};

#[derive(Serialize)]
pub struct ResponseData {
    username: Option<String>,
}

pub async fn request(cookies: CookieJar) -> (StatusCode, Json<ResponseData>) {
    let forbidden_status = || (StatusCode::FORBIDDEN, Json(ResponseData { username: None }));

    let Some(token) = cookies.get("token").map(|x| x.value()) else {
        return forbidden_status()
    };

    let jwt_secret = jwt_secret();
    let Ok(header) = jsonwebtoken::decode_header(token) else {
        return forbidden_status()
    };
    let Ok(claims) = jsonwebtoken::decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(&jwt_secret),
        &Validation::new(header.alg.clone()),
    ) else {
        return forbidden_status()
    };

    (
        StatusCode::OK,
        Json(ResponseData {
            username: Some(claims.claims.username),
        }),
    )
}
