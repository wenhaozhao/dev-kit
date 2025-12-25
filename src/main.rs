//! dev-kit

use anyhow::anyhow;
use clap::Parser;
use std::path::PathBuf;

mod kit;

type Result<T> = anyhow::Result<T>;

fn main() -> Result<()> {
    let CLI {
        command,
        verbose
    } = CLI::parse();
    let _ = init_logger(verbose)?;
    match command {
        Command::DecodeURI { input } => {
            let result = kit::decode_uri_component(input)?;
            println!("{}", result);
        }
        Command::JsonBeautify { input, file } => {
            let result = kit::json_beautify(input)?;
            if let Some(file) = file {
                std::fs::write(&file, result).map_err(|err| anyhow!("write to {} failed, {}", file.display(), err))?;
                println!("write to {}", file.display());
            } else {
                println!("{}", result);
            }
        }
        Command::JsonQuery { input, query, file } => {
            let result = kit::json_query(input, &query)?;
            if let Some(file) = file {
                let content = result.join("\n");
                std::fs::write(&file, content)?;
                println!("write to {}", file.display());
            } else {
                for row in result {
                    println!("{}", row);
                }
            }
        }
        Command::TimeNow { timezone, format } => {
            let result = kit::get_time(
                kit::TimeReq::Now,
                timezone,
                format.unwrap_or_default(),
            )?;
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
    DecodeURI {
        #[arg(help = "uri component to decode")]
        input: String
    },
    #[clap(about = "json beautify, alias jb", alias = "jb")]
    JsonBeautify {
        #[arg(help = "json-string or json-file-path")]
        input: String,
        #[arg(short, long, help = "file to write output")]
        file: Option<PathBuf>,
    },
    #[clap(about = "json extract, alias jq", alias = "jq")]
    JsonQuery {
        #[arg(help = "json-string or json-file-path")]
        input: String,
        #[arg(short, long, help = "json path to extract")]
        query: String,
        #[arg(short, long, help = "file to write output")]
        file: Option<PathBuf>,
    },
    #[clap(about = "get current time, alias now", alias = "now")]
    TimeNow {
        #[arg(long,short, help = "timezone, alias tz", alias = "tz")]
        timezone: Option<chrono::FixedOffset>,
        #[arg(long, short, help = "time format")]
        format: Option<kit::TimeFormat>,
    },
}

fn init_logger(verbose: bool) -> Result<()> {
    env_logger::Builder::default()
        .filter_level(if verbose { log::LevelFilter::Debug } else { log::LevelFilter::Info })
        .try_init()?;
    Ok(())
}