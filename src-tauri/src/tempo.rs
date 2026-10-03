use chrono::{DateTime, Local, TimeZone};

pub fn agora() -> i64 {
    Local::now().timestamp_millis()
}

pub fn quando(ms: i64) -> String {
    let Some(momento) = Local.timestamp_millis_opt(ms).single() else { return String::new() };
    let hoje = Local::now().date_naive();
    let dia = momento.date_naive();
    let hora = momento.format("%H:%M");
    if dia == hoje {
        format!("hoje às {hora}")
    } else if hoje.pred_opt() == Some(dia) {
        format!("ontem às {hora}")
    } else {
        format!("{} às {hora}", momento.format("%d/%m"))
    }
}

pub fn para_texto(ms: i64) -> String {
    Local.timestamp_millis_opt(ms).single().map(|d| d.to_rfc3339()).unwrap_or_default()
}

pub fn de_texto(texto: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(texto).ok().map(|d| d.timestamp_millis())
}
