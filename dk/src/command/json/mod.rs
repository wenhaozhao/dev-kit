use crate::command::http_parser::HttpRequest;
use anyhow::anyhow;
use derive_more::Display;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use strum::EnumIter;

#[derive(clap::Subcommand)]
pub enum JsonCommand {
    #[clap(about = "json beautify, alias 'format'", alias = "format")]
    Beauty {
        #[arg(help = "json input, support string, file-path, url, cmd", default_value = "-")]
        json: Json,
        #[arg(short, long, help = "json path to extract")]
        query: Option<String>,
        #[arg(short, long, help = "file to write output")]
        file: Option<PathBuf>,
    },
    #[clap(about = "json query, alias 'search'", alias = "search")]
    Query {
        #[arg(help = "json input, support string, file-path, url, cmd", default_value = "-")]
        json: Json,
        #[arg(short, long, help = "json path to extract")]
        query: String,
        #[arg(long, help = "beauty output", alias = "format", default_value = "false")]
        beauty: bool,
        #[arg(short, long, help = "file to write output")]
        file: Option<PathBuf>,
    },
    #[clap(about = "json diff with left and right")]
    Diff {
        #[arg(help = "json input, support string, file-path, url, cmd", default_value = "-")]
        left: Json,
        #[arg(help = "json input, support string, file-path, url, cmd", default_value = "-")]
        right: Json,
        #[arg(short, long, help = "json path to extract")]
        query: Option<String>,
        #[arg(long, help = "diff tool to use, alias dt, support idea/zed/vscode, and will auto detect if not set", alias = "dt")]
        diff_tool: Option<DiffTool>,
    },
}

impl super::Command for JsonCommand {
    fn run(&self) -> crate::Result<()> {
        match self {
            JsonCommand::Beauty { json, query, file } => {
                let result = json.beautify(query.as_deref())?;
                if let Some(file) = file {
                    fs::write(&file, result).map_err(|err|
                        anyhow!("write to {} failed, {}", file.display(), err)
                    )?;
                    println!("write to {}", file.display())
                } else {
                    println!("{result}");
                }
                Ok(())
            }
            JsonCommand::Query { json, query, beauty, file } => {
                let result = json.query(query, *beauty)?;
                if let Some(file) = file {
                    let content = result.join("\n");
                    fs::write(&file, content)?;
                    println!("write to {}", file.display())
                } else {
                    for row in result {
                        println!("{}", row);
                    }
                }
                Ok(())
            }
            JsonCommand::Diff { left, right, query, diff_tool } => {
                let _ = left.diff(right, query.as_deref(), diff_tool.map(|it| it))?;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Display)]
pub enum Json {
    #[display("{_0}")]
    Cmd(String),
    #[display("{_0}")]
    HttpRequest(HttpRequest),
    #[display("{}", _0.display())]
    Path(PathBuf),
    #[display("{_0}")]
    String(String),
    #[display("{}", _0.to_string())]
    JsonValue(Arc<serde_json::Value>),
}

impl Json {
    fn name(&self) -> &'static str {
        match self {
            Json::Cmd(_) => "Cmd",
            Json::HttpRequest(_) => "HttpRequest",
            Json::Path(_) => "Path",
            Json::String(_) => "String",
            Json::JsonValue(_) => "JsonValue",
        }
    }
}

mod json;

#[derive(Debug, Copy, Clone, Display, EnumIter)]
pub enum DiffTool {
    #[display("{_0}")]
    JetbrainsIDE(JetbrainsIDE),
    #[display("zed")]
    Zed,
    #[display("vscode")]
    VSCode,
}

mod difftool;

#[derive(Debug, Copy, Clone, Display, EnumIter, Default)]
#[display(rename_all = "lowercase")]
pub enum JetbrainsIDE {
    #[default]
    Idea,
    CLion,
    RustRover,
    GoLand,
    PyCharm,
    WebStorm,
    Rider,
    DataGrip,
    AppCode,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() -> crate::Result<()> {
        let json1 = r#"{"a":1,"b":2,"c":3}"#;
        let json1: serde_json::Value = serde_json::from_str(json1)?;
        let json1 = serde_json::to_string(&json1)?;
        println!("{}", json1);
        let json2 = r#"{"c":3,"b":2,"a":1, "d":{"g":"gg","f":"ff","e":"ee"}}"#;
        let json2: serde_json::Value = serde_json::from_str(json2)?;
        let json2 = serde_json::to_string(&json2)?;
        println!("{}", json2);
        Ok(())
    }
}