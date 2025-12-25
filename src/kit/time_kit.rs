use std::str::FromStr;

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
            "timestamp" => Ok(TimeFormat::Timestamp),
            _ => Ok(TimeFormat::Format(val.to_string())),
        }
    }
}

pub enum TimeReq {
    Now,
}

pub fn get_time(
    req: TimeReq,
    timezone: Option<chrono::FixedOffset>,
    format: TimeFormat,
) -> crate::Result<String> {
    let result = match req {
        TimeReq::Now => {
            let now = chrono::Local::now();
            let timezone = timezone.unwrap_or(*now.offset());
            let time = chrono::Local::now().with_timezone(&timezone);
            match format {
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
        let result = get_time(TimeReq::Now, None, TimeFormat::RFC3339);
        assert!(result.is_ok());
        dbg!(result.unwrap());
    }

    #[test]
    fn test_get_time_with_timezone_east_8() {
        let result = get_time(TimeReq::Now, Some(chrono::FixedOffset::east_opt(8 * 3600).unwrap()), TimeFormat::RFC3339);
        assert!(result.is_ok());
        dbg!(result.unwrap());
    }

    #[test]
    fn test_get_time_with_timezone_0() {
        let result = get_time(TimeReq::Now, Some(chrono::FixedOffset::east_opt(0).unwrap()), TimeFormat::RFC3339);
        assert!(result.is_ok());
        dbg!(result.unwrap());
    }
}