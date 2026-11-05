use crate::contracts::daemon::{LibraryWatchMode, LibraryWatchStatus, OutputStatus};
use crate::contracts::settings::keys;
use crate::frontend::settings::place::HOME;
use crate::frontend::settings::test_source::{FakeSettingsSource, place_source};
use crate::frontend::settings::{ActionId, Control, SectionId};

use super::{FitState, Shot, capture_live, library, localized};

const CHANGED: &str = "settings-home-changed";
const SOUND: &str = "settings-wallpapers-sound";
const COLOURS: &str = "settings-tab-theme";
const ROTATION: &str = "settings-section-random-rotation";
const TRANSITIONS: &str = "settings-section-wallpaper-transitions";
const DISPLAYS: [&str; 3] = ["DP-1", "DP-2", "HDMI-A-1"];
const EIGHT: usize = 8;
const MANY: usize = 24;
const PREFIX: &str = "home-";
const BOARD: &str = "home-board";
const ARMED: &str = "home-armed";
const DOWN: &str = "home-watch-down";
const DEFAULTS: &str = "home-defaults";
const OFF: &str = "home-off";
const SOME: &str = "home-eight";
const LOTS: &str = "home-many";

fn leaked<T>(value: T) -> &'static T {
    Box::leak(Box::new(value))
}

pub(super) fn shelf(name: &str) -> library::Shelf<'static> {
    if !name.starts_with(PREFIX) {
        return library::shelf(name);
    }
    let mode = if name == DOWN { LibraryWatchMode::UNAVAILABLE } else { LibraryWatchMode::NATIVE };
    let watch = LibraryWatchStatus { mode: mode.to_string(), ok: true, ..Default::default() };
    library::Shelf {
        watch: Some(leaked(watch)),
        armed: (name == ARMED).then(|| leaked(ActionId::ResetEverything)),
        ..library::Shelf::default()
    }
}

fn outputs() -> Vec<OutputStatus> {
    DISPLAYS
        .map(|name| OutputStatus {
            name: name.to_string(),
            current: format!("{name}.png"),
            ..Default::default()
        })
        .into()
}

fn board() -> FakeSettingsSource {
    FakeSettingsSource::default()
        .with_text(keys::wallpaper::VOLUME, "40")
        .with_flag(keys::wallpaper::MUTE, false)
        .with_flag(keys::playback::MUTE_ON_OTHER_AUDIO, true)
        .with_flag(keys::paper::VIDEO_MULTI_PROCESS, false)
        .with_flag(keys::general::RANDOM_ROTATE, true)
        .with_text(keys::general::RANDOM_INTERVAL, "300")
        .with_flag(keys::transition::ENABLED, true)
        .with_text(keys::transition::DURATION_MS, "1200")
}

fn toggled(count: usize) -> FakeSettingsSource {
    let plain = FakeSettingsSource::default();
    let paths: Vec<(String, bool)> = place_source(&plain)
        .places()
        .iter()
        .flat_map(|place| &place.sections)
        .flat_map(crate::frontend::settings::Section::rows)
        .filter_map(|row| match &row.control {
            Control::Toggle { path, value } => Some((path.clone(), !value)),
            _ => None,
        })
        .take(count)
        .collect();
    paths
        .into_iter()
        .fold(FakeSettingsSource::default(), |cfg, (path, value)| cfg.with_flag(&path, value))
}

fn state(name: &str, cfg: FakeSettingsSource, statuses: Vec<OutputStatus>) -> FitState {
    let connected = !statuses.is_empty();
    FitState {
        name: name.to_string(),
        cfg,
        weather: None,
        statuses,
        drawing: None,
        playback: None,
        connected,
        apps: None,
        backends: Vec::new(),
        sddm: Vec::new(),
    }
}

pub(super) fn states() -> Vec<FitState> {
    let quiet = FakeSettingsSource::default()
        .with_flag(keys::wallpaper::MUTE, true)
        .with_flag(keys::general::RANDOM_ROTATE, false)
        .with_flag(keys::transition::ENABLED, false);
    vec![
        state(DEFAULTS, FakeSettingsSource::default(), Vec::new()),
        state(BOARD, board(), outputs()),
        state(ARMED, board(), outputs()),
        state(OFF, quiet, outputs()),
        state(DOWN, board(), Vec::new()),
        state(SOME, toggled(EIGHT), outputs()),
        state(LOTS, toggled(MANY), outputs()),
    ]
}

#[test]
#[ignore = "manual offscreen render of the Home place"]
fn render_home() {
    let out = localized();
    let changed = Some(SectionId::new(HOME, CHANGED));
    let at = |key| Some(SectionId::new(HOME, key));
    let shots = [
        ("home", BOARD, None),
        ("home-defaults", DEFAULTS, None),
        ("home-snd", BOARD, at(SOUND)),
        ("home-rot", BOARD, at(ROTATION)),
        ("home-trn", BOARD, at(TRANSITIONS)),
        ("home-colours", BOARD, at(COLOURS)),
        ("home-chg", BOARD, changed),
        ("home-armed", ARMED, changed),
        ("home-off", OFF, None),
        ("home-off-open", OFF, changed),
        ("home-down", DOWN, None),
        ("home-eight", SOME, changed),
        ("home-many", LOTS, changed),
    ];
    let table = states();
    let mut heights = Vec::new();
    for (name, wanted, open) in shots {
        let fit = table.iter().find(|fit| fit.name == wanted).expect("state");
        let shot = Shot { place: HOME, cfg: &fit.cfg, open, pop: None, edit: None, weather: None };
        let (view, used) = capture_live(&shot, &fit.live(), Some(&format!("{out}/{name}.png")));
        heights.push(format!("{name}: used {used:.0} of {view:.0}"));
    }
    std::fs::write(format!("{out}/heights.txt"), heights.join("\n")).expect("heights");
}
