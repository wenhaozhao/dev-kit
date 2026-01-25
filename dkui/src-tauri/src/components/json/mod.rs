use crate::components::json::jsonparser::JsonParserTabState;
use crate::components::jsonparser::JsonParserTab;
use crate::SharedAppState;
use dev_kit::command::json::{DiffTool, JsonpathMatch, QueryType};
use itertools::Itertools;
use std::str::FromStr;

pub mod jsonparser;
pub mod jsondiff;

#[tauri::command]
pub async fn jsonparser_init_tabs(
    state: tauri::State<'_, SharedAppState>,
) -> Result<Vec<JsonParserTab>, String> {
    let state = state.read().await;
    let tabs = state.jsonparser.get_tabs().await.map_err(|e| e.to_string())?;
    Ok(tabs)
}

#[tauri::command]
pub async fn jsonparser_add_tab(
    state: tauri::State<'_, SharedAppState>,
) -> Result<JsonParserTabState, String> {
    let mut app_state = state.write().await;
    let jsonparser_path = app_state.jsonparser_path().await?;
    let tab = app_state.jsonparser.add_tab(&jsonparser_path).await?;
    Ok(tab)
}

#[tauri::command]
pub async fn jsonparser_remove_tab(
    state: tauri::State<'_, SharedAppState>,
    tab_id: String,
) -> Result<(), String> {
    let mut app_state = state.write().await;
    let jsonparser_path = app_state.jsonparser_path().await?;
    app_state.jsonparser.remove_tab(&jsonparser_path, &tab_id).await?;
    Ok(())
}

#[tauri::command]
pub async fn jsonparser_query_json(
    state: tauri::State<'_, SharedAppState>,
    json: String,
    query: Option<String>,
    query_type: Option<String>,
    reload: bool,
    tab_id: String,
) -> Result<String, String> {
    let mut app_state = state.write().await;
    let jsonparser_path = app_state.jsonparser_path().await?;
    let value = app_state.jsonparser.get_or_parse(&jsonparser_path, &tab_id, &json, reload).await?;
    let arr = value.query(
        query.as_deref(), query_type.and_then(|s|
            QueryType::from_str(&s).ok()
        ), true,
    ).map_err(|e| e.to_string())?;
    Ok(arr)
}

#[tauri::command]
pub async fn jsonparser_search_json_paths(
    state: tauri::State<'_, SharedAppState>,
    tab_id: String,
    query: Option<String>,
) -> Result<Vec<JsonpathMatch>, String> {
    let mut app_state = state.write().await;
    let jsonparser_path = app_state.jsonparser_path().await?;
    let array = app_state.jsonparser.search_paths(
        &jsonparser_path, &tab_id, query.as_deref().unwrap_or_default()
    ).await?;
    Ok(array)
}

#[tauri::command]
pub async fn jsondiff_query_json(
    state: tauri::State<'_, SharedAppState>,
    json: String,
    query: Option<String>,
    query_type: Option<String>,
    reload: bool,
) -> Result<String, String> {
    let app_state = state.write().await;
    let value = app_state.jsondiff.get_or_parse(&json, reload).await?;
    let arr = value.query(
        query.as_deref(), query_type.and_then(|s|
            QueryType::from_str(&s).ok()
        ), true,
    ).map_err(|e| e.to_string())?;
    Ok(arr)
}

#[tauri::command]
pub async fn jsondiff_search_json_paths(
    state: tauri::State<'_, SharedAppState>,
    json: String,
    query: Option<String>,
    query_type: Option<String>,
) -> Result<Vec<JsonpathMatch>, String> {
    let app_state = state.read().await;
    let value = app_state.jsondiff.get_or_parse(&json, false).await?;
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
pub async fn jsondiff_diff_json(
    state: tauri::State<'_, SharedAppState>,
    left: String,
    right: String,
    query: Option<String>,
    query_type: Option<String>,
    diff_tool: Option<String>,
) -> Result<(), String> {
    let app_state = state.read().await;
    let left_val = app_state.jsondiff.get_or_parse(&left, false).await?;
    let right_val = app_state.jsondiff.get_or_parse(&right, false).await?;
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