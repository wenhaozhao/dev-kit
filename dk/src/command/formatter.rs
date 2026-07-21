use crate::command::text::ContentType;
use anyhow::{anyhow, Context};
use derive_more::{Deref, Display};
use serde_json::Value;

pub fn format_text(input: &str, content_type: ContentType) -> crate::Result<String> {
    match content_type {
        ContentType::Json | ContentType::Jsonl => {
            let json_value = parse_json_or_jsonl(input)?;
            Ok(serde_json::to_string_pretty(&*json_value)?)
        }
        ContentType::Toml => {
            let table: toml::Table = input.parse().context("Invalid TOML")?;
            Ok(toml::to_string_pretty(&table)?)
        }
        ContentType::Yaml => {
            let value: serde_yaml::Value = serde_yaml::from_str(input).context("Invalid YAML")?;
            Ok(serde_yaml::to_string(&value)?.trim_end().to_string())
        }
        _ => Ok(normalize_plain_text(input)),
    }
}

pub fn normalize_plain_text(input: &str) -> String {
    input
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}

#[derive(Debug, Clone, Display)]
pub enum JsonInputType {
    Json,
    Jsonl,
}

#[derive(Debug, Clone, Deref)]
pub struct JsonValue {
    #[deref]
    value: Value,
    intput_type: JsonInputType,
}

/// Parses a JSON document, or a JSON Lines stream when the whole input is not JSON.
///
/// JSON Lines records are converted into a JSON array so existing JSONPath and query
/// behaviour remains unchanged.
pub fn parse_json_or_jsonl(input: &str) -> crate::Result<JsonValue> {
    match serde_json::from_str(input) {
        Ok(value) => Ok(JsonValue {
            value,
            intput_type: JsonInputType::Json,
        }),
        Err(json_error) => {
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
            if values.is_empty() && !input.trim().is_empty() {
                log::debug!("{}", json_error);
                Err(anyhow!("Invalid JSON format: {}", json_error))
            } else {
                Ok(JsonValue {
                    value: Value::Array(values),
                    intput_type: JsonInputType::Jsonl,
                })
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{format_text, normalize_plain_text, parse_json_or_jsonl};
    use crate::command::text::ContentType;
    use serde_json::json;

    #[test]
    fn parses_standard_json_without_changing_it() {
        assert_eq!(
            &*parse_json_or_jsonl(r#"{"name":"devkit"}"#).unwrap(),
            &json!({"name": "devkit"})
        );
    }

    #[test]
    fn parses_jsonl_records_into_an_array() {
        let input = "{\"name\":\"first\"}\n\n2\ntrue\nnull\n\"last\"";
        assert_eq!(
            &*parse_json_or_jsonl(input).unwrap(),
            &json!([{"name": "first"}, 2, true, null, "last"]),
        );
    }

    #[test]
    fn reports_the_invalid_jsonl_line() {
        let error = parse_json_or_jsonl("{\"valid\":true}\nnot-json\n{}").unwrap_err();
        assert!(error.to_string().contains("line 2"));
    }


    #[test]
    fn formats_structured_content() {
        assert_eq!(
            format_text("{\"b\":2,\"a\":1}", ContentType::Json).unwrap(),
            "{\n  \"a\": 1,\n  \"b\": 2\n}"
        );
        assert_eq!(
            format_text("{\"a\":1}\n{\"b\":2}", ContentType::Jsonl).unwrap(),
            "[\n  {\n    \"a\": 1\n  },\n  {\n    \"b\": 2\n  }\n]"
        );
        assert!(
            format_text("name = \"devkit\"", ContentType::Toml)
                .unwrap()
                .contains("name = \"devkit\"")
        );
        assert!(
            format_text("name: devkit", ContentType::Yaml)
                .unwrap()
                .contains("name: devkit")
        );
    }

    #[test]
    fn normalizes_plain_text_without_external_tools() {
        assert_eq!(normalize_plain_text("first  \nsecond\n\n"), "first\nsecond");
    }
}
