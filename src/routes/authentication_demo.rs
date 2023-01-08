use jsonwebtoken::{Algorithm, EncodingKey};
use rocket::form::Form;
use rocket::http::Cookie;
use rocket::{post, FromForm, Responder};
use rsa::pkcs1::LineEnding;
use rsa::pkcs8::EncodePrivateKey;
use serde::{Deserialize, Serialize};

use crate::mutex_lock;
use crate::security::PRIVATE_KEY;

#[derive(FromForm)]
pub struct Input<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct Data<'a> {
    token: &'a str,
}

#[derive(Serialize)]
pub struct ResponseData<'a, 'b> {
    status: u8,
    message: &'a str,
    data: Option<Data<'b>>,
}

#[derive(Serialize, Deserialize)]
struct Claim<'a> {
    username: &'a str,
    /// issued at
    iat: u64,
}

#[derive(Responder)]
#[response(content_type = "json")]
pub struct Response {
    json: String,
    token_cookie: Cookie<'static>,
}

enum ResponseType<'a> {
    Success { jwt: &'a str },
    InvalidForm,
    WrongPassword,
}

impl Response {
    fn new(r#type: ResponseType) -> Self {
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
        Self {
            json: serde_json::to_string(&data).unwrap(),
            token_cookie: Cookie::new(
                "token",
                data.data
                    .map(|x| String::from(x.token))
                    .unwrap_or(String::default()),
            ),
        }
    }
}

#[post("/authenticate", data = "<form>")]
pub fn authenticate(form: Option<Form<Input>>) -> Response {
    let Some(form) = form else {
        return Response::new(ResponseType::InvalidForm)
    };

    let login_match = check_login((form.username, form.password));
    if !login_match {
        return Response::new(ResponseType::WrongPassword);
    }

    let pem = mutex_lock!(PRIVATE_KEY)
        .as_ref()
        .unwrap()
        .to_pkcs8_pem(LineEnding::LF)
        .unwrap();
    let jwt_secret = pem.as_bytes();

    let header = jsonwebtoken::Header::new(Algorithm::HS512);
    let claim = Claim {
        iat: jsonwebtoken::get_current_timestamp(),
        username: form.username,
    };
    let jwt = jsonwebtoken::encode(&header, &claim, &EncodingKey::from_secret(jwt_secret)).unwrap();

    Response::new(ResponseType::Success { jwt: &jwt })
}

fn check_login(credential: (&str, &str)) -> bool {
    credential == ("bczhc", "123")
}
