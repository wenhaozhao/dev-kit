use super::Json;
use anyhow::anyhow;
use itertools::Itertools;
use jsonpath_rust::JsonPath;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
impl Json {
    pub fn beautify(&self) -> crate::Result<String> {
        let json = serde_json::Value::try_from(self)?;
        Ok(serde_json::to_string_pretty(&json)?)
    }

    pub fn query(&self, query: &str) -> crate::Result<Vec<String>> {
        let json = serde_json::Value::try_from(self)?;
        let query_result = json.query(query).map_err(|err| anyhow!(r#"Invalid json path: {}"#, err))?;
        let arr = query_result.iter().flat_map(|&it|
            serde_json::to_string(&it).map_err(|err|
                anyhow!(r#"Invalid json {}"#, err)
            )).collect_vec();
        Ok(arr)
    }

    pub fn diff_prepare(&self, query: Option<&str>) -> crate::Result<String> {
        let json = serde_json::Value::try_from(self)?;
        if let Some(query) = query {
            let array = json.query(query)?;
            let pretty = serde_json::to_string_pretty(&array)?;
            Ok(pretty)
        } else {
            Ok(serde_json::to_string_pretty(&json)?)
        }
    }
}

impl TryFrom<&Json> for serde_json::Value {
    type Error = anyhow::Error;

    fn try_from(input: &Json) -> Result<Self, Self::Error> {
        let json = match input {
            Json::Stdin(input) | Json::String(input) => {
                serde_json::from_str::<serde_json::Value>(&input).map_err(|err|
                    anyhow!(r#"
                    Invalid json format:
                    {}"#, err)
                )?
            }
            Json::Path(path) => {
                let file = std::fs::File::open(&path).map_err(|err| anyhow!("open file {} failed, {}", path.display(), err))?;
                serde_json::from_reader::<_, serde_json::Value>(file).map_err(|err|
                    anyhow!(r#"
                    Invalid json format:
                    {}"#, err)
                )?
            }
            Json::Uri(url) => {
                let url = url.clone();
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(1usize)
                    .enable_io().build()?;
                let text = futures::executor::block_on(async {
                    let h = rt.spawn(async {
                        let text = reqwest::get(url).await.map_err(|err| anyhow!(err))?.text().await.map_err(|err| anyhow!(err))?;
                        Ok::<_, anyhow::Error>(text)
                    });
                    h.await
                })??;
                serde_json::from_str(&text)?
            }
        };
        Ok(json)
    }
}

impl FromStr for Json {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq("-") {
            let mut string = String::new();
            let _ = std::io::stdin().lock().read_to_string(&mut string)
                .map_err(|err| anyhow!("read from stdin failed, {}", err))?;
            Ok(Json::Stdin(string))
        } else if let Ok(url) = url::Url::parse(value) {
            return Ok(Json::Uri(url));
        } else {
            let path = PathBuf::from_str(value)?;
            if fs::exists(&path).unwrap_or(false) {
                Ok(Json::Path(path))
            } else {
                Ok(Json::String(value.to_string()))
            }
        }
    }
}

impl TryFrom<&String> for Json {
    type Error = anyhow::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}