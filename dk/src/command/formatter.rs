use crate::command::json::parse_json_or_jsonl;
use crate::command::text::ContentType;
use anyhow::Context;

pub fn format_text(input: &str, content_type: ContentType) -> crate::Result<String> {
    match content_type {
        ContentType::Json | ContentType::Jsonl => {
            Ok(serde_json::to_string_pretty(&parse_json_or_jsonl(input)?)?)
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

#[cfg(test)]
mod tests {
    use super::{format_text, normalize_plain_text};
    use crate::command::text::ContentType;

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
