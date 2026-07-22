use derive_more::{Deref, DerefMut, From};
use dev_kit::command::formatter::FormattedValue;
use dev_kit::command::json::{Json, JsonpathMatch};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::HashMap;
use std::ops::{Add, Deref};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::fs;

#[derive(Debug, Deref, DerefMut)]
pub struct JsonParserState {
    data_path: PathBuf,
    #[deref]
    #[deref_mut]
    inner: JsonParserStateInner,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct JsonParserStateInner {
    active_tab_index: usize,
    tabs: HashMap<String, JsonParserTabState>,
}

impl JsonParserState {
    pub fn init<P: AsRef<Path>>(data_path: P) -> Result<JsonParserState, String> {
        let data_path = data_path.as_ref().to_owned();
        let config_path = data_path.join("state.json");
        let inner = if config_path.exists() {
            let json_string = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
            serde_json::from_str(&json_string)
                .map_err(|e| e.to_string())
                .map_err(|e| e.to_string())?
        } else {
            let mut state = JsonParserStateInner::default();
            let tab0 = JsonParserTabState::new(0);
            state.tabs.insert(tab0.id.clone(), tab0);
            let json_string = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
            std::fs::write(&config_path, &json_string).map_err(|e| e.to_string())?;
            state
        };
        Ok(JsonParserState { inner, data_path })
    }

    async fn update_state(&self) -> Result<(), String> {
        let config_path = self.data_path.join("state.json");
        let tab_json_str = serde_json::to_string_pretty(&**self).map_err(|e| e.to_string())?;
        fs::write(&config_path, tab_json_str)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl JsonParserState {
    pub async fn get_tabs(&self) -> Result<Vec<JsonParserTab>, String> {
        let mut datas = Vec::with_capacity(self.tabs.len());
        for JsonParserTabState {
            id,
            idx,
            json_input,
            json_output,
            json_query,
            selected_index,
            ..
        } in self.tabs.values()
        {
            datas.push(JsonParserTab {
                id: id.to_string(),
                idx: *idx,
                json_input: {
                    if let Some(InputSource { path, .. }) = json_input {
                        let data = fs::read_to_string(path).await.map_err(|e| e.to_string())?;
                        Some(data)
                    } else {
                        None
                    }
                },
                json_output: {
                    if let Some(OutputSource { path, .. }) = json_output {
                        let data = fs::read_to_string(path).await.map_err(|e| e.to_string())?;
                        Some(data)
                    } else {
                        None
                    }
                },
                json_query: json_query.clone(),
                selected_index: *selected_index,
            })
        }
        datas.sort_by_key(|a| a.idx);
        Ok(datas)
    }

    pub async fn add_tab(&mut self) -> Result<JsonParserTabState, String> {
        let tab_idx = self
            .tabs
            .values()
            .map(|it| it.idx)
            .max()
            .unwrap_or_default()
            .incr();
        let tab = JsonParserTabState::new(tab_idx);
        let tab_id = tab.id.clone();
        let _ = self.tabs.insert(tab_id.clone(), tab);
        self.update_state().await?;
        let tab = self.tabs.get(&tab_id).ok_or("Tab not found")?;
        Ok(tab.clone())
    }

    pub async fn remove_tab(&mut self, tab_id: &str) -> Result<(), String> {
        if let Some(JsonParserTabState {
            json_input,
            json_output,
            ..
        }) = self.tabs.remove(tab_id)
        {
            if let Some(InputSource { path, .. }) = json_input {
                let _ = fs::remove_file(path).await;
            }
            if let Some(OutputSource { path, .. }) = json_output {
                let _ = fs::remove_file(path).await;
            }
        };
        self.update_state().await?;
        Ok(())
    }

    pub async fn search_paths(
        &mut self,
        tab_id: &str,
        query: &str,
    ) -> Result<Vec<JsonpathMatch>, String> {
        let array = {
            let Some(tab) = self.tabs.get_mut(tab_id) else {
                return Err("Tab not found".to_string());
            };
            {
                let old_json_query = tab.json_query.as_deref().unwrap_or_default();
                if let (true, Some(json_query_cache)) =
                    (old_json_query.eq(query), &tab.json_query_cache)
                {
                    return Ok(json_query_cache.clone());
                }
            }
            let formatted_value = Self::get_only(tab).await?;
            let json_value = formatted_value.try_into().map_err(|e| format!("{e}"))?;
            let arr = dev_kit::command::json::Json::search_paths(&json_value, Some(query), None)
                .map_err(|e| e.to_string())?;
            let _ = tab.json_query_cache.replace(arr);
            let _ = tab.json_query.replace(JsonQuery(query.to_string()));
            tab.json_query_cache
                .as_ref()
                .ok_or("No json-query-cache found".to_string())?
                .clone()
        };
        self.update_state().await?;
        Ok(array)
    }

    pub async fn get_or_parse(
        &mut self,
        tab_id: &str,
        json_input_string: &str,
        reload: bool,
    ) -> Result<&FormattedValue, String> {
        let config_dir = self.data_path.to_owned();

        let cache_state = {
            let Some(tab) = self.tabs.get(tab_id) else {
                return Err("Tab not found".to_string());
            };
            let json_input_string_sha = hex::encode(sha2::Sha256::digest(json_input_string));
            let sha_eq =
                json_input_string_sha.eq(tab.json_input_sha.as_deref().unwrap_or_default());
            (
                sha_eq,
                reload,
                tab.json_output_cache.is_some(),
                tab.json_output.is_some(),
                json_input_string_sha,
            )
        };
        match cache_state {
            (true, false, true, _, _) => {}
            (true, false, false, true, _) => {
                let tab = self.tabs.get_mut(tab_id).expect("unexpected none tab");
                let OutputSource { path, ctype } = tab
                    .json_output
                    .as_ref()
                    .expect("unexpected none json_output");
                let json = Json::Filepath(path.to_path_buf());
                let json_value = FormattedValue::try_from(&json)
                    .and_then(|it| it.convert(*ctype))
                    .map_err(|err| format!("{err}"))?;
                let _ = tab.json_output_cache.replace(json_value);
            }
            (.., json_input_string_sha) => {
                let tab = self.tabs.get_mut(tab_id).expect("unexpected none tab");
                // update json-input
                if let Some(InputSource { path }) = &tab.json_input {
                    fs::write(path, json_input_string)
                        .await
                        .map_err(|e| e.to_string())?;
                } else {
                    let path = config_dir.join(format!("input-{}", uuid::Uuid::new_v4()));
                    fs::write(&path, json_input_string)
                        .await
                        .map_err(|e| e.to_string())?;
                    let _ = tab.json_input.replace(InputSource { path });
                }
                let (formatted_value, pretty) = {
                    let formatted_value =
                        FormattedValue::from_str(json_input_string).map_err(|e| e.to_string())?;
                    let pretty = formatted_value
                        .to_string_pretty()
                        .map_err(|e| e.to_string())?;
                    (formatted_value, pretty)
                };
                {
                    let path = if let Some(OutputSource { path, .. }) = &tab.json_output {
                        path.clone()
                    } else {
                        config_dir.join(format!("output-{}.json", uuid::Uuid::new_v4()))
                    };
                    fs::write(&path, pretty).await.map_err(|e| e.to_string())?;
                    let _ = &tab.json_output.replace(OutputSource {
                        path,
                        ctype: formatted_value.type_(),
                    });
                }
                let _ = tab.json_output_cache.replace(formatted_value);
                let _ = tab.json_query_cache.take();
                let _ = tab.json_input_sha.replace(json_input_string_sha);
                self.update_state().await?;
            }
        };
        let json_value = self
            .tabs
            .get(tab_id)
            .expect("unexpected none tab")
            .json_output_cache
            .as_ref()
            .expect("unexpected none json_output_cache");
        Ok(json_value)
    }
}

impl JsonParserState {
    async fn get_only(tab: &mut JsonParserTabState) -> Result<&FormattedValue, String> {
        if tab.json_output_cache.is_none() {
            let Some(OutputSource { path, ctype }) = &tab.json_output else {
                return Err("No json-input found".to_string());
            };
            let json_output_string = tokio::fs::read_to_string(path)
                .await
                .map_err(|err| format!("{err}"))?;
            let json_value = FormattedValue::from_str(&json_output_string)
                .and_then(|it| it.convert(*ctype))
                .map_err(|err| format!("{err}"))?;
            tab.json_output_cache.replace(json_value);
        }
        Ok(tab
            .json_output_cache
            .as_ref()
            .expect("unexpected none json_output_cache"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonParserTabState {
    id: String,
    idx: TabIdx,
    json_input: Option<InputSource>,
    json_input_sha: Option<String>,
    json_output: Option<OutputSource>,
    #[serde(skip)]
    json_output_cache: Option<FormattedValue>,
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

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, From, Eq, PartialEq, Hash, Default, Ord, PartialOrd,
)]
pub struct TabIdx(usize);
impl TabIdx {
    fn incr(&self) -> Self {
        Self(self.0.add(1))
    }
}

use crate::components::jsonparser::output_source::OutputSource;
use input_source::InputSource;

mod input_source {
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serialize};
    use std::path::PathBuf;
    use std::str::FromStr;

    #[derive(Debug, Clone, Serialize)]
    pub struct InputSource {
        pub path: PathBuf,
    }
    impl<'de> Deserialize<'de> for InputSource {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let json = serde_json::Value::deserialize(deserializer)?;
            match json {
                serde_json::Value::String(string) => Ok(Self {
                    path: PathBuf::from_str(&string).map_err(|err| {
                        D::Error::custom(format!("expect filepath but got {string}, err: {err}"))
                    })?,
                }),
                serde_json::Value::Object(mut jsonobj) => {
                    let path = jsonobj
                        .remove("path")
                        .and_then(|it| serde_json::from_value::<PathBuf>(it).ok())
                        .ok_or(D::Error::custom("unexpected path field"))?;
                    Ok(Self { path })
                }
                _ => {
                    let input = serde_json::to_string(&json).map_err(D::Error::custom)?;
                    Err(D::Error::custom(format!(
                        "deserialize failed, got unexpected json: {input}",
                    )))
                }
            }
        }
    }
}

mod output_source {
    use dev_kit::command::formatter::FormattedValueType as CType;
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serialize};
    use std::path::PathBuf;
    use std::str::FromStr;

    #[derive(Debug, Clone, Serialize)]
    pub struct OutputSource {
        pub path: PathBuf,
        pub ctype: CType,
    }
    impl<'de> Deserialize<'de> for OutputSource {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let json = serde_json::Value::deserialize(deserializer)?;
            match json {
                serde_json::Value::String(string) => Ok(Self {
                    path: PathBuf::from_str(&string).map_err(|err| {
                        D::Error::custom(format!("expect filepath but got {string}, err: {err}"))
                    })?,
                    ctype: CType::Json,
                }),
                serde_json::Value::Object(mut jsonobj) => {
                    let path = jsonobj
                        .remove("path")
                        .and_then(|it| serde_json::from_value::<PathBuf>(it).ok())
                        .ok_or(D::Error::custom("unexpected path field"))?;
                    let ctype = jsonobj
                        .remove("ctype")
                        .and_then(|it| serde_json::from_value::<CType>(it).ok())
                        .ok_or(D::Error::custom("unexpected path ctype"))?;
                    Ok(Self { path, ctype })
                }
                _ => {
                    let input = serde_json::to_string(&json).map_err(D::Error::custom)?;
                    Err(D::Error::custom(format!(
                        "deserialize failed, got unexpected json: {input}",
                    )))
                }
            }
        }
    }
}

#[cfg(test)]
mod source_tests {
    use super::output_source::OutputSource;
    use dev_kit::command::formatter::FormattedValueType;
    use std::path::PathBuf;

    #[test]
    fn deserializes_legacy_output_path_as_json() {
        let source: OutputSource = serde_json::from_str("\"/tmp/output.json\"").unwrap();
        assert_eq!(source.path, PathBuf::from("/tmp/output.json"));
        assert!(matches!(source.ctype, FormattedValueType::Json));
    }

    #[test]
    fn deserializes_persisted_output_type_without_recursive_deserialization() {
        let source: OutputSource =
            serde_json::from_str(r#"{"path":"/tmp/output.json","ctype":"jsonl"}"#).unwrap();
        assert_eq!(source.path, PathBuf::from("/tmp/output.json"));
        assert!(matches!(source.ctype, FormattedValueType::Jsonl));
    }
}

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
