use crate::command::http_parser::HttpRequest;
use derive_more::Display;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use strum::EnumIter;

#[derive(clap::Subcommand)]
pub enum JsonCommand {
    #[clap(about = "json format, alias 'beauty(b)/query(q)/search(s)/format(f)'", aliases = ["b", "query", "q", "search", "s", "format", "f"])]
    Beauty {
        #[arg(help = "json input, support string, file-path, url, cmd", default_value = "")]
        json: Json,
        #[arg(short, long, help = "extract content using jsonpath/key/value pattern")]
        query: Option<String>,
        #[arg(long, help = "json query type, alias `qt`, jsonpath(jp)/prefix(p)/suffix(s)/contains(c)/regex(r), and will auto detect if not set", alias = "qt")]
        query_type: Option<QueryType>,
        #[arg(long, help = "beauty output", alias = "format", default_value = "true")]
        beauty: bool,
        #[arg(short, long, help = "file to write output")]
        file: Option<PathBuf>,
    },
    #[clap(about = "json diff with left and right, alias 'd'", aliases=["d"])]
    Diff {
        #[arg(help = "json input, support string, file-path, url, cmd", default_value = "")]
        left: Json,
        #[arg(help = "json input, support string, file-path, url, cmd", default_value = "")]
        right: Json,
        #[arg(short, long, help = "extract content using jsonpath/key/value pattern")]
        query: Option<String>,
        #[arg(long, help = "json query type, alias `qt`, jsonpath(jp)/prefix(p)/suffix(s)/contains(c)/regex(r), and will auto detect if not set", alias = "qt")]
        query_type: Option<QueryType>,
        #[arg(long, help = "diff tool to use, alias dt, support idea/zed/vscode, and will auto detect if not set", alias = "dt")]
        diff_tool: Option<DiffTool>,
    },
}

impl super::Command for JsonCommand {
    fn run(&self) -> crate::Result<()> {
        match self {
            JsonCommand::Beauty { json, query, query_type, beauty, file } => {
                let content = json.query(query.as_deref(), *query_type, *beauty)?;
                if let Some(file) = file {
                    fs::write(&file, content)?;
                    println!("write to {}", file.display());
                } else {
                    println!("{content}");
                }
                Ok(())
            }
            JsonCommand::Diff { left, right, query, query_type, diff_tool } => {
                let _ = left.diff(right, query.as_deref(), *query_type, diff_tool.map(|it| it))?;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Display, )]
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

#[derive(Debug, Clone, Copy)]
pub enum QueryType {
    JsonPath,
    KeyPattern(KeyPatternType),
}

impl Default for QueryType {
    fn default() -> Self {
        Self::KeyPattern(KeyPatternType::default())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum KeyPatternType {
    Prefix,
    Suffix,
    #[default]
    Contains,
    Regex,
}

mod type_;
mod json;
pub use json::JsonpathMatch;

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