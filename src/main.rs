use axum::headers::{Header, HeaderValue};
use axum::routing::{get, post};
use axum::{headers, Router, TypedHeader};
use clap::{value_parser, Arg, Command};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = Command::new("web-app")
        .arg(
            Arg::new("port")
                .default_value("8080")
                .short('p')
                .long("port")
                .value_parser(value_parser!(u16)),
        )
        .get_matches();

    let port = *matches.get_one::<u16>("port").unwrap();
    start(port).await?;
    Ok(())
}

async fn start(port: u16) -> anyhow::Result<()> {
    web_app::security::init();

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
