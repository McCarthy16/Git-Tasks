//! Terminal rendering helpers: aligned tables, key-value blocks, timestamps.

use std::time::{Duration, UNIX_EPOCH};

use serde_json::Value;

/// Print an aligned table, or `(none)` when there are no rows.
pub fn table(headers: &[&str], rows: Vec<Vec<String>>) {
    if rows.is_empty() {
        println!("(none)");
        return;
    }
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(cell.len());
        }
    }
    let line = |cells: &[&str]| {
        let joined = cells
            .iter()
            .enumerate()
            .map(|(i, cell)| format!("{cell:<width$}", width = widths[i]))
            .collect::<Vec<_>>()
            .join("  ");
        println!("{}", joined.trim_end());
    };
    line(headers);
    for row in &rows {
        line(&row.iter().map(String::as_str).collect::<Vec<_>>());
    }
}

/// Print an aligned key-value block (the `show` views).
pub fn kv(pairs: &[(&str, String)]) {
    let width = pairs.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    for (key, value) in pairs {
        println!("{key:<width$}  {value}");
    }
}

/// A millisecond epoch timestamp as local time, or `—` when absent.
pub fn time(millis: Option<u64>) -> String {
    match millis {
        Some(ms) => {
            let time: chrono::DateTime<chrono::Local> =
                (UNIX_EPOCH + Duration::from_millis(ms)).into();
            time.format("%Y-%m-%d %H:%M").to_string()
        }
        None => "—".into(),
    }
}

/// An optional string, or `—` when absent/empty.
pub fn opt(value: Option<String>) -> String {
    match value {
        Some(s) if !s.is_empty() => s,
        _ => "—".into(),
    }
}

/// An event payload as a compact `key: value, key: value` summary.
pub fn payload(payload: Option<&Value>) -> String {
    let Some(Value::Object(fields)) = payload else {
        return String::new();
    };
    fields
        .iter()
        .map(|(key, value)| {
            let value = match value {
                Value::String(s) => s.clone(),
                Value::Null => "—".into(),
                other => other.to_string(),
            };
            format!("{key}: {value}")
        })
        .collect::<Vec<_>>()
        .join(", ")
}
