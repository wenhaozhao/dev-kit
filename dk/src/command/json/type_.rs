use crate::command::http_parser::HttpRequest;
use crate::command::json::{FormattedValue, Json, KeyPatternType, QueryType};
use crate::command::read_stdin;
use anyhow::{anyhow, Context};
use lazy_static::lazy_static;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

impl TryFrom<&Json> for FormattedValue {
    type Error = anyhow::Error;

    fn try_from(input: &Json) -> Result<Self, Self::Error> {
        let json = match input {
            Json::Cmd(input) | Json::String(input) => {
                super::parse_formatted_value(input)
            }
            Json::Filepath(path) => {
                let input = fs::read_to_string(path)
                    .with_context(|| format!("read file {} failed", path.display()))?;
                super::parse_formatted_value(&input)
            }
            Json::HttpRequest(http_request) => http_request.try_into()?,
        };
        Ok(json)
    }
}

impl FromStr for FormattedValue {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(super::parse_formatted_value(s))
    }
}


lazy_static! {
    static ref CMD_SPLIT_PATTERN: regex::Regex = {
        regex::RegexBuilder::new(r"^([\w\d]+).*")
            .multi_line(true)
            .case_insensitive(true)
            .build()
            .unwrap()
    };
}
impl FromStr for Json {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Some(string) = read_stdin()
            && !string.is_empty()
        {
            return Self::from_str(&string);
        }
        if value.is_empty() {
            Err(anyhow!("Invalid input"))
        } else if let Ok(http_request) = HttpRequest::from_str(value) {
            Ok(Json::HttpRequest(http_request))
        } else if let Some(_cmd_path) = CMD_SPLIT_PATTERN
            .captures(value)
            .map(|c| c.extract())
            .and_then(|(_, [cmd])| which::which(cmd).ok())
        {
            Ok(Json::Cmd(run_cmd(value)?))
        } else if let Ok(path) = {
            let path = PathBuf::from_str(value)?;
            if fs::exists(&path).unwrap_or(false) {
                Ok(path)
            } else {
                Err(anyhow!("Not a valid path: {}", value))
            }
        } {
            Ok(Json::Filepath(path))
        } else {
            Ok(Json::String(value.to_string()))
        }
    }
}

impl TryFrom<&String> for Json {
    type Error = anyhow::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

fn run_cmd(value: &str) -> crate::Result<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(value)
        .output()
        .map_err(|err| {
            anyhow!(
                r#"
failed to execute command: {}
{}
"#,
                err,
                value
            )
        })?;
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)
            .map_err(|err| anyhow!("failed to parse output as UTF-8: {}", err))?;
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("run command failed: {}", stderr))
    }
}

impl FromStr for QueryType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "jsonpath" | "jp" => Ok(Self::JsonPath),
            _ => Ok(Self::KeyPattern(
                KeyPatternType::from_str(s.as_str())
                    .map_err(|err| anyhow!("Invalid query type: {}", err))?,
            )),
        }
    }
}

impl FromStr for KeyPatternType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "prefix" | "p" => Ok(Self::Prefix),
            "suffix" | "s" => Ok(Self::Suffix),
            "contains" | "c" => Ok(Self::Contains),
            "regex" | "r" => Ok(Self::Regex),
            _ => Err(anyhow!("Invalid key pattern type: {}", s)),
        }
    }
}
