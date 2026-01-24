use derive_more::Deref;
use dev_kit as devkit;
use std::fs;
use std::sync::{Arc};
use tauri::async_runtime::RwLock;

mod components;
use components::*;
mod command_line;
use command_line::*;

mod app_state;
use app_state::AppState;

#[tauri::command]
fn save_to_file(path: String, content: String) -> Result<(), String> {
    fs::write(&path, content).map_err(|e| e.to_string())
}

#[derive(Clone, Deref)]
struct SharedAppState(Arc<RwLock<AppState>>);
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    match AppState::init() {
        Ok(app_state) => {
            let logdir = app_state.proj_dirs.data_local_dir().join("logs");
            let _ = fs::create_dir_all(&logdir);
            let logfile_path = logdir.join("stdout.log");
            let logfile = Box::new(fs::File::options().create(true).append(true).open(&logfile_path)
                .expect(&format!("Failed to open stdout log file: {}", logfile_path.display())));
            env_logger::Builder::from_default_env()
                .target(env_logger::Target::Pipe(logfile))
                .init();
            match tauri_init(app_state) {
                Ok(_) => {}
                Err(err) => {
                    let err_msg = format!("Failed to initialize tauri: {}", err);
                    log::error!("{err_msg}");
                    panic!("{err_msg}");
                }
            }
        }
        Err(err) => {
            panic!("init app state failed: {}", err);
        }
    }
}

fn tauri_init(app_state: AppState) -> Result<(), String> {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(SharedAppState(Arc::new(RwLock::new(app_state))))
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
        .run(tauri::generate_context!());
    match result {
        Ok(_) => {
            log::info!("Tauri application started.");
            Ok(())
        }
        Err(err) => {
            Err(format!("Error while running tauri application: {}", err))
        }
    }
}
