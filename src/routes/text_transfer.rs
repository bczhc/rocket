use crate::mutex_lock;
use once_cell::sync::Lazy;
use rocket::get;
use rocket::response::content::RawJson;
use serde::Serialize;
use std::sync::Mutex;

static TEXT: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

#[derive(Serialize)]
struct Response<'a> {
    result: &'a str,
}

#[get("/text-transfer?<text>")]
pub fn text(text: Option<&str>) -> RawJson<String> {
    let mut guard = mutex_lock!(TEXT);
    let result_text = match text {
        None => guard.as_str(),
        Some(set_text) => {
            guard.clear();
            guard.push_str(set_text);
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
