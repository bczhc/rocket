use axum::Json;
use std::fs::File;
use std::io;
use std::io::Read;

use serde::Serialize;

use crate::{mutex_lock, CONFIG};

#[derive(Serialize)]
pub struct Response {
    status: u32,
    data: Option<serde_json::Value>,
}

pub async fn get_info() -> Json<Response> {
    let guard = mutex_lock!(CONFIG);
    let config = guard.as_ref().unwrap();
    let ccit_info_file = &config.app.ccit_info_file;

    let read: Result<String, io::Error> = try {
        let mut file = File::open(ccit_info_file)?;
        let mut read = String::new();
        file.read_to_string(&mut read)?;
        read
    };
    let response = match read {
        Ok(r) => {
            let value: serde_json::Value = serde_json::from_str(&r).unwrap();
            Response {
                status: 0,
                data: Some(value),
            }
        }
        Err(_) => Response {
            status: 1,
            data: None,
        },
    };
    Json(response)
}
