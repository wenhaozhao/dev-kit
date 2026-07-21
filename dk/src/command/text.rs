use crate::command::formatter::parse_formatted_value;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Json,
    Jsonl,
    Toml,
    // Yaml,
    Text,
}

impl ContentType {
    pub fn detect(input: &str) -> Self {
        let input = input.trim();
        if input.is_empty() {
            return Self::Text;
        }
        let value = parse_formatted_value(input);
        (&value).into()
    }
}

impl FromStr for ContentType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "jsonl" | "ndjson" => Ok(Self::Jsonl),
            "toml" => Ok(Self::Toml),
            //"yaml" | "yml" => Ok(Self::Yaml),
            "text" | "plain" => Ok(Self::Text),
            other => Err(format!("unknown content type: {other}")),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::ContentType;

    #[test]
    fn detects_supported_content_types() {
        let cases = [
            (r#"{"name":"devkit"}"#, ContentType::Json),
            ("{\"a\":1}\n{\"b\":2}", ContentType::Jsonl),
            ("[package]\nname = \"devkit\"", ContentType::Toml),
            //("name: devkit\nitems:\n  - one", ContentType::Yaml),
            ("just ordinary prose", ContentType::Text),
        ];
        for (input, expected) in cases {
            assert_eq!(ContentType::detect(input), expected, "{input}");
        }
    }
}
