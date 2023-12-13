use crate::CONFIG;
use axum::Router;

pub mod some_tools;

pub fn router() -> Router {
    let guard = CONFIG.lock().unwrap();
    if guard.app.some_tools.is_none() {
        return Router::new();
    }
    Router::new().nest("/some-tools", some_tools::router())
}
