use crate::{SharedAppState, devkit};
use base64::Engine;
use serde_json::json;
#[tauri::command]
pub async fn base64_decode(
    _: tauri::State<'_, SharedAppState>,
    input: String,
    url_safe: bool,
    no_pad: bool,
) -> Result<serde_json::Value, String> {
    let (data, mime) = {
        let (data, mime) =
            devkit::command::base64::decode(&input, url_safe, no_pad).map_err(|e| e.to_string())?;
        (data, mime.unwrap_or(mime::TEXT_PLAIN))
    };

    let data = match mime.type_() {
        mime::TEXT => String::from_utf8_lossy(&data).to_string(),
        mime::IMAGE => {
            let content = base64::engine::general_purpose::STANDARD.encode(&data);
            format!("data:{mime};base64,{content}")
        }
        other => {
            format!("{} is not supported", other)
        }
    };
    Ok(json!({
        "mime": mime.as_ref(),
        "type": mime.type_().as_str(),
        "subtype": mime.subtype().as_str(),
        "data": data,
    }))
}

#[tauri::command]
pub fn base64_encode(
    input: String,
    url_safe: bool,
    no_pad: bool,
) -> Result<serde_json::Value, String> {
    let data =
        devkit::command::base64::encode(&input, url_safe, no_pad).map_err(|e| e.to_string())?;
    let mime = mime::TEXT_PLAIN;
    Ok(json!({
        "mime": mime.as_ref(),
        "type": mime.type_().as_str(),
        "subtype": mime.subtype().as_str(),
        "data": data,
    }))
}
