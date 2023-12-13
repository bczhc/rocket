use std::fs::{create_dir, File};
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use axum::extract::BodyStream;
use axum::response::IntoResponse;
use chrono::Utc;
use futures::StreamExt;
use serde::Serialize;

use crate::routes::app::some_tools::decompress_bzip3;
use crate::{mutex_lock, ResponseJson, CONFIG};

#[derive(Serialize)]
struct Response {}

fn write_to_file(content: &[u8]) -> io::Result<()> {
    let guard = mutex_lock!(CONFIG);
    let path = &guard.app.some_tools.as_ref().unwrap().crash_report_dir;
    let path = Path::new(path);
    if !path.exists() {
        create_dir(path)?;
    }

    let timestamp = Utc::now().timestamp_millis();

    File::options()
        .create(true)
        .create_new(true)
        .read(true)
        .write(true)
        .open(PathBuf::from(path).join(timestamp.to_string()))?
        .write_all(content)?;

    println!("Crash report: {}", timestamp);

    Ok(())
}

pub async fn upload(mut body: BodyStream) -> impl IntoResponse {
    let result: anyhow::Result<()> = try {
        let mut data = Vec::new();
        let mut total_size = 0_usize;
        while let Some(chunk) = body.next().await {
            let chunk = chunk?;
            data.extend_from_slice(&chunk);
            total_size += chunk.len();
            if total_size > 1048576 {
                Err(anyhow!("Data size exceeded"))?;
            }
        }

        let decompressed = decompress_bzip3(&data)?;
        write_to_file(&decompressed)?;
    };

    match result {
        Ok(_) => ResponseJson::ok(()),
        Err(e) => ResponseJson::error(1, format!("{}", e)),
    }
}
