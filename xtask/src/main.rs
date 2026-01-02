use clap::Parser;
use derive_more::Display;
use itertools::Itertools;
use os_xtask_utils::{Cargo, CommandExt};
use std::env;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::str::FromStr;
use strum::{EnumIter, IntoEnumIterator};

#[derive(clap::Parser)]
struct Cli {
    #[clap(long, default_value = "false")]
    update_crates: bool,
    #[clap(long)]
    features: Option<Vec<String>>,
    #[clap(long, default_value = "false")]
    no_default_features: bool,
    #[clap(long)]
    target: Option<String>,
    #[clap(short, long, default_value = "false")]
    release: bool,
    #[clap(long, default_value = "false")]
    verbose: bool,
}

#[derive(Debug, Clone, Copy, Display, EnumIter)]
enum BuildTarget {
    #[display("aarch64-apple-darwin")]
    Aarch64AppleDarwin,
    #[display("x86_64-apple-darwin")]
    X8664AppleDarwin,
    #[display("x86_64-pc-windows-gnu")]
    X8664PcWindowsGnu,
    #[display("x86_64-unknown-linux-gnu")]
    X8664UnknownLinuxGnu,
}

impl FromStr for BuildTarget {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BuildTarget::iter().find_or_first(|&it|
            it.to_string().eq_ignore_ascii_case(s)
        ).ok_or_else(|| format!("target {} is not supported", s))
    }
}

const BIN_NAME: &str = "devkit";

impl BuildTarget {
    fn bin_path<P: AsRef<Path>>(&self, target_path: P, release: bool) -> PathBuf {
        let target_path = target_path.as_ref();
        let release_dir = if release { "release" } else { "debug" };
        match self {
            Self::Aarch64AppleDarwin | Self::X8664AppleDarwin | Self::X8664UnknownLinuxGnu => {
                target_path.join(self.to_string()).join(release_dir).join(BIN_NAME)
            }
            BuildTarget::X8664PcWindowsGnu => {
                target_path.join(self.to_string()).join(release_dir).join(format!("{}.exe", BIN_NAME))
            }
        }
    }

    fn default_targets() -> &'static [Self] {
        &[Self::Aarch64AppleDarwin, Self::X8664AppleDarwin]
    }
}

fn main() {
    let project_root = project_root();
    let BuildDirs {
        project_root: _,
        target_path, releases_path, debugs_path,
    } = prepare_build_dirs(&project_root);

    let Cli {
        update_crates,
        features, no_default_features,
        target, release,
        ..
    } = Cli::parse();
    if update_crates {
        Cargo::update().invoke();
    }
    let targets = if let Some(target) = &target {
        match BuildTarget::from_str(target) {
            Ok(it) => vec![it],
            Err(err) => {
                eprintln!("Error: {}", err);
                exit(0);
            }
        }
    } else {
        BuildTarget::default_targets().to_vec()
    };
    for target in targets {
        println!("cargo build --target {} --release", target.to_string());
        Cargo::build().conditional(features.is_some(), |cargo| {
            cargo.features(
                !no_default_features,
                features.as_ref().unwrap(),
            );
        }).conditional(release, |cargo| {
            cargo.release();
        }).conditional(true, |cargo| {
            cargo.target(target.to_string());
        }).invoke();
        let bin_path = target.bin_path(&target_path, release);
        let bin_name = bin_path.file_name().expect(&format!("unexpected bin name with {}", bin_path.display()));
        let bin_name = bin_name.to_str().expect("unexpected bin name").replace(BIN_NAME, &format!("{}_{}", BIN_NAME, target.to_string()));
        if release {
            std::fs::copy(&bin_path, releases_path.join(bin_name)).expect("failed to copy binary");
        } else {
            std::fs::copy(&bin_path, debugs_path.join(bin_name)).expect("failed to copy binary");
        }
    }
    std::fs::copy(project_root.join("README.md"), releases_path.join("README.md")).expect("failed to copy README.md");
    std::fs::copy(project_root.join("README.zh-cn.md"), releases_path.join("README.zh-cn.md")).expect("failed to copy README.zh-cn.md");
}

struct BuildDirs {
    #[allow(dead_code)]
    project_root: PathBuf,
    target_path: PathBuf,
    releases_path: PathBuf,
    debugs_path: PathBuf,
}
fn prepare_build_dirs<P: AsRef<Path>>(project_root: P) -> BuildDirs {
    let project_root = project_root.as_ref();
    let target_path = project_root.join("target");
    let releases = target_path.join("releases");
    if releases.exists() {
        std::fs::remove_dir_all(&releases).expect("failed to remove releases");
    }
    std::fs::create_dir_all(&releases).expect("failed to create releases");
    let debugs = target_path.join("debugs");
    if debugs.exists() {
        std::fs::remove_dir_all(&debugs).expect("failed to remove debugs");
    }
    std::fs::create_dir_all(&debugs).expect("failed to create debugs");
    BuildDirs {
        project_root: project_root.to_owned(),
        target_path,
        releases_path: releases,
        debugs_path: debugs,
    }
}
fn project_root() -> PathBuf {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
    let cargo_manifest_dir = PathBuf::from(cargo_manifest_dir);
    let project = cargo_manifest_dir.parent().expect(&format!("unexpected cargo_manifest_dir: {}", cargo_manifest_dir.display()));
    PathBuf::from(project)
}