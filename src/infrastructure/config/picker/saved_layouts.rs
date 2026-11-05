use serde_json::{Map, Value, json};
use skwd_config::schema::{self, DefaultValue};

use crate::contracts::settings::{
    ACTIVE_LAYOUT, Premade, SAVED_LAYOUTS, SavedLayouts, Stored, layout_paths,
};

use super::Config;

const NAME: &str = "name";
const VALUES: &str = "values";
const OLD_PRESETS: &str = "components.wallpaperSelector.presets";
const OLD_PARAMS: &str = "params";
const SELECTOR_PREFIX: &str = "components.wallpaperSelector.";
const NUMBER_TOLERANCE: f64 = 1e-9;

fn same_value(left: &Value, right: &Value) -> bool {
    match (left.as_f64(), right.as_f64()) {
        (Some(left), Some(right)) => (left - right).abs() < NUMBER_TOLERANCE,
        _ => left == right,
    }
}

fn stored(value: Stored) -> Value {
    match value {
        Stored::Number(number) => json!(number),
        Stored::Text(text) => json!(text),
    }
}

impl Config {
    fn layouts_stored(&self) -> Vec<Value> {
        self.array_values(SAVED_LAYOUTS)
    }

    fn layout_named<'v>(entries: &'v [Value], name: &str) -> Option<&'v Value> {
        entries.iter().find(|entry| entry.get(NAME).and_then(Value::as_str) == Some(name))
    }

    fn active_at(&self, entries: &[Value]) -> Option<usize> {
        let active = self.str_at(ACTIVE_LAYOUT, "");
        entries
            .iter()
            .position(|entry| entry.get(NAME).and_then(Value::as_str) == Some(active.as_str()))
    }

    fn path_default(&self, path: &str) -> Option<Value> {
        match schema::find(path)?.default {
            DefaultValue::Boolean(value) => Some(json!(value)),
            DefaultValue::Number(value) => Some(json!(value)),
            DefaultValue::ResponsiveNumber { regular, compact } => {
                Some(json!(if self.small { compact } else { regular }))
            }
            DefaultValue::Text(value) => Some(json!(value)),
            DefaultValue::EmptyArray => None,
        }
    }

    fn snapshot_values(&self) -> Map<String, Value> {
        layout_paths()
            .filter_map(|path| self.get(path).map(|value| (path.to_string(), value.clone())))
            .collect()
    }

    fn changed_paths(&self, saved: Option<&Map<String, Value>>) -> usize {
        let fallback = Map::new();
        let saved = saved.unwrap_or(&fallback);
        layout_paths()
            .filter(|path| {
                let was = saved.get(*path).cloned().or_else(|| self.path_default(path));
                let now = self.get(path).cloned().or_else(|| self.path_default(path));
                match (was, now) {
                    (Some(was), Some(now)) => !same_value(&was, &now),
                    (None, None) => false,
                    _ => true,
                }
            })
            .count()
    }

    pub fn saved_layouts_view(&self) -> SavedLayouts {
        let entries = self.array_slice(SAVED_LAYOUTS);
        let active = self.active_at(entries);
        let names = entries
            .iter()
            .filter_map(|entry| entry.get(NAME).and_then(Value::as_str).map(str::to_string))
            .collect();
        let changes = active
            .and_then(|at| entries.get(at))
            .map_or(0, |entry| self.changed_paths(entry.get(VALUES).and_then(Value::as_object)));
        SavedLayouts { names, active, changes }
    }

    fn write_layouts(&mut self, entries: Vec<Value>, active: &str) {
        self.write_key(SAVED_LAYOUTS, Value::Array(entries));
        self.write_key(ACTIVE_LAYOUT, json!(active));
        self.persist();
    }

    fn legacy_layouts(&self) -> Vec<(String, Map<String, Value>)> {
        let Some(modes) = self.get(OLD_PRESETS).and_then(Value::as_object) else {
            return Vec::new();
        };
        let mut found = Vec::new();
        for (mode, presets) in modes {
            for preset in presets.as_array().map_or(&[][..], Vec::as_slice) {
                let Some(name) = preset.get(NAME).and_then(Value::as_str) else {
                    continue;
                };
                let mut overrides = Map::new();
                overrides
                    .insert(skwd_config::keys::selector::DISPLAY_MODE.to_string(), json!(mode));
                for (key, value) in
                    preset.get(OLD_PARAMS).and_then(Value::as_object).into_iter().flatten()
                {
                    overrides.insert(format!("{SELECTOR_PREFIX}{key}"), value.clone());
                }
                found.push((name.to_string(), overrides));
            }
        }
        found
    }

    fn imported(entries: &[Value], name: &str, overrides: &Map<String, Value>) -> bool {
        Self::layout_named(entries, name)
            .and_then(|entry| entry.get(VALUES))
            .and_then(Value::as_object)
            .is_some_and(|values| {
                overrides
                    .iter()
                    .all(|(path, want)| values.get(path).is_some_and(|have| same_value(have, want)))
            })
    }

    fn drop_imported_presets(&mut self) {
        let Some(modes) = self.get(OLD_PRESETS).and_then(Value::as_object) else {
            return;
        };
        let kept: Map<String, Value> = modes
            .iter()
            .filter_map(|(mode, presets)| {
                let Some(items) = presets.as_array() else {
                    return Some((mode.clone(), presets.clone()));
                };
                let rest: Vec<Value> = items
                    .iter()
                    .filter(|preset| preset.get(NAME).and_then(Value::as_str).is_none())
                    .cloned()
                    .collect();
                (!rest.is_empty()).then(|| (mode.clone(), Value::Array(rest)))
            })
            .collect();
        if kept.is_empty() {
            self.remove_key(OLD_PRESETS);
        } else {
            self.write_key(OLD_PRESETS, Value::Object(kept));
        }
    }

    pub fn seed_saved_layouts(&mut self, name: &str) {
        let mut entries = self.layouts_stored();
        let fresh = entries.is_empty();
        let legacy = self.legacy_layouts();
        if !fresh && legacy.is_empty() {
            return;
        }
        if fresh {
            entries.push(json!({ NAME: name, VALUES: self.snapshot_values() }));
        }
        for (old, overrides) in legacy {
            if Self::imported(&entries, &old, &overrides) {
                continue;
            }
            let unique =
                unique_layout_name(&old, |wanted| Self::layout_named(&entries, wanted).is_some());
            let mut values = self.snapshot_values();
            values.extend(overrides);
            entries.push(json!({ NAME: unique, VALUES: values }));
        }
        self.drop_imported_presets();
        self.write_key(SAVED_LAYOUTS, Value::Array(entries));
        if fresh {
            self.write_key(ACTIVE_LAYOUT, json!(name));
        }
        self.persist();
    }

    fn apply_values(&mut self, values: &Map<String, Value>) -> Result<(), String> {
        for path in layout_paths().filter(|path| !values.contains_key(*path)) {
            self.try_remove_key(path)?;
        }
        let pairs: Vec<(&str, Value)> =
            values.iter().map(|(path, value)| (path.as_str(), value.clone())).collect();
        self.set_keys(&pairs)
    }

    fn transact(
        &mut self,
        change: impl FnOnce(&mut Self) -> Result<(), String>,
    ) -> Result<(), String> {
        let before = self.data.clone();
        let result = change(self).and_then(|()| self.try_persist());
        if let Err(error) = &result {
            log::warn!("config: saved layout rejected: {error}");
            self.replace_document(before);
        }
        result
    }

    fn enter(&mut self, entry: &Value) -> Result<(), String> {
        if let Some(values) = entry.get(VALUES).and_then(Value::as_object) {
            self.apply_values(values)?;
        }
        self.write_key(ACTIVE_LAYOUT, entry.get(NAME).cloned().unwrap_or_default());
        Ok(())
    }

    pub fn save_layout(&mut self) {
        let mut entries = self.layouts_stored();
        let Some(at) = self.active_at(&entries) else {
            return;
        };
        let values = Value::Object(self.snapshot_values());
        let Some(entry) = entries.get_mut(at).and_then(Value::as_object_mut) else {
            return;
        };
        entry.insert(VALUES.to_string(), values);
        let name = entry.get(NAME).and_then(Value::as_str).unwrap_or_default().to_string();
        self.write_layouts(entries, &name);
    }

    pub fn save_named_layout(&mut self, name: &str) {
        let mut entries = self.layouts_stored();
        let values = Value::Object(self.snapshot_values());
        let found = entries
            .iter_mut()
            .find(|entry| entry.get(NAME).and_then(Value::as_str) == Some(name))
            .and_then(Value::as_object_mut);
        match found {
            Some(entry) => {
                entry.insert(VALUES.to_string(), values);
            }
            None => entries.push(json!({ NAME: name, VALUES: values })),
        }
        self.write_layouts(entries, name);
    }

    pub fn revert_layout(&mut self) -> Result<(), String> {
        let entries = self.layouts_stored();
        let saved = self
            .active_at(&entries)
            .and_then(|at| entries.get(at))
            .and_then(|entry| entry.get(VALUES))
            .and_then(Value::as_object);
        match saved {
            Some(values) => self.transact(|conf| conf.apply_values(values)),
            None => Ok(()),
        }
    }

    pub fn switch_layout(&mut self, at: usize) -> Result<(), String> {
        let entries = self.layouts_stored();
        match entries.get(at) {
            Some(entry) => self.transact(|conf| conf.enter(entry)),
            None => Ok(()),
        }
    }

    pub fn add_layout(&mut self, name: &str) {
        let mut entries = self.layouts_stored();
        entries.push(json!({ NAME: name, VALUES: self.snapshot_values() }));
        self.write_layouts(entries, name);
    }

    pub fn rename_layout(&mut self, name: &str) -> bool {
        let name = name.trim();
        let mut entries = self.layouts_stored();
        let Some(at) = self.active_at(&entries) else {
            return false;
        };
        let current = entries.get(at).and_then(|entry| entry.get(NAME)).and_then(Value::as_str);
        if name.is_empty() || current == Some(name) || Self::layout_named(&entries, name).is_some()
        {
            return false;
        }
        let Some(entry) = entries.get_mut(at).and_then(Value::as_object_mut) else {
            return false;
        };
        entry.insert(NAME.to_string(), json!(name));
        self.write_layouts(entries, name);
        true
    }

    pub fn delete_layout(&mut self) -> Result<(), String> {
        let mut entries = self.layouts_stored();
        let Some(at) = self.active_at(&entries).filter(|_| entries.len() > 1) else {
            return Ok(());
        };
        entries.remove(at);
        let first = entries[0].clone();
        self.transact(|conf| {
            conf.write_key(SAVED_LAYOUTS, Value::Array(entries));
            conf.enter(&first)
        })
    }

    pub fn apply_premade(&mut self, premade: &Premade) -> Result<(), String> {
        let mut values: Map<String, Value> = premade
            .values
            .iter()
            .map(|(path, value)| ((*path).to_string(), stored(*value)))
            .collect();
        values.insert(
            skwd_config::keys::selector::DISPLAY_MODE.to_string(),
            json!(premade.mode.as_key()),
        );
        self.transact(|conf| conf.apply_values(&values))
    }

    pub fn unique_layout(&self, wanted: &str) -> String {
        let entries = self.layouts_stored();
        unique_layout_name(wanted, |name| Self::layout_named(&entries, name).is_some())
    }
}

fn unique_layout_name(wanted: &str, taken: impl Fn(&str) -> bool) -> String {
    if !taken(wanted) {
        return wanted.to_string();
    }
    let mut number = 2;
    loop {
        let name = format!("{wanted} {number}");
        if !taken(&name) {
            return name;
        }
        number += 1;
    }
}
