use clap::Parser;
use derive_more::Display;
use itertools::Itertools;
use os_xtask_utils::{Cargo, CommandExt};
use std::{env, fs};
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
    fn bin_path<P: AsRef<Path>>(&self, target_path: P) -> PathBuf {
        let target_path = target_path.as_ref().join(self.to_string()).join("release");
        match self {
            Self::Aarch64AppleDarwin | Self::X8664AppleDarwin | Self::X8664UnknownLinuxGnu => {
                target_path.join(BIN_NAME)
            }
            BuildTarget::X8664PcWindowsGnu => {
                target_path.join(format!("{}.exe", BIN_NAME))
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
        target_path, deployment_path,
    } = prepare_build_dirs(&project_root);

    let Cli {
        update_crates,
        features, no_default_features,
        target,
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
        }).release().conditional(true, |cargo| {
            cargo.target(target.to_string());
        }).invoke();
        let bin_path = target.bin_path(&target_path);
        let deployment_path = deployment_path.join(target.to_string());
        if deployment_path.exists() {
            fs::remove_dir_all(&deployment_path).expect(
                &format!("failed to remove existing deployment path, path: {}", deployment_path.display())
            )
        }
        fs::create_dir_all(&deployment_path).expect(
            &format!("failed to create deployment path, path: {}", deployment_path.display())
        );
        fs::copy(&bin_path, deployment_path.join("devkit")).expect(
            &format!("failed to copy binary, path: {}", bin_path.display())
        );
        fs::copy(project_root.join("README.md"), deployment_path.join("README.md")).expect("failed to copy README.md");
    }
}

struct BuildDirs {
    #[allow(dead_code)]
    project_root: PathBuf,
    target_path: PathBuf,
    deployment_path: PathBuf,
}
fn prepare_build_dirs<P: AsRef<Path>>(project_root: P) -> BuildDirs {
    let project_root = project_root.as_ref();
    let target_path = project_root.join("target");
    let deployment_path = target_path.join("deployment");
    if deployment_path.exists() {
        fs::remove_dir_all(&deployment_path).expect(
            &format!("failed to remove existing deployment path: {}", deployment_path.display())
        );
    }
    fs::create_dir_all(&deployment_path).expect(
        &format!("failed to create deployment path: {}", deployment_path.display())
    );
    BuildDirs {
        project_root: project_root.to_owned(),
        target_path,
        deployment_path,
    }
}
fn project_root() -> PathBuf {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
    let cargo_manifest_dir = PathBuf::from(cargo_manifest_dir);
    let project = cargo_manifest_dir.parent().expect(
        &format!("unexpected cargo_manifest_dir: {}", cargo_manifest_dir.display())
    );
    PathBuf::from(project)
}