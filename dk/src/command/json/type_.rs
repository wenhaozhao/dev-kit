use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::sync::Arc;
use anyhow::anyhow;
use lazy_static::lazy_static;
use serde_json::Value;
use crate::command::http_parser::HttpRequest;
use crate::command::json::{Json, KeyPatternType, QueryType};
use crate::command::read_stdin;

impl TryFrom<&Json> for Arc<Value> {
    type Error = anyhow::Error;

    fn try_from(input: &Json) -> Result<Self, Self::Error> {
        let json = match input {
            Json::Cmd(input) | Json::String(input) => {
                let json = serde_json::from_str::<Value>(&input).map_err(|err| {
                    log::debug!("{}", err);
                    anyhow!("Invalid json format")
                })?;
                Arc::new(json)
            }
            Json::Path(path) => {
                let file = fs::File::open(&path)
                    .map_err(|err| anyhow!("open file {} failed, {}", path.display(), err))?;
                let json =
                    serde_json::from_reader::<_, Value>(file).map_err(|err| {
                        log::debug!("{}", err);
                        anyhow!("Invalid json format")
                    })?;
                Arc::new(json)
            }
            Json::HttpRequest(http_request) => Arc::new(http_request.try_into()?),
            Json::JsonValue(val) => Arc::clone(val),
        };
        Ok(json)
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
        if let Some(string) = read_stdin() {
            if !string.is_empty() {
                return Ok(Self::from_str(&string)?);
            }
        }
        if value.is_empty() {
            Err(anyhow!("Invalid input"))
        } else if let Ok(http_request) = HttpRequest::from_str(value) {
            Ok(Json::HttpRequest(http_request))
        } else if let Some(_cmd_path) = CMD_SPLIT_PATTERN
            .captures(&value)
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
            Ok(Json::Path(path))
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
            _ => {
                Ok(Self::KeyPattern(KeyPatternType::from_str(s.as_str())
                    .map_err(|err| anyhow!("Invalid query type: {}", err))?))
            }
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
            _ => Err(anyhow!("Invalid key pattern type: {}", s))
        }
    }
}