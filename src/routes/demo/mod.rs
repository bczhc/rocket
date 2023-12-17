use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

pub mod authentication;

pub fn router() -> Router {
    Router::new()
        .nest("/authentication", authentication::router())
        .route("/", get(demo))
}

async fn demo() -> impl IntoResponse {
    "hello, world"
}
