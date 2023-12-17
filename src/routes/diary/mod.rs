use std::sync::Mutex;

use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use hex::ToHex;
use once_cell::sync::Lazy;
use rand::distributions::Standard;
use rand::rngs::OsRng;
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};

use crate::routes::diary::database::{Database, DatabaseInfo};
use crate::{lazy_option_initializer, mutex_lock, LazyOption, ResponseJson, CONFIG};

pub mod database;
pub mod diary_book;
pub mod diary_entry;
pub mod session;
pub mod user;

static DATABASE_FILE: Lazy<String> = Lazy::new(|| {
    mutex_lock!(CONFIG)
        .app
        .diary
        .as_ref()
        .expect("Missing config")
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
    user_id: u64,
    /// issued at
    iat: u64,
    /// expired at
    exp: u64,
}

fn random_salt(length: usize) -> String {
    OsRng
        .sample_iter::<u8, _>(Standard)
        .map(char::from)
        .filter(|x| x.is_ascii_alphanumeric() || x.is_ascii_punctuation())
        .take(length)
        .collect()
}

const PASSWORD_SALT_LENGTH: usize = 16;

/// Returns: (password hash, associated salt string)
pub(crate) fn generate_password_hash(password: &str) -> (String, String) {
    let salt = random_salt(PASSWORD_SALT_LENGTH);
    let hash = crate::security::hash_password(password, salt.as_bytes());
    (hash, salt)
}

pub fn init() {}

/// Timestamp in seconds
pub(crate) fn timestamp() -> u64 {
    chrono::Utc::now()
        .timestamp()
        .try_into()
        .expect("Timestamp error")
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
