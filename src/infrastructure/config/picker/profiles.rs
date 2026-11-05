use serde_json::Value;
use skwd_config::picker_layout::{self, Component, LayoutProfile, Layouts, Position, Size};

use super::Config;
use crate::contracts::settings::OverviewMemo;

const OUTPUTS: &str = "outputs";
const VERSION: &str = "version";
const VALUES: &str = "values";
const VIEWPORT: &str = "viewport";
const POSITIONS: &str = "positions";

const MISSING_OUTPUT: &str = "layout output missing";
const INVALID_RELATIONSHIP: &str = "invalid layout relationship";
const INVALID_VIEWPORT: &str = "invalid layout viewport";
const MISSING_DOCUMENT: &str = "configuration object missing";
const MISSING_OUTPUTS: &str = "layout outputs missing";
const MISSING_PROFILE: &str = "layout profile missing";
const MISSING_VALUES: &str = "layout values missing";

pub(super) struct LayoutTarget {
    pub output: String,
    pub legacy: Option<String>,
    pub viewport: Size,
}

impl Config {
    pub(crate) fn project_layout(
        &self,
        output: &str,
        profile: &LayoutProfile,
    ) -> Result<Self, String> {
        let mut config = Self {
            data: self.data.clone(),
            layout: None,
            effective: std::sync::OnceLock::new(),
            layout_cache: std::sync::OnceLock::new(),
            palette_previews: std::sync::OnceLock::new(),
            overview: OverviewMemo::default(),
            config_path: self.config_path.clone(),
            small: self.small,
            transient: true,
            writes: super::document::Writes::default(),
            on_battery: self.on_battery,
        };
        config.set_screen_width(profile.viewport.width);
        config.set_layout_profile(output, profile)?;
        config.select_layout_output(Some(output.to_owned()), profile.viewport);
        Ok(config)
    }

    pub(crate) fn document(&self) -> &Value {
        &self.data
    }

    pub(crate) fn select_layout_output(&mut self, output: Option<String>, viewport: Size) {
        self.select_layout_monitor(output, None, viewport);
    }

    pub(crate) fn select_layout_monitor(
        &mut self,
        output: Option<String>,
        connector: Option<String>,
        viewport: Size,
    ) {
        self.layout = output.filter(|output| !output.trim().is_empty()).map(|output| {
            let legacy = connector.filter(|connector| *connector != output);
            LayoutTarget { output, legacy, viewport }
        });
        self.effective.take();
        self.palette_previews.take();
        self.overview.take();
    }

    fn resolved(&self) -> Option<(String, &LayoutProfile)> {
        let target = self.layout.as_ref()?;
        let layouts = self.layouts().ok()?;
        if let Some(profile) = layouts.outputs.get(&target.output) {
            return Some((target.output.clone(), profile));
        }
        if let Some((legacy, profile)) =
            target.legacy.as_ref().and_then(|legacy| layouts.outputs.get_key_value(legacy))
        {
            return Some((legacy.clone(), profile));
        }
        similar(layouts, target.viewport).map(|(key, profile)| (key.clone(), profile))
    }

    pub(crate) fn remove_layout_profile(&mut self, output: &str) -> bool {
        let removed = self
            .data
            .get_mut(picker_layout::ROOT)
            .and_then(|layouts| layouts.get_mut(OUTPUTS))
            .and_then(Value::as_object_mut)
            .is_some_and(|outputs| outputs.remove(output).is_some());
        if removed {
            self.layout_cache.take();
            self.effective.take();
            self.palette_previews.take();
            self.overview.take();
        }
        removed
    }

    pub(crate) fn layout_output(&self) -> Option<&str> {
        self.layout.as_ref().map(|layout| layout.output.as_str())
    }

    #[cfg(test)]
    pub(crate) fn layout_outputs(&self) -> Vec<String> {
        self.layouts().map(|layouts| layouts.outputs.keys().cloned().collect()).unwrap_or_default()
    }

    pub(crate) fn layout_profile(
        &self,
        output: &str,
        viewport: Size,
    ) -> Result<LayoutProfile, String> {
        let layouts = self.layouts()?;
        let mut document = layout_base(&self.data);
        let current = self.layout_output() == Some(output);
        let stored = layouts
            .outputs
            .get(output)
            .or_else(|| current.then(|| self.resolved().map(|(_, profile)| profile)).flatten());
        if let Some(profile) = stored {
            profile.apply(&mut document);
        }
        let mut profile = LayoutProfile::capture(&document, viewport);
        if let Some(stored) = stored {
            profile.positions.clone_from(&stored.positions);
        }
        profile.validate()?;
        Ok(profile)
    }

    pub(crate) fn set_layout_profile(
        &mut self,
        output: &str,
        profile: &LayoutProfile,
    ) -> Result<(), String> {
        if output.trim().is_empty() {
            return Err(MISSING_OUTPUT.to_owned());
        }
        profile.validate()?;
        let mut layouts = self.layout_cache.take().unwrap_or_else(|| Layouts::read(&self.data))?;
        layouts.outputs.insert(output.to_owned(), profile.clone());
        self.write_profile(output, profile)?;
        let _ = self.layout_cache.set(Ok(layouts));
        Ok(())
    }

    pub(crate) fn position(&self, component: Component) -> Option<Position> {
        self.resolved()?.1.positions.get(&component).copied()
    }

    pub(crate) fn set_position(
        &mut self,
        component: Component,
        position: Option<Position>,
    ) -> Result<(), String> {
        self.edit_layout(&ProfileEdit::Position(component, position))
    }

    pub(super) fn effective_document(&self) -> &Value {
        let Some(target) = &self.layout else { return &self.data };
        self.effective.get_or_init(|| {
            let mut effective = layout_base(&self.data);
            if let Some((_, profile)) = self.resolved() {
                let mut profile = profile.clone();
                profile.viewport = target.viewport;
                profile.apply(&mut effective);
            }
            effective
        })
    }

    fn layouts(&self) -> Result<&Layouts, String> {
        self.layout_cache.get_or_init(|| Layouts::read(&self.data)).as_ref().map_err(Clone::clone)
    }

    fn write_profile(&mut self, output: &str, profile: &LayoutProfile) -> Result<(), String> {
        let value = serde_json::to_value(profile).map_err(|error| error.to_string())?;
        let root = self.data.as_object_mut().ok_or_else(|| MISSING_DOCUMENT.to_owned())?;
        let layouts = root.entry(picker_layout::ROOT).or_insert_with(|| {
            serde_json::json!({
                VERSION: picker_layout::VERSION, OUTPUTS: {}
            })
        });
        layouts
            .get_mut(OUTPUTS)
            .and_then(Value::as_object_mut)
            .ok_or_else(|| MISSING_OUTPUTS.to_owned())?
            .insert(output.to_owned(), value);
        self.effective.take();
        self.palette_previews.take();
        self.overview.take();
        Ok(())
    }

    pub(crate) fn set_keys(&mut self, values: &[(&str, Value)]) -> Result<(), String> {
        let layout_values: Vec<_> = values
            .iter()
            .filter(|(path, _)| self.layout.is_some() && picker_layout::owns(path))
            .map(|(path, value)| (*path, Some(value.clone())))
            .collect();
        if !layout_values.is_empty() {
            self.edit_layout(&ProfileEdit::Values(&layout_values))?;
        }
        for (path, value) in values {
            if self.layout.is_none() || !picker_layout::owns(path) {
                self.write_key(path, value.clone());
            }
        }
        Ok(())
    }

    pub(super) fn remove_layout_value(&mut self, path: &str) -> Result<bool, String> {
        if self.layout.is_none() || !picker_layout::owns(path) {
            return Ok(false);
        }
        self.edit_layout(&ProfileEdit::Values(&[(path, None)]))?;
        Ok(true)
    }

    fn stored_profile(
        &mut self,
        output: &str,
    ) -> Result<&mut serde_json::Map<String, Value>, String> {
        self.data
            .get_mut(picker_layout::ROOT)
            .and_then(|layouts| layouts.get_mut(OUTPUTS))
            .and_then(|outputs| outputs.get_mut(output))
            .and_then(Value::as_object_mut)
            .ok_or_else(|| MISSING_PROFILE.to_owned())
    }

    fn edit_layout(&mut self, edit: &ProfileEdit<'_>) -> Result<(), String> {
        if let ProfileEdit::Position(component, Some(position)) = edit
            && !position.valid_for(*component)
        {
            return Err(INVALID_RELATIONSHIP.to_owned());
        }
        let target = self.layout.as_ref().ok_or_else(|| MISSING_OUTPUT.to_owned())?;
        let output = target.output.clone();
        let viewport = target.viewport;
        if !viewport.valid() {
            return Err(INVALID_VIEWPORT.to_owned());
        }
        let seed = self.resolved().map(|(key, profile)| (key, profile.clone()));
        let legacy = target
            .legacy
            .clone()
            .filter(|legacy| seed.as_ref().is_some_and(|(key, _)| key == legacy));
        let mut layouts = self.layout_cache.take().unwrap_or_else(|| Layouts::read(&self.data))?;
        let existing = layouts.outputs.contains_key(&output);
        let profile = layouts.outputs.entry(output.clone()).or_insert_with(|| {
            seed.map_or_else(
                || LayoutProfile::capture(&self.data, viewport),
                |(_, profile)| profile,
            )
        });
        let changed = edit.apply(profile) || profile.viewport != viewport;
        profile.viewport = viewport;
        if !changed && existing {
            let _ = self.layout_cache.set(Ok(layouts));
            return Ok(());
        }
        if existing {
            let stored = self.stored_profile(&output)?;
            match edit {
                ProfileEdit::Values(values) => {
                    let stored_values = stored
                        .get_mut(VALUES)
                        .and_then(Value::as_object_mut)
                        .ok_or_else(|| MISSING_VALUES.to_owned())?;
                    for (path, _) in *values {
                        match profile.values.get(*path) {
                            Some(value) => stored_values
                                .insert((*path).to_owned(), value.clone().unwrap_or(Value::Null)),
                            None => stored_values.remove(*path),
                        };
                    }
                }
                ProfileEdit::Position(..) => {
                    stored.insert(
                        POSITIONS.to_owned(),
                        serde_json::to_value(&profile.positions)
                            .map_err(|error| error.to_string())?,
                    );
                }
            }
            stored.insert(
                VIEWPORT.to_owned(),
                serde_json::to_value(viewport).map_err(|error| error.to_string())?,
            );
            self.effective.take();
            self.palette_previews.take();
            self.overview.take();
        } else {
            self.write_profile(&output, profile)?;
            if let Some(legacy) = legacy {
                layouts.outputs.remove(&legacy);
                self.remove_layout_profile(&legacy);
            }
        }
        let _ = self.layout_cache.set(Ok(layouts));
        Ok(())
    }
}

fn similar(layouts: &Layouts, viewport: Size) -> Option<(&String, &LayoutProfile)> {
    let aspect = |size: Size| size.width / size.height.max(1.0);
    let portrait = |size: Size| size.height > size.width;
    layouts
        .outputs
        .iter()
        .filter(|(_, profile)| portrait(profile.viewport) == portrait(viewport))
        .min_by(|(_, one), (_, other)| {
            let gap = |profile: &LayoutProfile| {
                let exact = profile.viewport == viewport;
                (!exact, (aspect(profile.viewport) - aspect(viewport)).abs())
            };
            let (one, other) = (gap(one), gap(other));
            one.0.cmp(&other.0).then(one.1.total_cmp(&other.1))
        })
}

fn layout_base(document: &Value) -> Value {
    Value::Object(
        document
            .as_object()
            .expect("configuration object")
            .iter()
            .filter(|(key, _)| key.as_str() != picker_layout::ROOT)
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    )
}

enum ProfileEdit<'a> {
    Values(&'a [(&'a str, Option<Value>)]),
    Position(Component, Option<Position>),
}

impl ProfileEdit<'_> {
    fn apply(&self, profile: &mut LayoutProfile) -> bool {
        match self {
            Self::Values(values) => {
                let mut changed = false;
                for (path, value) in *values {
                    let before = profile.values.get(*path).cloned();
                    profile.set(path, value.clone());
                    changed |= profile.values.get(*path) != before.as_ref();
                }
                changed
            }
            Self::Position(component, position) => {
                let changed = profile.positions.get(component).copied() != *position;
                if let Some(position) = position {
                    profile.positions.insert(*component, *position);
                } else {
                    profile.positions.remove(component);
                }
                changed
            }
        }
    }
}
