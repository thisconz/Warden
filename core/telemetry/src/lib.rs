use std::time::Duration;
use uuid::Uuid;

pub fn log_request(
    method: &str,
    path: &str,
    status: u16,
    latency: Duration,
) {
    let log = serde_json::json!({
        "request_id": Uuid::new_v4().to_string(),
        "method": method,
        "path": path,
        "status": status,
        "latency_ms": latency.as_secs_f64() * 1000.0
    });

    println!("{}", log);
}