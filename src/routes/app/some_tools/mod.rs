use axum::routing::get;
use axum::Router;
use std::io;
use std::io::Cursor;

pub mod crash_report;

pub fn decompress_bzip3(data: &[u8]) -> bzip3::errors::Result<Vec<u8>> {
    let mut reader = Cursor::new(data);
    let mut writer = Cursor::new(Vec::new());
    let mut decoder = bzip3::read::Bz3Decoder::new(&mut reader)?;
    io::copy(&mut decoder, &mut writer).unwrap();
    Ok(writer.into_inner())
}

pub fn router() -> Router {
    Router::new().route("/crash-report", get(crash_report::upload))
}
