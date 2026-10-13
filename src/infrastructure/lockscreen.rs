use std::path::PathBuf;
use std::process::Command;

pub use wall_proto::SddmChoices;

const HELM_NAME: &str = "skwd-helm";
const PKEXEC_PATH: &str = "/usr/bin/pkexec";
const ENABLE: &str = "enable";
const DISABLE: &str = "disable";
const SDDM_FAILURE: &str = "SDDM setup failed";

#[derive(Clone, Debug)]
pub struct SddmSetup {
    pub path: String,
    pub theme: Option<String>,
}

fn sibling_helm() -> Result<PathBuf, String> {
    let path = std::env::current_exe()
        .map_err(|error| error.to_string())?
        .parent()
        .ok_or("Cannot locate the installed Skwd suite")?
        .join(HELM_NAME);
    if !super::executable::is_executable(&path) {
        return Err(format!("Cannot run {}", path.display()));
    }
    Ok(path)
}

fn output(command: &mut Command) -> Result<String, String> {
    let result = command.output().map_err(|error| error.to_string())?;
    if !result.status.success() {
        let reason = String::from_utf8_lossy(&result.stderr).trim().to_owned();
        return Err(if reason.is_empty() { SDDM_FAILURE.into() } else { reason });
    }
    String::from_utf8(result.stdout).map_err(|error| error.to_string())
}

pub fn sddm_choices() -> Result<SddmChoices, String> {
    let mut command = Command::new(sibling_helm()?);
    command.args([wall_proto::helm::LOCKSCREEN_SYSTEM, wall_proto::helm::LOCKSCREEN_CHOICES]);
    serde_json::from_str(&output(&mut command)?).map_err(|error| error.to_string())
}

pub fn set_sddm(enabled: bool, theme: Option<&str>) -> Result<String, String> {
    if !super::executable::is_executable(std::path::Path::new(PKEXEC_PATH)) {
        return Err("pkexec is not installed".into());
    }
    let mut command = Command::new(PKEXEC_PATH);
    command.arg(sibling_helm()?).arg(wall_proto::helm::LOCKSCREEN_SYSTEM);
    command.arg(if enabled { ENABLE } else { DISABLE });
    if let Some(theme) = theme {
        command.arg(theme);
    }
    let path = output(&mut command)?.trim().to_owned();
    if !std::path::Path::new(&path).is_absolute() {
        return Err("SDDM helper returned an invalid image path".into());
    }
    Ok(path)
}

pub fn prepare_sddm(enabled: bool, theme: Option<&str>) -> Result<SddmSetup, String> {
    let previous = if enabled { None } else { Some(sddm_choices()?.current) };
    let path = set_sddm(enabled, theme)?;
    Ok(SddmSetup { path, theme: previous.filter(|name| !name.is_empty()) })
}
