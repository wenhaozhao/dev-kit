use super::{DiffTool, Json};
use crate::command::http_parser::HttpRequest;
use crate::command::read_stdin;
use anyhow::anyhow;
use itertools::Itertools;
use jsonpath_rust::JsonPath;
use lazy_static::lazy_static;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::sync::Arc;
use std::{env, fs};

impl Json {
    pub fn keys(&self, query: Option<&str>) -> crate::Result<Vec<String>> {
        let json = Arc::<serde_json::Value>::try_from(self)?;
        let result = if let Some(query) = query {
            json.query(query)?
        } else {
            vec![&*json]
        };
        let keys = result.iter().flat_map(|it| match it {
            serde_json::Value::Object(map) => map.keys().map(|k| k.to_string()).collect_vec(),
            serde_json::Value::Array(_) => vec!["*".to_string()],
            _ => vec![]
        }).unique_by(|it| it.clone()).collect_vec();
        Ok(keys)
    }
    pub fn beautify(&self, query: Option<&str>) -> crate::Result<String> {
        let json = Arc::<serde_json::Value>::try_from(self)?;
        let result = if let Some(query) = query {
            let json = json.query(query)?;
            serde_json::to_string_pretty(&json)
        } else {
            serde_json::to_string_pretty(&*json)
        };
        Ok(result.map_err(|err| {
            log::debug!("{}",err);
            anyhow!("Invalid json format")
        })?)
    }

    pub fn query(&self, query: &str, beauty: bool) -> crate::Result<Vec<String>> {
        let json = Arc::<serde_json::Value>::try_from(self)?;
        let query_result = json.query(query).map_err(|err| {
            log::debug!("{}",err);
            anyhow!("Invalid json path: {query}")
        })?;
        let arr = query_result.iter().flat_map(|&it| {
            if beauty {
                serde_json::to_string_pretty(&it)
            } else {
                serde_json::to_string(&it)
            }.map_err(|err| {
                log::debug!("{}",err);
                anyhow!("Invalid json format")
            })
        }).collect_vec();
        Ok(arr)
    }

    pub fn diff(&self, other: &Self, query: Option<&str>, diff_tool: Option<DiffTool>) -> crate::Result<()> {
        let tmp_dir = env::temp_dir().join("jsondiff").join(chrono::Local::now().format("%Y%m%d%H%M%S%f").to_string());
        if tmp_dir.exists() {
            fs::remove_dir_all(&tmp_dir)?;
        }
        let left = self;
        let right = other;
        let _ = fs::create_dir_all(&tmp_dir)?;
        let left = left.diff_prepare(query.as_deref())?;
        let left_path = tmp_dir.join("left.json");
        fs::write(&left_path, left)?;
        println!("write left to file {}", left_path.display());
        let right = right.diff_prepare(query.as_deref())?;
        let right_path = tmp_dir.join("right.json");
        fs::write(&right_path, right)?;
        println!("write right to file {}", right_path.display());
        let diff_tool = diff_tool.unwrap_or_default();
        if diff_tool.is_available() {
            println!("diff with {}", diff_tool);
            diff_tool.diff(&left_path, &right_path)?;
        } else {
            eprintln!("diff tool {} is not installed", diff_tool);
            println!(
                r#"
install {} command-line interface, see:
{}"#,
                diff_tool, diff_tool.how_to_install()
            )
        }
        Ok(())
    }
}

impl Json {
    fn diff_prepare(&self, query: Option<&str>) -> crate::Result<String> {
        let json = Arc::<serde_json::Value>::try_from(self)?;
        if let Some(query) = query {
            let array = json.query(query)?;
            let pretty = serde_json::to_string_pretty(&array)?;
            Ok(pretty)
        } else {
            Ok(serde_json::to_string_pretty(&*json)?)
        }
    }
}

lazy_static! {
    static ref ASYNC_RT: tokio::runtime::Runtime = {
         tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(1usize)
                        .enable_all()
        .build().unwrap()
    };
}

impl TryFrom<&Json> for Arc<serde_json::Value> {
    type Error = anyhow::Error;

    fn try_from(input: &Json) -> Result<Self, Self::Error> {
        let json = match input {
            Json::Cmd(input) | Json::String(input) => {
                let json = serde_json::from_str::<serde_json::Value>(&input).map_err(|err| {
                    log::debug!("{}",err);
                    anyhow!("Invalid json format")
                })?;
                Arc::new(json)
            }
            Json::Path(path) => {
                let file = fs::File::open(&path).map_err(|err| anyhow!("open file {} failed, {}", path.display(), err))?;
                let json = serde_json::from_reader::<_, serde_json::Value>(file).map_err(|err| {
                    log::debug!("{}",err);
                    anyhow!("Invalid json format")
                })?;
                Arc::new(json)
            }
            Json::HttpRequest(http_request) => {
                Arc::new(http_request.try_into()?)
            }
            Json::JsonValue(val) => Arc::clone(val)
        };
        Ok(json)
    }
}

lazy_static! {
    static ref CMD_SPLIT_PATTERN: regex::Regex = {
        regex::RegexBuilder::new(r"^([\w\d]+).*")
        .multi_line(true)
        .case_insensitive(true)
        .build().unwrap()
    };
}
impl FromStr for Json {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let result = if let Some(string) = read_stdin() {
            Ok(Self::from_str(&string)?)
        } else if let Ok(http_request) = HttpRequest::from_str(value) {
            Ok(Json::HttpRequest(http_request))
        } else if let Some(_cmd_path) = CMD_SPLIT_PATTERN.captures(&value)
            .map(|c| c.extract())
            .and_then(|(_, [cmd])| which::which(cmd).ok()) {
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
        };
        match result {
            Ok(json) => {
                log::debug!("guess str to Json ok, {} => str: {value}", json.name());
                Ok(json)
            }
            Err(err) => {
                log::debug!("guess str to Json failed, str: {value}, err: {err}");
                Err(err)
            }
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
        .map_err(|err| anyhow!(r#"
failed to execute command: {}
{}
"#, err, value
                ))?;
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout).map_err(|err|
            anyhow!("failed to parse output as UTF-8: {}", err)
        )?;
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("run command failed: {}", stderr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use url::Url;

    #[test]
    fn test_json_from_str_string() {
        let input = r#"{"a": 1}"#;
        let json = Json::from_str(input).unwrap();
        match json {
            Json::String(s) => assert_eq!(s, input),
            _ => panic!("Expected Json::String, got {:?}", json),
        }
    }

    #[test]
    fn test_json_from_str_path() {
        let mut file = NamedTempFile::new().unwrap();
        let content = r#"{"a": 1}"#;
        writeln!(file, "{}", content).unwrap();
        let path_str = file.path().to_str().unwrap();

        let json = Json::from_str(path_str).unwrap();
        match json {
            Json::Path(p) => assert_eq!(p, file.path()),
            _ => panic!("Expected Json::Path, got {:?}", json),
        }
    }

    #[test]
    fn test_json_from_str_url_http() {
        let input = "http://example.com/api.json";
        let json = Json::from_str(input).unwrap();
        match json {
            Json::HttpRequest(http_request) => {
                let url = Url::try_from(&http_request).unwrap();
                assert_eq!(url.as_str(), input)
            }
            _ => panic!("Expected Json::Uri, got {:?}", json),
        }
    }

    #[test]
    fn test_json_from_str_cmd() {
        // Assume 'echo' is available
        let input = "echo '{\"a\": 1}'";
        let json = Json::from_str(input).unwrap();
        match json {
            Json::Cmd(s) => assert!(s.contains("\"a\": 1")),
            _ => panic!("Expected Json::Cmd, got {:?}", json),
        }
    }

    #[test]
    fn test_json_beautify() {
        let input = r#"{"a":1,"b":2}"#;
        let json = Json::String(input.to_string());
        let beautified = json.beautify(None).unwrap();
        assert!(beautified.contains("\n  \"a\": 1,"));
        assert!(beautified.contains("\n  \"b\": 2"));
    }

    #[test]
    fn test_json_query() {
        let input = r#"{"a":{"b":1},"c":2}"#;
        let json = Json::String(input.to_string());
        let result = json.query("$.a.b", false).unwrap();
        assert_eq!(result, vec!["1"]);

        let result = json.query("$.a", false).unwrap();
        assert_eq!(result, vec![r#"{"b":1}"#]);
    }

    #[test]
    fn test_json_diff_prepare() {
        let input = r#"{"a":1,"b":2}"#;
        let json = Json::String(input.to_string());

        // No query
        let prepared = json.diff_prepare(None).unwrap();
        assert!(prepared.contains("\"a\": 1"));

        // With query
        let prepared = json.diff_prepare(Some("$.a")).unwrap();
        assert_eq!(prepared, "[\n  1\n]");
    }

    #[test]
    fn test_run_cmd_success() {
        let result = run_cmd("echo 'hello'").unwrap();
        assert_eq!(result.trim(), "hello");
    }

    #[test]
    fn test_try_from_json_for_value() {
        // Test Json::String
        let json_str = Json::String(r#"{"a": 1}"#.to_string());
        let value = Arc::<serde_json::Value>::try_from(&json_str).unwrap();
        assert_eq!(value["a"], 1);

        // Test Json::Path
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"b": 2}}"#).unwrap();
        let json_path = Json::Path(file.path().to_path_buf());
        let value = Arc::<serde_json::Value>::try_from(&json_path).unwrap();
        assert_eq!(value["b"], 2);

        // Test Json::Cmd
        let json_cmd = Json::Cmd(r#"{"c": 3}"#.to_string());
        let value = Arc::<serde_json::Value>::try_from(&json_cmd).unwrap();
        assert_eq!(value["c"], 3);
    }

    #[test]
    fn test_json_from_str_invalid() {
        // This will be treated as Json::String because it's not a valid path, url or cmd
        let input = "invalid json";
        let json = Json::from_str(input).unwrap();
        match json {
            Json::String(s) => assert_eq!(s, input),
            _ => panic!("Expected Json::String"),
        }

        // Test parsing invalid json from Json::String
        let json_obj = Json::String(input.to_string());
        let result = Arc::<serde_json::Value>::try_from(&json_obj);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_cmd_error_output() {
        // command exists but fails
        let result = run_cmd("ls /non_existent_directory_12345");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("run command failed"));
    }
}