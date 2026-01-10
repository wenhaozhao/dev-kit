use std::path::PathBuf;
use std::{env, fs};

fn main() -> Result<(), String> {
    let Workspace {
        target_triple, devkit_bin, ..
    } = get_workspace()?;
    if target_triple.to_lowercase().ends_with("darwin") {
        println!("cargo:rerun-if-changed={}", devkit_bin.display());
        let dst_bin_path = {
            let path = PathBuf::from(format!("binaries/devkit-{target_triple}"));
            let parent = path.parent().expect(&format!("failed to get parent directory, path: {}", path.display()));
            if !parent.exists() {
                fs::create_dir_all(&parent).expect(&format!("failed to create directory: {}", parent.display()));
            }
            path
        };
        let _ = fs::copy(&devkit_bin, &dst_bin_path).expect(&format!("failed to copy devkit binary: {}", devkit_bin.display()));
    }
    tauri_build::build();
    Ok(())
}

fn get_workspace() -> Result<Workspace, String> {
    let target_triple = env::var("TARGET").expect("TARGET is not set");
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
    let cargo_manifest_dir = PathBuf::from(cargo_manifest_dir);
    let project_path = cargo_manifest_dir.parent().and_then(|it| it.parent()).expect(
        &format!("unexpected cargo_manifest_dir: {}", cargo_manifest_dir.display())
    ).to_owned();
    let _dkui_package_path = cargo_manifest_dir.parent().expect(
        &format!("unexpected cargo_manifest_dir: {}", cargo_manifest_dir.display())
    );
    let target_path = project_path.join("target");
    let devkit_bin = target_path.join("deployment").join(&target_triple).join("devkit");
    Ok(Workspace {
        target_triple,
        project_path,
        target_path,
        devkit_bin,
    })
}

struct Workspace {
    target_triple: String,
    #[allow(unused)]
    project_path: PathBuf,
    #[allow(unused)]
    target_path: PathBuf,
    devkit_bin: PathBuf,
}
