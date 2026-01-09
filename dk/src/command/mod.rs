
#[derive(clap::Subcommand)]
pub enum Commands {
    #[clap(about = "uri-tools")]
    Uri {
        #[clap(subcommand)]
        command: uri::UriCommand,
    },
    #[clap(about = "json-tools")]
    Json {
        #[clap(subcommand)]
        command: json::JsonCommand,
    },
    #[clap(about = "time-tools")]
    Time {
        #[clap(subcommand)]
        command: time::TimeCommand,
    },
    #[clap(name = "qrcode", about = "qrcode generator, alias 'qr'", alias = "qr")]
    QrCode(qrcode::QrCodeArgs,)
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
            Commands::QrCode (args) => args.run(),
        }
    }
}

mod http_parser;
pub mod uri;
pub mod json;
pub mod time;
pub mod qrcode;


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

#[cfg(not(feature = "read_stdin"))]
fn read_stdin() -> Option<String> {
    None
}

