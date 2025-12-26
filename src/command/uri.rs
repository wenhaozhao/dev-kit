use anyhow::anyhow;
use derive_more::Display;
use percent_encoding::percent_decode_str;
use std::io::Read;
use std::str::FromStr;

#[derive(clap::Subcommand)]
pub enum UriCommand {
    #[clap(about = "decode uri component")]
    Decode {
        #[arg(help = "uri component to decode", default_value = "-")]
        uri: Uri
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
        }
    }
}

#[derive(Debug, Clone, Display)]
pub enum Uri {
    #[display("{_0}")]
    Url(url::Url),
    #[display("{_0}")]
    String(String),
}


impl Uri {
    pub fn decode(&self) -> crate::Result<String> {
        match self {
            Uri::Url(url) => {
                Ok(percent_decode_str(url.as_str()).decode_utf8()?.to_string())
            }
            Uri::String(string) => {
                Ok(percent_decode_str(string.as_str()).decode_utf8()?.to_string())
            }
        }
    }
}


impl FromStr for Uri {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq("-") {
            let mut string = String::new();
            let _ = std::io::stdin().lock().read_to_string(&mut string)
                .map_err(|err| anyhow!("read from stdin failed, {}", err))?;
            match string.trim() {
                "-" => Err(anyhow!("Not a valid input")),
                _ => Ok(Self::from_str(&string)?)
            }
        } else {
            match url::Url::parse(&value) {
                Ok(url) => Ok(Uri::Url(url)),
                Err(_) => Ok(Uri::String(value.to_string())),
            }
        }
    }
}

impl TryFrom<&String> for Uri {
    type Error = anyhow::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}


