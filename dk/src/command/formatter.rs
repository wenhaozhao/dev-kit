use crate::command::json::Json;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FormattedValueType {
    #[default]
    Json,
    Jsonl,
    //Yaml,
    Toml,
    Text,
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
    pub fn convert(self, type_to: FormattedValueType) -> crate::Result<Self> {
        match type_to {
            FormattedValueType::Json => match self {
                FormattedValue::Json(_) => Ok(self),
                FormattedValue::Jsonl(value) => Ok(FormattedValue::Json(Value::Array(value))),
                FormattedValue::Toml(value) => {
                    Ok(FormattedValue::Json(serde_json::to_value(value)?))
                }
                FormattedValue::Text(value) => {
                    Ok(FormattedValue::Json(serde_json::to_value(value)?))
                }
            },
            FormattedValueType::Jsonl => match self {
                FormattedValue::Json(value) => {
                    if let Value::Array(value) = value {
                        Ok(FormattedValue::Jsonl(value))
                    } else {
                        Ok(FormattedValue::Jsonl(vec![value]))
                    }
                }
                FormattedValue::Jsonl(_) => Ok(self),
                _ => Ok(self
                    .convert(FormattedValueType::Json)?
                    .convert(FormattedValueType::Jsonl)?),
            },
            FormattedValueType::Toml => match self {
                FormattedValue::Json(value) => {
                    Ok(FormattedValue::Toml(serde_json::from_value(value)?))
                }
                FormattedValue::Toml(_) => Ok(self),
                _ => Ok(self
                    .convert(FormattedValueType::Json)?
                    .convert(FormattedValueType::Toml)?),
            },
            FormattedValueType::Text => match self {
                FormattedValue::Json(value) => {
                    Ok(FormattedValue::Text(serde_json::to_string_pretty(&value)?))
                }
                FormattedValue::Text(_) => Ok(self),
                _ => Ok(self
                    .convert(FormattedValueType::Json)?
                    .convert(FormattedValueType::Text)?),
            },
        }
    }

    pub fn type_(&self) -> FormattedValueType {
        match self {
            FormattedValue::Json(_) => FormattedValueType::Json,
            FormattedValue::Jsonl(_) => FormattedValueType::Jsonl,
            //FormattedValue::Yaml(_) => FormattedValueType::Yaml,
            FormattedValue::Toml(_) => FormattedValueType::Toml,
            FormattedValue::Text(_) => FormattedValueType::Text,
        }
    }
    pub fn to_string_pretty(&self) -> crate::Result<String> {
        match self {
            FormattedValue::Json(value) => Ok(serde_json::to_string_pretty(value)?),
            FormattedValue::Jsonl(value) => Ok(serde_json::to_string_pretty(value)?),
            //FormattedValue::Yaml(value) => Ok(serde_yaml::to_string(value)?),
            FormattedValue::Toml(value) => {
                if let Ok(string) = toml::to_string_pretty(value) {
                    Ok(string)
                } else {
                    FormattedValue::Json(serde_json::to_value(value.clone())?).to_string_pretty()
                }
            }
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

pub fn parse_formatted_value(input: &str) -> FormattedValue {
    if input.is_empty() {
        return FormattedValue::Text(input.to_string());
    }
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

    if let Ok(value) = serde_json::from_str(input) {
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

#[cfg(test)]
mod tests {
    use super::{FormattedValue, FormattedValueType, parse_formatted_value};
    use serde_json::json;

    #[test]
    fn parses_standard_json_without_changing_it() {
        match parse_formatted_value(r#"{"name":"devkit"}"#) {
            FormattedValue::Json(value) => {
                assert_eq!(value, json!({"name": "devkit"}));
            }
            _ => panic!("expected Json"),
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
            _ => panic!("expected Jsonl"),
        }
    }

    #[test]
    fn prefers_json_for_pretty_multiline_documents() {
        let input = "{\n  \"items\": [\n    1,\n    2\n  ]\n}";
        assert!(matches!(
            parse_formatted_value(input),
            FormattedValue::Json(_)
        ));
    }

    #[test]
    fn converts_jsonl_to_json_array_without_losing_records() {
        let value = parse_formatted_value("{\"id\":1}\n{\"id\":2}");
        assert_eq!(value.type_(), FormattedValueType::Jsonl);
        let converted = value.convert(FormattedValueType::Json).unwrap();
        assert!(
            matches!(converted, FormattedValue::Json(value) if value.as_array().unwrap().len() == 2)
        );
    }

    #[test]
    fn preserves_plain_text_that_is_not_a_supported_document() {
        let value = parse_formatted_value("curl --request GET https://example.test");
        assert!(matches!(value, FormattedValue::Text(_)));
    }

    #[test]
    fn treats_empty_input_as_empty_text() {
        let value = parse_formatted_value("");
        assert!(matches!(value, FormattedValue::Text(ref text) if text.is_empty()));
        assert_eq!(value.type_(), FormattedValueType::Text);
    }

    #[test]
    fn parses_toml_after_json_and_jsonl_fallbacks() {
        let value = parse_formatted_value("name = \"devkit\"");
        assert!(matches!(value, FormattedValue::Toml(_)));
    }
}
