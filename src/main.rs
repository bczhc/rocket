use std::net::SocketAddr;

use axum::headers::{Header, HeaderValue};
use axum::routing::{get, post};
use axum::{headers, Router, TypedHeader};
use clap::{Arg, Command, ValueHint};

use web_app::{mutex_lock, read_config, CONFIG, ROUTES};

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
    let mut app = Router::new();

    let mut routes_guard = mutex_lock!(ROUTES);

    macro_rules! add_route {
        (GET $x:expr, $p:expr) => {
            app = app.route($x, get($p));
            routes_guard.push(format!("GET {} {}", $x, stringify!($p)));
        };
        (POST $x:expr, $p:expr) => {
            app = app.route($x, post($p));
            routes_guard.push(format!("POST {} {}", $x, stringify!($p)));
        };
    }
    add_route!(GET "/login", routes::authentication_demo::login::authenticate);
    add_route!(POST "/text-transfer", routes::text_transfer::text);
    add_route!(GET "/request", routes::authentication_demo::request::request);
    add_route!(GET "/ccit-info", routes::ccit_info::get_info);
    add_route!(GET "/server-network-log/get", routes::server_network_log::route::get);
    add_route!(GET "/server-network-log/info", routes::server_network_log::info::info);
    add_route!(GET "/app/some-tools/crash-report", routes::app::some_tools::crash_report::upload);
    add_route!(GET "/random", routes::random::stream_random);
    add_route!(GET "/routes", routes::routes::list);
    add_route!(GET "/test", test_route);

    drop(routes_guard);

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
