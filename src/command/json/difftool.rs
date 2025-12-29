use crate::command::json::{DiffTool, JetbrainsIDE};
use anyhow::anyhow;
use itertools::Itertools;
use std::path::Path;
use std::process::{exit, Command};
use std::str::FromStr;
use strum::IntoEnumIterator;
use which::which;

impl FromStr for DiffTool {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "zed" => Ok(DiffTool::Zed),
            "vscode" => Ok(DiffTool::VSCode),
            val => JetbrainsIDE::iter().find(|it|
                it.to_string().eq(val)
            ).map(DiffTool::JetbrainsIDE).ok_or_else(||
                anyhow!("invalid diff tool: {}", val)
            )
        }
    }
}

impl Default for DiffTool {
    fn default() -> Self {
        let difftool = DiffTool::iter().filter(|it|
            it.is_available()
        ).next().or_else(|| JetbrainsIDE::iter().find_or_first(|it|
            which(it.to_string()).is_ok()
        ).map(|it|
            Self::JetbrainsIDE(it)
        ));
        if let Some(difftool) = difftool {
            difftool
        } else {
            eprintln!("no diff tool available, you can install one of the following:");
            for it in Self::iter() {
                println!("{}", it.how_to_install())
            }
            exit(0)
        }
    }
}

impl DiffTool {
    pub fn diff<L: AsRef<Path>, R: AsRef<Path>>(&self, left: L, right: R) -> crate::Result<()> {
        let left = left.as_ref();
        let right = right.as_ref();
        match self {
            DiffTool::JetbrainsIDE(ide) => {
                let status = Command::new(ide.to_string())
                    .arg("diff")
                    .arg(left.display().to_string())
                    .arg(right.display().to_string())
                    .status()
                    .map_err(|err| anyhow!(
r#"
failed to execute idea diff {} {}
error: {}"#,
                        left.display(), right.display(), err
                    ))?;
                if status.success() {
                    Ok(())
                } else {
                    Err(anyhow!("idea diff command failed with status: {}", status))
                }
            }
            DiffTool::Zed => {
                let status = Command::new("zed")
                    .arg("--diff")
                    .arg(left.display().to_string())
                    .arg(right.display().to_string())
                    .status()
                    .map_err(|err| anyhow!(
r#"
failed to execute zed --diff {} {}
error: {}"#,
                        left.display(), right.display(), err
                    ))?;
                if status.success() {
                    Ok(())
                } else {
                    Err(anyhow!("zed --diff command failed with status: {}", status))
                }
            }
            &DiffTool::VSCode => {
                let status = Command::new("code")
                    .arg("--diff")
                    .arg(left.display().to_string())
                    .arg(right.display().to_string())
                    .status()
                    .map_err(|err| anyhow!(
r#"
failed to execute code --diff {} {}
error: {}"#,
                        left.display(), right.display(), err
                    ))?;
                if status.success() {
                    Ok(())
                } else {
                    Err(anyhow!("code --diff command failed with status: {}", status))
                }
            }
        }
    }

    pub fn is_available(&self) -> bool {
        which(self.to_string()).is_ok()
    }

    pub fn how_to_install(&self) -> String {
        match self {
            DiffTool::JetbrainsIDE(ide) => format!("https://www.jetbrains.com/help/{}/working-with-the-ide-features-from-command-line.html", ide.to_string()),
            DiffTool::Zed => "https://zed.dev/docs/command-line-interface".to_string(),
            DiffTool::VSCode => "https://code.visualstudio.com/docs/configure/command-line".to_string(),
        }
    }
}