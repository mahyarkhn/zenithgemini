#![allow(dead_code)]
use std::time::{Duration, SystemTime};

pub fn unix_timestamp() -> i64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64
}

pub fn system_time_to_unix_timestamp(time: SystemTime) -> i64 {
    time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64
}


fn unix_timestamp_to_system_time(timestamp: i64) -> SystemTime {
    let duration = Duration::from_secs(timestamp as u64); // Convert i64 to u64
    std::time::UNIX_EPOCH + duration
}
