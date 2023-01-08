use crate::mutex_lock;
use crate::security::PRIVATE_KEY;
use jsonwebtoken::{Algorithm, EncodingKey};
use rocket::form::Form;
use rocket::serde::json::Json;
use rocket::{post, FromForm};
use rsa::pkcs1::LineEnding;
use rsa::pkcs8::EncodePrivateKey;
use serde::{Deserialize, Serialize};

#[derive(FromForm)]
pub struct Input<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct Data {
    token: String,
}

#[derive(Serialize)]
pub struct Response {
    status: u8,
    message: String,
    data: Option<Data>,
}

#[derive(Serialize, Deserialize)]
struct Claim<'a> {
    username: &'a str,
    /// issued at
    iat: u64,
}

#[post("/authenticate", data = "<form>")]
pub fn authenticate(form: Form<Input>) -> Json<Response> {
    let login_match = check_login((form.username, form.password));
    if !login_match {
        return Json(Response {
            status: 1,
            message: "Wrong username or password".into(),
            data: None,
        });
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

    // TODO: set cookie

    let response = Response {
        status: 0,
        message: "Login succeeded".into(),
        data: Some(Data { token: jwt }),
    };
    Json(response)
}

fn check_login(credential: (&str, &str)) -> bool {
    credential == ("bczhc", "123")
}
