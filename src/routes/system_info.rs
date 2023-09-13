use axum::response::IntoResponse;
use axum::Json;
use bytesize::ByteSize;
use chrono::Duration;
use serde::Serialize;
use sysinfo::{CpuExt, System, SystemExt};

#[derive(Serialize, Debug)]
struct SystemInfo {
    cpu_usage: Vec<String>,
    total_memory: String,
    used_memory: String,
    total_swap: String,
    used_swap: String,
    process_count: usize,
    uptime: String,
    system_name: String,
    kernel_version: String,
    os_version: String,
    host_name: String,
}

fn get_system_info() -> SystemInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_usage = sys
        .cpus()
        .iter()
        .map(|p| format!("{:.2}%", p.cpu_usage()))
        .collect();
    let total_memory = format_size(sys.total_memory());
    let used_memory = format_size(sys.used_memory());
    let total_swap = format_size(sys.total_swap());
    let used_swap = format_size(sys.used_swap());
    let process_count = sys.processes().len();
    let uptime = format_uptime(Duration::seconds(sys.uptime() as i64));
    let system_name = sys.name().unwrap_or_else(|| String::from("Unknown"));
    let kernel_version = sys
        .kernel_version()
        .unwrap_or_else(|| String::from("Unknown"));
    let os_version = sys.os_version().unwrap_or_else(|| String::from("Unknown"));
    let host_name = sys.host_name().unwrap_or_else(|| String::from("Unknown"));

    SystemInfo {
        cpu_usage,
        total_memory,
        used_memory,
        total_swap,
        used_swap,
        process_count,
        uptime,
        system_name,
        kernel_version,
        os_version,
        host_name,
    }
}

fn format_size(size: u64) -> String {
    ByteSize(size).to_string_as(true)
}

fn format_uptime(duration: Duration) -> String {
    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;

    if days > 0 {
        format!("{}d {:02}h {:02}m {:02}s", days, hours, minutes, seconds)
    } else {
        format!("{:02}h {:02}m {:02}s", hours, minutes, seconds)
    }
}

pub async fn system_info() -> impl IntoResponse {
    Json(get_system_info())
}

#[test]
fn display() {
    println!("{:#?}", get_system_info());
}
