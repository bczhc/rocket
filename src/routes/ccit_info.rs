use std::fs::File;
use std::io;
use std::io::Read;

use serde_json::Value;

use crate::{mutex_lock, ResponseJson, CONFIG};

pub async fn get_info() -> ResponseJson<Value> {
    let guard = mutex_lock!(CONFIG);
    let config = guard.as_ref().unwrap();
    let ccit_info_file = &config.app.ccit_info_file;

    let read: Result<String, io::Error> = try {
        let mut file = File::open(ccit_info_file)?;
        let mut read = String::new();
        file.read_to_string(&mut read)?;
        read
    };
    let response = match read {
        Ok(r) => {
            let value: Result<Value, _> = serde_json::from_str(&r);

            let value = match value {
                Ok(v) => v,
                Err(e) => {
                    return ResponseJson::error(format!("{}", e));
                }
            };

            ResponseJson::ok(value)
        }
        Err(e) => ResponseJson::error(format!("{}", e)),
    };
    response
}
