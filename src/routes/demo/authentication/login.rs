use crate::routes::demo::authentication::{jwt_secret, JwtClaims};
use axum::headers::{Header, HeaderValue, SetCookie};
use axum::response::IntoResponse;
use axum::{Form, Json, TypedHeader};
use jsonwebtoken::{Algorithm, EncodingKey};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct Data {
    token: String,
}

#[derive(Serialize)]
pub struct ResponseData {
    status: u8,
    message: &'static str,
    data: Option<Data>,
}

enum ResponseType {
    Success { jwt: String },
    InvalidForm,
    WrongPassword,
}

fn create_response(r#type: ResponseType) -> (TypedHeader<SetCookie>, Json<ResponseData>) {
    let data = match r#type {
        ResponseType::Success { jwt } => ResponseData {
            message: "Login succeeded",
            status: 0,
            data: Some(Data { token: jwt }),
        },
        ResponseType::WrongPassword => ResponseData {
            message: "Wrong username or password",
            status: 1,
            data: None,
        },
        ResponseType::InvalidForm => ResponseData {
            message: "Invalid form",
            status: 2,
            data: None,
        },
    };
    let token = data
        .data
        .as_ref()
        .map(|x| String::from(&x.token))
        .unwrap_or(String::default());

    let values = [HeaderValue::from_str(&format!("token={}", token)).unwrap()];

    let header = TypedHeader(SetCookie::decode(&mut values.iter()).unwrap());
    (header, Json(data))
}

pub async fn authenticate(form: Option<Form<Input>>) -> impl IntoResponse {
    let Some(form) = form else {
        return create_response(ResponseType::InvalidForm);
    };

    let login_match = check_login((&form.username, &form.password));
    if !login_match {
        return create_response(ResponseType::WrongPassword);
    }

    let jwt_secret = jwt_secret();

    let header = jsonwebtoken::Header::new(Algorithm::HS512);
    let issued_at = jsonwebtoken::get_current_timestamp();
    let claim = JwtClaims {
        iat: issued_at,
        exp: issued_at + 3600, /* 1h */
        username: form.username.clone(),
    };
    let jwt =
        jsonwebtoken::encode(&header, &claim, &EncodingKey::from_secret(&jwt_secret)).unwrap();

    create_response(ResponseType::Success { jwt })
}

fn check_login(credential: (&str, &str)) -> bool {
    credential == ("bczhc", "123")
}
