use std::str::FromStr;
use chrono::FixedOffset;
use dev_kit::command::time::{Time, TimeCommand, TimeFormat, TimestampUnit};

#[tauri::command]
pub fn parse_time(
    time: String,
    timezone: Option<String>,
    format: Option<String>,
) -> Result<String, String> {
    let cmd = TimeCommand::Parse {
        time: Time::from_str(&time).map_err(|e| e.to_string())?,
        input_unit: Some(TimestampUnit::Milliseconds),
        timezone: timezone.as_deref().and_then(|tz| FixedOffset::from_str(tz).ok()),
        format: format.as_deref().and_then(|fmt| TimeFormat::from_str(fmt).ok()),
        output_unit: Some(TimestampUnit::Milliseconds),
    };
    let result = cmd.run_actual().map_err(|e| e.to_string())?;
    Ok(serde_json::to_string(&result).map_err(|err| err.to_string())?)
}