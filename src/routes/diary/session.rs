use axum::headers::{Header, HeaderValue, SetCookie};
use axum::response::{Html, IntoResponse};
use axum::{Form, TypedHeader};
use axum_extra::extract::CookieJar;
use chrono::Duration;
use jsonwebtoken::{Algorithm, EncodingKey};
use serde::{Deserialize, Serialize};

use crate::routes::demo::authentication::jwt_secret;
use crate::routes::diary::{failure_response, AuthForm, JwtClaims, ResponseStatus};
use crate::security::resolve_jwt;
use crate::{lock_database, ResponseJson};

#[inline]
pub(crate) fn validate_session(cookies: &CookieJar) -> Option<JwtClaims> {
    resolve_jwt::<JwtClaims>(&cookies).map(|x| x.claims)
}

#[macro_export]
macro_rules! get_session {
    ($cookies:expr) => {
        match crate::routes::diary::session::validate_session($cookies) {
            Some(claims) => claims,
            None => {
                return <_ as ::axum::response::IntoResponse>::into_response((
                    ::axum::http::StatusCode::FORBIDDEN,
                    crate::routes::diary::failure_response(
                        crate::routes::diary::ResponseStatus::InvalidSession,
                    ),
                ))
            }
        }
    };
}

#[derive(Serialize)]
pub struct ResponseData {
    jwt: JwtClaims,
}

/// `#[serde(flatten)]` doesn't work now
///
/// https://github.com/nox/serde_urlencoded/issues/33
#[derive(Serialize, Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
    callback: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CallbackExtras {
    succeeded: bool,
    username: Option<String>,
}

pub async fn login(Form(form): Form<LoginForm>) -> impl IntoResponse {
    fn response_html(callback_url: &str, extras: CallbackExtras) -> String {
        let json = serde_json::to_string(&extras).unwrap();
        let redirect_url = format!("{callback_url}?extras={}", urlencoding::encode(&json));
        let redirect_html = include_str!("session-redirect.html").replace('#', &redirect_url);
        redirect_html
    }

    let database = lock_database!();
    let valid = database.verify_password(&form.username, &form.password);

    if !valid {
        return match form.callback {
            None => failure_response(ResponseStatus::AuthenticationFailed).into_response(),
            Some(c) => Html(response_html(
                &c,
                CallbackExtras {
                    succeeded: false,
                    username: None,
                },
            ))
            .into_response(),
        };
    }

    // unwrap: user must exists here
    let user_id = database.query_user_id(&form.username).unwrap();
    drop(database);
    let timestamp = jsonwebtoken::get_current_timestamp();
    let claims = JwtClaims {
        username: form.username.clone(),
        user_id,
        iat: timestamp,
        exp: timestamp + Duration::days(1).num_seconds() as u64,
    };

    let jwt_header = jsonwebtoken::Header::new(Algorithm::HS512);
    let key = EncodingKey::from_secret(&jwt_secret());
    let jwt = jsonwebtoken::encode(&jwt_header, &claims, &key).unwrap();

    let set_cookies = [HeaderValue::from_str(&format!("token={}", jwt)).unwrap()];
    let header = TypedHeader(SetCookie::decode(&mut set_cookies.iter()).unwrap());

    match form.callback {
        None => {
            let data = ResponseData {
                jwt: claims.clone(),
            };
            (header, ResponseJson::ok(data)).into_response()
        }
        Some(c) => {
            let extras = CallbackExtras {
                succeeded: true,
                username: Some(claims.username),
            };
            (header, Html(response_html(&c, extras))).into_response()
        }
    }
}
