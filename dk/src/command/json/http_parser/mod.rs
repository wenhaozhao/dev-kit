mod jetbrains_http;

use anyhow::anyhow;
use derive_more::Display;
pub use jetbrains_http::*;
use lazy_static::lazy_static;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Clone, Display)]
pub enum HttpRequest {
    JetBrainsHttp(JetBrainsHttp),
    Uri(url::Url),
    #[display("{}", _0.display())]
    FilePath(PathBuf),
}

impl TryFrom<&HttpRequest> for Url {
    type Error = anyhow::Error;

    fn try_from(value: &HttpRequest) -> Result<Self, Self::Error> {
        match value {
            HttpRequest::JetBrainsHttp(it) => {
                Url::try_from(&***it)
            }
            HttpRequest::Uri(url) => {
                Ok(url.clone())
            }
            HttpRequest::FilePath(path) => {
                Url::from_file_path(path).map_err(|_| anyhow!("Invalid file path: {}", path.display()))
            }
        }
    }
}

impl FromStr for HttpRequest {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Ok(jetbrains_http) = JetBrainsHttp::from_str(value) {
            Ok(Self::JetBrainsHttp(jetbrains_http))
        } else if let Ok(url) = Url::parse(value) {
            let schema = url.scheme().to_lowercase();
            match schema.as_str() {
                "https" | "http" => {
                    Ok(Self::Uri(url))
                }
                "file" => {
                    Ok(Self::FilePath(PathBuf::from_str(url.path())?))
                }
                _ => Err(anyhow!("Not a valid url: {value}"))
            }
        } else {
            Err(anyhow!("Not a valid http request: {value}"))
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


impl TryFrom<&HttpRequest> for serde_json::Value {
    type Error = anyhow::Error;

    fn try_from(http_request: &HttpRequest) -> Result<Self, Self::Error> {
        match http_request.clone() {
            HttpRequest::JetBrainsHttp(jetbrains_http) => {
                let text = futures::executor::block_on(async move {
                    let h = ASYNC_RT.spawn(async move {
                        let text = reqwest::Client::default().execute(
                            reqwest::Request::try_from(&**jetbrains_http)?
                        ).await.map_err(|err| {
                            log::debug!("{}",err);
                            anyhow!("Invalid http request, {jetbrains_http}")
                        })?.text().await.map_err(|err| {
                            log::debug!("{}",err);
                            anyhow!("Invalid http response, {jetbrains_http}")
                        })?;
                        Ok::<_, anyhow::Error>(text)
                    });
                    h.await
                })??;
                serde_json::from_str(&text).map_err(|err| {
                    log::debug!("{}",err);
                    anyhow!("Invalid json format")
                })
            }
            HttpRequest::Uri(url) => {
                let url = url.clone();
                let text = futures::executor::block_on(async move {
                    let h = ASYNC_RT.spawn(async move {
                        let text = reqwest::get(url.clone()).await.map_err(|err| {
                            log::debug!("{}",err);
                            anyhow!("Invalid http request, url: {url}")
                        })?.text().await.map_err(|err| {
                            log::debug!("{}",err);
                            anyhow!("Invalid http response, url: {url}")
                        })?;
                        Ok::<_, anyhow::Error>(text)
                    });
                    h.await
                })??;
                serde_json::from_str(&text).map_err(|err| {
                    log::debug!("{}",err);
                    anyhow!("Invalid json format")
                })
            }
            HttpRequest::FilePath(path) => {
                let file = fs::File::open(&path).map_err(|err|
                    anyhow!("open file {} failed, {}", path.display(), err)
                )?;
                serde_json::from_reader::<_, serde_json::Value>(file).map_err(|err| {
                    log::debug!("{}",err);
                    anyhow!("Invalid json format")
                })
            }
        }
    }
}