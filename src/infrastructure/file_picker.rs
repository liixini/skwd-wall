use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ZENITY: &str = "zenity";
const FILE_SELECTION: &str = "--file-selection";
const DIRECTORY: &str = "--directory";
const TITLE: &str = "--title";
const FILTER: &str = "--file-filter";
const FILENAME: &str = "--filename";
const CANCELLED: i32 = 1;

pub struct FilePicker<'a> {
    pub title: &'a str,
    pub directory: bool,
    pub filters: &'a [String],
    pub start: Option<&'a Path>,
}

pub fn choose(spec: &FilePicker<'_>) -> Result<Option<PathBuf>, String> {
    let mut command = Command::new(ZENITY);
    command.arg(FILE_SELECTION).arg(format!("{TITLE}={}", spec.title));
    if spec.directory {
        command.arg(DIRECTORY);
    }
    for filter in spec.filters {
        command.arg(format!("{FILTER}={filter}"));
    }
    if let Some(start) = spec.start {
        command.arg(format!("{FILENAME}={}/", start.display()));
    }
    selected(command.output().map_err(|err| format!("start file picker: {err}"))?)
}

fn selected(output: Output) -> Result<Option<PathBuf>, String> {
    if output.status.code() == Some(CANCELLED) {
        return Ok(None);
    }
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(if error.is_empty() {
            format!("file picker exited: {}", output.status)
        } else {
            error
        });
    }
    let path = String::from_utf8(output.stdout).map_err(|err| err.to_string())?;
    let path = path.strip_suffix('\n').unwrap_or(&path);
    Ok((!path.is_empty()).then(|| PathBuf::from(path)))
}

#[cfg(all(test, unix))]
mod tests;
