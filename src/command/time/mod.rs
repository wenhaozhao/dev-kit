use anyhow::anyhow;
use chrono::{FixedOffset, Utc};
use derive_more::{Deref, Display, From};
use std::io::Read;
use std::str::FromStr;

#[derive(clap::Subcommand)]
pub enum TimeCommand {
    #[clap(about = "get current time")]
    Now {
        #[arg(long, short, help = "timezone, alias tz, default to LOCAL", alias = "tz")]
        timezone: Option<FixedOffset>,
        #[arg(long, short, help = "time format: rfc3339(default), timestamp(ts) or custom format")]
        format: Option<TimeFormat>,
    },
    #[clap(about = "time paser")]
    Parse {
        #[arg(help = "time to parse, support unix-timestamp or string time, eg. 2023-01-01 12:00:00", default_value = "-")]
        time: Time,
        #[arg(short, long, help = "unix-timestamp unit, seconds(s) or milliseconds(ms, default)")]
        unit: Option<TimeUnit>,
        #[arg(long, short, help = "timezone, alias tz, default to local", alias = "tz")]
        timezone: Option<FixedOffset>,
        #[arg(long, short, help = "time format: rfc3339(default), timestamp(ts) or custom format")]
        format: Option<TimeFormat>,
    },
}


#[derive(Debug, Clone, Display)]
pub enum Time {
    StringTime(Timestring),
    Timestamp(Timestamp),
}
#[derive(Debug, Clone, Display, Deref, From)]
#[display("{_0}")]
pub struct Timestring(String);
mod timestring_guess;
#[derive(Debug, Copy, Clone, Display, Deref, From)]
#[display("{_0}")]
pub struct Timestamp(i64);

#[derive(Debug, Copy, Clone, Display, Default)]
pub enum TimeUnit {
    #[display("s")]
    Seconds,
    #[default]
    #[display("ms")]
    Milliseconds,
}

#[derive(Debug, Clone, Default)]
pub enum TimeFormat {
    #[default]
    RFC3339,
    Timestamp,
    Format(String),
}

impl super::Command for TimeCommand {
    fn run(&self) -> crate::Result<()> {
        match self {
            TimeCommand::Now { timezone, format } => {
                let timezone = timezone.unwrap_or(*chrono::Local::now().offset());
                let time = chrono::Local::now().with_timezone(&timezone);
                let result = Self::time_formatter(&time, &format)?;
                println!("{result}");
                Ok(())
            }
            TimeCommand::Parse { time, unit, timezone, format, } => {
                let unit = unit.unwrap_or_default();
                let timezone = timezone.unwrap_or(*chrono::Local::now().offset());
                let time = match time {
                    Time::Timestamp(time) => {
                        match unit {
                            TimeUnit::Seconds => chrono::DateTime::from_timestamp(**time, 0),
                            TimeUnit::Milliseconds => chrono::DateTime::from_timestamp_millis(**time),
                        }.ok_or(anyhow!("Invalid timestamp {}{}", time, unit))?.with_timezone(&timezone)
                    }
                    Time::StringTime(time) => {
                        chrono::DateTime::<Utc>::try_from(time).map_err(|err| {
                            log::debug!("Failed to parse time string: {}, error: {}", time, err);
                            anyhow!("Invalid string time {time}")
                        })?.with_timezone(&timezone)
                    }
                };
                let result = Self::time_formatter(&time, &format)?;
                println!("{result}");
                Ok(())
            }
        }
    }
}

impl TimeCommand {
    fn time_formatter(time: &chrono::DateTime<FixedOffset>, format: &Option<TimeFormat>) -> crate::Result<String> {
        let format = format.clone().unwrap_or_default();
        let result = match format {
            TimeFormat::RFC3339 => time.to_rfc3339(),
            TimeFormat::Timestamp => time.timestamp_millis().to_string(),
            TimeFormat::Format(format) => time.format(&format).to_string(),
        };
        Ok(result)
    }
}

impl FromStr for Time {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let val = if value.eq("-") {
            let mut string = String::new();
            let _ = std::io::stdin().lock().read_to_string(&mut string)
                .map_err(|err| anyhow!("read from stdin failed, {}", err))?;
            string.trim().to_string()
        } else {
            value.to_string()
        };
        if let Ok(val) = value.parse::<i64>() {
            Ok(Self::Timestamp(val.into()))
        } else {
            Ok(Self::StringTime(val.into()))
        }
    }
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