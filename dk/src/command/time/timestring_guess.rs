use super::Timestring;
use std::convert::TryFrom;

impl TryFrom<&Timestring> for chrono::DateTime<chrono::Utc> {
    type Error = anyhow::Error;

    fn try_from(Timestring(val): &Timestring) -> Result<Self, Self::Error> {
        parse_from_rfc3339(val)
            .or_else(|| parse_from_rfc2822(val))
            .or_else(|| guess_from_known_formats(val))
            .ok_or_else(|| anyhow::anyhow!("Invalid time string: {}", val))
    }
}

fn parse_from_rfc3339(val: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    match chrono::DateTime::parse_from_rfc3339(val) {
        Ok(val) => Some(val.to_utc()),
        Err(err) => {
            log::debug!("parse time {val} by rfc3339 failed, err: {err:?}");
            None
        }
    }
}

fn parse_from_rfc2822(val: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    match chrono::DateTime::parse_from_rfc2822(val) {
        Ok(val) => Some(val.to_utc()),
        Err(err) => {
            log::debug!("parse time {val} by rfc2822 failed, err: {err:?}");
            None
        }
    }
}

fn guess_from_known_formats(val: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    match dateparser::parse(val) {
        Ok(utc_dt) => {
            Some(utc_dt)
        }
        Err(err) => {
            log::debug!("guess time {val} failed, err: {err:?}");
            None
        }
    }
}