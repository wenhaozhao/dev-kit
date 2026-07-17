use std::env;

fn main() {
    let target = env::var("TARGET").expect("TARGET is not set");
    let binary_name = if target.contains("windows") {
        format!("devkit-{target}.exe")
    } else {
        format!("devkit-{target}")
    };

    println!("cargo:rerun-if-changed=binaries/{binary_name}");
    tauri_build::build();
}
