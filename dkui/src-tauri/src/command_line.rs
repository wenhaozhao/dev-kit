use std::path::PathBuf;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use lazy_static::lazy_static;

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
pub fn show_add_to_path_bth() -> Result<String, String> {
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

#[tauri::command]
pub fn add_to_path() -> Result<String, String> {
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

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn get_current_shell() -> Option<String> {
    std::env::var("SHELL").ok().and_then(|shell_path| {
        PathBuf::from(&shell_path)
            .file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
    })
}