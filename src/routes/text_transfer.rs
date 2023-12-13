use std::sync::Mutex;

use axum::response::IntoResponse;
use axum::{Form, Json, Router};
use axum::routing::get;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::mutex_lock;

static TEXT: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

#[derive(Serialize)]
struct Response {
    result: String,
}

#[derive(Deserialize, Debug)]
pub struct Input {
    text: String,
}

pub async fn text(text: Option<Form<Input>>) -> impl IntoResponse {
    let mut guard = mutex_lock!(TEXT);
    let result_text = match text {
        None => (*guard).clone(),
        Some(input) => {
            *guard = input.text.clone();
            (*guard).clone()
        }
    };
    Json(Response {
        result: result_text,
    })
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(text).post(text))
}
