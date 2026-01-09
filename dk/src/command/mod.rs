use anyhow::anyhow;
use std::ops::Deref;
use std::str::FromStr;

#[derive(clap::Subcommand)]
pub enum Commands {
    #[clap(about = "uri tools", alias = "url")]
    Uri {
        #[clap(subcommand)]
        command: uri::UriCommand,
    },
    #[clap(about = "json tools")]
    Json {
        #[clap(subcommand)]
        command: json::JsonCommand,
    },
    #[clap(about = "time tools")]
    Time {
        #[clap(subcommand)]
        command: time::TimeCommand,
    },
    #[clap(name = "qrcode", about = "qrcode generator tools, alias 'qr'", alias = "qr")]
    QrCode(qrcode::QrCodeArgs),
    #[clap(about = "base64 tools, alias 'b64'", alias = "b64")]
    Base64 {
        #[clap(subcommand)]
        command: base64::Base64Command,
    },
}

pub trait Command {
    fn run(&self) -> crate::Result<()>;
}

impl Command for Commands {
    fn run(&self) -> crate::Result<()> {
        match self {
            Commands::Uri { command } => command.run(),
            Commands::Json { command } => command.run(),
            Commands::Time { command } => command.run(),
            Commands::QrCode(args) => args.run(),
            Commands::Base64 { command } => command.run(),
        }
    }
}

pub mod base64;
mod http_parser;
pub mod json;
pub mod qrcode;
pub mod time;
pub mod uri;

#[cfg(feature = "read_stdin")]
fn read_stdin() -> Option<String> {
    use std::io::{BufRead, IsTerminal};
    let stdin = std::io::stdin().lock();
    if stdin.is_terminal() {
        None
    } else {
        let mut lines = vec![];
        for line in stdin.lines() {
            match line {
                Ok(line) => {
                    let _ = lines.push(line);
                }
                Err(err) => {
                    log::error!("read from stdin failed, {}", err);
                    break;
                }
            }
        }
        let string = lines.join("\n");
        Some(string)
    }
}

#[derive(Debug, Clone)]
pub struct StringInput(String);

impl Deref for StringInput {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for StringInput {
    type Err = anyhow::Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty()
            && let Some(string) = read_stdin()
        {
            if !string.is_empty() {
                return Ok(Self::from_str(&string)?);
            }
        }
        if value.is_empty() {
            Err(anyhow!("Invalid input"))
        } else {
            Ok(Self(value.to_string()))
        }
    }
}

#[cfg(not(feature = "read_stdin"))]
fn read_stdin() -> Option<String> {
    None
}
