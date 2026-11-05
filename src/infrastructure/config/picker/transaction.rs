use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;

use super::Config;
use super::document::canonicalize_picker_config;

const CONFIG_MODE: u32 = 0o600;
const NO_DOCUMENT: &str = "layout document missing";
const UNRESOLVED: &str = "layout conflict unresolved";

struct Unresolved;

#[derive(Debug)]
pub(crate) enum LayoutSaveError {
    Conflict(Box<Value>),
    Failure(String),
}

impl LayoutSaveError {
    fn failure(error: impl std::fmt::Display) -> Self {
        Self::Failure(error.to_string())
    }
}

impl Config {
    pub(crate) fn finish_layout(
        &mut self,
        original: &Value,
        commit: bool,
    ) -> Result<(), LayoutSaveError> {
        let latest = finish(&self.config_path, original, self.document(), commit)?;
        self.restore_transient(latest);
        Ok(())
    }

    pub(crate) fn resolve_layout(
        &mut self,
        original: &Value,
        reviewed: &Value,
        keep_draft: bool,
    ) -> Result<(), LayoutSaveError> {
        let draft = merge(Some(original), Some(self.document()), Some(reviewed), Some(keep_draft))
            .map_err(|Unresolved| LayoutSaveError::failure(UNRESOLVED))?
            .ok_or_else(|| LayoutSaveError::failure(NO_DOCUMENT))?;
        let latest = finish(&self.config_path, reviewed, &draft, true)?;
        self.restore_transient(latest);
        Ok(())
    }
}

fn finish(
    path: &Path,
    original: &Value,
    draft: &Value,
    commit: bool,
) -> Result<Value, LayoutSaveError> {
    let mut latest = match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).map_err(LayoutSaveError::failure)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => original.clone(),
        Err(error) => return Err(LayoutSaveError::failure(error)),
    };
    canonicalize_picker_config(&mut latest);
    if !commit {
        return Ok(latest);
    }
    latest = merge(Some(original), Some(draft), Some(&latest), None)
        .map_err(|Unresolved| LayoutSaveError::Conflict(Box::new(latest.clone())))?
        .ok_or_else(|| LayoutSaveError::failure(NO_DOCUMENT))?;
    canonicalize_picker_config(&mut latest);
    let text = serde_json::to_string_pretty(&latest).map_err(LayoutSaveError::failure)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(LayoutSaveError::failure)?;
    }
    skwd_config::atomic_write_mode(path, format!("{text}\n").as_bytes(), Some(CONFIG_MODE))
        .map_err(LayoutSaveError::failure)?;
    Ok(latest)
}

fn merge(
    original: Option<&Value>,
    draft: Option<&Value>,
    latest: Option<&Value>,
    choice: Option<bool>,
) -> Result<Option<Value>, Unresolved> {
    if original == draft || latest == draft {
        return Ok(latest.cloned());
    }
    if latest == original {
        return Ok(draft.cloned());
    }
    if let (Some(Value::Object(draft)), Some(Value::Object(latest))) = (draft, latest)
        && original.is_none_or(Value::is_object)
    {
        let empty = serde_json::Map::new();
        let original = original.and_then(Value::as_object).unwrap_or(&empty);
        let keys: BTreeSet<_> = original.keys().chain(draft.keys()).chain(latest.keys()).collect();
        let mut merged = serde_json::Map::new();
        for key in keys {
            if let Some(value) = merge(original.get(key), draft.get(key), latest.get(key), choice)?
            {
                merged.insert(key.clone(), value);
            }
        }
        return Ok(Some(Value::Object(merged)));
    }
    choice.map(|keep| if keep { draft.cloned() } else { latest.cloned() }).ok_or(Unresolved)
}
