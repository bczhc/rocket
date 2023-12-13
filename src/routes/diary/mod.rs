use std::sync::Mutex;

use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use hex::ToHex;
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::routes::diary::database::{Database, DatabaseInfo};
use crate::{lazy_option_initializer, mutex_lock, LazyOption, ResponseJson, CONFIG};

pub mod database;
pub mod diary_book;
pub mod diary_entry;
pub mod session;
pub mod user;

static DATABASE_FILE: Lazy<String> = Lazy::new(|| {
    // mutex_lock!(CONFIG)
    //     .as_ref()
    //     .unwrap()
    //     .app
    //     .diary
    //     .unwrap()
    //     .database_file
    //     .clone()
    // TODO
    String::default()
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
    user_id: u64,
    /// issued at
    iat: u64,
    /// expired at
    exp: u64,
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

pub fn router() -> Router {
    let guard = CONFIG.lock().unwrap();
    if guard.app.diary.is_none() {
        return Router::new();
    }
    Router::new()
        /* --------------- user --------------- */
        .route("/user", post(user::create_user).patch(user::update_user))
        .route("/user/:username", get(user::user_info))
        /* --------------- login --------------- */
        .route("/session", post(session::login))
        /* --------------- diary book --------------- */
        .route(
            "/book",
            post(diary_book::create)
                .patch(diary_book::update)
                .delete(diary_book::delete),
        )
        .route("/books", get(diary_book::list))
        /* --------------- diary entry --------------- */
        .route(
            "/diary/:id",
            get(diary_entry::fetch).delete(diary_entry::delete),
        )
        .route("/diaries", get(diary_entry::list))
}

#[macro_export]
macro_rules! lock_database {
    () => {
        crate::mutex_lock!(crate::routes::diary::DATABASE)
    };
}
