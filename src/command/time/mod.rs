use anyhow::anyhow;
use chrono::Utc;
use derive_more::{Deref, Display, From};
use std::io::Read;
use std::str::FromStr;

#[derive(clap::Subcommand)]
pub enum TimeCommand {
    #[clap(about = "get current time")]
    Now {
        #[arg(long, short, help = "timezone, alias tz, default to LOCAL", alias = "tz")]
        timezone: Option<chrono::FixedOffset>,
        #[arg(long, short, help = "time format: rfc3339(default), timestamp(ts) or custom format")]
        format: Option<TimeFormat>,
    },
    #[clap(about = "parse timestamp")]
    Timestamp {
        #[arg(help = "timestamp to parse", default_value = "-")]
        time: Timestamp,
        #[arg(short, long, help = "time unit, seconds(s) or milliseconds(ms, default)")]
        unit: Option<TimeUnit>,
        #[arg(long, short, help = "timezone, alias tz, default to LOCAL", alias = "tz")]
        timezone: Option<chrono::FixedOffset>,
        #[arg(long, short, help = "time format: rfc3339(default), timestamp(ts) or custom format")]
        format: Option<TimeFormat>,
    },
    #[clap(about = "parse timestring")]
    Timestring {
        #[arg(help = "time string to parse, eg. 2023-01-01 12:00:00, or default stdin with '-'", default_value = "-")]
        time: StringTime,
        #[arg(long, short, help = "timezone, alias tz, default to LOCAL", alias = "tz")]
        timezone: Option<chrono::FixedOffset>,
        #[arg(long, short, help = "time format: rfc3339(default), timestamp(ts) or custom format")]
        format: Option<TimeFormat>,
    },
}

impl super::Command for TimeCommand {
    fn run(&self) -> crate::Result<()> {
        match self {
            TimeCommand::Now { timezone, format } => {
                let timezone = timezone.unwrap_or(*chrono::Local::now().offset());
                let format = format.clone().unwrap_or_default();
                let time = chrono::Local::now().with_timezone(&timezone);
                let result = match format {
                    TimeFormat::RFC3339 => time.to_rfc3339(),
                    TimeFormat::Timestamp => time.timestamp_millis().to_string(),
                    TimeFormat::Format(format) => time.format(&format).to_string(),
                };
                println!("{result}");
                Ok(())
            }
            TimeCommand::Timestamp { time, unit, timezone, format, } => {
                let unit = unit.unwrap_or_default();
                let timezone = timezone.unwrap_or(*chrono::Local::now().offset());
                let time = match unit {
                    TimeUnit::Seconds => chrono::DateTime::from_timestamp(**time, 0),
                    TimeUnit::Milliseconds => chrono::DateTime::from_timestamp_millis(**time),
                }.ok_or(anyhow!("Invalid timestamp {}{}", time, unit))?.with_timezone(&timezone);
                let format = format.clone().unwrap_or_default();
                let result = match format {
                    TimeFormat::RFC3339 => time.to_rfc3339(),
                    TimeFormat::Timestamp => time.timestamp_millis().to_string(),
                    TimeFormat::Format(format) => time.format(&format).to_string(),
                };
                println!("{result}");
                Ok(())
            }
            TimeCommand::Timestring { time, timezone, format, } => {
                let timezone = timezone.unwrap_or(*chrono::Local::now().offset());
                let time = chrono::DateTime::<Utc>::from_str(time).map_err(|err| {
                    log::debug!("Failed to parse time string: {}, error: {}", time, err);
                    anyhow!("Invalid string time {time}")
                })?.with_timezone(&timezone);
                let format = format.clone().unwrap_or_default();
                let result = match format {
                    TimeFormat::RFC3339 => time.to_rfc3339(),
                    TimeFormat::Timestamp => time.timestamp_millis().to_string(),
                    TimeFormat::Format(format) => time.format(&format).to_string(),
                };
                println!("{result}");
                Ok(())
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Display, Deref, From)]
#[display("{_0}")]
pub struct Timestamp(i64);

impl FromStr for Timestamp {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let val: i64 = if value.eq("-") {
            let mut string = String::new();
            let _ = std::io::stdin().lock().read_to_string(&mut string)
                .map_err(|err| anyhow!("read from stdin failed, {}", err))?;
            string.trim().parse().map_err(|err|
                anyhow!("Invalid timestamp: {}, {}", value, err)
            )?
        } else {
            value.trim().parse().map_err(|err|
                anyhow!("Invalid timestamp: {}, {}", value, err)
            )?
        };
        Ok(Timestamp(val))
    }
}

#[derive(Debug, Copy, Clone, Display, Default)]
pub enum TimeUnit {
    #[display("s")]
    Seconds,
    #[default]
    #[display("ms")]
    Milliseconds,
}

impl FromStr for TimeUnit {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "s" | "seconds" => Ok(TimeUnit::Seconds),
            "ms" | "milliseconds" => Ok(TimeUnit::Milliseconds),
            _ => Err(anyhow!("Invalid time unit: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Display, Deref)]
#[display("{_0}")]
pub struct StringTime(String);
mod string_time_parser;

impl FromStr for StringTime {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq("-") {
            let mut string = String::new();
            let _ = std::io::stdin().lock().read_to_string(&mut string)
                .map_err(|err| anyhow!("read from stdin failed, {}", err))?;
            Ok(StringTime(string.trim().to_string()))
        } else {
            Ok(StringTime(value.trim().to_string()))
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum TimeFormat {
    #[default]
    RFC3339,
    Timestamp,
    Format(String),
}

impl FromStr for TimeFormat {
    type Err = anyhow::Error;

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        let s = val.to_lowercase();
        match s.as_str() {
            "rfc3339" => Ok(TimeFormat::RFC3339),
            "timestamp" | "ts" => Ok(TimeFormat::Timestamp),
            _ => Ok(TimeFormat::Format(val.to_string())),
        }
    }
}