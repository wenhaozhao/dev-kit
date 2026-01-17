use crate::devkit;
#[tauri::command]
pub fn base64_decode(input: String, url_safe: bool, no_pad: bool) -> Result<String, String> {
    let data = devkit::command::base64::decode(&input, url_safe, no_pad).map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&data).to_string())
}

#[tauri::command]
pub fn base64_encode(input: String, url_safe: bool, no_pad: bool) -> Result<String, String> {
    devkit::command::base64::encode(&input, url_safe, no_pad).map_err(|e| e.to_string())
}
