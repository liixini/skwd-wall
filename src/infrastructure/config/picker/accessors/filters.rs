use serde_json::Value;

use crate::domain::library::filter::ResolutionPreset;

use super::super::Config;

impl Config {
    pub fn filter_bar_offset(&self) -> (f32, f32) {
        (
            skwd_config::schema::setting::filter_bar::OFFSET_X.read(self.root()) as f32,
            skwd_config::schema::setting::filter_bar::OFFSET_Y.read(self.root()) as f32,
        )
    }

    pub fn filter_bar_orientation(&self) -> &'static str {
        match skwd_config::schema::setting::filter_bar::ORIENTATION.read(self.root()).as_str() {
            "vertical" => "vertical",
            _ => "horizontal",
        }
    }

    pub fn filter_bar_visual_style(&self) -> String {
        use crate::contracts::picker::Mode;
        let configured = skwd_config::schema::setting::filter_bar::VISUAL_STYLE.read(self.root());
        let mode =
            Mode::try_from_key(&configured).unwrap_or_else(|| Mode::from_key(&self.display_mode()));
        String::from(match mode {
            Mode::Sandy | Mode::Hand | Mode::Depth | Mode::Collection => Mode::Slices.as_key(),
            mode => mode.as_key(),
        })
    }

    pub fn filter_show(&self, key: &str) -> bool {
        self.get(&format!("filterBar.show.{key}")).and_then(Value::as_bool) != Some(false)
    }

    pub fn last_filter_color(&self) -> i64 {
        self.num_at(skwd_config::keys::filter_bar::LAST_COLOR, -1.0) as i64
    }

    pub fn last_filter_folder(&self) -> String {
        let folder =
            self.str_at(skwd_config::keys::filter_bar::LAST_FOLDER, &self.default_folder());
        if folder.is_empty() {
            String::from(crate::domain::library::filter::ALL_WALLPAPERS)
        } else {
            folder
        }
    }

    pub fn resolution_presets(&self) -> Vec<ResolutionPreset> {
        let Some(entries) =
            self.get(skwd_config::keys::filter_bar::RESOLUTION_PRESETS).and_then(Value::as_array)
        else {
            return Vec::new();
        };
        let mut seen = std::collections::HashSet::new();
        let pixels = |value: &Value| {
            value.as_i64().or_else(|| {
                let value = value.as_f64()?;
                (value > 0.0 && value.fract() == 0.0 && value <= i64::MAX as f64)
                    .then_some(value as i64)
            })
        };
        entries
            .iter()
            .filter_map(|entry| {
                let label = entry.get("label")?.as_str()?.trim();
                let legacy = || {
                    let width = pixels(entry.get("width")?)?;
                    let height = pixels(entry.get("height")?)?;
                    Some(((width, height), Some((width, height))))
                };
                let range = if let Some(from) = entry.get("from").and_then(Value::as_str) {
                    let from = crate::domain::library::filter::parse_resolution(from)?;
                    let to = match entry.get("to").and_then(Value::as_str) {
                        Some(value) if !value.trim().is_empty() => {
                            Some(crate::domain::library::filter::parse_resolution(value)?)
                        }
                        _ => None,
                    };
                    (from, to)
                } else {
                    legacy()?
                };
                let ((from_width, from_height), to) = range;
                let orientation = match entry.get("orientation").and_then(Value::as_str) {
                    Some("tall") => "tall",
                    Some("wide") => "wide",
                    _ if from_height > from_width => "tall",
                    _ => "wide",
                };
                let orient = |width: i64, height: i64| match orientation {
                    "tall" => (width.min(height), width.max(height)),
                    _ => (width.max(height), width.min(height)),
                };
                let from_edges = orient(from_width, from_height);
                if to.is_some_and(|(width, height)| {
                    let to_edges = orient(width, height);
                    from_edges.0 > to_edges.0 || from_edges.1 > to_edges.1
                }) {
                    return None;
                }
                let (to_width, to_height) =
                    to.map_or((None, None), |(width, height)| (Some(width), Some(height)));
                let preset = ResolutionPreset {
                    label: label.to_string(),
                    orientation: orientation.to_string(),
                    from_width,
                    from_height,
                    to_width,
                    to_height,
                };
                (!label.is_empty() && seen.insert(preset.key())).then_some(preset)
            })
            .collect()
    }
}
