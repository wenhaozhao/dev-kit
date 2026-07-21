use dev_kit::command::qrcode::generator::QrCodeImageVal;
use dev_kit::command::qrcode::{generator, OutputType, QrContent, QrEcLevel, QrVersion};
use dev_kit::command::time::{Time, TimeCommand, TimeFormat, TimestampUnit};
use std::str::FromStr;


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
