//! dev-kit

use devkit::command::Command;
use clap::Parser;

type Result<T> = anyhow::Result<T>;

fn main() -> Result<()> {
    env_logger::init();
    let CLI {
        command,
    } = CLI::parse();
    match command.run() {
        Ok(_) => {}
        Err(err) => {
            log::error!("{}", err);
        }
    }
    Ok(())
}

#[derive(clap::Parser)]
struct CLI {
    #[clap(subcommand)]
    command: devkit::command::Commands,
}
