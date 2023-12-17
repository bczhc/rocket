use axum_extra::extract::CookieJar;
use std::sync::Mutex;

use hex::ToHex;
use jsonwebtoken::{DecodingKey, TokenData, Validation};
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::de::DeserializeOwned;

use crate::routes::demo::authentication::jwt_secret;
use crate::{lazy_option_initializer, mutex_lock, print_flush, LazyOption};

pub static JWT_SECRET: LazyOption<[u8; 300]> = lazy_option_initializer!();

pub fn init() {
    print_flush!("Generating JWT secret... ");
    let mut secret = [0_u8; 300];
    OsRng.fill_bytes(&mut secret);
    mutex_lock!(JWT_SECRET).replace(secret);
    println!("done");
}

pub fn hash_password(pw: &str, salt: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(pw.as_bytes());
    hasher.update(salt);
    hasher.finalize().as_bytes().encode_hex()
}

pub fn resolve_jwt<C: DeserializeOwned>(cookies: &CookieJar) -> Option<TokenData<C>> {
    let Some(token) = cookies.get("token").map(|x| x.value()) else {
        return None;
    };

    let jwt_secret = jwt_secret();
    let Ok(header) = jsonwebtoken::decode_header(token) else {
        return None;
    };
    let result = jsonwebtoken::decode::<C>(
        token,
        &DecodingKey::from_secret(&jwt_secret),
        &Validation::new(header.alg.clone()),
    );
    let Ok(claims) = result else {
        return None;
    };

    Some(claims)
}
