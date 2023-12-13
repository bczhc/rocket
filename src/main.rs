#![feature(decl_macro)]

use std::net::SocketAddr;

use anyhow::anyhow;
use axum::Router;
use clap::Parser;

use web_app::{mutex_lock, read_config, CONFIG};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = web_app::cli::Args::parse();

    if !args.config.exists() {
        return Err(anyhow!(
            "Config file doesn't exist: {}",
            args.config.display()
        ));
    }
    let config = read_config(args.config)?;
    println!("Config: {:?}", config);

    *CONFIG.lock().unwrap() = config;

    start().await?;
    Ok(())
}

fn initialize() {
    web_app::security::init();
    web_app::routes::diary::init();
    web_app::routes::system_info::start_update_thread();
}

async fn start() -> anyhow::Result<()> {
    initialize();

    let (addr, port) = {
        let guard = mutex_lock!(CONFIG);
        let config = &*guard;

        let addr = config
            .server
            .addr
            .clone()
            .unwrap_or_else(|| String::from("0.0.0.0"));
        let port = config.server.port;
        (addr, port)
    };

    let app = router();

    let addr = SocketAddr::new(addr.parse()?, port);
    println!("Server started on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

fn router() -> Router {
    let router = Router::new();
    macro nest_module($($x:tt),+ $(,)?) {
        router
        $(
            .nest(concat!("/", stringify!($x)), web_app::routes:: $x ::router())
        )*
    }

    nest_module!(
        app,
        demo,
        diary,
        server_network_log,
        ccit_info,
        random,
        system_info,
        text_transfer
    )
}
