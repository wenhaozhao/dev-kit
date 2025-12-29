use devkit::command::uri::Uri;
use devkit::command::json::{Json, DiffTool};
use devkit::command::time::TimeFormat;
use std::str::FromStr;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn decode_uri(uri: String) -> Result<String, String> {
    let uri = Uri::from_str(&uri).map_err(|e| e.to_string())?;
    uri.decode().map_err(|e| e.to_string())
}

#[tauri::command]
fn format_json(json: String) -> Result<String, String> {
    let json = Json::from_str(&json).map_err(|e| e.to_string())?;
    json.beautify().map_err(|e| e.to_string())
}

#[tauri::command]
fn query_json(json: String, query: String) -> Result<Vec<String>, String> {
    let json = Json::from_str(&json).map_err(|e| e.to_string())?;
    json.query(&query).map_err(|e| e.to_string())
}

#[tauri::command]
fn diff_json(left: String, right: String, query: Option<String>, diff_tool: Option<String>) -> Result<(), String> {
    let left = Json::from_str(&left).map_err(|e| e.to_string())?;
    let right = Json::from_str(&right).map_err(|e| e.to_string())?;
    let tool = if let Some(t) = diff_tool {
        DiffTool::from_str(&t).map_err(|e| e.to_string())?
    } else {
        DiffTool::default()
    };

    use devkit::command::Command;
    let cmd = devkit::command::json::JsonCommand::Diff {
        left,
        right,
        query,
        diff_tool: Some(tool),
    };
    cmd.run().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_available_diff_tools() -> Vec<String> {
    use strum::IntoEnumIterator;
    DiffTool::iter()
        .filter(|t: &DiffTool| t.is_available())
        .map(|t| t.to_string())
        .collect()
}

#[tauri::command]
fn now_time(timezone: Option<String>, format: Option<String>) -> Result<String, String> {
    let tz = if let Some(tz_str) = timezone {
        Some(chrono::FixedOffset::from_str(&tz_str).map_err(|e| e.to_string())?)
    } else {
        None
    };
    let fmt = if let Some(fmt_str) = format {
        Some(TimeFormat::from_str(&fmt_str).map_err(|e: anyhow::Error| e.to_string())?)
    } else {
        None
    };

    let timezone = tz.unwrap_or(*chrono::Local::now().offset());
    let time = chrono::Local::now().with_timezone(&timezone);
    
    let fmt_enum = fmt.unwrap_or_default();
    match fmt_enum {
        TimeFormat::RFC3339 => Ok(time.to_rfc3339()),
        TimeFormat::Timestamp => Ok(time.timestamp_millis().to_string()),
        TimeFormat::Format(f) => Ok(time.format(&f).to_string()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            decode_uri,
            format_json,
            query_json,
            diff_json,
            get_available_diff_tools,
            now_time
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
