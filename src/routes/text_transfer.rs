use crate::mutex_lock;
use once_cell::sync::Lazy;
use rocket::form::Form;
use rocket::response::content::RawJson;
use rocket::{post, FromForm};
use serde::Serialize;
use std::mem;
use std::sync::Mutex;

static TEXT: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

#[derive(Serialize)]
struct Response<'a> {
    result: &'a str,
}

#[derive(FromForm)]
pub struct Input<'a> {
    text: &'a str,
}

#[post("/text-transfer", data = "<text>")]
pub fn text(text: Option<Form<Input>>) -> RawJson<String> {
    let mut guard = mutex_lock!(TEXT);
    let result_text = match text {
        None => guard.as_str(),
        Some(input) => {
            *guard = input.text.into();
            guard.as_str()
        }
    };
    RawJson(
        serde_json::to_string(&Response {
            result: result_text,
        })
        .unwrap(),
    )
}
