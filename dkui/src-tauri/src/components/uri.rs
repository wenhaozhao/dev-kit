use crate::devkit;
use std::str::FromStr;
#[tauri::command]
pub fn decode_uri(uri: String) -> Result<String, String> {
    let uri = devkit::command::uri::Uri::from_str(&uri).map_err(|e| e.to_string())?;
    uri.decode().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn encode_uri(uri: String) -> Result<String, String> {
    let uri = devkit::command::uri::Uri::from_str(&uri).map_err(|e| e.to_string())?;
    uri.encode().map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct UriComponentResult {
    name: String,
    value: serde_json::Value,
}

#[tauri::command]
pub fn parse_uri(uri: String, filter: Option<Vec<String>>) -> Result<Vec<UriComponentResult>, String> {
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
