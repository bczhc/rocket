use rsa::pkcs8::EncodePrivateKey;
use serde::{Deserialize, Serialize};

use crate::mutex_lock;
use crate::security::JWT_SECRET;

pub mod login;
pub mod request;

#[derive(Serialize, Deserialize)]
pub(crate) struct JwtClaims {
    username: String,
    /// issued at
    iat: u64,
    /// expired at
    exp: u64,
}

pub(crate) fn jwt_secret() -> Vec<u8> {
    mutex_lock!(JWT_SECRET).unwrap().into()
}
