use base64::Engine;
use dev_kit::command::qrcode::{generator, OutputType, QrContent, QrEcLevel, QrVersion};
use std::io::Read;
use std::ops::Deref;
use std::str::FromStr;

#[tauri::command]
pub fn save_image_to_file(path: String, base64_content: String) -> Result<(), String> {
    let base64_data = if base64_content.contains(",") {
        base64_content.split(',').nth(1).unwrap_or(&base64_content)
    } else {
        &base64_content
    };
    let buffer = base64::engine::general_purpose::STANDARD.decode(base64_data).map_err(|e|
        e.to_string()
    )?;
    std::fs::write(&path, buffer).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct QrCodeResponse {
    data: String,
    ec_level: String,
    version: String,
}

#[tauri::command]
pub fn generate_qrcode(
    content: String,
    ec_level: Option<String>,
    version: Option<serde_json::Value>,
    output_type: Option<String>,
) -> Result<QrCodeResponse, String> {
    let content = QrContent::from_str(&content).map_err(|e| e.to_string())?;
    let ec_level = ec_level
        .map(|s| QrEcLevel::from_str(&s).unwrap_or_default())
        .unwrap_or_default();
    let version = version
        .map(|v| {
            let s = match v {
                serde_json::Value::String(s) => s,
                serde_json::Value::Number(n) => n.to_string(),
                _ => "auto".to_string(),
            };
            QrVersion::from_str(&s).unwrap_or_default()
        })
        .unwrap_or_default();
    let output_type = output_type
        .map(|s| OutputType::from_str(&s).unwrap_or(OutputType::Svg))
        .unwrap_or(OutputType::Svg);

    let result = generator::generate(
        &content,
        &ec_level,
        &version,
        output_type,
    ).map_err(|e| e.to_string())?;

    let data = match result.deref() {
        generator::QrCodeImageVal::Svg(path) => {
            std::fs::read_to_string(path).map_err(|e| e.to_string())?
        }
        generator::QrCodeImageVal::Image(path) => {
            let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
            let base64 = base64::engine::general_purpose::STANDARD.encode(buffer);
            format!("data:image/png;base64,{}", base64)
        }
        _ => return Err("Unexpected QR code output type".to_string()),
    };

    Ok(QrCodeResponse {
        data,
        ec_level: result.ec_level.to_string(),
        version: result.version.to_string(),
    })
}