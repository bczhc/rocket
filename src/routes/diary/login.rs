use crate::routes::authentication_demo::jwt_secret;
use crate::routes::diary::{
    failure_response, hash_password, AuthForm, JwtClaims, ResponseStatus, DATABASE,
};
use crate::{mutex_lock, ResponseJson};
use axum::headers::{Header, HeaderValue, SetCookie};
use axum::response::IntoResponse;
use axum::{Form, TypedHeader};
use chrono::Duration;
use jsonwebtoken::{Algorithm, EncodingKey};
use serde::Serialize;

#[derive(Serialize)]
pub struct ResponseData {
    jwt: JwtClaims,
}

pub async fn login(Form(form): Form<AuthForm>) -> impl IntoResponse {
    let database = mutex_lock!(DATABASE);
    let pw_hash = hash_password(&form.password);
    let valid = database.check_existence(&form.username, Some(&pw_hash));

    if !valid {
        return failure_response(ResponseStatus::AuthenticationFailed).into_response();
    }

    let user_id = database.query_user_id(&form.username).unwrap();
    drop(database);

    let timestamp = jsonwebtoken::get_current_timestamp();
    let claims = JwtClaims {
        username: form.username.clone(),
        user_id,
        iat: timestamp,
        exp: timestamp + Duration::days(1).num_seconds() as u64,
    };

    let jwt_header = jsonwebtoken::Header::new(Algorithm::HS512);
    let key = EncodingKey::from_secret(&jwt_secret());
    let jwt = jsonwebtoken::encode(&jwt_header, &claims, &key).unwrap();

    let set_cookies = [HeaderValue::from_str(&format!("token={}", jwt)).unwrap()];
    let header = TypedHeader(SetCookie::decode(&mut set_cookies.iter()).unwrap());

    let data = ResponseData { jwt: claims };
    (header, ResponseJson::ok(data)).into_response()
}
