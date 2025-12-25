//! dev-kit

use clap::Parser;

mod kit;

type Result<T> = anyhow::Result<T>;

fn main() -> Result<()> {
    let CLI {
        command,
        verbose
    } = CLI::parse();
    let _ = init_logger(verbose)?;
    match command {
        Command::DecodeURI { uri_component } => {
            let result = kit::decode_uri_component(uri_component)?;
            println!("{}", result);
        }
    }

    Ok(())
}

#[derive(clap::Parser)]
struct CLI {
    #[arg(short, long, help = "enable verbose output")]
    verbose: bool,
    #[clap(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    #[clap(about = "decode uri component, alias du", alias = "du")]
    DecodeURI{
        #[arg(help = "uri component to decode")]
        uri_component: String
    }
}

fn init_logger(verbose: bool) -> Result<()> {
    env_logger::Builder::default()
        .filter_level(if verbose { log::LevelFilter::Debug } else { log::LevelFilter::Info })
        .try_init()?;
    Ok(())
}