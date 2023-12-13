use axum::Router;

pub mod some_tools;

pub fn router() -> Router {
    Router::new()
        .nest("/some-tools", some_tools::router())
}
