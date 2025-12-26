use crate::command::json::DiffTool;
use anyhow::anyhow;
use std::path::Path;
use std::process::Command;
use strum::IntoEnumIterator;
use which::which;

impl Default for DiffTool {
    fn default() -> Self {
        DiffTool::iter().filter(|it|
            it.is_available()
        ).next().unwrap_or(
            DiffTool::Idea
        )
    }
}

impl DiffTool {}

impl DiffTool {
    pub fn diff<L: AsRef<Path>, R: AsRef<Path>>(&self, left: L, right: R) -> crate::Result<()> {
        let left = left.as_ref();
        let right = right.as_ref();
        match self {
            DiffTool::Idea => {
                let status = Command::new("idea")
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

    pub fn how_to_install(&self) -> &'static str {
        match self {
            DiffTool::Idea => "https://www.jetbrains.com/help/idea/working-with-the-ide-features-from-command-line.html",
            DiffTool::Zed => "https://zed.dev/docs/command-line-interface",
            DiffTool::VSCode => "https://code.visualstudio.com/docs/configure/command-line",
        }
    }
}