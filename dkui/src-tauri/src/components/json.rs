use dev_kit::command::json::{DiffTool, Json, QueryType};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct JsonCache {
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

#[tauri::command]
pub fn query_json(
    cache: tauri::State<'_, Mutex<JsonCache>>,
    json: String,
    query: Option<String>,
    query_type: Option<String>,
) -> Result<String, String> {
    let value = cache.lock().unwrap().get_or_parse(&json)?;
    let arr = value.query(
        query.as_deref(), query_type.and_then(|s|
            QueryType::from_str(&s).ok()
        ), true,
    ).map_err(|e| e.to_string())?;
    Ok(arr)
}

#[tauri::command]
pub fn search_json_paths(
    cache: tauri::State<'_, Mutex<JsonCache>>,
    json: String,
    query: Option<String>,
    query_type: Option<String>,
) -> Result<Vec<String>, String> {
    let value = cache.lock().unwrap().get_or_parse(&json)?;
    let query_type = query_type.and_then(|s| QueryType::from_str(&s).ok());
    let keys = value.search_paths(
        query.as_deref(), query_type,
    ).map(|arr|arr.into_iter().map(|it|it.into()).collect::<_>()).map_err(|e| e.to_string())?;
    Ok(keys)
}

#[tauri::command]
pub fn diff_json(
    cache: tauri::State<'_, Mutex<JsonCache>>,
    left: String,
    right: String,
    query: Option<String>,
    query_type: Option<String>,
    diff_tool: Option<String>,
) -> Result<(), String> {
    let left_val = cache.lock().unwrap().get_or_parse(&left)?;
    let right_val = cache.lock().unwrap().get_or_parse(&right)?;
    let query_type = query_type.and_then(|s|
        QueryType::from_str(&s).ok()
    );
    let tool = if let Some(t) = diff_tool {
        DiffTool::from_str(&t).map_err(|e| e.to_string())?
    } else {
        DiffTool::default()
    };
    let _ = left_val
        .diff(&right_val, query.as_deref(), query_type, Some(tool))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_available_diff_tools() -> Vec<String> {
    use strum::IntoEnumIterator;
    DiffTool::iter()
        .filter(|t: &DiffTool| t.is_available())
        .map(|t| t.to_string())
        .collect()
}