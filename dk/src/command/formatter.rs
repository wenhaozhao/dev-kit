use crate::command::json::Json;
use crate::command::text::ContentType;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormattedValue {
    Json(Value),
    Jsonl(Vec<Value>),
    //Yaml(serde_yaml::Value),
    Toml(toml::Value),
    Text(String),
}

impl TryFrom<&FormattedValue> for Value {
    type Error = anyhow::Error;

    fn try_from(value: &FormattedValue) -> Result<Self, Self::Error> {
        value.clone().try_into()
    }
}

impl TryFrom<FormattedValue> for Value {
    type Error = anyhow::Error;

    fn try_from(value: FormattedValue) -> Result<Self, Self::Error> {
        let value = match value {
            FormattedValue::Json(value) => value,
            FormattedValue::Jsonl(value) => Value::Array(value),
            //FormattedValue::Yaml(value) => serde_json::to_value(value)?,
            FormattedValue::Toml(value) => serde_json::to_value(value)?,
            FormattedValue::Text(value) => serde_json::Value::String(value),
        };
        Ok(value)
    }
}

impl FormattedValue {
    pub fn to_string_pretty(&self) -> crate::Result<String> {
        match self {
            FormattedValue::Json(value) => Ok(serde_json::to_string_pretty(value)?),
            FormattedValue::Jsonl(value) => Ok(serde_json::to_string_pretty(value)?),
            //FormattedValue::Yaml(value) => Ok(serde_yaml::to_string(value)?),
            FormattedValue::Toml(value) => Ok(toml::to_string_pretty(value)?),
            FormattedValue::Text(value) => Ok(value.clone()),
        }
    }

    pub fn to_string(&self) -> crate::Result<String> {
        match self {
            FormattedValue::Json(value) => Ok(serde_json::to_string(value)?),
            FormattedValue::Jsonl(value) => Ok(serde_json::to_string(value)?),
            //FormattedValue::Yaml(value) => Ok(serde_yaml::to_string(value)?),
            FormattedValue::Toml(value) => Ok(toml::to_string(value)?),
            FormattedValue::Text(value) => Ok(value.clone()),
        }
    }
}

impl From<&FormattedValue> for ContentType {
    fn from(value: &FormattedValue) -> Self {
        match value {
            FormattedValue::Json(_) => Self::Json,
            FormattedValue::Jsonl(_) => Self::Jsonl,
            //FormattedValue::Yaml(_) => Self::Yaml,
            FormattedValue::Toml(_) => Self::Toml,
            FormattedValue::Text(_) => Self::Text,
        }
    }
}

pub fn parse_formatted_value(input: &str) -> FormattedValue {
    if let Ok(json) = Json::from_str(input) {
        match json {
            Json::Cmd(input) => {
                return parse_formatted_value(&input);
            }
            Json::Filepath(path) => {
                if let Ok(input) = fs::read_to_string(path) {
                    return parse_formatted_value(&input);
                }
            }
            Json::HttpRequest(http_request) => {
                if let Ok(value) = (&http_request).try_into() {
                    return value;
                }
            }
            Json::String(_) => {}
        }
    }

    fn guess_jsonl(input: &str) -> crate::Result<Vec<Value>> {
        let mut values = Vec::new();
        for (index, line) in input.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let value = serde_json::from_str(line)
                .map_err(|error| anyhow!("Invalid JSONL at line {}: {}", index + 1, error))?;
            values.push(value);
        }
        Ok(values)
    }
    if let Ok(value) = serde_json::from_str(&input) {
        return FormattedValue::Json(value);
    }
    if let Ok(values) = guess_jsonl(input) {
        return FormattedValue::Jsonl(values);
    }
    // if let Ok(value) = serde_yaml::from_str(input) {
    //     return FormattedValue::Yaml(value);
    // }
    if let Ok(value) = toml::from_str(input) {
        return FormattedValue::Toml(value);
    }
    FormattedValue::Text(input.to_string())
}


#[cfg(test)]
mod tests {
    use super::{parse_formatted_value, FormattedValue};
    use serde_json::json;

    #[test]
    fn parses_standard_json_without_changing_it() {
        match parse_formatted_value(r#"{"name":"devkit"}"#) {
            FormattedValue::Json(value) => {
                assert_eq!(
                    value,
                    json!({"name": "devkit"})
                );
            }
            _ => panic!("expected Json")
        }
    }

    #[test]
    fn parses_jsonl_records_into_an_array() {
        let input = r#"
{"name": "first"}
2
true
null
"last"
        "#;
        match parse_formatted_value(input) {
            FormattedValue::Jsonl(values) => {
                let value = serde_json::to_value(values).unwrap();
                assert_eq!(
                    value,
                    json!([
  {
    "name": "first"
  },
  2,
  true,
  null,
  "last"
])
                );
            }
            _ => panic!("expected Jsonl")
        }
    }
}
