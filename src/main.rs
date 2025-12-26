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
            let result = kit::time_convertor(
                kit::InputTime::Now,
                timezone,
                format.unwrap_or_default(),
            )?;
            println!("{}", result);
        }
        Command::TimestampParser {
            timestamp, unit, timezone, format,
        } => {
            let result = kit::time_convertor(
                kit::InputTime::Timestamp { val: timestamp.try_into()?, unit: unit.unwrap_or_default() },
                timezone,
                format.unwrap_or_default(),
            )?;
            println!("{}", result);
        }
        Command::TimestringParser {
            timestring, timezone, format,
        } => {
            let result = kit::time_convertor(
                kit::InputTime::StringTime(timestring.try_into()?),
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
        #[arg(help = "uri component to decode, or default stdin with '-'", default_value = "-")]
        input: String
    },
    #[clap(about = "json beautify, alias jb", alias = "jb")]
    JsonBeautify {
        #[arg(help = "json-string, json-file-path, or default stdin with '-'", default_value = "-")]
        input: String,
        #[arg(short, long, help = "file to write output")]
        file: Option<PathBuf>,
    },
    #[clap(about = "json extract, alias jq", alias = "jq")]
    JsonQuery {
        #[arg(help = "json-string, json-file-path, or default stdin with '-'", default_value = "-")]
        input: String,
        #[arg(short, long, help = "json path to extract")]
        query: String,
        #[arg(short, long, help = "file to write output")]
        file: Option<PathBuf>,
    },
    #[clap(about = "get current time, alias now", alias = "now")]
    TimeNow {
        #[arg(long, short, help = "timezone, alias tz, default to LOCAL", alias = "tz")]
        timezone: Option<chrono::FixedOffset>,
        #[arg(long, short, help = "time format: rfc3339(default), timestamp(ts) or custom format")]
        format: Option<kit::TimeFormat>,
    },
    #[clap(about = "parse timestamp, alias ts", alias = "ts")]
    TimestampParser {
        #[arg(help = "timestamp to parse, or default stdin with '-'", default_value = "-")]
        timestamp: String,
        #[arg(short, long, help = "time unit, seconds(s) or milliseconds(ms, default)")]
        unit: Option<kit::TimeUnit>,
        #[arg(long, short, help = "timezone, alias tz, default to LOCAL", alias = "tz")]
        timezone: Option<chrono::FixedOffset>,
        #[arg(long, short, help = "time format: rfc3339(default), timestamp(ts) or custom format")]
        format: Option<kit::TimeFormat>,
    },
    #[clap(about = "parse timestring, alias tp", alias = "tp")]
    TimestringParser {
        #[arg(help = "time string to parse, eg. 2023-01-01 12:00:00, or default stdin with '-'", default_value = "-")]
        timestring: String,
        #[arg(long, short, help = "timezone, alias tz, default to LOCAL", alias = "tz")]
        timezone: Option<chrono::FixedOffset>,
        #[arg(long, short, help = "time format: rfc3339(default), timestamp(ts) or custom format")]
        format: Option<kit::TimeFormat>,
    },
}

fn init_logger(verbose: bool) -> Result<()> {
    env_logger::Builder::default()
        .filter_level(if verbose { log::LevelFilter::Debug } else { log::LevelFilter::Info })
        .try_init()?;
    Ok(())
}