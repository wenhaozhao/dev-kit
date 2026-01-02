use super::Timestring;
use chrono::TimeZone;
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
    const FORMATES: &[&str] = &[
        "%a %b %d %H:%M:%S %Z %Y",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y/%m/%d %H:%M:%S",
        "%Y/%m/%dT%H:%M:%S",
        "%Y/%m/%d %H:%M:%S%.f",
        "%Y/%m/%dT%H:%M:%S%.f",
        "%m-%d-%Y %H:%M:%S",
        "%m-%d-%YT%H:%M:%S",
        "%m-%d-%Y %H:%M:%S%.f",
        "%m-%d-%YT%H:%M:%S%.f",
        "%m/%d/%Y %H:%M:%S",
        "%m/%d/%YT%H:%M:%S",
        "%m/%d/%Y %H:%M:%S%.f",
        "%m/%d/%YT%H:%M:%S%.f",
        "%d-%m-%Y %H:%M:%S",
        "%d-%m-%YT%H:%M:%S",
        "%d-%m-%Y %H:%M:%S%.f",
        "%d-%m-%YT%H:%M:%S%.f",
        "%d/%m/%Y %H:%M:%S",
        "%d/%m/%YT%H:%M:%S",
        "%d/%m/%Y %H:%M:%S%.f",
        "%d/%m/%YT%H:%M:%S%.f",
        "%Y-%m-%d",
        "%Y/%m/%d",
        "%m-%d-%Y",
        "%d-%m-%Y",
    ];
    let default_timezone = *chrono::Local::now().offset();
    for &format in FORMATES {
        match chrono::NaiveDateTime::parse_from_str(val, format) {
            Ok(naive_dt) => {
                if let Some(time) = default_timezone.from_local_datetime(&naive_dt).single() {
                    return Some(time.to_utc());
                }
            }
            Err(err) => {
                log::debug!("guess time {val} by {format} failed, err: {err:?}");
            }
        }
        match chrono::NaiveDate::parse_from_str(val, format).map(|it| it.and_hms_opt(0, 0, 0)) {
            Ok(naive_date) => {
                if let Some(time) = naive_date.and_then(|it| default_timezone.from_local_datetime(&it).single()) {
                    return Some(time.to_utc());
                }
            }
            Err(err) => {
                log::debug!("guess time {val} by {format} failed, err: {err:?}");
            }
        }
    }
    log::debug!("guess tim {val} failed with all known formats");
    None
}