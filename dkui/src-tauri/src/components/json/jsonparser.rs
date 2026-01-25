use derive_more::From;
use dev_kit::command::json::{Json, JsonpathMatch};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::HashMap;
use std::ops::{Add, Deref};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use tokio::fs;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct JsonParserState {
    active_tab_index: usize,
    tabs: HashMap<String, JsonParserTabState>,
}

impl JsonParserState {
    pub fn init<P: AsRef<Path>>(config_dir: P) -> Result<JsonParserState, String> {
        let config_path = config_dir.as_ref().join("state.json");
        let jsonparser_state = if config_path.exists() {
            let json_string = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
            serde_json::from_str(&json_string).map_err(|e| e.to_string()).map_err(|e| e.to_string())?
        } else {
            let mut state = JsonParserState::default();
            let tab0 = JsonParserTabState::new(0);
            state.tabs.insert(tab0.id.clone(), tab0);
            let json_string = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
            let _ = std::fs::write(&config_path, &json_string).map_err(|e| e.to_string())?;
            state
        };
        Ok(jsonparser_state)
    }

    async fn update_state<P: AsRef<Path>>(&self, config_dir: P) -> Result<(), String> {
        let config_path = config_dir.as_ref().join("state.json");
        let tab_json_str = serde_json::to_string_pretty(&self).map_err(|e| e.to_string())?;
        let _ = fs::write(&config_path, tab_json_str).await.map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl JsonParserState {
    pub async fn get_tabs(&self) -> Result<Vec<JsonParserTab>, String> {
        let mut datas = Vec::with_capacity(self.tabs.len());
        for (_, JsonParserTabState {
            id,
            idx,
            json_input,
            json_output,
            json_query,
            selected_index,
            ..
        }) in &self.tabs {
            datas.push(JsonParserTab {
                id: id.to_string(),
                idx: idx.clone(),
                json_input: {
                    if let Some(JsonInput(path)) = json_input {
                        let data = fs::read_to_string(path).await.map_err(|e| e.to_string())?;
                        Some(data)
                    } else {
                        None
                    }
                },
                json_output: {
                    if let Some(JsonOutput(path)) = json_output {
                        let data = fs::read_to_string(path).await.map_err(|e| e.to_string())?;
                        Some(data)
                    } else {
                        None
                    }
                },
                json_query: json_query.clone(),
                selected_index: selected_index.clone(),
            })
        }
        let _ = datas.sort_by(|a, b| a.idx.cmp(&b.idx));
        Ok(datas)
    }

    pub async fn add_tab<P: AsRef<Path>>(&mut self, config_dir: P) -> Result<JsonParserTabState, String> {
        let tab_idx = self.tabs.values().map(|it| it.idx).max().unwrap_or_default().incr();
        let tab = JsonParserTabState::new(tab_idx);
        let tab_id = tab.id.clone();
        let _ = self.tabs.insert(tab_id.clone(), tab);
        let _ = self.update_state(config_dir).await?;
        let tab = self.tabs.get(&tab_id).ok_or("Tab not found")?;
        Ok(tab.clone())
    }

    pub async fn remove_tab(&mut self, config_dir: &Path, tab_id: &str) -> Result<(), String> {
        if let Some(JsonParserTabState {
                        json_input, json_output, ..
                    }) = self.tabs.remove(tab_id) {
            if let Some(JsonInput(path)) = json_input {
                let _ = fs::remove_file(path).await;
            }
            if let Some(JsonOutput(path)) = json_output {
                let _ = fs::remove_file(path).await;
            }
        };
        let _ = self.update_state(config_dir).await?;
        Ok(())
    }

    pub async fn search_paths(&mut self, config_dir: &Path, tab_id: &str, query: &str) -> Result<Vec<JsonpathMatch>, String> {
        let array = {
            let Some(tab) = self.tabs.get_mut(tab_id) else {
                return Err("Tab not found".to_string());
            };
            {
                let old_json_query = tab.json_query.as_deref().unwrap_or_default();
                if let (true, Some(json_query_cache)) = (old_json_query.eq(query), &tab.json_query_cache,) {
                    return Ok(json_query_cache.clone());
                }
            }
            let json = Self::get_only(tab).await?;
            let arr = json.search_paths(Some(query), None).map_err(|e| e.to_string())?;
            let _ = tab.json_query_cache.replace(arr);
            let _ = tab.json_query.replace(JsonQuery(query.to_string()));
            tab.json_query_cache.as_ref().ok_or("No json-query-cache found".to_string())?.clone()
        };
        self.update_state(config_dir).await?;
        Ok(array)
    }

    pub async fn get_or_parse<P: AsRef<Path>>(&mut self, config_dir: P, tab_id: &str, json_input_string: &str, reload: bool) -> Result<Json, String> {
        let json_value = {
            let Some(tab) = self.tabs.get_mut(tab_id) else {
                return Err("Tab not found".to_string());
            };
            let json_input_string_sha = hex::encode(sha2::Sha256::digest(json_input_string));
            let sha_eq = json_input_string_sha.eq(tab.json_input_sha.as_deref().unwrap_or_default());
            if let (true, false, Some(json_value), _) = (sha_eq, reload, &tab.json_output_cache, &tab.json_output) {
                return Ok(Json::clone(json_value));
            }
            if let (true, false, None, Some(JsonOutput(path))) = (sha_eq, reload, &tab.json_output_cache, &tab.json_output) {
                let json = fs::read_to_string(path).await.map_err(|e| e.to_string()).and_then(|it| {
                    serde_json::from_str::<serde_json::Value>(&it).map_err(|e| e.to_string())
                }).map(|it|
                    Arc::new(Json::JsonValue(Arc::new(it)))
                )?;
                let _ = tab.json_output_cache.replace(json);
                return Ok(Json::clone(tab.json_output_cache.as_deref().ok_or("No json-output found")?));
            }
            // update json-input
            let config_dir = config_dir.as_ref();
            if let Some(JsonInput(path)) = &tab.json_input {
                let _ = fs::write(path, json_input_string).await.map_err(|e| e.to_string())?;
            } else {
                let path = config_dir.join(format!("input-{}", uuid::Uuid::new_v4()));
                let _ = fs::write(&path, json_input_string).await.map_err(|e| e.to_string())?;
                let _ = tab.json_input.replace(JsonInput(path));
            }
            let (json_value, json_value_stringify) = {
                let json_value = Json::from_str(json_input_string).map_err(|e| e.to_string()).and_then(|json| {
                    Ok(Arc::<serde_json::Value>::try_from(&json).map_err(|e| e.to_string())?)
                }).map_err(|e| e.to_string())?;
                let json_value_stringify = serde_json::to_string_pretty(&json_value).map_err(|e| e.to_string())?;
                (json_value, json_value_stringify)
            };
            let json = if let Some(JsonOutput(path)) = &tab.json_output {
                let _ = fs::write(path, json_value_stringify).await.map_err(|e| e.to_string())?;
                Arc::new(Json::JsonValue(json_value))
            } else {
                let path = config_dir.join(format!("output-{}.json", uuid::Uuid::new_v4()));
                let _ = fs::write(&path, json_value_stringify).await.map_err(|e| e.to_string())?;
                let _ = &tab.json_output.replace(JsonOutput(path));
                Arc::new(Json::JsonValue(json_value))
            };
            let _ = tab.json_output_cache.replace(json);
            let _ = tab.json_query_cache.take();
            let _ = tab.json_input_sha.replace(json_input_string_sha);
            Json::clone(tab.json_output_cache.as_deref().ok_or("No json-output found")?)
        };
        self.update_state(config_dir).await?;
        Ok(json_value)
    }
}

impl JsonParserState {
    async fn get_only(tab: &mut JsonParserTabState) -> Result<Json, String> {
        if let Some(json) = &tab.json_output_cache {
            return Ok(Json::clone(json));
        }
        let Some(JsonOutput(path)) = &tab.json_output else {
            return Err("No json-input found".to_string());
        };
        let json = fs::read_to_string(&path).await.map_err(|e| e.to_string()).and_then(|it| {
            serde_json::from_str(&it).map_err(|e| e.to_string())
        }).map(|it|
            Arc::new(Json::JsonValue(Arc::new(it)))
        )?;
        let _ = tab.json_output_cache.replace(json);
        Ok(Json::clone(tab.json_output_cache.as_deref().ok_or("No json-output found")?))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonParserTabState {
    id: String,
    idx: TabIdx,
    json_input: Option<JsonInput>,
    json_input_sha: Option<String>,
    json_output: Option<JsonOutput>,
    #[serde(skip)]
    json_output_cache: Option<Arc<Json>>,
    json_query: Option<JsonQuery>,
    #[serde(skip)]
    json_query_cache: Option<Vec<JsonpathMatch>>,
    selected_index: SelectedIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonParserTab {
    pub id: String,
    pub idx: TabIdx,
    pub json_input: Option<String>,
    pub json_output: Option<String>,
    pub json_query: Option<JsonQuery>,
    pub selected_index: SelectedIndex,
}

impl JsonParserTabState {
    pub fn new<Idx: Into<TabIdx>>(idx: Idx) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            idx: idx.into(),
            json_input: Default::default(),
            json_input_sha: None,
            json_output: Default::default(),
            json_output_cache: None,
            json_query: Default::default(),
            json_query_cache: Default::default(),
            selected_index: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, From, Eq, PartialEq, Hash, Default, Ord, PartialOrd)]
pub struct TabIdx(usize);
impl TabIdx {
    fn incr(&self) -> Self {
        Self(self.0.add(1))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonInput(PathBuf);
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonOutput(PathBuf);
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonQuery(String);

impl Deref for JsonQuery {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SelectedIndex(isize);
impl Default for SelectedIndex {
    fn default() -> Self {
        Self(-1)
    }
}

impl JsonParserState {}
