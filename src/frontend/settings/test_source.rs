#![cfg(test)]

use std::collections::HashMap;
use std::sync::Arc;

use crate::contracts::settings::{
    ExclusionScope, GraphicsCard, GraphicsTier, Overview, OverviewMemo, ProfileState,
    ProfileStates, SavedLayouts, Scan, SettingsSource, keys, schema,
};
use crate::domain::input::InputMap;
use crate::frontend::settings::{PlaceSource, RuntimeStatus};
use crate::frontend::ui::calm::{VALUE_CLOSE, VALUE_OPEN, sentinel};

const PLAIN_SLOTS: u32 = 5;
const FETCHED_SECONDS_AGO: u64 = 120;
const WEATHER_PLACE: &str = "Stockholm";
const WEATHER_CREDITS: &str = "/assets/outdoors/sources.json";
const WEATHER_DISPLAYS: [(&str, &str); 2] = [("DP-1", "Woods"), ("HDMI-A-1", "Harbour")];
const WEATHER_NOW: &str = "cloudy";
const WEATHER_QUERY: &str = "grey overcast sky";
const WEATHER_PLAYLIST: &str = "Grey days";
const WEATHER_MATCHES: usize = 12;

pub(crate) fn place_source(cfg: &dyn SettingsSource) -> PlaceSource<'_> {
    PlaceSource {
        cfg,
        themes: &[],
        folders: &[],
        analysis: "",
        backends: &[],
        runtime: RuntimeStatus::default(),
    }
}

pub(crate) fn plain(text: &str) -> String {
    let text = text.replace([VALUE_OPEN, VALUE_CLOSE], "");
    (0..PLAIN_SLOTS)
        .fold(text, |text, slot| text.replace(sentinel(slot).expect("slot"), &format!("[{slot}]")))
}

pub(crate) fn weather_status(
    source: crate::contracts::daemon::WeatherSource,
) -> crate::contracts::daemon::WeatherStatus {
    use crate::contracts::daemon::{
        LightPhase, OutdoorMatch, OutdoorOutput, OutdoorSeason, OutdoorStatus, WeatherSource,
        WeatherState, WeatherStatus,
    };
    let output = |name: &str, wallpaper: &str| OutdoorOutput {
        output: name.into(),
        key: format!("{wallpaper}.png"),
        name: wallpaper.into(),
        preview: None,
        held: false,
    };
    let outdoors = (source == WeatherSource::Outdoors).then(|| OutdoorStatus {
        season: OutdoorSeason::Spring,
        light: LightPhase::Day,
        match_kind: OutdoorMatch::Library,
        location: WEATHER_PLACE.into(),
        credits: Some(WEATHER_CREDITS.into()),
        outputs: WEATHER_DISPLAYS.iter().map(|(name, wallpaper)| output(name, wallpaper)).collect(),
        ..Default::default()
    });
    WeatherStatus {
        enabled: true,
        source,
        status: WeatherState::Active,
        weather: vec![WEATHER_NOW.into()],
        query: (source == WeatherSource::Semantic).then(|| WEATHER_QUERY.into()),
        playlist_name: (source == WeatherSource::Playlists).then(|| WEATHER_PLAYLIST.into()),
        resolved_count: WEATHER_MATCHES,
        fetched_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|now| now.as_secs().saturating_sub(FETCHED_SECONDS_AGO)),
        outdoors,
        ..Default::default()
    }
}

pub(crate) fn render(tag: &str, key: &str, pairs: &[(&str, &str)]) -> String {
    let mut args = fluent::FluentArgs::new();
    for (name, value) in pairs {
        args.set(*name, *value);
    }
    crate::i18n::Catalog::for_locale(tag).format(key, Some(&args))
}

pub(crate) struct FakeSettingsSource {
    text: HashMap<String, String>,
    flags: HashMap<String, bool>,
    lengths: HashMap<String, usize>,
    saved_themes: Vec<String>,
    palette_presets: Vec<(String, String)>,
    palette_previews: Vec<(String, Vec<String>)>,
    graphics: GraphicsCard,
    devices: Vec<crate::contracts::capabilities::GraphicsDevice>,
    niri: std::cell::Cell<bool>,
    on_battery: bool,
    bindings: InputMap,
    profile_states: HashMap<String, ProfileState>,
    exclusion_scopes: Vec<ExclusionScope>,
    layouts: SavedLayouts,
    overview: OverviewMemo,
}

impl Default for FakeSettingsSource {
    fn default() -> Self {
        Self {
            text: HashMap::new(),
            flags: HashMap::new(),
            lengths: HashMap::new(),
            saved_themes: Vec::new(),
            palette_presets: vec![
                (String::from("nord"), String::from("Nord")),
                (String::from("dracula"), String::from("Dracula")),
            ],
            palette_previews: Vec::new(),
            graphics: GraphicsCard { name: String::from("Test GPU"), tier: GraphicsTier::Other },
            devices: Vec::new(),
            niri: std::cell::Cell::new(false),
            on_battery: false,
            bindings: InputMap::default(),
            profile_states: HashMap::new(),
            exclusion_scopes: Vec::new(),
            layouts: SavedLayouts::default(),
            overview: OverviewMemo::default(),
        }
    }
}

impl FakeSettingsSource {
    pub(crate) fn with_layouts(
        mut self,
        names: &[&str],
        active: Option<usize>,
        changes: usize,
    ) -> Self {
        self.layouts = SavedLayouts {
            names: names.iter().map(ToString::to_string).collect(),
            active,
            changes,
        };
        self
    }

    pub(crate) fn with_palettes(mut self, palettes: &[(&str, &str, [&str; 4])]) -> Self {
        self.palette_presets =
            palettes.iter().map(|(key, label, _)| ((*key).into(), (*label).into())).collect();
        self.palette_previews = palettes
            .iter()
            .map(|(key, _, colours)| ((*key).into(), colours.map(String::from).to_vec()))
            .collect();
        self
    }

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

    pub(crate) fn set_niri(&self, niri: bool) {
        self.niri.set(niri);
    }

    pub(crate) fn with_niri(self, niri: bool) -> Self {
        self.niri.set(niri);
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

    fn saved_layouts(&self) -> SavedLayouts {
        self.layouts.clone()
    }

    fn sandy_grain(&self) -> f32 {
        3.0
    }

    fn sandy_swap_style(&self) -> String {
        String::from("vortex")
    }

    fn launch_animation(&self) -> String {
        self.text(keys::launch::ANIMATION)
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
        self.palette_previews.clone()
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
        self.niri.get()
    }

    fn on_battery_power(&self) -> bool {
        self.on_battery
    }

    fn effective_picker_fps(&self) -> u32 {
        let normal = self.number(keys::general::MAX_FPS).max(1.0);
        let battery = self.number(keys::performance::BATTERY_FPS);
        let effective = if self.on_battery && self.flag(keys::performance::BATTERY_SAVER) {
            if battery == 0.0 { normal } else { normal.min(battery) }
        } else {
            normal
        };
        effective.round().clamp(1.0, u32::MAX as f64) as u32
    }

    fn overview(&self, stamp: u64, scan: Scan<'_>) -> Arc<Overview> {
        self.overview.get(stamp, scan)
    }

    fn effective_video_idle_seconds(&self) -> u32 {
        let normal = self.number(keys::paper::IDLE_PAUSE_SECONDS);
        if !(self.on_battery && self.flag(keys::performance::BATTERY_SAVER)) {
            return normal.round().clamp(0.0, u32::MAX as f64) as u32;
        }
        let battery = self.number(keys::performance::BATTERY_VIDEO_IDLE_SECONDS);
        match (normal, battery) {
            (0.0, value) | (value, 0.0) => value,
            (normal, battery) => normal.min(battery),
        }
        .round()
        .clamp(0.0, u32::MAX as f64) as u32
    }
}
