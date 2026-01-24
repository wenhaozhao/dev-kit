use crate::SharedAppState;
use dev_kit::command::json::{DiffTool, Json, JsonpathMatch, QueryType};
use itertools::Itertools;
use sha2::Digest;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

mod jsonparser;


#[derive(Default)]
pub struct JsonCache {
    cache: HashMap<String, Json>,
}

impl JsonCache {
    fn get_or_parse(&mut self, tab_id: &str, json_str: &str, reload: bool) -> Result<Json, String> {
        let json_sha = {
            let json_sha = &sha2::Sha256::digest(json_str.as_bytes())[..];
            hex::encode(json_sha)
        };
        let cache_key = format!("{}:{}", tab_id, json_sha);
        if let (false, Some(parsed)) = (reload, self.cache.get(&cache_key)) {
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
        self.cache.insert(cache_key, json.clone());
        Ok(json)
    }
}

#[tauri::command]
pub async fn query_json(
    state: tauri::State<'_, SharedAppState>,
    json: String,
    query: Option<String>,
    query_type: Option<String>,
    reload: bool,
) -> Result<String, String> {
    let mut app_state = state.write().await;
    let cache = &mut app_state.json_cache;
    let value = cache.get_or_parse("tab_0", &json, reload)?;
    let arr = value.query(
        query.as_deref(), query_type.and_then(|s|
            QueryType::from_str(&s).ok()
        ), true,
    ).map_err(|e| e.to_string())?;
    Ok(arr)
}

#[tauri::command]
pub async fn search_json_paths(
    state: tauri::State<'_, SharedAppState>,
    json: String,
    query: Option<String>,
    query_type: Option<String>,
) -> Result<Vec<JsonpathMatch>, String> {
    let mut app_state = state.write().await;
    let cache = &mut app_state.json_cache;
    let value = cache.get_or_parse("tab_0", &json, false)?;
    let query_type = query_type.and_then(|s| QueryType::from_str(&s).ok());
    match value.search_paths(query.as_deref(), query_type) {
        Ok(arr) => {
            Ok(arr.into_iter().map(|it| it.into()).collect_vec())
        }
        Err(err) => {
            Err(err.to_string())
        }
    }
}

#[tauri::command]
pub async fn diff_json(
    state: tauri::State<'_, SharedAppState>,
    left: String,
    right: String,
    query: Option<String>,
    query_type: Option<String>,
    diff_tool: Option<String>,
) -> Result<(), String> {
    let mut app_state = state.write().await;
    let cache = &mut app_state.json_cache;
    let left_val = cache.get_or_parse("tab_0", &left, false)?;
    let right_val = cache.get_or_parse("tab_0", &right, false)?;
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