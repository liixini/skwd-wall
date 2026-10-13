#[cfg(all(test, unix))]
mod tests;

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct PickerLabels {
    pub title: String,
    pub packs: String,
    pub all_files: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImportedSemanticPack {
    pub format: u32,
    pub id: String,
    pub version: String,
    pub dimensions: usize,
    pub manifest: PathBuf,
    pub bytes: u64,
}

pub fn choose_and_import(
    bin: &Path,
    runtime: &Path,
    labels: &PickerLabels,
) -> Result<Option<ImportedSemanticPack>, String> {
    let Some(source) = choose_source(labels)? else {
        return Ok(None);
    };
    import_source(bin, runtime, &source, &models_directory()?).map(Some)
}

pub fn import_source(
    bin: &Path,
    runtime: &Path,
    source: &Path,
    models_dir: &Path,
) -> Result<ImportedSemanticPack, String> {
    let output = Command::new(bin)
        .arg("--install-pack")
        .arg(source)
        .arg("--models-dir")
        .arg(models_dir)
        .arg("--runtime")
        .arg(runtime)
        .output()
        .map_err(|error| format!("start Lens model-pack installer: {error}"))?;
    let output = checked(output, "install model pack")?;
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("read Lens install report: {error}"))
}

fn choose_source(labels: &PickerLabels) -> Result<Option<PathBuf>, String> {
    const DOWNLOADS: &str = "Downloads";
    const PACKS: &str =
        "semantic-pack.json *.skwdmodel *.zip *.tar *.tar.gz *.tgz *.tar.xz *.txz *.tar.zst *.tzst";
    let filters = [format!("{} | {PACKS}", labels.packs), format!("{} | *", labels.all_files)];
    let downloads = home_directory().map(|home| home.join(DOWNLOADS)).filter(|path| path.is_dir());
    super::file_picker::choose(&super::file_picker::FilePicker {
        title: &labels.title,
        directory: false,
        filters: &filters,
        start: downloads.as_deref(),
    })
}

fn models_directory() -> Result<PathBuf, String> {
    let data = env::var_os("XDG_DATA_HOME")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .or_else(|| home_directory().map(|home| home.join(".local/share")))
        .ok_or_else(|| String::from("cannot locate the user data directory"))?;
    Ok(data.join("skwd-lens/models/semantic/packs"))
}

fn home_directory() -> Option<PathBuf> {
    env::var_os("HOME").filter(|path| !path.is_empty()).map(PathBuf::from)
}

fn checked(output: Output, operation: &str) -> Result<Output, String> {
    if output.status.success() {
        return Ok(output);
    }
    Err(output_error(&output, operation))
}

fn output_error(output: &Output, operation: &str) -> String {
    let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if detail.is_empty() { format!("cannot {operation}") } else { detail }
}

pub fn delete_model(manifest: &Path, managed_id: Option<&str>) -> Result<(), String> {
    let bin = super::semantic::SemanticTooling::helper()?;
    if let Some(id) = managed_id {
        return remove_pack(&bin, &models_directory()?, manifest, id);
    }
    let output = Command::new(bin)
        .arg("--remove-manifest")
        .arg(manifest)
        .output()
        .map_err(|error| format!("start Lens model removal: {error}"))?;
    removal_report(&checked(output, "delete model")?)
}

pub fn remove_pack(bin: &Path, models_dir: &Path, manifest: &Path, id: &str) -> Result<(), String> {
    let models_dir = models_dir.canonicalize().map_err(|error| error.to_string())?;
    let root = manifest.parent().ok_or_else(|| String::from("invalid model manifest path"))?;
    let resolved = root.canonicalize().map_err(|error| error.to_string())?;
    if resolved != root
        || resolved.parent() != Some(models_dir.as_path())
        || manifest.file_name() != Some(std::ffi::OsStr::new("semantic-pack.json"))
    {
        return Err(String::from(
            "only models imported into the user model directory can be deleted",
        ));
    }
    let component = resolved.file_name().ok_or_else(|| String::from("invalid model directory"))?;
    let output = Command::new(bin)
        .arg("--remove-pack")
        .arg(id)
        .arg("--models-dir")
        .arg(models_dir)
        .arg("--pack-component")
        .arg(component)
        .arg("--allow-active")
        .output()
        .map_err(|error| format!("start Lens model removal: {error}"))?;
    let output = checked(output, "delete model pack")?;
    removal_report(&output)
}

fn removal_report(output: &Output) -> Result<(), String> {
    let report: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("read Lens removal report: {error}"))?;
    if report.get("removed").and_then(serde_json::Value::as_bool) != Some(true) {
        return Err(String::from("Lens did not confirm model removal"));
    }
    Ok(())
}
