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
        }
    }
}
pub mod uri;
pub mod json;
pub mod time;
mod http_parser;


