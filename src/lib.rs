#![feature(try_blocks)]

use std::path::Path;
use std::sync::Mutex;

use figment::providers::{Format, Toml};
use once_cell::sync::Lazy;
use serde::Deserialize;

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
pub struct AppConfig {
    pub ccit_info_file: String,
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
