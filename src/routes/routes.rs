use axum::response::IntoResponse;

use crate::{mutex_lock, ROUTES};

pub async fn list() -> impl IntoResponse {
    let mut response = String::new();
    let guard = mutex_lock!(ROUTES);
    use std::fmt::Write;
    for x in guard.iter() {
        writeln!(&mut response, "{}", x).unwrap();
    }
    drop(guard);
    response
}
