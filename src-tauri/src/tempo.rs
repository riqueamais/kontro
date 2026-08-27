use chrono::{DateTime, Local, TimeZone};

pub fn agora() -> i64 {
    Local::now().timestamp_millis()
}

pub fn para_texto(ms: i64) -> String {
    Local
        .timestamp_millis_opt(ms)
        .single()
        .map(|d| d.to_rfc3339())
        .unwrap_or_default()
}

pub fn de_texto(texto: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(texto)
        .ok()
        .map(|d| d.timestamp_millis())
}
