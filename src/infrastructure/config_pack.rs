use std::path::PathBuf;
use std::process::{Command, Output};

const HELM_NAME: &str = "skwd-helm";
const PACK_EXTENSION: &str = "skwdpack";

pub struct Labels<'a> {
    pub export_title: &'a str,
    pub import_title: &'a str,
}

fn helm() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join(HELM_NAME)))
        .filter(|path| super::executable::is_executable(path))
        .unwrap_or_else(|| PathBuf::from(HELM_NAME))
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").filter(|path| !path.is_empty()).map(PathBuf::from)
}

fn checked(output: &Output, operation: &str) -> Result<(), String> {
    if output.status.success() {
        return Ok(());
    }
    let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(if detail.is_empty() { format!("cannot {operation}") } else { detail })
}

fn pack_path(path: PathBuf) -> PathBuf {
    if path.extension().and_then(|extension| extension.to_str()) == Some(PACK_EXTENSION) {
        path
    } else {
        let mut name = path.into_os_string();
        name.push(format!(".{PACK_EXTENSION}"));
        PathBuf::from(name)
    }
}

pub fn export(labels: &Labels<'_>) -> Result<Option<PathBuf>, String> {
    let suggested = home().unwrap_or_else(|| PathBuf::from(".")).join("skwd-settings.skwdpack");
    let Some(destination) = super::file_picker::save(labels.export_title, &suggested)? else {
        return Ok(None);
    };
    let destination = pack_path(destination);
    let output = Command::new(helm())
        .arg("export")
        .arg("--out")
        .arg(&destination)
        .arg("--name")
        .arg("Skwd settings")
        .output()
        .map_err(|error| format!("start configuration export: {error}"))?;
    checked(&output, "export configuration")?;
    Ok(Some(destination))
}

pub fn import(labels: &Labels<'_>) -> Result<Option<PathBuf>, String> {
    let selected = super::file_picker::choose(&super::file_picker::FilePicker {
        title: labels.import_title,
        directory: true,
        filters: &[],
        start: home().as_deref(),
    })?;
    let Some(source) = selected else { return Ok(None) };
    let output = Command::new(helm())
        .arg("import")
        .arg(&source)
        .arg("--merge")
        .output()
        .map_err(|error| format!("start configuration import: {error}"))?;
    checked(&output, "import configuration")?;
    Ok(Some(source))
}

#[cfg(test)]
#[path = "config_pack_tests.rs"]
mod tests;
