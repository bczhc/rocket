use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

pub mod authentication;
pub mod html2canvas;

pub fn router() -> Router {
    Router::new()
        .nest("/authentication", authentication::router())
        .nest("/html2canvas", html2canvas::router())
        .route("/", get(demo))
}

async fn demo() -> impl IntoResponse {
    "hello, world"
}
