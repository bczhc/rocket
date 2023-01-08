use crate::{mutex_lock, print_flush};
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use rsa::RsaPrivateKey;
use std::sync::Mutex;

pub static PRIVATE_KEY: Lazy<Mutex<Option<RsaPrivateKey>>> = Lazy::new(|| Mutex::new(None));

pub fn init() {
    print_flush!("Generating private key...");
    let private_key = RsaPrivateKey::new(&mut OsRng, 4096).unwrap();
    mutex_lock!(PRIVATE_KEY).replace(private_key);
    println!(" done");
}
