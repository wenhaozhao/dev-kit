//!
//! DevKit
//!
use clap::{CommandFactory, Parser};
use dev_kit as devkit;
use devkit::command::Command;

type Result<T> = anyhow::Result<T>;

fn main() -> Result<()> {
    env_logger::init();
    let Cli { command, version } = Cli::parse();
    if let Some(command) = command {
        match command.run() {
            Ok(_) => {}
            Err(err) => {
                log::error!("{}", err);
            }
        }
    } else if version {
        println!("DevKit v{}", env!("CARGO_PKG_VERSION"));
    } else {
        Cli::command().print_help()?;
    }
    Ok(())
}

#[derive(clap::Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Option<devkit::command::Commands>,
    #[clap(long, short, help = "show version")]
    version: bool,
}
