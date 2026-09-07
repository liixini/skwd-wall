#![cfg(test)]

use std::collections::HashMap;

use crate::contracts::settings::{GraphicsCard, GraphicsTier, SettingsSource, keys, schema};
use crate::domain::input::InputMap;

pub(crate) struct FakeSettingsSource {
    text: HashMap<String, String>,
    flags: HashMap<String, bool>,
    lengths: HashMap<String, usize>,
    presets: HashMap<String, Vec<String>>,
    selected_presets: HashMap<String, String>,
    saved_themes: Vec<String>,
    palette_presets: Vec<(String, String)>,
    graphics: GraphicsCard,
    devices: Vec<crate::contracts::capabilities::GraphicsDevice>,
    niri: bool,
    on_battery: bool,
}

impl Default for FakeSettingsSource {
    fn default() -> Self {
        Self {
            text: HashMap::new(),
            flags: HashMap::new(),
            lengths: HashMap::new(),
            presets: HashMap::new(),
            selected_presets: HashMap::new(),
            saved_themes: Vec::new(),
            palette_presets: vec![
                (String::from("nord"), String::from("Nord")),
                (String::from("dracula"), String::from("Dracula")),
            ],
            graphics: GraphicsCard { name: String::from("Test GPU"), tier: GraphicsTier::Other },
            devices: Vec::new(),
            niri: false,
            on_battery: false,
        }
    }
}

impl FakeSettingsSource {
    pub(crate) fn with_flag(mut self, path: &str, value: bool) -> Self {
        self.flags.insert(path.to_string(), value);
        self
    }

    pub(crate) fn with_on_battery(mut self, on_battery: bool) -> Self {
        self.on_battery = on_battery;
        self
    }

    pub(crate) fn with_devices(
        mut self,
        devices: Vec<crate::contracts::capabilities::GraphicsDevice>,
    ) -> Self {
        self.devices = devices;
        self
    }

    pub(crate) fn with_text(mut self, path: &str, value: &str) -> Self {
        self.text.insert(path.to_string(), value.to_string());
        self
    }

    pub(crate) fn with_array_len(mut self, path: &str, length: usize) -> Self {
        self.lengths.insert(path.to_string(), length);
        self
    }

    pub(crate) fn with_saved_theme(mut self, name: &str) -> Self {
        self.saved_themes.push(name.to_string());
        self
    }
}

impl SettingsSource for FakeSettingsSource {
    fn flag(&self, path: &str) -> bool {
        self.flags.get(path).copied().or_else(|| schema::boolean_default(path)).unwrap_or(false)
    }

    fn flag_default_true(&self, path: &str) -> bool {
        self.flags.get(path).copied().unwrap_or(true)
    }

    fn text(&self, path: &str) -> String {
        self.text.get(path).cloned().unwrap_or_else(|| {
            schema::text_default(path)
                .map(str::to_string)
                .or_else(|| schema::number_default(path).map(|value| value.to_string()))
                .unwrap_or_else(|| match path {
                    keys::we_render::ENGINE => String::from("native"),
                    _ => String::new(),
                })
        })
    }

    fn number(&self, path: &str) -> f64 {
        self.text
            .get(path)
            .and_then(|value| value.parse().ok())
            .or_else(|| schema::number_default(path))
            .unwrap_or(0.0)
    }

    fn array_len(&self, path: &str) -> usize {
        self.lengths.get(path).copied().unwrap_or(0)
    }

    fn display_mode(&self) -> String {
        self.text(keys::selector::DISPLAY_MODE)
    }

    fn selector_mode(&self) -> String {
        match self.display_mode().as_str() {
            "grid" | "wall" => String::from("grid"),
            "hex" => String::from("hex"),
            "sandy" | "nova" => String::from("sandy"),
            _ => String::from("slices"),
        }
    }

    fn selector_preset_names(&self, mode: &str) -> Vec<String> {
        self.presets.get(mode).cloned().unwrap_or_default()
    }

    fn selected_preset(&self, mode: &str) -> Option<String> {
        self.selected_presets.get(mode).cloned()
    }

    fn sandy_grain(&self) -> f32 {
        3.0
    }

    fn sandy_swap_style(&self) -> String {
        String::from("vortex")
    }

    fn launch_animation(&self) -> String {
        String::from("fade")
    }

    fn theme_backend(&self) -> String {
        let legacy = self.text(keys::theme::BACKEND);
        let policy = self.text(keys::theme::POLICY);
        match policy.as_str() {
            "fixed" => return String::from("static"),
            "off" => return String::from("off"),
            "wallpaper" => {}
            _ if legacy == "static" || legacy == "off" => return legacy,
            _ => {}
        }
        let authority = self.text(keys::theme::AUTHORITY);
        match authority.as_str() {
            "caelestia" | "noctalia" | "dms" | "end4" => authority,
            "skwd" => {
                let engine = self.text(keys::theme::ENGINE);
                if engine.is_empty() { legacy } else { engine }
            }
            _ if matches!(legacy.as_str(), "caelestia" | "noctalia" | "dms" | "end4") => legacy,
            _ => legacy,
        }
    }

    fn motion_fast_ms(&self) -> f32 {
        180.0
    }

    fn motion_standard_ms(&self) -> f32 {
        250.0
    }

    fn motion_slow_ms(&self) -> f32 {
        450.0
    }

    fn saved_theme_names(&self) -> Vec<String> {
        self.saved_themes.clone()
    }

    fn palette_presets(&self) -> Vec<(String, String)> {
        self.palette_presets.clone()
    }

    fn bindings(&self) -> InputMap {
        InputMap::default()
    }

    fn graphics_devices(&self) -> Vec<crate::contracts::capabilities::GraphicsDevice> {
        self.devices.clone()
    }

    fn graphics_card(&self) -> GraphicsCard {
        self.graphics.clone()
    }

    fn is_niri(&self) -> bool {
        self.niri
    }

    fn on_battery_power(&self) -> bool {
        self.on_battery
    }
}
