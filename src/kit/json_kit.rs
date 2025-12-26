use anyhow::anyhow;
use derive_more::Display;
use itertools::Itertools;
use jsonpath_rust::JsonPath;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Clone, Display)]
pub enum JsonInput {
    #[display("{_0}")]
    String(String),
    #[display("{}", _0.display())]
    Path(PathBuf),
}

impl TryFrom<String> for JsonInput {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.eq("-") {
            let mut string = String::new();
            let _ = std::io::stdin().lock().read_to_string(&mut string)
                .map_err(|err| anyhow!("read from stdin failed, {}", err))?;
            return Ok(JsonInput::String(string));
        }
        let path = PathBuf::from(&value);
        if fs::exists(&path).unwrap_or(false) {
            Ok(JsonInput::Path(path))
        } else {
            Ok(JsonInput::String(value))
        }
    }
}

impl TryFrom<&JsonInput> for serde_json::Value {
    type Error = anyhow::Error;

    fn try_from(input: &JsonInput) -> Result<Self, Self::Error> {
        let json = match input {
            JsonInput::String(input) => {
                serde_json::from_str::<serde_json::Value>(&input).map_err(|err|
                    anyhow!(r#"
                    Invalid json format:
                    {}"#, err)
                )?
            }
            JsonInput::Path(path) => {
                let file = std::fs::File::open(&path).map_err(|err| anyhow!("open file {} failed, {}", path.display(), err))?;
                serde_json::from_reader::<_, serde_json::Value>(file).map_err(|err|
                    anyhow!(r#"
                    Invalid json format:
                    {}"#, err)
                )?
            }
        };
        Ok(json)
    }
}

pub fn json_beautify<Input: TryInto<JsonInput, Error=anyhow::Error>>(input: Input) -> crate::Result<String> {
    let input = input.try_into()?;
    let json = serde_json::Value::try_from(&input)?;
    Ok(serde_json::to_string_pretty(&json)?)
}

pub fn json_query<Input: TryInto<JsonInput, Error=anyhow::Error>>(input: Input, query: &str) -> crate::Result<Vec<String>> {
    let input = input.try_into()?;
    let json = serde_json::Value::try_from(&input)?;
    let query_result = json.query(query).map_err(|err| anyhow!(r#"Invalid json path: {}"#, err))?;
    let arr = query_result.iter().flat_map(|&it|
        serde_json::to_string(&it).map_err(|err|
            anyhow!(r#"Invalid json {}"#, err)
        )).collect_vec();
    Ok(arr)
}