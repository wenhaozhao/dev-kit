use dev_kit::command::base64;
use dev_kit::command::formatter::format_text;
use dev_kit::command::json::parse_json_or_jsonl;
use dev_kit::command::qrcode::generator::QrCodeImageVal;
use dev_kit::command::qrcode::{OutputType, QrContent, QrEcLevel, QrVersion, generator};
use dev_kit::command::text::{ContentType, detect_content_type};
use dev_kit::command::textdiff::diff_lines;
use dev_kit::command::time::{Time, TimeCommand, TimeFormat, TimestampUnit};
use dev_kit::command::uri::Uri;
use std::str::FromStr;

#[test]
fn jsonl_to_formatted_json_diff_workflow() {
    let parsed = parse_json_or_jsonl("{\"name\":\"first\"}\n\n{\"name\":\"second\"}").unwrap();
    assert!(parsed.is_array());
    let formatted = format_text("{\"b\":2,\"a\":1}", ContentType::Json).unwrap();
    assert_eq!(formatted.lines().next(), Some("{"));
    let changes = diff_lines("one\ntwo", "one\nthree", None, None);
    assert!(changes.iter().any(|line| line.content == "two"));
}

#[test]
fn detection_and_legacy_utilities_work_without_network() {
    assert_eq!(detect_content_type("name: devkit", None), ContentType::Yaml);
    assert_eq!(
        Uri::from_str("foo%20bar").unwrap().decode().unwrap(),
        "foo bar"
    );
    let encoded = base64::encode("devkit", false, false).unwrap();
    assert_eq!(base64::decode(&encoded, false, false).unwrap().0, b"devkit");
}

#[test]
fn invalid_jsonl_reports_the_source_line() {
    let error = parse_json_or_jsonl("{}\ninvalid").unwrap_err();
    assert!(error.to_string().contains("line 2"));
}

#[test]
fn time_and_qrcode_work_offline() {
    let command = TimeCommand::Parse {
        time: Time::from_str("0").unwrap(),
        input_unit: Some(TimestampUnit::Seconds),
        timezone: None,
        format: Some(TimeFormat::Timestamp),
        output_unit: Some(TimestampUnit::Seconds),
    };
    assert!(command.run_actual().is_ok());

    let content = QrContent::from_str("devkit").unwrap();
    let level = QrEcLevel::from_str("q").unwrap();
    let image = generator::generate(&content, &level, &QrVersion::Auto, OutputType::Text).unwrap();
    assert!(matches!(&*image, QrCodeImageVal::Text(text) if !text.is_empty()));
}
