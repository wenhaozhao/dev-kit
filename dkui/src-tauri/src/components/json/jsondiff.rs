use dev_kit::command::json::Json;
use sha2::Digest;
use std::str::FromStr;
use std::sync::Arc;

pub struct JsonDiffState {
    cache: moka::future::Cache<String, Json>,
}

impl JsonDiffState {
    pub fn init() -> Result<JsonDiffState, String> {
        Ok(Self {
            cache: moka::future::CacheBuilder::new(10).build(),
        })
    }
}

impl JsonDiffState {
    pub async fn get_or_parse(&self, json_str: &str, reload: bool) -> Result<Json, String> {
        let json_sha = {
            let json_sha = &sha2::Sha256::digest(json_str.as_bytes())[..];
            hex::encode(json_sha)
        };
        if let (false, Some(parsed)) = (reload, self.cache.get(&json_sha).await) {
            return Ok(parsed);
        }
        let json = {
            let json = Json::from_str(json_str).map_err(|e| e.to_string())?;
            let json_value = Arc::<serde_json::Value>::try_from(&json).map_err(|e| e.to_string())?;
            Json::JsonValue(json_value)
        };
        let _ = self.cache.insert(json_sha, json.clone()).await;
        Ok(json)
    }
}