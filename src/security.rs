use crate::{mutex_lock, print_flush};
use cfg_if::cfg_if;
use once_cell::sync::Lazy;
use rsa::RsaPrivateKey;
use std::sync::Mutex;

pub static PRIVATE_KEY: Lazy<Mutex<Option<RsaPrivateKey>>> = Lazy::new(|| Mutex::new(None));

pub fn init() {
    print_flush!("Generating private key... ");
    #[allow(clippy::needless_late_init)]
    let private_key;
    cfg_if! {
        if #[cfg(debug_assertions)] {
            private_key = RsaPrivateKey::new(&mut rand::thread_rng(), 1024).unwrap();
        } else {
            use rand::rngs::OsRng;
            private_key = RsaPrivateKey::new(&mut OsRng, 4096).unwrap();
        }
    }
    mutex_lock!(PRIVATE_KEY).replace(private_key);
    println!("done");
}
