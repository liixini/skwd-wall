use std::path::PathBuf;

use log::{info, warn};
use serde_json::Value;

use crate::contracts::picker::format_config_number as fmt_num;

#[cfg(unix)]
fn secure_config_file(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    let Ok(metadata) = std::fs::symlink_metadata(path) else { return false };
    if !metadata.file_type().is_file() || metadata.permissions().mode() & 0o7777 == 0o600 {
        return false;
    }
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).is_ok()
}

#[cfg(not(unix))]
fn secure_config_file(_path: &std::path::Path) -> bool {
    false
}

#[allow(clippy::struct_field_names)]
pub struct Config {
    pub(super) data: Value,
    pub config_path: PathBuf,
    pub(super) small: bool,
    pub(super) transient: bool,
    pub(super) on_battery: bool,
}

impl Config {
    skwd_config::getters! {
        close_on_selection: bool_setting(skwd_config::schema::setting::general::CLOSE_ON_SELECTION);
        default_folder: text_setting(skwd_config::schema::setting::filter_bar::DEFAULT_FOLDER);
        display_mode: text_setting(skwd_config::schema::setting::selector::DISPLAY_MODE);
        filter_bar_always_visible: bool_setting(skwd_config::schema::setting::general::FILTER_BAR_ALWAYS_VISIBLE);
        filter_bar_sticky: bool_setting(skwd_config::schema::setting::filter_bar::STICKY);
        hex_arc: on_unless_off(skwd_config::keys::selector::HEX_ARC);
        last_filter_favourites: off_unless_on(skwd_config::keys::filter_bar::LAST_FAVOURITES_ONLY);
        last_filter_show_hidden_folders: bool_setting(skwd_config::schema::setting::filter_bar::LAST_SHOW_HIDDEN_FOLDERS);
        last_filter_kind: str(skwd_config::keys::filter_bar::LAST_KIND, "");
        launch_animation: str(skwd_config::keys::launch::ANIMATION, "fade");
        last_filter_orient: str(skwd_config::keys::filter_bar::LAST_ORIENT, "");
        last_filter_resolution: str(skwd_config::keys::filter_bar::LAST_RESOLUTION, "");
        last_filter_sort: str(skwd_config::keys::filter_bar::LAST_SORT, "color");
        main_monitor: str(skwd_config::keys::system::MONITOR, "");
        sandy_swap_loop: off_unless_on(skwd_config::keys::selector::SANDY_SWAP_LOOP);
        sandy_video_out_live: on_unless_off(skwd_config::keys::selector::SANDY_OUTGOING_LIVE);
        slice_wobble: off_unless_on(skwd_config::keys::selector::SLICE_WOBBLE);
        weather_match: off_unless_on(skwd_config::keys::general::WEATHER_MATCH);
    }

    #[cfg(test)]
    pub(crate) fn from_data(data: Value) -> Self {
        let mut data = data;
        canonicalize_picker_config(&mut data);
        Self {
            data,
            config_path: std::env::temp_dir().join("skwd_wall_test.json"),
            small: false,
            transient: false,
            on_battery: false,
        }
    }

    pub fn load() -> Self {
        let config_path = skwd_config::config_path();
        let _ = secure_config_file(&config_path);
        let mut data = if let Ok(text) = std::fs::read_to_string(&config_path) {
            match serde_json::from_str(&text) {
                Ok(val) => val,
                Err(err) => {
                    let backup = config_path.with_extension("json.corrupt");
                    let _ = std::fs::copy(&config_path, &backup);
                    let _ = secure_config_file(&backup);
                    warn!(
                        "config parse error: {err}; the unparseable file was backed up to {} before any setting change overwrites it",
                        backup.display()
                    );
                    Value::Object(serde_json::Map::new())
                }
            }
        } else {
            info!("no config at {}, using defaults", config_path.display());
            Value::Object(serde_json::Map::new())
        };
        canonicalize_picker_config(&mut data);
        Self {
            data,
            config_path,
            small: false,
            transient: false,
            on_battery: skwd_config::on_battery_power(),
        }
    }

    pub fn reload(&mut self) -> bool {
        if self.transient {
            return false;
        }
        self.on_battery = skwd_config::on_battery_power();
        let _ = secure_config_file(&self.config_path);
        if let Ok(text) = std::fs::read_to_string(&self.config_path)
            && let Ok(mut val) = serde_json::from_str::<Value>(&text)
            && {
                canonicalize_picker_config(&mut val);
                true
            }
            && val != self.data
        {
            self.data = val;
            return true;
        }
        false
    }

    pub fn set_screen_width(&mut self, width: f32) {
        self.small = width <= 1600.0;
    }

    pub fn on_battery_power(&self) -> bool {
        self.on_battery
    }

    pub(crate) fn set_on_battery_power(&mut self, on_battery: bool) -> bool {
        if self.on_battery == on_battery {
            return false;
        }
        self.on_battery = on_battery;
        true
    }

    pub(super) fn get(&self, path: &str) -> Option<&Value> {
        skwd_config::get(&self.data, path)
    }

    pub(super) fn str_at(&self, path: &str, default: &str) -> String {
        skwd_config::str_at(&self.data, path, default)
    }

    pub(super) fn num_at(&self, path: &str, default: f64) -> f64 {
        skwd_config::num_at(&self.data, path, default)
    }

    pub(super) fn bool_false_unless_true(&self, path: &str) -> bool {
        skwd_config::bool_false_unless_true(&self.data, path)
    }

    pub(super) fn sel_num(&self, key: &str, large: f64, small: f64) -> f64 {
        let def = if self.small { small } else { large };
        self.num_at(key, def)
    }

    fn write_selector_presets(&mut self, mode: &str, list: Vec<(String, Value)>) {
        let arr: Vec<Value> = list
            .into_iter()
            .map(|(name, params)| {
                let mut obj = serde_json::Map::new();
                obj.insert("name".into(), Value::String(name));
                obj.insert("params".into(), params);
                Value::Object(obj)
            })
            .collect();
        self.save_key(&format!("components.wallpaperSelector.presets.{mode}"), Value::Array(arr));
    }

    pub fn set_selected_preset(&mut self, mode: &str, name: Option<&str>) {
        let path = format!("components.wallpaperSelector.activePreset.{mode}");
        self.save_key(&path, Value::String(name.unwrap_or("").to_string()));
    }

    pub fn save_selector_preset(&mut self, mode: &str, name: &str) {
        let params = self.selector_preset_snapshot();
        let mut list = self.selector_presets(mode);
        if let Some(slot) = list.iter_mut().find(|(label, _)| label == name) {
            slot.1 = params;
        } else {
            list.push((name.to_string(), params));
        }
        self.write_selector_presets(mode, list);
        self.set_selected_preset(mode, Some(name));
    }

    pub fn delete_selector_preset(&mut self, mode: &str, name: &str) {
        let list: Vec<(String, Value)> =
            self.selector_presets(mode).into_iter().filter(|(label, _)| label != name).collect();
        self.write_selector_presets(mode, list);
        if self.selected_preset(mode).as_deref() == Some(name) {
            self.set_selected_preset(mode, None);
        }
    }

    pub fn rename_selected_preset(&mut self, mode: &str, new_name: &str) -> bool {
        let new_name = new_name.trim();
        let Some(old) = self.selected_preset(mode) else {
            return false;
        };
        if new_name.is_empty() || new_name == old {
            return false;
        }
        let mut list = self.selector_presets(mode);
        let Some(slot) = list.iter_mut().find(|(label, _)| label == &old) else {
            return false;
        };
        slot.0 = new_name.to_string();
        self.write_selector_presets(mode, list);
        self.set_selected_preset(mode, Some(new_name));
        true
    }

    pub fn apply_selector_preset(&mut self, mode: &str, name: &str) {
        let Some((_, params)) =
            self.selector_presets(mode).into_iter().find(|(label, _)| label == name)
        else {
            return;
        };
        if let Some(obj) = params.as_object() {
            for (key, value) in obj {
                self.set_key(&format!("components.wallpaperSelector.{key}"), value.clone());
            }
        }
        self.set_selected_preset(mode, Some(name));
    }

    pub fn set_key(&mut self, path: &str, value: Value) {
        let parts: Vec<&str> = path.split('.').collect();
        if !self.data.is_object() {
            self.data = Value::Object(serde_json::Map::new());
        }
        let mut cur = &mut self.data;
        for part in &parts[..parts.len() - 1] {
            match descend(cur, part) {
                Some(next) => cur = next,
                None => return,
            }
        }
        put(cur, parts[parts.len() - 1], value);
    }

    pub fn persist(&self) {
        if self.transient {
            return;
        }
        let mut data = self.data.clone();
        canonicalize_picker_config(&mut data);
        let text = match serde_json::to_string_pretty(&data) {
            Ok(json) => format!("{json}\n"),
            Err(err) => {
                warn!("config serialize failed: {err}");
                return;
            }
        };
        if let Some(dir) = self.config_path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        match skwd_config::atomic_write_mode(&self.config_path, text.as_bytes(), Some(0o600)) {
            Ok(()) => info!("config saved to {}", self.config_path.display()),
            Err(err) => warn!("config write failed: {err}"),
        }
    }

    pub fn save_key(&mut self, path: &str, value: Value) {
        self.set_key(path, value);
        self.persist();
    }

    pub fn root(&self) -> &Value {
        &self.data
    }

    pub fn begin_transient(&mut self) -> Value {
        self.transient = true;
        self.data.clone()
    }

    pub fn restore_transient(&mut self, snapshot: Value) {
        self.data = snapshot;
        self.transient = false;
    }

    #[cfg(test)]
    pub(crate) fn is_transient(&self) -> bool {
        self.transient
    }

    pub fn remove_key(&mut self, path: &str) {
        let parts: Vec<&str> = path.split('.').collect();
        let mut cur = &mut self.data;
        for part in &parts[..parts.len() - 1] {
            match cur.as_object_mut().and_then(|obj| obj.get_mut(*part)) {
                Some(next) => cur = next,
                None => return,
            }
        }
        if let Some(obj) = cur.as_object_mut() {
            obj.remove(parts[parts.len() - 1]);
        }
    }

    pub fn browser_apply_button(&self, source: crate::contracts::browser::Source) -> bool {
        use crate::contracts::browser::Source;
        use skwd_config::keys::sources;
        let key = match source {
            Source::Wallhaven => sources::WALLHAVEN_SHOW_APPLY_BUTTON,
            Source::Steam => sources::STEAM_SHOW_APPLY_BUTTON,
            Source::Unsplash => sources::UNSPLASH_SHOW_APPLY_BUTTON,
            Source::Pexels => sources::PEXELS_SHOW_APPLY_BUTTON,
            Source::Youtube => sources::YOUTUBE_SHOW_APPLY_BUTTON,
            Source::Bing => return false,
        };
        self.flag_default_config(key)
    }

    pub fn flag_default_true(&self, path: &str) -> bool {
        self.get(path).and_then(Value::as_bool) != Some(false)
    }

    pub fn flag_default_config(&self, path: &str) -> bool {
        skwd_config::schema::boolean_default(path).map_or_else(
            || self.get(path).and_then(Value::as_bool) == Some(true),
            |default| skwd_config::bool_at(self.root(), path, default),
        )
    }

    pub fn str_path(&self, path: &str) -> String {
        if path == skwd_config::keys::we_render::ENGINE {
            return String::from("native");
        }
        if let Some(val) = self.get(path) {
            if let Some(text) = val.as_str() {
                return text.to_string();
            }
            if let Some(num) = val.as_f64() {
                return fmt_num(num);
            }
        }
        if let Some(value) = self.legacy_resolution_bound(path) {
            return value;
        }
        skwd_config::schema::text_default(path).map_or_else(
            || skwd_config::schema::number_default(path).map_or_else(String::new, fmt_num),
            str::to_string,
        )
    }

    fn legacy_resolution_bound(&self, path: &str) -> Option<String> {
        let base = path.strip_suffix(".from").or_else(|| path.strip_suffix(".to"))?;
        if !base.starts_with(skwd_config::keys::filter_bar::RESOLUTION_PRESETS) {
            return None;
        }
        let width = self.get(&format!("{base}.width"))?.as_i64()?;
        let height = self.get(&format!("{base}.height"))?.as_i64()?;
        (width > 0 && height > 0).then(|| format!("{width}x{height}"))
    }

    #[allow(clippy::match_same_arms)]
    pub fn num_path(&self, path: &str) -> f64 {
        if let Some(value) = skwd_config::schema::read_number_with(self.root(), path, self.small) {
            return value;
        }
        if let Some(num) = self.get(path).and_then(Value::as_f64) {
            return num;
        }
        match path {
            key if key.ends_with("sliceWidth") => self.slice_width() as f64,
            key if key.ends_with("expandedWidth") => self.expanded_width() as f64,
            key if key.ends_with("sliceHeight") => self.slice_height() as f64,
            key if key.ends_with("visibleCount") => self.visible_count() as f64,
            key if key.ends_with("sliceSpacing") => self.slice_spacing() as f64,
            key if key.ends_with("skewOffset") => self.skew_offset() as f64,
            key if key.ends_with("hexRadius") => self.hex_radius() as f64,
            key if key.ends_with("hexRows") => self.hex_rows() as f64,
            key if key.ends_with("hexCols") => self.hex_cols() as f64,
            key if key.ends_with("hexScrollStep") => self.hex_scroll_step() as f64,
            key if key.ends_with("hexArcIntensityX10") => {
                (self.hex_arc_intensity() * 10.0).round() as f64
            }
            key if key.ends_with("tagCloudWidth") => self.tag_cloud_width() as f64,
            key if key.ends_with("tagCloudHeight") => self.tag_cloud_height() as f64,
            key if key.ends_with("gridColumns") => self.grid_columns() as f64,
            key if key.ends_with("gridRows") => self.grid_rows() as f64,
            key if key.ends_with("gridThumbWidth") => self.grid_thumb_width() as f64,
            key if key.ends_with("gridThumbHeight") => self.grid_thumb_height() as f64,
            key if key.ends_with("sandyGrain") => self.sandy_grain() as f64,
            key if key.ends_with("sandyRingSize") => (self.sandy_ring_size() * 100.0) as f64,
            key if key.ends_with("sandyResScale") => (self.sandy_res_scale() * 100.0) as f64,
            key if key.ends_with("sandyLod") => self.sandy_lod() as f64,
            key if key.ends_with("cornerTL")
                || key.ends_with("cornerTR")
                || key.ends_with("cornerBR")
                || key.ends_with("cornerBL") =>
            {
                16.0
            }
            key if key.ends_with("wallhavenColumns") || key.ends_with("steamColumns") => 6.0,
            key if key.ends_with("wallhavenRows") || key.ends_with("steamRows") => 3.0,
            key if key.ends_with("wallhavenThumbWidth") || key.ends_with("steamThumbWidth") => {
                300.0
            }
            key if key.ends_with("wallhavenThumbHeight") || key.ends_with("steamThumbHeight") => {
                169.0
            }
            _ => skwd_config::schema::number_default(path).unwrap_or(0.0),
        }
    }

    pub fn array_len(&self, path: &str) -> usize {
        self.get(path).and_then(Value::as_array).map_or(0, std::vec::Vec::len)
    }

    pub fn array_values(&self, path: &str) -> Vec<Value> {
        self.get(path).and_then(Value::as_array).cloned().unwrap_or_default()
    }

    pub fn array_push(&mut self, path: &str, value: Value) {
        let needs_init = !matches!(self.get(path), Some(Value::Array(_)));
        if needs_init {
            self.set_key(path, Value::Array(Vec::new()));
        }
        let parts: Vec<&str> = path.split('.').collect();
        let mut cur = &mut self.data;
        for part in &parts {
            let Some(next) = cur.as_object_mut().and_then(|obj| obj.get_mut(*part)) else {
                return;
            };
            cur = next;
        }
        if let Some(arr) = cur.as_array_mut() {
            arr.push(value);
        }
        self.persist();
    }

    pub fn array_remove(&mut self, path: &str, idx: usize) {
        let parts: Vec<&str> = path.split('.').collect();
        let mut cur = &mut self.data;
        for part in &parts {
            match cur.as_object_mut().and_then(|obj| obj.get_mut(*part)) {
                Some(val) => cur = val,
                None => return,
            }
        }
        if let Some(arr) = cur.as_array_mut()
            && idx < arr.len()
        {
            arr.remove(idx);
        }
        self.persist();
    }

    pub fn ensure_selector_enabled(&mut self) {
        let exists =
            self.get(skwd_config::keys::selector::COMPONENTS).is_some_and(Value::is_object);
        if !exists {
            self.save_key(skwd_config::keys::selector::ENABLED, Value::Bool(true));
        }
    }
}

fn canonicalize_picker_config(data: &mut Value) {
    skwd_config::canonicalize_paper_engine(data);
    skwd_config::canonicalize_we_renderer(data);
    canonicalize_resolution_presets(data);
    canonicalize_browser_apply_button(data);
}

fn canonicalize_browser_apply_button(data: &mut Value) {
    let Some(sources) = data.get_mut("sources").and_then(Value::as_object_mut) else {
        return;
    };
    let Some(previous) = sources.remove("showApplyButton").and_then(|value| value.as_bool()) else {
        return;
    };
    for source in
        crate::contracts::browser::Source::ALL.into_iter().filter(|source| source.searchable())
    {
        let settings =
            sources.entry(source.key()).or_insert_with(|| Value::Object(serde_json::Map::new()));
        if let Some(settings) = settings.as_object_mut() {
            settings.entry("showApplyButton").or_insert(Value::Bool(previous));
        }
    }
}

fn canonicalize_resolution_presets(data: &mut Value) {
    let Some(entries) = data
        .get_mut("filterBar")
        .and_then(|value| value.get_mut("resolutionPresets"))
        .and_then(Value::as_array_mut)
    else {
        return;
    };
    let bounds: Vec<Option<(i64, i64, bool)>> = entries
        .iter()
        .map(|entry| {
            let dimensions = entry
                .get("from")
                .and_then(Value::as_str)
                .and_then(crate::domain::library::filter::parse_resolution)
                .or_else(|| {
                    entry
                        .get("width")
                        .and_then(Value::as_i64)
                        .zip(entry.get("height").and_then(Value::as_i64))
                })?;
            let tall = match entry.get("orientation").and_then(Value::as_str) {
                Some("tall") => true,
                Some("wide") => false,
                _ => dimensions.1 > dimensions.0,
            };
            Some((dimensions.0, dimensions.1, tall))
        })
        .collect();
    for (index, entry) in entries.iter_mut().enumerate() {
        let Some(object) = entry.as_object_mut() else { continue };
        let tall = bounds[index].is_some_and(|(_, _, tall)| tall);
        object
            .entry("orientation")
            .or_insert_with(|| Value::String(if tall { "tall" } else { "wide" }.to_string()));
        let legacy = object
            .get("width")
            .and_then(Value::as_i64)
            .zip(object.get("height").and_then(Value::as_i64))
            .map(|(width, height)| format!("{width}x{height}"));
        if let Some(legacy) = legacy {
            object.entry("from").or_insert_with(|| Value::String(legacy.clone()));
            object.entry("to").or_insert_with(|| {
                let upper = bounds[index + 1..]
                    .iter()
                    .flatten()
                    .find(|(_, _, candidate_tall)| *candidate_tall == tall)
                    .map_or_else(String::new, |(width, height, _)| {
                        format!("{}x{}", width.saturating_sub(1), height.saturating_sub(1))
                    });
                Value::String(upper)
            });
        }
        object.remove("width");
        object.remove("height");
    }
}

fn descend<'a>(cur: &'a mut Value, part: &str) -> Option<&'a mut Value> {
    if cur.is_array() {
        let idx = part.parse::<usize>().ok()?;
        return cur.as_array_mut()?.get_mut(idx);
    }
    if !cur.is_object() {
        *cur = Value::Object(serde_json::Map::new());
    }
    Some(
        cur.as_object_mut()
            .unwrap()
            .entry(part.to_string())
            .or_insert_with(|| Value::Object(serde_json::Map::new())),
    )
}

fn put(cur: &mut Value, last: &str, value: Value) {
    if cur.is_array() {
        if let Some(slot) = last
            .parse::<usize>()
            .ok()
            .and_then(|idx| cur.as_array_mut().and_then(|arr| arr.get_mut(idx)))
        {
            *slot = value;
        }
        return;
    }
    if !cur.is_object() {
        *cur = Value::Object(serde_json::Map::new());
    }
    cur.as_object_mut().unwrap().insert(last.to_string(), value);
}

#[cfg(all(test, unix))]
mod tests;
