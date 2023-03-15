#![feature(try_blocks)]

use axum::response::{IntoResponse, Response};
use axum::Json;
use std::path::Path;
use std::sync::Mutex;

use figment::providers::{Format, Toml};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub mod blake3;
pub mod routes;
pub mod security;

pub static CONFIG: Lazy<Mutex<Option<Config>>> = Lazy::new(|| Mutex::new(None));

#[macro_export]
macro_rules! mutex_lock {
    ($e:expr) => {
        $e.lock().unwrap()
    };
}

#[macro_export]
macro_rules! print_flush {
    () => {
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    };
    ($($arg:tt)*) => {
        print!($($arg)*);
        print_flush!();
    };
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    pub ccit_info_file: String,
    pub server_network_log_file: String,
}

#[derive(Deserialize, Debug)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
}

pub fn read_config(config_path: impl AsRef<Path>) -> Config {
    let config: Config = figment::Figment::new()
        .merge(Toml::file(config_path))
        .extract()
        .unwrap();
    config
}

#[derive(Serialize)]
pub struct ResponseJson<T>
where
    T: Serialize,
{
    status: u32,
    message: Option<String>,
    data: Option<T>,
}

impl<T> ResponseJson<T>
where
    T: Serialize,
{
    pub fn error<S>(message: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            status: 1,
            data: None,
            message: Some(message.into()),
        }
    }

    pub fn ok(data: T) -> Self {
        Self {
            status: 0,
            message: Some("OK".into()),
            data: Some(data),
        }
    }
}

impl<T> IntoResponse for ResponseJson<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}
