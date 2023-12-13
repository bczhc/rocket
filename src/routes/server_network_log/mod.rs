use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Cursor, Write};
use std::str::FromStr;

use anyhow::anyhow;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use bzip3::write::Bz3Encoder;
use serde::{Deserialize, Serialize};

use crate::CONFIG;

pub mod info;
pub mod route;

#[derive(Deserialize, Debug)]
pub struct Input {
    time: String,
    bzip3: Option<bool>,
}

enum Mode {
    Single(u64),
    Range(u64, u64),
}

#[derive(Serialize)]
pub struct ResponseData {
    status: u32,
    message: String,
    data: Option<LogEntry>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    timestamp: u64,
    rx_size: u64,
    tx_size: u64,
}

impl FromStr for LogEntry {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(' ');
        let result: Option<Self> = try {
            let f1 = split.next()?.parse::<u64>().ok()?;
            let f2 = split.next()?.parse::<u64>().ok()?;
            let f3 = split.next()?.parse::<u64>().ok()?;
            Self {
                timestamp: f1,
                rx_size: f2,
                tx_size: f3,
            }
        };
        match result {
            None => Err(()),
            Some(r) => Ok(r),
        }
    }
}

pub fn read_entries() -> anyhow::Result<Vec<LogEntry>> {
    let guard = CONFIG.lock().unwrap();
    let network_log_file = &guard.as_ref().unwrap().app.server_network_log_file;
    let file = File::open(network_log_file)?;
    drop(guard);

    let mut entries = Vec::new();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        let mut split = line.split(' ');
        let Some((timestamp, rx_size, tx_size)) = (try {
            let timestamp = split.next()?.parse::<u64>().ok()?;
            let rx_size = split.next()?.parse::<u64>().ok()?;
            let tx_size = split.next()?.parse::<u64>().ok()?;
            (timestamp, rx_size, tx_size)
        }) else {
            return Err(anyhow!("Failed to parse log file"));
        };
        let last_timestamp = entries
            .last()
            .map(|x: &LogEntry| x.timestamp)
            .unwrap_or_default();
        if timestamp < last_timestamp {
            return Err(anyhow!("Timestamp goes backwards"));
        }
        entries.push(LogEntry {
            timestamp,
            rx_size,
            tx_size,
        });
    }
    Ok(entries)
}

fn search_index(entries: &Vec<LogEntry>, timestamp: u64) -> usize {
    match entries.binary_search_by(|x| x.timestamp.cmp(&timestamp)) {
        Ok(index) => index,
        Err(index) => {
            if index == 0 {
                0
            } else {
                index - 1
            }
        }
    }
}

pub fn search_entry_range(from: u64, to: u64) -> anyhow::Result<Vec<LogEntry>> {
    let mut entries = read_entries()?;
    if entries.is_empty() {
        return Err(anyhow!("Empty entry list"));
    }

    let from_index = search_index(&entries, from);
    let to_index = search_index(&entries, to);

    if from_index >= to_index {
        return Ok(vec![entries[from_index].clone()]);
    }

    let mut part = entries.split_off(from_index);
    part.truncate(to_index - from_index + 1);
    Ok(part)
}

pub fn search_entry_single(timestamp: u64) -> anyhow::Result<LogEntry> {
    let entries = read_entries()?;
    if entries.is_empty() {
        return Err(anyhow!("Empty entry list"));
    }
    let index = search_index(&entries, timestamp);
    Ok(entries[index].clone())
}

fn write_entries_text<W>(entries: &Vec<LogEntry>, writer: &mut W)
where
    W: std::fmt::Write,
{
    for e in entries {
        writeln!(writer, "{} {} {}", e.timestamp, e.rx_size, e.tx_size).unwrap();
    }
}

pub fn compress_entries(entries: &Vec<LogEntry>) -> anyhow::Result<Vec<u8>> {
    let mut cursor = Cursor::new(Vec::new());
    let mut encoder = Bz3Encoder::new(&mut cursor, 1048576)?;
    let mut writer = BufWriter::new(&mut encoder);

    for e in entries {
        writeln!(&mut writer, "{} {} {}", e.timestamp, e.rx_size, e.tx_size)?;
    }
    drop(writer);
    drop(encoder);
    Ok(cursor.into_inner())
}

pub fn router() -> Router {
    Router::new().route("/", get(route::get))
}
