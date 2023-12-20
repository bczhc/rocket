use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Form, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::routes::diary::session::validate_session;
use crate::routes::diary::{
    failure_response, generate_password_hash, AuthForm, JwtClaims, ResponseStatus,
};
use crate::{get_session, lock_database, ResponseJson};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub signup_time: u64,
    pub username: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub gender: Gender,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "tag", content = "value")]
pub enum Gender {
    Unknown,
    Male,
    Female,
    Other(String),
}

impl Gender {
    pub fn from_db_int(gender_code: u8, gender_other: Option<String>) -> Gender {
        match (gender_code, gender_other) {
            (0, _) => Gender::Unknown,
            (1, _) => Gender::Male,
            (2, _) => Gender::Female,
            (3, Some(other)) => Gender::Other(other),
            _ => Gender::Unknown,
        }
    }

    pub fn to_db_int(&self) -> (u8, Option<&str>) {
        match self {
            Gender::Unknown => (0, None),
            Gender::Male => (1, None),
            Gender::Female => (2, None),
            Gender::Other(x) => (3, Some(x)),
        }
    }
}

pub async fn user_info(Path(username): Path<String>) -> impl IntoResponse {
    let database = lock_database!();

    let result: Option<UserProfile> = try {
        let user_id = database.query_user_id(&username)?;
        database.query_user_profile(user_id)?
    };
    match result {
        Some(a) => ResponseJson::ok(a).into_response(),
        None => failure_response(ResponseStatus::NoRecord).into_response(),
    }
}

pub async fn me_user_info(cookies: CookieJar) -> impl IntoResponse {
    let c = get_session!(&cookies);

    let database = lock_database!();
    match database.query_user_profile(c.user_id) {
        None => failure_response(ResponseStatus::NoRecord).into_response(),
        Some(x) => ResponseJson::ok(x).into_response(),
    }
}

pub async fn create_user(Form(form): Form<AuthForm>) -> impl IntoResponse {
    let (pw_hash, salt) = generate_password_hash(&form.password);
    let database = lock_database!();

    if database.check_existence(&form.username) {
        // user exists
        return failure_response(ResponseStatus::UserExists).into_response();
    }

    database.add_user(&form.username, &pw_hash, &salt);

    ResponseJson::ok(()).into_response()
}

pub async fn update_user(cookies: CookieJar, Json(form): Json<UserProfile>) -> impl IntoResponse {
    let claims = get_session!(&cookies);

    let database = lock_database!();

    database.update_user_profile(claims.user_id, &form);
    ResponseJson::ok(()).into_response()
}
