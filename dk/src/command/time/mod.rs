use crate::command::read_stdin;
use anyhow::anyhow;
use chrono::{FixedOffset, Utc};
use derive_more::{Deref, Display, From, FromStr};
use serde::Serialize;
use std::panic;
use std::str::FromStr;

#[derive(clap::Subcommand)]
pub enum TimeCommand {
    #[clap(about = "get current time")]
    Now {
        #[arg(long, short, help = "output timezone, alias tz, default to LOCAL", alias = "tz")]
        timezone: Option<FixedOffset>,
        #[arg(long, short, help = "output time format: rfc3339(default), timestamp(ts) or custom format")]
        format: Option<TimeFormat>,
        #[arg(long, help = "output unix-timestamp unit, s or ms, alias iu, default to ms", alias = "iu")]
        output_unit: Option<TimestampUnit>,
    },
    #[clap(about = "time paser")]
    Parse {
        #[arg(help = "input time, support unix-timestamp or string time, eg. 2023-01-01 12:00:00", default_value = "-")]
        time: Time,
        #[arg(long, help = "input unix-timestamp unit, s or ms, alias iu, default to ms", alias = "iu")]
        input_unit: Option<TimestampUnit>,
        #[arg(long, short, help = "input timezone, alias tz, default to local", alias = "tz")]
        timezone: Option<FixedOffset>,
        #[arg(long, short, help = "output time format: rfc3339(default), timestamp(ts) or custom format")]
        format: Option<TimeFormat>,
        #[arg(long, help = "output unix-timestamp unit, s or ms, alias ou, default to ms", alias = "ou")]
        output_unit: Option<TimestampUnit>,
    },
}


#[derive(Debug, Clone, Display, Serialize)]
pub enum Time {
    StringTime(Timestring),
    Timestamp(Timestamp),
}
#[derive(Debug, Clone, Display, Deref, From, FromStr, Serialize)]
#[display("{_0}")]
pub struct Timestring(String);
mod timestring_guess;
#[derive(Debug, Copy, Clone, Display, Deref, From, Serialize)]
#[display("{_0}")]
pub struct Timestamp(i64);

#[derive(Debug, Copy, Clone, Display, Default)]
pub enum TimestampUnit {
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
        let TimeFormatterVal { output: result, .. } = self.run_actual()?;
        println!("{}", result);
        Ok(())
    }
}

impl TimeCommand {
    pub fn run_actual(&self) -> crate::Result<TimeFormatterVal> {
        match self {
            TimeCommand::Now { timezone, format, output_unit, } => {
                let time = Utc::now().with_timezone(
                    &timezone.unwrap_or(*chrono::Local::now().offset())
                );
                let result = Self::time_formatter(&time, &format, &output_unit.unwrap_or_default())?;
                Ok(TimeFormatterVal {
                    intput: Time::StringTime(Timestring(result.clone())),
                    timestamp: Timestamp(time.timestamp_millis()),
                    output: result,
                })
            }
            TimeCommand::Parse { time: input_time, input_unit, timezone, format, output_unit, } => {
                let time = match (input_time, input_unit.unwrap_or_default()) {
                    (Time::Timestamp(time), TimestampUnit::Seconds) => {
                        chrono::DateTime::from_timestamp(**time, 0).ok_or(anyhow!("Invalid timestamp {}seconds", time))?
                    }
                    (Time::Timestamp(time), TimestampUnit::Milliseconds) => {
                        chrono::DateTime::from_timestamp_millis(**time).ok_or(anyhow!("Invalid timestamp {}milliseconds", time))?
                    }
                    (Time::StringTime(time), _) => {
                        chrono::DateTime::<Utc>::try_from(time).map_err(|err| {
                            log::debug!("Failed to parse time string: {}, error: {}", time, err);
                            anyhow!("Invalid string time {time}")
                        })?
                    }
                }.with_timezone(
                    &timezone.unwrap_or(*chrono::Local::now().offset())
                );
                let result = Self::time_formatter(&time, &format, &output_unit.unwrap_or_default())?;
                Ok(TimeFormatterVal {
                    intput: input_time.clone(),
                    timestamp: Timestamp(time.timestamp_millis()),
                    output: result,
                })
            }
        }
    }
}

impl TimeCommand {
    fn time_formatter(time: &chrono::DateTime<FixedOffset>, format: &Option<TimeFormat>, unit: &TimestampUnit) -> crate::Result<String> {
        let format = format.clone().unwrap_or_default();
        let result = match format {
            TimeFormat::RFC3339 => time.to_rfc3339(),
            TimeFormat::Timestamp => match unit {
                TimestampUnit::Seconds => time.timestamp().to_string(),
                TimestampUnit::Milliseconds => time.timestamp_millis().to_string(),
            },
            TimeFormat::Format(format) => panic::catch_unwind(|| {
                time.format(&format).to_string()
            }).map_err(|_| anyhow!("Invalid time format"))?,
        };
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TimeFormatterVal {
    intput: Time,
    timestamp: Timestamp,
    output: String,
}

impl FromStr for Time {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Some(string) = read_stdin() {
            Ok(Self::from_str(string.trim())?)
        } else {
            if let Ok(val) = value.parse::<i64>() {
                Ok(Self::Timestamp(val.into()))
            } else {
                Ok(Self::StringTime(value.to_string().into()))
            }
        }
    }
}

impl FromStr for TimestampUnit {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "s" | "seconds" => Ok(TimestampUnit::Seconds),
            "ms" | "milliseconds" => Ok(TimestampUnit::Milliseconds),
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