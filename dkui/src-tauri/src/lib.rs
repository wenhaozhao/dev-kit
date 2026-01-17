use dev_kit as devkit;
use std::sync::Mutex;

mod components;
use components::*;
mod command_line;
use command_line::*;

#[tauri::command]
fn save_to_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(JsonCache::default()))
        .invoke_handler(tauri::generate_handler![
            show_add_to_path_bth,
            add_to_path,
            save_to_file,
            query_json,
            search_json_paths,
            diff_json,
            get_available_diff_tools,
            decode_uri,
            encode_uri,
            parse_uri,
            parse_time,
            save_image_to_file,
            generate_qrcode,
            base64_decode,
            base64_encode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
