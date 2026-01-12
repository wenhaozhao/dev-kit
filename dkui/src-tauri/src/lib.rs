use base64::Engine;
use chrono::FixedOffset;
use dev_kit as devkit;
use devkit::command::json::{DiffTool, Json};
use devkit::command::qrcode::{OutputType, QrContent, QrEcLevel, QrVersion};
use devkit::command::time::{Time, TimeCommand, TimeFormat, TimestampUnit};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::Read;
use std::ops::Deref;
use std::path::PathBuf;
use std::ptr;
use std::str::FromStr;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct JsonCache {
    cache: HashMap<String, Json>,
}

impl JsonCache {
    fn get_or_parse(&mut self, json_str: &str) -> Result<Json, String> {
        if let Some(parsed) = self.cache.get(json_str) {
            return Ok(parsed.clone());
        }
        let json = {
            let json = Json::from_str(json_str).map_err(|e| e.to_string())?;
            let json_value = Arc::<serde_json::Value>::try_from(&json).map_err(|e| e.to_string())?;
            Json::JsonValue(json_value)
        };
        // Limit cache size to avoid memory issues
        if self.cache.len() > 1 {
            self.cache.clear();
        }
        self.cache.insert(json_str.to_string(), json.clone());
        Ok(json)
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn decode_uri(uri: String) -> Result<String, String> {
    let uri = devkit::command::uri::Uri::from_str(&uri).map_err(|e| e.to_string())?;
    uri.decode().map_err(|e| e.to_string())
}

#[tauri::command]
fn encode_uri(uri: String) -> Result<String, String> {
    let uri = devkit::command::uri::Uri::from_str(&uri).map_err(|e| e.to_string())?;
    uri.encode().map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
struct UriComponentResult {
    name: String,
    value: serde_json::Value,
}

#[tauri::command]
fn parse_uri(uri: String, filter: Option<Vec<String>>) -> Result<Vec<UriComponentResult>, String> {
    let uri = devkit::command::uri::Uri::from_str(&uri).map_err(|e| e.to_string())?;
    let filter = filter.map(|f| {
        f.into_iter()
            .map(|s| devkit::command::uri::UriComponent::from_str(&s))
            .collect::<Result<Vec<_>, _>>()
    }).transpose().map_err(|e| e.to_string())?;
    let components = uri.parse(&filter).map_err(|e| e.to_string())?;
    let result = components
        .into_iter().map(|c| {
        let name = c.name().to_string();
        let value = match c {
            devkit::command::uri::UriComponentValue::Scheme(s) => serde_json::Value::String(s),
            devkit::command::uri::UriComponentValue::Authority(Some(a)) => serde_json::Value::String(a),
            devkit::command::uri::UriComponentValue::Host(h) => serde_json::Value::String(h),
            devkit::command::uri::UriComponentValue::Port(p) => serde_json::json!(p),
            devkit::command::uri::UriComponentValue::Path(p) => serde_json::Value::String(p),
            devkit::command::uri::UriComponentValue::Query(q) => {
                let mut map = serde_json::Map::new();
                for (k, v) in q {
                    let val = match v {
                        devkit::command::uri::QueryPartVal::Single(s) => {
                            s.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null)
                        }
                        devkit::command::uri::QueryPartVal::Multi(m) => {
                            serde_json::Value::Array(m.into_iter().map(serde_json::Value::String).collect())
                        }
                    };
                    map.insert(k.to_string(), val);
                }
                serde_json::Value::Object(map)
            }
            _ => serde_json::Value::Null
        };
        UriComponentResult { name, value }
    }).filter(|UriComponentResult { value, .. }| !value.is_null()).collect();
    Ok(result)
}

#[tauri::command]
fn format_json(
    cache: tauri::State<'_, Mutex<JsonCache>>,
    json: String,
    query: Option<String>,
) -> Result<String, String> {
    let value = cache.lock().unwrap().get_or_parse(&json)?;
    let result = value.beautify(query.as_deref()).map_err(|e| e.to_string())?;
    Ok(result)
}

#[tauri::command]
fn query_json(
    cache: tauri::State<'_, Mutex<JsonCache>>,
    json: String,
    query: String,
) -> Result<Vec<String>, String> {
    let value = cache.lock().unwrap().get_or_parse(&json)?;
    let arr = value.query(&query, true).map_err(|e| e.to_string())?;
    Ok(arr)
}

#[tauri::command]
fn get_json_keys(
    cache: tauri::State<'_, Mutex<JsonCache>>,
    json: String,
    query: Option<String>,
) -> Result<Vec<String>, String> {
    let value = cache.lock().unwrap().get_or_parse(&json)?;
    let keys = value.keys(query.as_deref()).map_err(|e| e.to_string())?;
    Ok(keys)
}

#[tauri::command]
fn diff_json(
    cache: tauri::State<'_, Mutex<JsonCache>>,
    left: String,
    right: String,
    query: Option<String>,
    diff_tool: Option<String>,
) -> Result<(), String> {
    let left_val = cache.lock().unwrap().get_or_parse(&left)?;
    let right_val = cache.lock().unwrap().get_or_parse(&right)?;
    let tool = if let Some(t) = diff_tool {
        DiffTool::from_str(&t).map_err(|e| e.to_string())?
    } else {
        DiffTool::default()
    };
    let _ = left_val
        .diff(&right_val, query.as_deref(), Some(tool))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_available_diff_tools() -> Vec<String> {
    use strum::IntoEnumIterator;
    DiffTool::iter()
        .filter(|t: &DiffTool| t.is_available())
        .map(|t| t.to_string())
        .collect()
}

#[tauri::command]
fn parse_time(
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

#[tauri::command]
fn save_to_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_image_to_file(path: String, base64_content: String) -> Result<(), String> {
    let base64_data = if base64_content.contains(",") {
        base64_content.split(',').nth(1).unwrap_or(&base64_content)
    } else {
        &base64_content
    };
    let buffer = base64::engine::general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| e.to_string())?;
    std::fs::write(&path, buffer).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
struct QrCodeResponse {
    data: String,
    ec_level: String,
    version: String,
}

#[tauri::command]
fn generate_qrcode(
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

    let result = devkit::command::qrcode::generator::generate(
        &content,
        &ec_level,
        &version,
        output_type,
    )
        .map_err(|e| e.to_string())?;

    let data = match result.deref() {
        devkit::command::qrcode::generator::QrCodeImageVal::Svg(path) => {
            std::fs::read_to_string(path).map_err(|e| e.to_string())?
        }
        devkit::command::qrcode::generator::QrCodeImageVal::Image(path) => {
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

lazy_static! {
    static ref DEVKIT_PATH: AtomicPtr<PathBuf> = {
        match which::which("devkit"){
            Ok(path) => {
                let ptr = Box::into_raw( Box::new(path));
                AtomicPtr::new(ptr)
            },
            Err(_) => {
                AtomicPtr::new(ptr::null_mut())
            }
        }
    };
}
#[tauri::command]
fn base64_decode(input: String, url_safe: bool, no_pad: bool) -> Result<String, String> {
    let data = devkit::command::base64::decode(&input, url_safe, no_pad).map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&data).to_string())
}

#[tauri::command]
fn base64_encode(input: String, url_safe: bool, no_pad: bool) -> Result<String, String> {
    devkit::command::base64::encode(&input, url_safe, no_pad).map_err(|e| e.to_string())
}

#[tauri::command]
fn show_add_to_path_bth() -> Result<String, String> {
    let devkit_path_ptr = DEVKIT_PATH.load(Ordering::Relaxed);
    if devkit_path_ptr.is_null() {
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            return Ok("".to_string());
        }
        #[cfg(target_os = "windows")]{
            return Err("Unsupported operating system".to_string());
        }
    } else {
        let path = unsafe { &*devkit_path_ptr };
        Ok(path.display().to_string())
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn get_current_shell() -> Option<String> {
    std::env::var("SHELL").ok().and_then(|shell_path| {
        PathBuf::from(&shell_path)
            .file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
    })
}

#[tauri::command]
fn add_to_path() -> Result<String, String> {
    if !DEVKIT_PATH.load(Ordering::Relaxed).is_null() {
        return Ok("devkit already in PATH".to_string());
    }
    #[cfg(any(target_os = "macos", target_os = "linux"))]{
        use std::{fs, io::Write};
        let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let bin_dir = current_exe.parent().ok_or("Failed to get bin directory")?;
        let home_path = PathBuf::from(std::env::var("HOME").map_err(|e| e.to_string())?);
        let shell = get_current_shell().ok_or("Failed to get current shell")?.to_lowercase();
        match shell.as_str() {
            "fish" => {
                let fish_config_path = home_path.join(".config/fish/conf.d/devkit.fish");
                let mut file = fs::File::options().create(true).append(true).open(&fish_config_path).map_err(|e| e.to_string())?;
                file.write(format!(r#"
set -x PATH {} $PATH
alias dk "devkit"
                "#, bin_dir.display()).as_bytes()).map_err(|e| e.to_string())?;
            }
            "zsh" => {
                let zsh_config_path = home_path.join(".zshrc");
                let mut file = fs::File::options().create(true).append(true).open(&zsh_config_path).map_err(|e| e.to_string())?;
                file.write(format!(r#"
export PATH={}:$PATH
alias dk="devkit"
                "#, bin_dir.display()).as_bytes()).map_err(|e| e.to_string())?;
            }
            "bash" => {
                let bash_config_path = home_path.join(".bashrc");
                let mut file = fs::File::options().create(true).append(true).open(&bash_config_path).map_err(|e| e.to_string())?;
                file.write(format!(r#"
export PATH={}:$PATH
alias dk="devkit"
                "#, bin_dir.display()).as_bytes()).map_err(|e| e.to_string())?;
            }
            _ => {}
        }
        DEVKIT_PATH.store(Box::into_raw(Box::new(bin_dir.join("devkit"))), Ordering::Relaxed);
        return Ok("add devkit to PATH ok, you can use devkit or dk in command-line now!!!".to_string());
    }
    #[cfg(target_os = "windows")]{
        return Err("Unsupported operating system".to_string());
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(JsonCache::default()))
        .invoke_handler(tauri::generate_handler![
            greet,
            decode_uri,
            encode_uri,
            parse_uri,
            format_json,
            query_json,
            get_json_keys,
            diff_json,
            get_available_diff_tools,
            parse_time,
            save_to_file,
            save_image_to_file,
            show_add_to_path_bth,
            add_to_path,
            generate_qrcode,
            base64_decode,
            base64_encode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
