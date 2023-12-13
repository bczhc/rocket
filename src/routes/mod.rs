use axum::Router;

pub mod app;
pub mod ccit_info;
pub mod demo;
pub mod diary;
pub mod random;
pub mod server_network_log;
pub mod system_info;
pub mod text_transfer;

pub fn router() -> Router {
    Router::new()
        .nest("/app", app::router())
        .nest("/demo", demo::router())
        .nest("/server-network-log", server_network_log::router())
}
