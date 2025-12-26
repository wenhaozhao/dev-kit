//! dev-kit

use crate::command::Command;
use clap::Parser;

type Result<T> = anyhow::Result<T>;

mod command;

fn main() -> Result<()> {
    let CLI {
        command,
        verbose
    } = CLI::parse();
    let _ = init_logger(verbose)?;
    let _ = command.run()?;
    Ok(())
}

#[derive(clap::Parser)]
struct CLI {
    #[arg(short, long, help = "enable verbose output")]
    verbose: bool,
    #[clap(subcommand)]
    command: command::Commands,
}


fn init_logger(verbose: bool) -> Result<()> {
    env_logger::Builder::default()
        .filter_level(if verbose { log::LevelFilter::Debug } else { log::LevelFilter::Info })
        .try_init()?;
    Ok(())
}