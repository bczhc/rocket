use std::net::SocketAddr;

use axum::extract::Multipart;
use axum::headers::{Header, HeaderValue};
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

fn initialize() {
    web_app::security::init();
    web_app::routes::diary::init();
}

async fn start() -> anyhow::Result<()> {
    initialize();

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
        ($verb:ident $x:literal, $p:expr) => {
            ::paste::paste! {
                let verb_name = stringify!($verb);
                app = app.route($x, ::axum::routing::[<$verb:lower>]($p));
                routes_guard.push(format!("{} {} {}", verb_name, $x, stringify!($p)));
            }
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
    add_route!(POST "/test", test_route);

    // log in
    add_route!(POST "/diary/session", routes::diary::session::login);
    // create user
    add_route!(POST "/diary/user", routes::diary::users::create_user);
    // update user profile
    // TODO
    add_route!(PATCH "/diary/user", routes::diary::users::create_user);
    // fetch diary
    add_route!(GET "/diary/diaries/:id", routes::diary::fetch);
    // get user profile
    add_route!(GET "/diary/user/:username", routes::diary::users::user_info);
    // delete diary
    add_route!(DELETE "/diary/diary/:id", routes::diary::users::user_info);
    // create a diary book
    add_route!(POST "/diary/books", routes::diary::diary_books::create_diary_book);
    // list diary books of the session
    // TODO
    add_route!(GET "/diary/books", routes::diary::users::user_info);
    // list diaries of a diary book
    // TODO
    add_route!(GET "/diary/diaries", routes::diary::users::user_info);
    // delete a diary book
    // TODO
    add_route!(GET "/diary/books/:id", routes::diary::users::user_info);

    drop(routes_guard);

    let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), port);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn test_route(mut multipart: Multipart) {
    while let Some(f) = multipart.next_field().await.unwrap() {
        // println!("{:?}", f);
        let name = f.name().map(|x| x.to_string());
        println!("{:?}", f.headers().get("Content-Transfer-EncodinG"));
        let bytes = f.bytes().await.unwrap().to_vec();
        println!("{:?}", (name, bytes));
    }
}
