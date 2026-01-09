use derive_more::{Deref, Display, FromStr};
use itertools::Itertools;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(clap::Subcommand)]
pub enum UriCommand {
    #[clap(about = "decode uri component, alias d", alias = "d")]
    Decode {
        #[arg(help = "uri component to decode", default_value = "")]
        uri: Uri
    },
    #[clap(about = "encode uri component, alias e", alias = "e")]
    Encode {
        #[arg(help = "uri component to encode", default_value = "")]
        uri: Uri
    },
    #[clap(about = "parse uri component, alias p", alias = "p")]
    Parse {
        #[arg(help = "uri component to encode", default_value = "")]
        uri: Uri,
        #[arg(long, help = "component filter of uri: scheme, authority, host, port, path, query", value_delimiter = ',')]
        filter: Option<Vec<UriComponent>>,
    },
}

impl super::Command for UriCommand {
    fn run(&self) -> crate::Result<()> {
        match self {
            UriCommand::Decode { uri } => {
                let result = uri.decode()?;
                println!("{result}");
                Ok(())
            }
            UriCommand::Encode { uri } => {
                let result = uri.encode()?;
                println!("{result}");
                Ok(())
            }
            UriCommand::Parse { uri, filter } => {
                let result = uri.parse(filter)?;
                let filter_len = filter.as_ref().map(|it| it.len()).unwrap_or(0);
                for it in result {
                    match filter_len {
                        1 => {
                            if let UriComponentValue::Query(parts) = it {
                                let string = parts.into_iter().map(|(_, v)|
                                    v.to_string()
                                ).next().unwrap_or_default();
                                println!("{}", string)
                            } else {
                                println!("{}", it.string_value())
                            }
                        }
                        _ => {
                            if let UriComponentValue::Query(parts) = it {
                                let parts = parts.into_iter().map(|(k, v)|
                                    format!("   {}={}", k, v)
                                ).join("\n");
                                println!("query:\n{parts}")
                            } else {
                                println!("{}: {}", it.name(), it.string_value())
                            }
                        }
                    }
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Display)]
pub enum Uri {
    Url(url::Url),
    String(String),
    HttpRequest(super::http_parser::HttpRequest),
}

mod uri;

#[derive(Debug, Clone, Display, Deserialize)]
pub enum UriComponent {
    Scheme,
    Authority,
    Host,
    Port,
    Path,
    #[display("{_0:?}")]
    Query(Option<QueryPartName>),
}

#[derive(Debug, Clone)]
pub enum UriComponentValue {
    Scheme(String),
    Authority(Option<String>),
    Host(String),
    Port(u16),
    Path(String),
    Query(BTreeMap<QueryPartName, QueryPartVal>),
}

#[derive(Debug, Clone, Deserialize, Deref, Display, Eq, PartialEq, Ord, PartialOrd, FromStr)]
pub struct QueryPartName(String);
#[derive(Debug, Clone, Deserialize)]
pub enum QueryPartVal {
    Single(Option<String>),
    Multi(Vec<String>),
}

mod components;
