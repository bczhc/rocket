use std::pin::Pin;
use std::task::{Context, Poll};
use std::{cmp, slice};

use axum::body::StreamBody;
use axum::extract::Query;
use axum::headers::HeaderMap;
use axum::http::header;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::Deserialize;
use tokio::io::{AsyncRead, ReadBuf};
use tokio_util::io::ReaderStream;

#[derive(Deserialize, Debug)]
pub struct Form {
    size: usize,
}

struct RandomReader {
    limit: usize,
    filled: usize,
}

impl RandomReader {
    fn new(limit: usize) -> Self {
        Self { limit, filled: 0 }
    }
}

impl AsyncRead for RandomReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        unsafe {
            let amount = cmp::min(self.limit - self.filled, buf.remaining());
            let end = buf.filled().len() + amount;

            let unfilled = buf.unfilled_mut();
            OsRng.fill_bytes(slice::from_raw_parts_mut(
                unfilled.as_mut_ptr().cast(),
                unfilled.len(),
            ));

            self.filled += amount;
            if end > buf.initialized().len() {
                buf.assume_init(end);
            }

            buf.set_filled(end);
        }
        Poll::Ready(Ok(()))
    }
}

pub async fn stream_random(Query(query): Query<Form>) -> impl IntoResponse {
    let random_reader = RandomReader::new(query.size);
    let response = StreamBody::new(ReaderStream::new(random_reader));

    let mut header_map = HeaderMap::new();
    header_map.insert(header::CONTENT_LENGTH, query.size.into());

    (header_map, response)
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(stream_random))
}
