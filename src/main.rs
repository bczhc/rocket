use std::net::SocketAddr;

use axum::headers::{Header, HeaderValue};
use axum::routing::{get, post};
use axum::{headers, Router, TypedHeader};
use clap::{Arg, Command, ValueHint};

use web_app::{mutex_lock, read_config, CONFIG};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = Command::new("web-app")
        .arg(
            Arg::new("config")
                .default_value("./config.toml")
                .short('c')
                .long("config")
                .value_hint(ValueHint::FilePath),
        )
        .get_matches();

    let config_path = matches.get_one::<String>("config").unwrap();

    let config = read_config(config_path);
    println!("Config: {:?}", config);

    mutex_lock!(CONFIG).replace(config);

    start().await?;
    Ok(())
}

async fn start() -> anyhow::Result<()> {
    web_app::security::init();

    let port = {
        let guard = mutex_lock!(CONFIG);
        let config = guard.as_ref().unwrap();
        config.server.port
    };

    println!("Server started");
    use web_app::routes;
    let app = Router::new()
        .route("/text-transfer", post(routes::text_transfer::text))
        .route(
            "/login",
            post(routes::authentication_demo::login::authenticate),
        )
        .route(
            "/request",
            get(routes::authentication_demo::request::request),
        )
        .route("/ccit-info", get(routes::ccit_info::get_info))
        .route("/server-network-log", get(routes::server_network_log::route::get))
        .route("/test", get(test_route));

    let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), port);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn test_route() -> (TypedHeader<headers::SetCookie>, &'static str) {
    let values = vec![
        HeaderValue::from_static("A=1"),
        HeaderValue::from_static("B=2"),
    ];
    (
        TypedHeader(headers::SetCookie::decode(&mut values.iter()).unwrap()),
        "hello",
    )
}
