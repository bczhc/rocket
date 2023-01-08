use crate::mutex_lock;
use crate::security::PRIVATE_KEY;
use once_cell::sync::Lazy;
use rsa::pkcs8::{EncodePrivateKey, LineEnding};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

pub mod login;
pub mod request;

static JWT_SECRET: Lazy<Mutex<Option<Vec<u8>>>> = Lazy::new(|| Mutex::new(None));

#[derive(Serialize, Deserialize)]
pub(crate) struct JwtClaims {
    username: String,
    /// issued at
    iat: u64,
    /// expired at
    exp: u64,
}

pub(crate) fn jwt_secret() -> Vec<u8> {
    // here just use the RSA private key as the JWT signing secret
    let generate_secret = || {
        let pem = mutex_lock!(PRIVATE_KEY)
            .as_ref()
            .unwrap()
            .to_pkcs8_pem(LineEnding::LF)
            .unwrap();
        pem.as_bytes().into()
    };

    // due to every time `generate_secret` generates the different key, cache it and make it singleton
    let mut guard = mutex_lock!(JWT_SECRET);
    if guard.is_none() {
        guard.replace(generate_secret());
    }
    guard.as_ref().unwrap().clone()
}
