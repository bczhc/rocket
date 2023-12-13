use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};

use axum::extract::Query;
use axum::headers::{HeaderMap, HeaderValue};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use serde::Deserialize;

use crate::{mutex_lock, ResponseJson, CONFIG};

#[derive(Deserialize)]
pub struct QueryData {
    pub text: String,
}

pub async fn generate_image(query: Option<Query<QueryData>>) -> impl IntoResponse {
    if query.is_none() {
        return ResponseJson::<()>::error(1, "Query: url?text=文本").into_response();
    }
    let query = query.unwrap().0;

    let guard = mutex_lock!(CONFIG);
    let config = guard.as_ref().unwrap();
    let port = config.app.html2canvas_demo_port;
    drop(guard);

    let text = &query.text;

    let result: anyhow::Result<Response> = try {
        let image_path = mktemp::Temp::new_file()?;
        let image_path = image_path.to_str().expect("Should be UTF-8-valid");
        let command_json = serde_json::to_string(&[text.as_str(), image_path]).unwrap();

        let mut stream = TcpStream::connect(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port))?;
        use std::io::Write;
        writeln!(&mut stream, "{}", command_json)?;
        stream.flush()?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        assert!(line.starts_with("Input a line"));
        line.clear();
        reader.read_line(&mut line)?;
        assert!(line.starts_with("Done"));

        // read to memory. for storage limit, the image file should
        // be deleted on this request end.
        let mut image_data = Vec::new();
        File::open(image_path)?.read_to_end(&mut image_data)?;
        fs::remove_file(image_path)?;

        let mut header_map = HeaderMap::new();
        header_map.insert(header::CONTENT_LENGTH, image_data.len().into());
        header_map.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::IMAGE_PNG.as_ref()),
        );

        (header_map, image_data).into_response()
    };

    return match result {
        Ok(r) => r,
        Err(e) => ResponseJson::<()>::error(1, format!("{}", e)).into_response(),
    };
}

pub fn router() -> Router {
    Router::new().route("/image", get(generate_image))
}
