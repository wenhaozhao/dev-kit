use anyhow::anyhow;
use derive_more::Display;
use itertools::Itertools;
use jsonpath_rust::JsonPath;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(clap::Subcommand)]
pub enum JsonCommand {
    #[clap(about = "json beautify, alias 'format'", alias = "format")]
    Beauty {
        #[arg(help = "json-string, json-file-path", default_value = "-")]
        json: Json,
        #[arg(short, long, help = "file to write output")]
        file: Option<PathBuf>,
    },
    #[clap(about = "json query, alias 'search'", alias = "search")]
    Query {
        #[arg(help = "json-string, json-file-path", default_value = "-")]
        json: Json,
        #[arg(short, long, help = "json path to extract")]
        query: String,
        #[arg(short, long, help = "file to write output")]
        file: Option<PathBuf>,
    },
}

impl super::Command for JsonCommand {
    fn run(&self) -> crate::Result<()> {
        match self {
            JsonCommand::Beauty { json, file } => {
                let result = json.beautify()?;
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
            JsonCommand::Query { json, query, file } => {
                let result = json.query(query)?;
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
        }
    }
}

#[derive(Debug, Clone, Display)]
pub enum Json {
    #[display("{_0}")]
    String(String),
    #[display("{}", _0.display())]
    Path(PathBuf),
}

impl Json {
    fn beautify(&self) -> crate::Result<String> {
        let json = serde_json::Value::try_from(self)?;
        Ok(serde_json::to_string_pretty(&json)?)
    }

    fn query(&self, query: &str) -> crate::Result<Vec<String>> {
        let json = serde_json::Value::try_from(self)?;
        let query_result = json.query(query).map_err(|err| anyhow!(r#"Invalid json path: {}"#, err))?;
        let arr = query_result.iter().flat_map(|&it|
            serde_json::to_string(&it).map_err(|err|
                anyhow!(r#"Invalid json {}"#, err)
            )).collect_vec();
        Ok(arr)
    }
}
impl TryFrom<&Json> for serde_json::Value {
    type Error = anyhow::Error;

    fn try_from(input: &Json) -> Result<Self, Self::Error> {
        let json = match input {
            Json::String(input) => {
                serde_json::from_str::<serde_json::Value>(&input).map_err(|err|
                    anyhow!(r#"
                    Invalid json format:
                    {}"#, err)
                )?
            }
            Json::Path(path) => {
                let file = std::fs::File::open(&path).map_err(|err| anyhow!("open file {} failed, {}", path.display(), err))?;
                serde_json::from_reader::<_, serde_json::Value>(file).map_err(|err|
                    anyhow!(r#"
                    Invalid json format:
                    {}"#, err)
                )?
            }
        };
        Ok(json)
    }
}

impl FromStr for Json {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq("-") {
            let mut string = String::new();
            let _ = std::io::stdin().lock().read_to_string(&mut string)
                .map_err(|err| anyhow!("read from stdin failed, {}", err))?;
            return Ok(Json::String(string));
        }
        let path = PathBuf::from_str(value)?;
        if fs::exists(&path).unwrap_or(false) {
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