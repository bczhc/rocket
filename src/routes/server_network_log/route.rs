use axum::extract::Query;
use axum::response::IntoResponse;
use axum::Json;

use crate::routes::server_network_log::{
    compress_entries, search_entry_range, search_entry_single, Input, Mode, ResponseData,
};

pub async fn get(query: Option<Query<Input>>) -> axum::response::Response {
    let mode: Option<Mode> = try {
        let query = query?;
        if query.time.contains("..") {
            let mut split = query.time.split("..");
            let from = split.next()?.parse::<u64>().ok()?;
            let to = split.next()?.parse::<u64>().ok()?;
            Mode::Range(from, to)
        } else {
            let timestamp = query.time.parse::<u64>().ok()?;
            Mode::Single(timestamp)
        }
    };
    let Some(mode) = mode else {
        return Json(ResponseData {
            status: 1,
            message: "Invalid query".into(),
            data: None,
        }).into_response();
    };

    let result: anyhow::Result<axum::response::Response> = try {
        match mode {
            Mode::Single(timestamp) => {
                let entry = search_entry_single(timestamp)?;
                Json(ResponseData {
                    status: 0,
                    message: "OK".into(),
                    data: Some(entry),
                })
                .into_response()
            }
            Mode::Range(from, to) => {
                let entries = search_entry_range(from, to)?;
                let Ok(data) = compress_entries(&entries) else {
                    return Json(ResponseData {
                        status: 1,
                        message: "Compression failed".into(),
                        data: None,
                    }).into_response();
                };
                data.into_response()
            }
        }
    };
    match result {
        Ok(r) => r,
        Err(e) => Json(ResponseData {
            status: 1,
            message: e.to_string(),
            data: None,
        })
        .into_response(),
    }
}
