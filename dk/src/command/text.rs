use crate::command::json::parse_json_or_jsonl;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Json,
    Jsonl,
    Toml,
    Yaml,
    Rust,
    JavaScript,
    TypeScript,
    Java,
    C,
    Cpp,
    Lua,
    Text,
}

impl ContentType {
    pub fn detect(input: &str) -> Self {
        let input = input.trim();
        if input.is_empty() {
            return Self::Text;
        }

        if serde_json::from_str::<serde_json::Value>(input).is_ok() {
            return Self::Json;
        }
        if is_jsonl(input) {
            return Self::Jsonl;
        }

        let lines: Vec<_> = input
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect();
        if input.contains("fn ")
            && (input.contains("let ") || input.contains("use ") || input.contains("impl "))
        {
            return Self::Rust;
        }
        if input.contains("interface ") || input.contains(": string") || input.contains(": number")
        {
            return Self::TypeScript;
        }
        if input.contains("function ") || input.contains("const ") || input.contains("=>") {
            return Self::JavaScript;
        }
        if input.contains("public class ") || input.contains("import java.") {
            return Self::Java;
        }
        if input.contains("#include") || input.contains("std::") || input.contains("class ") {
            return if input.contains("std::") || input.contains("class ") {
                Self::Cpp
            } else {
                Self::C
            };
        }
        if input.contains("local ") || input.contains("function ") && input.contains("end") {
            return Self::Lua;
        }
        if input.starts_with("---")
            || lines.iter().any(|line| line.starts_with("- "))
            || lines.iter().any(|line| line.contains(": "))
        {
            return Self::Yaml;
        }
        if lines
            .iter()
            .any(|line| line.starts_with('[') && line.ends_with(']'))
            || lines
                .iter()
                .any(|line| line.contains('=') && !line.contains("=="))
        {
            return Self::Toml;
        }
        Self::Text
    }
}

impl FromStr for ContentType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "jsonl" | "ndjson" => Ok(Self::Jsonl),
            "toml" => Ok(Self::Toml),
            "yaml" | "yml" => Ok(Self::Yaml),
            "rust" | "rs" => Ok(Self::Rust),
            "javascript" | "js" => Ok(Self::JavaScript),
            "typescript" | "ts" => Ok(Self::TypeScript),
            "java" => Ok(Self::Java),
            "c" => Ok(Self::C),
            "cpp" | "c++" | "cc" => Ok(Self::Cpp),
            "lua" => Ok(Self::Lua),
            "text" | "plain" => Ok(Self::Text),
            other => Err(format!("unknown content type: {other}")),
        }
    }
}

pub fn detect_content_type(input: &str, override_type: Option<ContentType>) -> ContentType {
    override_type.unwrap_or_else(|| ContentType::detect(input))
}

fn is_jsonl(input: &str) -> bool {
    input.lines().filter(|line| !line.trim().is_empty()).count() > 1
        && parse_json_or_jsonl(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::{ContentType, detect_content_type};

    #[test]
    fn detects_supported_content_types() {
        let cases = [
            (r#"{"name":"devkit"}"#, ContentType::Json),
            ("{\"a\":1}\n{\"b\":2}", ContentType::Jsonl),
            ("[package]\nname = \"devkit\"", ContentType::Toml),
            ("name: devkit\nitems:\n  - one", ContentType::Yaml),
            ("use std::io;\nfn main() { let x = 1; }", ContentType::Rust),
            ("const name = 'devkit';", ContentType::JavaScript),
            ("interface Config { name: string }", ContentType::TypeScript),
            ("public class App {}", ContentType::Java),
            ("#include <stdio.h>\nint main(void) {}", ContentType::C),
            (
                "#include <vector>\nstd::vector<int> values;",
                ContentType::Cpp,
            ),
            ("local name = 'devkit'", ContentType::Lua),
            ("just ordinary prose", ContentType::Text),
        ];
        for (input, expected) in cases {
            assert_eq!(ContentType::detect(input), expected, "{input}");
        }
    }

    #[test]
    fn override_wins_over_detection() {
        assert_eq!(
            detect_content_type("{\"a\":1}", Some(ContentType::Text)),
            ContentType::Text
        );
    }
}
