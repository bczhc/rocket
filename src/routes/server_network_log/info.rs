use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

use anyhow::anyhow;
use serde::Serialize;

use crate::routes::server_network_log::LogEntry;
use crate::{mutex_lock, ResponseJson, CONFIG};

#[derive(Serialize)]
pub struct Info {
    first: Option<LogEntry>,
    last: Option<LogEntry>,
    count: u64,
}

type R = ResponseJson<Info>;

pub async fn info() -> R {
    let guard = mutex_lock!(CONFIG);
    let log_file = guard.app.server_network_log_file.as_ref().unwrap();
    let result: anyhow::Result<R> = try {
        let file = File::open(log_file)?;
        let reader = BufReader::new(file);

        let mut count = 0_u64;
        let mut lines = reader.lines();
        let first = lines.next();
        if first.is_none() {
            // 0 entry
            return R::ok(Info {
                first: None,
                last: None,
                count: 0,
            });
        }
        count += 1;
        let first = &first.unwrap()?;
        let mut last = first.clone();
        let parse_failed_error = || anyhow!("Parse failed");
        let first = LogEntry::from_str(first).map_err(|_| parse_failed_error())?;

        for line in lines {
            let line = line?;
            last = line;
            count += 1;
        }

        let last = LogEntry::from_str(&last).map_err(|_| parse_failed_error())?;
        R::ok(Info {
            first: Some(first),
            last: Some(last),
            count,
        })
    };
    match result {
        Ok(r) => r,
        Err(e) => R::error(1, e.to_string()),
    }
}
