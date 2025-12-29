use chrono::FixedOffset;
use devkit::command::json::{DiffTool, Json};
use devkit::command::time::{Time, TimeCommand, TimeFormat, TimeUnit};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct JsonCache {
    cache: HashMap<String, Json>,
}

impl JsonCache {
    fn get_or_parse(&mut self, json_str: &str) -> Result<Json, String> {
        if let Some(parsed) = self.cache.get(json_str) {
            return Ok(parsed.clone());
        }
        let json = {
            let json = Json::from_str(json_str).map_err(|e| e.to_string())?;
            let json_value = Arc::<serde_json::Value>::try_from(&json).map_err(|e| e.to_string())?;
            Json::JsonValue(json_value)
        };
        // Limit cache size to avoid memory issues
        if self.cache.len() > 1 {
            self.cache.clear();
        }
        self.cache.insert(json_str.to_string(), json.clone());
        Ok(json)
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn decode_uri(uri: String) -> Result<String, String> {
    let uri = devkit::command::uri::Uri::from_str(&uri).map_err(|e| e.to_string())?;
    uri.decode().map_err(|e| e.to_string())
}

#[tauri::command]
fn format_json(
    cache: tauri::State<'_, Mutex<JsonCache>>,
    json: String,
    query: Option<String>,
) -> Result<String, String> {
    let value = cache.lock().unwrap().get_or_parse(&json)?;
    let result = value.beautify(query.as_deref()).map_err(|e| e.to_string())?;
    Ok(result)
}

#[tauri::command]
fn query_json(
    cache: tauri::State<'_, Mutex<JsonCache>>,
    json: String,
    query: String,
) -> Result<Vec<String>, String> {
    let value = cache.lock().unwrap().get_or_parse(&json)?;
    let arr = value.query(&query, true).map_err(|e| e.to_string())?;
    Ok(arr)
}

#[tauri::command]
fn get_json_keys(
    cache: tauri::State<'_, Mutex<JsonCache>>,
    json: String,
    query: Option<String>,
) -> Result<Vec<String>, String> {
    let value = cache.lock().unwrap().get_or_parse(&json)?;
    let keys = value.keys(query.as_deref()).map_err(|e| e.to_string())?;
    Ok(keys)
}

#[tauri::command]
fn diff_json(
    cache: tauri::State<'_, Mutex<JsonCache>>,
    left: String,
    right: String,
    query: Option<String>,
    diff_tool: Option<String>,
) -> Result<(), String> {
    let left_val = cache.lock().unwrap().get_or_parse(&left)?;
    let right_val = cache.lock().unwrap().get_or_parse(&right)?;
    let tool = if let Some(t) = diff_tool {
        DiffTool::from_str(&t).map_err(|e| e.to_string())?
    } else {
        DiffTool::default()
    };
    let _ = left_val
        .diff(&right_val, query.as_deref(), Some(tool))
        .map_err(|e| e.to_string())?;
    Ok(())
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
fn now_time(
    timezone: Option<String>,
    format: Option<String>,
    unit: Option<String>,
) -> Result<String, String> {
    let cmd = TimeCommand::Now {
        timezone: timezone.as_deref().and_then(|tz| FixedOffset::from_str(tz).ok()),
        format: format.as_deref().and_then(|fmt| TimeFormat::from_str(fmt).ok()),
        unit: unit.as_deref().and_then(|u| TimeUnit::from_str(u).ok()),
    };
    Ok(cmd.run_actual().map_err(|e| e.to_string())?)
}

#[tauri::command]
fn parse_time(
    time: String,
    unit: Option<String>,
    timezone: Option<String>,
    format: Option<String>,
) -> Result<String, String> {
    let cmd = TimeCommand::Parse {
        time: Time::from_str(&time).map_err(|e| e.to_string())?,
        unit: unit.as_deref().and_then(|u| TimeUnit::from_str(u).ok()),
        timezone: timezone.as_deref().and_then(|tz| FixedOffset::from_str(tz).ok()),
        format: format.as_deref().and_then(|fmt| TimeFormat::from_str(fmt).ok()),
    };
    Ok(cmd.run_actual().map_err(|e| e.to_string())?)
}

#[tauri::command]
fn save_to_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(JsonCache::default()))
        .invoke_handler(tauri::generate_handler![
            greet,
            decode_uri,
            format_json,
            query_json,
            get_json_keys,
            diff_json,
            get_available_diff_tools,
            now_time,
            parse_time,
            save_to_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
