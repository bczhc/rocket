use jsonwebtoken::{DecodingKey, Validation};
use rocket::get;
use rocket::http::{CookieJar, Status};
use rocket::response::status;
use rocket::serde::json::Json;
use serde::Serialize;

use crate::routes::authentication_demo::{jwt_secret, JwtClaims};

#[derive(Serialize)]
pub struct ResponseData {
    username: Option<String>,
}

#[get("/request")]
pub fn request(cookies: &CookieJar<'_>) -> status::Custom<Json<ResponseData>> {
    let forbidden_status =
        || status::Custom(Status::Forbidden, Json(ResponseData { username: None }));

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

    status::Custom(
        Status::Ok,
        Json(ResponseData {
            username: Some(claims.claims.username),
        }),
    )
}
