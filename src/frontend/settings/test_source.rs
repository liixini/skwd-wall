#![cfg(test)]

use std::collections::HashMap;

use crate::contracts::settings::{
    ExclusionScope, GraphicsCard, GraphicsTier, ProfileState, ProfileStates, SettingsSource, keys,
    schema,
};
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
    bindings: InputMap,
    profile_states: HashMap<String, ProfileState>,
    exclusion_scopes: Vec<ExclusionScope>,
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
            bindings: InputMap::default(),
            profile_states: HashMap::new(),
            exclusion_scopes: Vec::new(),
        }
    }
}

impl FakeSettingsSource {
    pub(crate) fn with_flag(mut self, path: &str, value: bool) -> Self {
        self.flags.insert(path.to_string(), value);
        self
    }

    pub(crate) fn with_profile_state(
        mut self,
        index: usize,
        folder: &str,
        state: ProfileState,
    ) -> Self {
        self.profile_states.insert(format!("{index}:{folder}"), state);
        self
    }

    pub(crate) fn with_exclusion_scopes(mut self, scopes: Vec<ExclusionScope>) -> Self {
        self.exclusion_scopes = scopes;
        self
    }

    pub(crate) fn with_niri(mut self, niri: bool) -> Self {
        self.niri = niri;
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

    pub(crate) fn with_bindings(mut self, bindings: InputMap) -> Self {
        self.bindings = bindings;
        self
    }
}

impl SettingsSource for FakeSettingsSource {
    fn default_semantic_model_available(&self) -> bool {
        self.flag_default_true("test.defaultModelAvailable")
    }

    fn flag(&self, path: &str) -> bool {
        self.flags.get(path).copied().or_else(|| schema::boolean_default(path)).unwrap_or(false)
    }

    fn flag_default_true(&self, path: &str) -> bool {
        self.flags.get(path).copied().unwrap_or(true)
    }

    fn text(&self, path: &str) -> String {
        let value = self.text.get(path).cloned().unwrap_or_else(|| {
            schema::text_default(path)
                .map(str::to_string)
                .or_else(|| schema::number_default(path).map(|value| value.to_string()))
                .unwrap_or_else(|| match path {
                    keys::we_render::ENGINE => String::from("native"),
                    _ => String::new(),
                })
        });
        if path == keys::theme::MODE && value.is_empty() {
            self.text(keys::matugen::MODE)
        } else {
            value
        }
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

    fn library_folders(&self) -> Vec<String> {
        if let Some(len) = self.lengths.get(keys::library::FOLDERS) {
            return (0..*len)
                .map(|index| self.text(&format!("{}.{index}", keys::library::FOLDERS)))
                .collect();
        }
        let mut folders = vec![self.text(keys::paths::WALLPAPER)];
        let videos = self.text(keys::paths::VIDEO_WALLPAPER);
        if !videos.is_empty() && !folders.contains(&videos) {
            folders.push(videos);
        }
        folders
    }

    fn exclusion_scopes(&self) -> Vec<ExclusionScope> {
        self.exclusion_scopes.clone()
    }

    fn resolved_folder(&self, folder: &str) -> String {
        folder.to_owned()
    }

    fn profile_states(&self, index: usize, folders: &[String]) -> ProfileStates {
        let state = |folder: &str| {
            self.profile_states.get(&format!("{index}:{folder}")).copied().unwrap_or_default()
        };
        (folders.iter().map(|folder| state(folder)).collect(), state(""))
    }

    fn display_mode(&self) -> String {
        self.text(keys::selector::DISPLAY_MODE)
    }

    fn selector_mode(&self) -> String {
        match self.display_mode().as_str() {
            "grid" | "wall" => String::from("grid"),
            "hex" => String::from("hex"),
            "sandy" | "nova" => String::from("sandy"),
            "hand" => String::from("hand"),
            "collection" => String::from("collection"),
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

    fn palette_colors(&self) -> Vec<(String, Vec<String>)> {
        Vec::new()
    }

    fn palette_presets(&self) -> Vec<(String, String)> {
        self.palette_presets.clone()
    }

    fn bindings(&self) -> InputMap {
        self.bindings.clone()
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
