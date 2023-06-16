use std::sync::Mutex;

use axum::extract::Query;
use axum::response::IntoResponse;
use hex::ToHex;
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::routes::diary::database::{Database, DatabaseInfo};
use crate::{lazy_option_initializer, mutex_lock, LazyOption, ResponseJson, CONFIG};

pub mod database;
pub mod diaries;
pub mod diary_books;
pub mod session;
pub mod users;

static DATABASE_FILE: Lazy<String> = Lazy::new(|| {
    mutex_lock!(CONFIG)
        .as_ref()
        .unwrap()
        .app
        .diary
        .database_file
        .clone()
});
static DATABASE: Lazy<Mutex<Database>> =
    Lazy::new(|| Mutex::new(Database::new(&*DATABASE_FILE).unwrap()));

#[derive(Deserialize)]
pub struct FetchQuery {
    pub diary_id: String,
    pub date: u32,
}

#[derive(Deserialize)]
pub struct AuthForm {
    pub username: String,
    pub password: String,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ResponseStatus {
    Ok = 0,
    UserExists,
    AuthenticationFailed,
    NoRecord,
}

impl ResponseStatus {
    pub fn message(&self) -> &'static str {
        match self {
            ResponseStatus::Ok => "User creation succeeded",
            ResponseStatus::UserExists => "User already exists",
            ResponseStatus::AuthenticationFailed => "Authentication failed",
            ResponseStatus::NoRecord => "No record",
        }
    }
}

pub(crate) fn failure_response(status: ResponseStatus) -> impl IntoResponse {
    ResponseJson::<()>::error(status as u32, status.message())
}

#[derive(Serialize, Deserialize)]
pub(crate) struct JwtClaims {
    username: String,
    /// issued at
    iat: u64,
    /// expired at
    exp: u64,
}

pub async fn fetch(Query(query): Query<FetchQuery>) -> impl IntoResponse {
    todo!()
}

type Salt = [u8; 16];
static SALT: LazyOption<Salt> = lazy_option_initializer!();

pub(crate) fn hash_password(password: &str) -> String {
    let guard = mutex_lock!(SALT);
    crate::security::hash_password(password, guard.as_ref().unwrap())
}

pub fn init() {
    let mut salt = Salt::default();
    OsRng.fill_bytes(&mut salt);
    let database = mutex_lock!(DATABASE);

    let info = database.fetch_info();
    if info.is_none() {
        database.update_info(&DatabaseInfo {
            hash_salt: salt.encode_hex(),
        });
    }
    let info = database.fetch_info().unwrap();
    let salt = hex::decode(&info.hash_salt).expect("Malformed salt string");
    mutex_lock!(SALT).replace(salt.try_into().expect("Wrong salt length"));
}

/// Timestamp in milliseconds
pub(crate) fn timestamp() -> u64 {
    chrono::Utc::now()
        .timestamp_millis()
        .try_into()
        .expect("Timestamp error")
}

pub(crate) fn generate_id() -> u64 {
    OsRng.next_u64()
}

#[macro_export]
macro_rules! lock_database {
    () => {
        crate::mutex_lock!(crate::routes::diary::DATABASE)
    };
}
