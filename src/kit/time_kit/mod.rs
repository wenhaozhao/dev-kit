use anyhow::anyhow;
use chrono::Utc;
use derive_more::{Deref, Display, From, FromStr};
use std::io::Read;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum InputTime {
    Now,
    Timestamp {
        val: Timestamp,
        unit: TimeUnit,
    },
    StringTime(StringTime),
}

#[derive(Debug, Copy, Clone, Display, Deref, From)]
#[display("{_0}")]
pub struct Timestamp(i64);

impl TryFrom<String> for Timestamp {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = if value.eq("-") {
            let mut string = String::new();
            let _ = std::io::stdin().lock().read_to_string(&mut string)
                .map_err(|err| anyhow!("read from stdin failed, {}", err))?;
            string
        } else {
            value
        };
        let val = value.trim().parse::<i64>().map_err(|err|
            anyhow!("Invalid timestamp: {}, {}", value, err)
        )?;
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

impl TryFrom<String> for StringTime {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = if value.eq("-") {
            let mut string = String::new();
            let _ = std::io::stdin().lock().read_to_string(&mut string)
                .map_err(|err| anyhow!("read from stdin failed, {}", err))?;
            string
        } else {
            value
        };
        Ok(StringTime(value.trim().to_string()))
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

pub fn time_convertor(
    req: InputTime,
    timezone: Option<chrono::FixedOffset>,
    format_to: TimeFormat,
) -> crate::Result<String> {
    let default_timezone = *chrono::Local::now().offset();
    let result = match req {
        InputTime::Now => {
            let timezone = timezone.unwrap_or(default_timezone);
            let time = chrono::Local::now().with_timezone(&timezone);
            match format_to {
                TimeFormat::RFC3339 => time.to_rfc3339(),
                TimeFormat::Timestamp => time.timestamp_millis().to_string(),
                TimeFormat::Format(format) => time.format(&format).to_string(),
            }
        }
        InputTime::Timestamp { val, unit, } => {
            let timezone = timezone.unwrap_or(default_timezone);
            let time = match unit {
                TimeUnit::Seconds => chrono::DateTime::from_timestamp(*val, 0),
                TimeUnit::Milliseconds => chrono::DateTime::from_timestamp_millis(*val),
            }.ok_or(anyhow!("Invalid timestamp {}{}", val, unit))?.with_timezone(&timezone);
            match format_to {
                TimeFormat::RFC3339 => time.to_rfc3339(),
                TimeFormat::Timestamp => time.timestamp_millis().to_string(),
                TimeFormat::Format(format) => time.format(&format).to_string(),
            }
        }
        InputTime::StringTime(val) => {
            let timezone = timezone.unwrap_or(default_timezone);
            let time = chrono::DateTime::<Utc>::try_from(&val).map_err(|err| {
                log::debug!("Failed to parse time string: {}, error: {}", val, err);
                anyhow!("Invalid string time {val}")
            })?.with_timezone(&timezone);
            match format_to {
                TimeFormat::RFC3339 => time.to_rfc3339(),
                TimeFormat::Timestamp => time.timestamp_millis().to_string(),
                TimeFormat::Format(format) => time.format(&format).to_string(),
            }
        }
    };
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_time() {
        let result = time_convertor(InputTime::Now, None, TimeFormat::RFC3339);
        assert!(result.is_ok());
        dbg!(result.unwrap());
    }

    #[test]
    fn test_get_time_with_timezone_east_8() {
        let result = time_convertor(InputTime::Now, Some(chrono::FixedOffset::east_opt(8 * 3600).unwrap()), TimeFormat::RFC3339);
        assert!(result.is_ok());
        dbg!(result.unwrap());
    }

    #[test]
    fn test_get_time_with_timezone_0() {
        let result = time_convertor(InputTime::Now, Some(chrono::FixedOffset::east_opt(0).unwrap()), TimeFormat::RFC3339);
        assert!(result.is_ok());
        dbg!(result.unwrap());
    }
}