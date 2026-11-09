mod home;
mod library;
mod search;

use std::collections::{HashMap, HashSet};

use iced::advanced::widget::{Operation, operation};
use iced::{Rectangle, Size, Vector};

use crate::contracts::daemon::{
    AppThemeStatus, AppThemesResult, OutputStatus, PlaybackStatus, WeatherSource, WeatherStatus,
};
use crate::contracts::media::MediaKind;
use crate::contracts::settings::drawing::{PackState, Phase, Status};
use crate::domain::input::{InputAction, InputMap};

use super::{TokenSide, switch_identity, token_key, token_side};
use crate::contracts::settings::SettingsSource;
use crate::frontend::settings::test_source::FakeSettingsSource;
use crate::frontend::settings::{
    CalmCtx, ChromeCtx, Control, FocusCtx, FoldersCtx, Group, GroupKind, KeysCtx, PageSnapshot,
    ProfilesCtx, Row, Section, SectionId, SettingsFocus, SourceCtx, WorkbenchInput, place_keys,
    settings_workbench,
};
use crate::frontend::ui::UI_FONT;
use crate::frontend::ui::calm::render_tests::board;
use crate::i18n::{Script, set_language};

const OUT: &str = "SKWD_SETTINGS_OUT";
const LANG: &str = "SKWD_LANG";
const ONLY_PLACE: &str = "SKWD_FIT_PLACE";
const SCREEN: Size = Size::new(1366.0, 768.0);
const FIT_MARGIN: f32 = super::SURFACE.foot + super::BODY.divider;
const LABEL_SLACK: f32 = 2.0;
const FIT_COMMANDS: usize = 9;
const FITTED: [&str; 10] = [
    crate::frontend::settings::place::HOME,
    crate::frontend::settings::place::LAYOUT,
    crate::frontend::settings::place::COLOURS,
    crate::frontend::settings::place::LIBRARY,
    crate::frontend::settings::place::KEYBINDS,
    crate::frontend::settings::place::WALLPAPERS,
    crate::frontend::settings::place::AUTOMATION,
    crate::frontend::settings::place::BAR,
    crate::frontend::settings::place::SYSTEM,
    crate::frontend::settings::place::SOURCES,
];
const STEAM_PATHS: [&str; 3] = [
    "/home/someone-with-a-long-name/.var/app/com.valvesoftware.Steam/.local/share/Steam",
    "/home/someone-with-a-long-name/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/workshop/content/431960",
    "/run/media/someone/Games Library/SteamLibrary/steamapps/common/wallpaper_engine/assets",
];
const MODEL_PATHS: [&str; 2] = [
    "/home/someone/.local/share/skwd/models/siglip2-large/semantic-pack.json",
    "/home/someone/.local/share/skwd/models/pe-core-l14-336/semantic-pack.json",
];
const INTERIM: [&str; 2] =
    ["settings-section-wallpaper-efficiency", "settings-general-general-card"];

#[test]
fn token_keys_split_sides() {
    let sentence = token_key("display.fps", 0, TokenSide::Sentence);
    assert_ne!(sentence, token_key("display.fps", 0, TokenSide::Body));
    assert_ne!(
        token_key("keys.next", 0, TokenSide::Body),
        token_key("keys.next", 1, TokenSide::Body)
    );
    assert_eq!(sentence, token_key("display.fps", 0, TokenSide::Sentence));
}

#[test]
fn master_menu_sides() {
    let row = |control| Row::of(String::new(), String::new(), control);
    let toggle = |path: &str| row(Control::Toggle { path: path.into(), value: true });
    let menu = row(Control::Dropdown {
        path: "gpu".into(),
        options: vec![("auto".into(), "Auto".into())],
        current: "auto".into(),
        palettes: Vec::new(),
    });
    let section = Section {
        master: Some(toggle("saver")),
        head: vec![menu],
        groups: vec![Group::new(GroupKind::Column, None, vec![toggle("detail")])],
        ..Section::new(SectionId::new("app", "power"), "Power", "")
    };
    let sides: Vec<_> = (0..3).map(|at| token_side(&section, at)).collect();
    assert_eq!(sides, [TokenSide::Sentence, TokenSide::Sentence, TokenSide::Body]);
}

#[test]
fn checks_join_picked() {
    let toggle = |title: &str, value| {
        Row::of(title.into(), String::new(), Control::Toggle { path: title.to_lowercase(), value })
    };
    let slot = crate::frontend::settings::Slot::Checks {
        rows: vec![0, 1, 2],
        heading: String::new(),
        blank: "no category".into(),
    };
    for (picks, expected) in [
        ([true, true, false], "General and Anime"),
        ([true, true, true], "General, Anime and People"),
        ([false, false, false], "no category"),
    ] {
        let rows: Vec<_> = ["General", "Anime", "People"]
            .into_iter()
            .zip(picks)
            .map(|(title, on)| toggle(title, on))
            .collect();
        let rows: Vec<&Row> = rows.iter().collect();
        let caption = crate::i18n::with_language("en-US", || super::checked(&slot, &rows));
        assert_eq!(caption, expected);
    }
}

#[test]
fn secrets_show_tail() {
    assert_eq!(crate::frontend::settings::masked("0123456789abcdef"), "••••cdef");
    assert_eq!(crate::frontend::settings::masked("ab"), "••••ab");
}

#[test]
fn flipped_toggles_keep_identity() {
    use crate::frontend::settings::ActionId;
    let identity = |value: bool| {
        let id = ActionId::SetThemeTarget("dms", !value);
        switch_identity(&Control::ToggleAction { id, value })
    };
    assert_eq!(identity(true), identity(false));
    assert!(identity(true).is_some());
}

fn source<'a>(cfg: &'a dyn SettingsSource, values: &'a HashMap<String, String>) -> SourceCtx<'a> {
    SourceCtx {
        layout_premade: None,
        config: cfg,
        values,
        themes: &[],
        folders: &[],
        backends: &[],
        outputs: &[],
        output_statuses: &[],
        output_previews: Box::leak(Box::default()),
        library_watch: None,
        app_themes: None,
        playback: None,
        weather: None,
        weather_pending: false,
        playlists: &[],
        sddm_themes: &[],
        drawing: None,
        drawing_selection: None,
        drawing_view_count: 0,
        drawing_library: false,
        connected: false,
        thumbnail_task: None,
        drawing_task: None,
        analysis: String::new(),
    }
}

fn live_source<'a>(
    cfg: &'a dyn SettingsSource,
    values: &'a HashMap<String, String>,
    live: &Live<'a>,
) -> SourceCtx<'a> {
    SourceCtx {
        output_statuses: live.statuses,
        drawing: live.drawing,
        drawing_selection: live.drawing.map(|status| status.key.as_str()),
        folders: live.shelf.folders,
        library_watch: live.shelf.watch,
        playback: live.playback,
        connected: live.connected,
        app_themes: live.apps,
        backends: live.backends,
        sddm_themes: live.sddm,
        ..source(cfg, values)
    }
}

fn seeded(cfg: &dyn SettingsSource, place: &str, live: &Live<'_>) -> HashMap<String, String> {
    use crate::contracts::settings::keys;
    let empty = HashMap::new();
    let mut values = HashMap::new();
    if place == crate::frontend::settings::place::LIBRARY {
        library::seed(cfg, live.shelf, &mut values);
    }
    for section in live_source(cfg, &empty, live).place(place).sections {
        for row in section.into_rows() {
            match row.control {
                Control::ActionBtn {
                    id: crate::frontend::settings::ActionId::ChooseRunningProcess,
                    ..
                } => {
                    let path = keys::playback::PROCESSES;
                    values.insert(path.to_string(), cfg.text(path));
                }
                Control::Number { key, path, .. } => {
                    values.insert(
                        key,
                        crate::contracts::picker::format_config_number(cfg.number(&path)),
                    );
                }
                Control::TextField { key, path, .. } | Control::Resolution { key, path, .. } => {
                    values.insert(key, cfg.text(&path));
                }
                _ => {}
            }
        }
    }
    values
}

thread_local! {
    static STRIP_UI: std::cell::Cell<crate::frontend::settings::LayoutsUi> = const {
        std::cell::Cell::new(crate::frontend::settings::LayoutsUi {
            confirm: None,
            armed: false,
            premade: None,
            renaming: false,
            rejected: false,
        })
    };
}

struct Shot<'a> {
    place: &'static str,
    cfg: &'a FakeSettingsSource,
    open: Option<SectionId>,
    pop: Option<u64>,
    edit: Option<u64>,
    weather: Option<&'a WeatherStatus>,
}

#[derive(Clone, Copy, Default)]
struct Live<'a> {
    statuses: &'a [OutputStatus],
    drawing: Option<&'a Status>,
    playback: Option<&'a PlaybackStatus>,
    connected: bool,
    shelf: library::Shelf<'a>,
    keys: KeysCtx<'a>,
    apps: Option<&'a AppThemesResult>,
    backends: &'a [String],
    sddm: &'a [String],
    search: Option<&'a search::Probe<'a>>,
}

thread_local! {
    static SWITCH: std::cell::RefCell<Option<(PageSnapshot, f32)>> =
        const { std::cell::RefCell::new(None) };
}

thread_local! {
    static FLOORS: std::cell::RefCell<HashMap<String, f32>> = std::cell::RefCell::new(HashMap::new());
}

thread_local! {
    static UNTRIMMED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(super) fn untrimmed(section: Section) -> Section {
    if UNTRIMMED.get() {
        return Section { trim: crate::frontend::settings::Trim::None, ..section };
    }
    section
}

pub(super) fn note_floor(id: &iced::widget::Id, floor: f32) {
    FLOORS.with(|floors| floors.borrow_mut().insert(format!("{id:?}"), floor));
}

struct Squeezed(Vec<String>);

impl Operation for Squeezed {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn container(&mut self, id: Option<&iced::widget::Id>, bounds: Rectangle) {
        let Some(id) = id else { return };
        let key = format!("{id:?}");
        let floor = FLOORS.with(|floors| floors.borrow().get(&key).copied());
        if let Some(floor) = floor.filter(|floor| bounds.width + LABEL_SLACK < *floor) {
            self.0.push(format!("{key} {:.1} of {floor:.1}", bounds.width));
        }
    }
}

trait Probe {
    fn run(&mut self, size: Size, view: iced::Element<'_, crate::app::Message>);
}

impl<T: Operation> Probe for T {
    fn run(&mut self, size: Size, view: iced::Element<'_, crate::app::Message>) {
        crate::frontend::headless::operate(size, UI_FONT, view, self);
    }
}

struct Measure {
    view: f32,
    used: f32,
    visible: f32,
    hidden: f32,
    heights: Vec<f32>,
}

struct Extent {
    target: iced::widget::Id,
    last: Option<iced::widget::Id>,
    bodies: Vec<iced::widget::Id>,
    found: Option<Rectangle>,
    bottom: f32,
    hidden: f32,
    heights: Vec<f32>,
}

impl Extent {
    fn new(
        target: iced::widget::Id,
        last: Option<iced::widget::Id>,
        bodies: Vec<iced::widget::Id>,
    ) -> Self {
        Self { target, last, bodies, found: None, bottom: 0.0, hidden: 0.0, heights: Vec::new() }
    }

    fn visible(&self) -> f32 {
        self.bottom - self.found.map_or(0.0, |scroller| scroller.y)
    }

    fn used(&self) -> f32 {
        let top = self.found.map_or(0.0, |scroller| scroller.y);
        self.bottom - top + self.hidden
    }
}

impl Operation for Extent {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn container(&mut self, id: Option<&iced::widget::Id>, bounds: Rectangle) {
        if id.is_some() && id == self.last.as_ref() {
            self.bottom = bounds.y + bounds.height;
        }
    }

    fn scrollable(
        &mut self,
        id: Option<&iced::widget::Id>,
        bounds: Rectangle,
        content: Rectangle,
        _translation: Vector,
        _state: &mut dyn operation::Scrollable,
    ) {
        if id == Some(&self.target) {
            self.found = Some(bounds);
        }
        if id.is_some_and(|id| self.bodies.contains(id)) {
            let hidden = (content.height - bounds.height).max(0.0);
            self.hidden += hidden;
            if hidden > 0.0 {
                self.heights.push(bounds.height);
            }
        }
    }
}

fn capture(shot: &Shot<'_>, out: Option<&str>) -> (f32, f32) {
    capture_live(shot, &Live::default(), out)
}

fn capture_live(shot: &Shot<'_>, live: &Live<'_>, out: Option<&str>) -> (f32, f32) {
    let Measure { view, used, .. } = capture_in(SCREEN, shot, live, out, &mut []);
    (view, used)
}

fn capture_in(
    size: Size,
    shot: &Shot<'_>,
    live: &Live<'_>,
    out: Option<&str>,
    probes: &mut [&mut dyn Probe],
) -> Measure {
    let Shot { place, cfg, open, pop, edit, weather } = *shot;
    let layouts = STRIP_UI.with(std::cell::Cell::get);
    let mut values = seeded(cfg, place, live);
    let found = live.search.and_then(|probe| probe.found);
    let landed = live.search.and_then(|probe| probe.landed);
    if let Some(found) = found {
        search::seed(cfg, found, &mut values);
    }
    let folds = HashSet::new();
    let bars = HashMap::new();
    let drafts: HashMap<_, _> =
        live.shelf.draft.map(|(kind, draft)| (kind.key(), draft.to_string())).into_iter().collect();
    let palette = board();
    let switch = SWITCH.take();
    let view = || {
        settings_workbench(WorkbenchInput {
            layouts,
            source: SourceCtx { weather, ..live_source(cfg, &values, live) },
            focus: FocusCtx {
                keyboard_focus: landed.map_or(SettingsFocus::Index, |_| SettingsFocus::Controls),
                focused_control: landed.map_or(0, |(_, row)| row),
                focused_choice: None,
                expanded_details: &folds,
                bar_reveals: &bars,
                active_input: None,
                armed: live.shelf.armed,
            },
            chrome: ChromeCtx {
                viewport: (size.width, size.height),
                scale: 1.0,
                entrance: 1.0,
                palette: &palette,
            },
            calm: CalmCtx {
                open,
                section: landed.map_or(0, |(section, _)| section),
                busy: f32::from(u8::from(open.is_some())),
                pop,
                edit,
                instant: true,
                closing: None,
            },
            profiles: ProfilesCtx { selected: live.shelf.selected, filter: live.shelf.filter },
            folders: FoldersCtx { drafts: &drafts, undo: live.shelf.undo },
            preview_path: "",
            transition_preview: None,
            tab_transition: 1.0,
            page_transition: switch.as_ref().map_or(1.0, |(_, at)| *at),
            emblem_transition: 1.0,
            previous: switch.as_ref().map(|(old, _)| old),
            tab: place,
            search_open: found.is_some(),
            search_query: found.map_or("", |found| found.query),
            search_results: found.map_or(&[], |found| found.results),
            search_total: found.map_or(0, |found| found.total),
            search_at: found.and_then(|found| found.at),
            keys: live.keys,
            process_picker: None,
            weather_review: None,
        })
    };
    if let Some(out) = out {
        crate::frontend::headless::capture(size, UI_FONT, out, view);
    }
    let sections = SourceCtx { weather, ..live_source(cfg, &values, live) }.place(place).sections;
    let last = sections.last().map(|section| super::section_widget_id(section.id));
    let target = crate::frontend::settings::settings_reading_id();
    let bodies = sections.iter().map(|section| super::body_scroll_id(section.id)).collect();
    let mut extent = Extent::new(target, last, bodies);
    crate::frontend::headless::operate(size, UI_FONT, view(), &mut extent);
    for probe in probes {
        probe.run(size, view());
    }
    let scroller = extent.found.expect("reading scroller");
    Measure {
        view: scroller.height,
        used: extent.used(),
        visible: extent.visible(),
        hidden: extent.hidden,
        heights: extent.heights,
    }
}

#[test]
#[ignore = "manual offscreen render of every settings place"]
fn render_places() {
    let out = std::env::var(OUT).expect(OUT);
    set_language("en-US");
    iced_wgpu::graphics::text::font_system()
        .write()
        .expect("font system")
        .load_font(crate::shell::ui_font_bytes(Script::Latin).into());
    let cfg = FakeSettingsSource::default();
    let empty = HashMap::new();
    for place in place_keys() {
        capture(
            &Shot { place, cfg: &cfg, open: None, pop: None, edit: None, weather: None },
            Some(&format!("{out}/{place}.png")),
        );
        let first = source(&cfg, &empty).place(place).sections.first().map(|section| section.id);
        if first.is_some() {
            let shot = Shot { place, cfg: &cfg, open: first, pop: None, edit: None, weather: None };
            capture(&shot, Some(&format!("{out}/{place}-open.png")));
        }
    }
}

#[test]
#[ignore = "manual offscreen render of the System place"]
fn render_system() {
    use crate::contracts::settings::keys;
    let out = localized();
    let cfg = system(true);
    let place = crate::frontend::settings::place::SYSTEM;
    let empty = HashMap::new();
    let sections = source(&cfg, &empty).place(place).sections;
    capture(
        &Shot { place, cfg: &cfg, open: None, pop: None, edit: None, weather: None },
        Some(&format!("{out}/system.png")),
    );
    for section in sections.iter().filter(|section| section.has_body()) {
        let shot =
            Shot { place, cfg: &cfg, open: Some(section.id), pop: None, edit: None, weather: None };
        capture(&shot, Some(&format!("{out}/system-{}.png", section.id.key)));
    }
    for (key, path) in [
        ("settings-performance-gpu-label", keys::performance::GPU_PREFERENCE),
        ("settings-language-choice-label", keys::general::LANGUAGE),
    ] {
        let pop = Some(token_key(path, 0, TokenSide::Sentence));
        capture(
            &Shot { place, cfg: &cfg, open: None, pop, edit: None, weather: None },
            Some(&format!("{out}/system-{key}-menu.png")),
        );
    }
    let off = system(false);
    let power = sections.first().map(|section| section.id);
    for (open, name) in [(None, "system-saver-off"), (power, "system-saver-off-open")] {
        capture(
            &Shot { place, cfg: &off, open, pop: None, edit: None, weather: None },
            Some(&format!("{out}/{name}.png")),
        );
    }
}

fn localized() -> String {
    let tag = std::env::var(LANG).unwrap_or_else(|_| String::from("en-US"));
    let language = crate::i18n::LANGUAGES.iter().find(|language| language.tag == tag).expect("tag");
    set_language(language.tag);
    iced_wgpu::graphics::text::font_system()
        .write()
        .expect("font system")
        .load_font(crate::shell::ui_font_bytes(language.script).into());
    std::env::var(OUT).expect(OUT)
}

fn shot(cfg: &FakeSettingsSource) -> Shot<'_> {
    let place = crate::frontend::settings::place::AUTOMATION;
    Shot { place, cfg, open: None, pop: None, edit: None, weather: None }
}

fn automation(count: usize) -> FakeSettingsSource {
    use crate::contracts::settings::keys;
    let cfg = FakeSettingsSource::default()
        .with_flag(keys::general::RANDOM_ROTATE, true)
        .with_flag(keys::general::RANDOM_INCLUDE_STATIC, true)
        .with_flag(keys::general::RANDOM_INCLUDE_VIDEO, true)
        .with_flag(keys::general::RANDOM_INCLUDE_WE, false)
        .with_flag(keys::general::RANDOM_INCLUDE_FAVOURITES, false)
        .with_flag(keys::general::NOTIFY_ON_WALLPAPER_CHANGE, false)
        .with_flag(keys::schedule::ENABLED, true)
        .with_flag(keys::schedule::APPLY_ON_START, true)
        .with_flag(keys::system::RESTORE_ON_STARTUP, true)
        .with_text(keys::general::RANDOM_INTERVAL, "300")
        .with_text(keys::general::LOCALE, "Stockholm")
        .with_text(keys::schedule::LATITUDE, "59.33")
        .with_text(keys::schedule::LONGITUDE, "18.06")
        .with_array_len(keys::post_processing::LIST, count);
    (0..count).fold(cfg, |cfg, at| {
        let kind = if at % 2 == 0 { "all" } else { "static" };
        cfg.with_text(&format!("postProcessing.{at}.type"), kind).with_text(
            &format!("postProcessing.{at}.command"),
            &format!("notify-send \"{at}\" %name%"),
        )
    })
}

#[test]
#[ignore = "manual offscreen render of the Automation place"]
fn render_automation() {
    use crate::contracts::settings::keys;
    use crate::frontend::settings::test_source::weather_status;
    let out = localized();
    let cfg = automation(2);
    let empty = HashMap::new();
    let sections =
        source(&cfg, &empty).place(crate::frontend::settings::place::AUTOMATION).sections;
    let mut heights = Vec::new();
    let mut take = |name: &str, shot: &Shot<'_>| {
        let (view, content) = capture(shot, Some(&format!("{out}/{name}.png")));
        heights.push(format!("{name}: used {content:.0} of {view:.0}"));
    };
    take("automation", &shot(&cfg));
    for section in &sections {
        let open = Some(section.id);
        take(&format!("automation-{}", section.id.key), &Shot { open, ..shot(&cfg) });
    }
    let edit = Some(token_key(keys::general::RANDOM_INTERVAL, 0, TokenSide::Sentence));
    take("automation-interval-editor", &Shot { edit, ..shot(&cfg) });
    let location = sections.get(2).map(|section| section.id);
    let modes = [
        ("off", keys::weather::SOURCE_PLAYLISTS, false, None),
        ("playlists", keys::weather::SOURCE_PLAYLISTS, true, Some(WeatherSource::Playlists)),
        ("semantic", keys::weather::SOURCE_SEMANTIC, true, Some(WeatherSource::Semantic)),
        ("outdoors", keys::weather::SOURCE_OUTDOORS, true, Some(WeatherSource::Outdoors)),
    ];
    for (mode, source, enabled, status) in modes {
        let status = status.map(weather_status);
        let cfg = automation(2)
            .with_flag(keys::weather::ENABLED, enabled)
            .with_text(keys::weather::SOURCE, source);
        let shot = Shot { open: location, weather: status.as_ref(), ..shot(&cfg) };
        take(&format!("automation-location-{mode}"), &shot);
    }
    let pop = Some(token_key(crate::frontend::settings::WEATHER_MODE_KEY, 0, TokenSide::Sentence));
    take("automation-mode-menu", &Shot { pop, ..shot(&cfg) });
    let many = automation(9);
    let open = sections.last().map(|section| section.id);
    take("automation-commands-many", &Shot { open, ..shot(&many) });
    let pop = Some(token_key("postProcessing.1.type", 0, TokenSide::Body));
    take("automation-runs-for-menu", &Shot { open, pop, ..shot(&cfg) });
    let off = FakeSettingsSource::default()
        .with_flag(keys::general::RANDOM_ROTATE, false)
        .with_flag(keys::schedule::ENABLED, false);
    take("automation-off", &shot(&off));
    std::fs::write(format!("{out}/heights.txt"), heights.join("\n")).expect("heights");
}

#[test]
#[ignore = "manual offscreen render of the Sources place"]
fn render_sources() {
    use crate::contracts::settings::keys;
    let out = localized();
    let place = crate::frontend::settings::place::SOURCES;
    let cfg = FakeSettingsSource::default().with_flag(keys::sources::BING_ENABLED, true);
    let shot = |cfg| Shot { place, cfg, open: None, pop: None, edit: None, weather: None };
    let empty = HashMap::new();
    let sections = source(&cfg, &empty).place(place).sections;
    let mut heights = Vec::new();
    let mut take = |name: &str, shot: &Shot<'_>| {
        let (view, content) = capture(shot, Some(&format!("{out}/{name}.png")));
        heights.push(format!("{name}: used {content:.0} of {view:.0}"));
    };
    take("sources", &shot(&cfg));
    for section in &sections {
        take(
            &format!("sources-{}", section.id.key),
            &Shot { open: Some(section.id), ..shot(&cfg) },
        );
    }
    let menus = [
        ("wallhaven-sort", keys::wallhaven::DEFAULT_SORT),
        ("wallhaven-categories", keys::wallhaven::DEFAULT_GENERAL),
        ("wallhaven-purity", keys::wallhaven::DEFAULT_SFW),
        ("workshop-category", keys::steam::DEFAULT_CATEGORY),
    ];
    for (name, path) in menus {
        let pop = Some(token_key(path, 0, TokenSide::Sentence));
        take(&format!("sources-{name}-menu"), &Shot { pop, ..shot(&cfg) });
    }
    let wallhaven = sections.first().map(|section| section.id);
    let pop = Some(token_key(keys::wallhaven::DEFAULT_ATLEAST, 0, TokenSide::Body));
    take("sources-minimum-menu", &Shot { open: wallhaven, pop, ..shot(&cfg) });
    let edit = Some(token_key(keys::wallhaven::API_KEY, 0, TokenSide::Body));
    take("sources-key-editor", &Shot { open: wallhaven, edit, ..shot(&cfg) });
    let busy = busy_sources();
    for section in &sections {
        take(
            &format!("sources-busy-{}", section.id.key),
            &Shot { open: Some(section.id), ..shot(&busy) },
        );
    }
    let off = sources(false);
    take("sources-off", &shot(&off));
    take("sources-off-open", &Shot { open: wallhaven, ..shot(&off) });
    std::fs::write(format!("{out}/heights.txt"), heights.join("\n")).expect("heights");
}

fn system(saver: bool) -> FakeSettingsSource {
    use crate::contracts::settings::keys;
    FakeSettingsSource::default()
        .with_flag(keys::performance::BATTERY_SAVER, saver)
        .with_flag(keys::performance::BATTERY_WALLPAPER_PERFORMANCE, true)
        .with_text(keys::general::MAX_FPS, "144")
        .with_text(keys::paper::IDLE_PAUSE_SECONDS, "30")
        .with_text(keys::performance::BATTERY_FPS, "30")
        .with_text(keys::performance::BATTERY_VIDEO_IDLE_SECONDS, "60")
}

fn sources(on: bool) -> FakeSettingsSource {
    use crate::contracts::settings::keys;
    FakeSettingsSource::default()
        .with_flag(keys::features::WALLHAVEN, on)
        .with_flag(keys::features::STEAM, on)
        .with_flag(keys::sources::UNSPLASH_ENABLED, on)
        .with_flag(keys::sources::PEXELS_ENABLED, on)
        .with_flag(keys::sources::BING_ENABLED, on)
        .with_flag(keys::sources::YOUTUBE_ENABLED, on)
        .with_flag(keys::wallhaven::DEFAULT_ANIME, false)
        .with_flag(keys::wallhaven::DEFAULT_PEOPLE, false)
}

fn busy_sources() -> FakeSettingsSource {
    use crate::contracts::settings::keys;
    sources(true)
        .with_flag(keys::wallhaven::DEFAULT_ANIME, true)
        .with_flag(keys::wallhaven::DEFAULT_PEOPLE, true)
        .with_flag(keys::wallhaven::DEFAULT_SKETCHY, true)
        .with_flag(keys::wallhaven::DEFAULT_NSFW, true)
        .with_text(keys::wallhaven::DEFAULT_RATIOS, "21x9")
        .with_text(keys::steam::DEFAULT_CATEGORY, "Technology")
        .with_text(keys::steam::DEFAULT_RESOLUTION, "3440 x 1440")
        .with_text(keys::steam::BACKEND, "steamcmd")
        .with_text(keys::steam::USERNAME, "wallpaper-collector")
        .with_text(keys::wallhaven::API_KEY, "0123456789abcdef")
        .with_text(keys::paths::STEAM, STEAM_PATHS[0])
        .with_text(keys::paths::STEAM_WORKSHOP, STEAM_PATHS[1])
        .with_text(keys::paths::STEAM_WE_ASSETS, STEAM_PATHS[2])
}

fn busy_bar() -> FakeSettingsSource {
    use crate::contracts::settings::keys;
    let presets = keys::filter_bar::RESOLUTION_PRESETS;
    let bands = [
        ("1080p", "1920x1080", "2559x1439"),
        ("", "2560x1440", ""),
        ("4K", "3840x2160", "7679x4319"),
    ];
    let cfg = FakeSettingsSource::default()
        .with_array_len(presets, bands.len())
        .with_array_len(keys::semantic::MODELS, 2)
        .with_text("semantic.models.0.name", "SigLIP2 B/16 large")
        .with_text("semantic.models.0.manifest", MODEL_PATHS[0])
        .with_text("semantic.models.1.name", "PE-Core L14 336")
        .with_text("semantic.models.1.manifest", MODEL_PATHS[1])
        .with_text(keys::semantic::MANIFEST, MODEL_PATHS[1])
        .with_text(keys::semantic::INDEX_PROFILE, "multiview")
        .with_text(keys::tagging::DEFAULT_SEARCH_MODE, "describe")
        .with_text(keys::filter_bar::ORIENTATION, "vertical")
        .with_flag(keys::filter_bar::SHOW_ORIENT, false)
        .with_text(keys::general::FILTER_BAR_VISIBILITY, "hidden");
    bands.iter().enumerate().fold(cfg, |cfg, (at, (label, from, to))| {
        cfg.with_text(&format!("{presets}.{at}.label"), label)
            .with_text(&format!("{presets}.{at}.from"), from)
            .with_text(&format!("{presets}.{at}.to"), to)
    })
}

fn long_model_bar() -> FakeSettingsSource {
    busy_bar()
        .with_text("semantic.models.0.name", "SigLIP2 ViT-B/16 large multiview index, quantized")
}

fn floats_off_bar() -> FakeSettingsSource {
    use crate::contracts::settings::keys;
    busy_bar()
        .with_flag(keys::filter_bar::SHOW_ARRANGE, false)
        .with_flag(keys::filter_bar::SHOW_SWATCH, false)
}

fn quiet_bar() -> FakeSettingsSource {
    use crate::contracts::settings::keys;
    FakeSettingsSource::default()
        .with_flag(keys::semantic::ENABLED, false)
        .with_flag("test.defaultModelAvailable", false)
}

#[test]
#[ignore = "manual offscreen render of the Bar place"]
fn render_bar() {
    use crate::contracts::settings::keys;
    let out = localized();
    let place = crate::frontend::settings::place::BAR;
    let cfg = busy_bar();
    let shot = |cfg| Shot { place, cfg, open: None, pop: None, edit: None, weather: None };
    let empty = HashMap::new();
    let sections = source(&cfg, &empty).place(place).sections;
    let ids: Vec<_> = sections.iter().map(|section| Some(section.id)).collect();
    let mut heights = Vec::new();
    let mut take = |name: &str, shot: &Shot<'_>| {
        let (view, content) = capture(shot, Some(&format!("{out}/{name}.png")));
        heights.push(format!("{name}: used {content:.0} of {view:.0}"));
    };
    take("bar", &shot(&cfg));
    for section in &sections {
        take(&format!("bar-{}", section.id.key), &Shot { open: Some(section.id), ..shot(&cfg) });
    }
    for (name, path) in [
        ("style", keys::filter_bar::VISUAL_STYLE),
        ("folder", keys::filter_bar::DEFAULT_FOLDER),
        ("model", keys::semantic::MANIFEST),
    ] {
        let pop = Some(token_key(path, 0, TokenSide::Sentence));
        take(&format!("bar-{name}-menu"), &Shot { pop, ..shot(&cfg) });
    }
    let filter = ids[0];
    for (name, path) in
        [("buttons", "filterBar.show.type.all"), ("sorts", "filterBar.show.sort.date")]
    {
        let pop = Some(token_key(path, 0, TokenSide::Body));
        take(&format!("bar-{name}-menu"), &Shot { open: filter, pop, ..shot(&cfg) });
    }
    let sizes = ids[1];
    let add = sections[1]
        .rows()
        .find(|row| row.look.cell == crate::frontend::settings::Cell::Menu)
        .and_then(|row| row.control.token())
        .expect("add menu");
    let pop = Some(token_key(&add, 0, TokenSide::Body));
    take("bar-add-menu", &Shot { open: sizes, pop, ..shot(&cfg) });
    let edit = Some(token_key("filterBar.resolutionPresets.0.from", 1, TokenSide::Body));
    take("bar-height-editor", &Shot { open: sizes, edit, ..shot(&cfg) });
    let quiet = quiet_bar();
    take("bar-quiet", &shot(&quiet));
    for open in [sizes, ids[3]] {
        let key = open.map_or("", |id| id.key);
        take(&format!("bar-quiet-{key}"), &Shot { open, ..shot(&quiet) });
    }
    let off = floats_off_bar();
    take("bar-floats-off", &Shot { open: filter, ..shot(&off) });
    let long = long_model_bar();
    take("bar-long-model", &Shot { open: ids[3], ..shot(&long) });
    std::fs::write(format!("{out}/heights.txt"), heights.join("\n")).expect("heights");
}

const OUTPUTS: [(&str, i32, i32, MediaKind, &str); 4] = [
    ("DP-1", 2560, 1440, MediaKind::Video, "/walls/aurora-ridge-at-night-long-cut.mp4"),
    ("DP-3", 1920, 1080, MediaKind::Static, "/walls/harbour-dusk.jpg"),
    ("HDMI-A-1", 1920, 1080, MediaKind::WallpaperEngine, "2953314812"),
    ("DP-2", 3440, 1440, MediaKind::Static, "/walls/moss.png"),
];

fn outputs(count: usize) -> Vec<OutputStatus> {
    OUTPUTS
        .iter()
        .take(count)
        .map(|(name, width, height, kind, file)| OutputStatus {
            name: (*name).to_string(),
            kind: kind.clone(),
            path: (*file).to_string(),
            we_id: (*file).to_string(),
            width: *width,
            height: *height,
            volume: 40,
            ..Default::default()
        })
        .collect()
}

const LONG_NAMES: [&str; 2] = [
    "/walls/aurora-ridge-at-night-over-the-northern-sea-with-a-long-exposure-and-stars-final-cut-v12.mp4",
    "/walls/harbourduskoverthetownwithnobreakpointsatallinthefilenameextendedfinalcut20260101.jpg",
];

fn long_outputs() -> Vec<OutputStatus> {
    let mut shown = outputs(OUTPUTS.len());
    for (status, file) in shown.iter_mut().zip(LONG_NAMES) {
        status.path = file.to_string();
    }
    shown
}

fn pack(phase: Phase, pack: PackState) -> Status {
    Status {
        pack,
        phase,
        pack_path: String::from("/home/someone/.local/share/skwd/drawing/manifest.json"),
        pack_bytes: 1_400_000_000,
        cache_bytes: 90_000_000,
        key: String::from("harbour-dusk.jpg"),
        stage: String::from("lineart"),
        progress: 0.4,
        completed: 3,
        total: 12,
        ..Default::default()
    }
}

fn pause_status() -> PlaybackStatus {
    PlaybackStatus {
        fullscreen_supported: true,
        maximized_supported: true,
        full_width_supported: true,
        focused_supported: true,
        other_audio_supported: true,
        ..Default::default()
    }
}

fn busy_wallpapers() -> FakeSettingsSource {
    use crate::contracts::settings::keys;
    FakeSettingsSource::default()
        .with_flag(keys::playback::FULLSCREEN, true)
        .with_flag(keys::playback::MAXIMIZED, true)
        .with_flag(keys::playback::FOCUSED, true)
        .with_flag(keys::playback::PROCESS_ENABLED, true)
        .with_flag(keys::playback::MUTE_ON_OTHER_AUDIO, true)
        .with_flag(keys::drawing::ENABLED, true)
        .with_text(keys::playback::PROCESSES, "steam*, mpv")
}

struct FitState {
    name: String,
    cfg: FakeSettingsSource,
    weather: Option<WeatherStatus>,
    statuses: Vec<OutputStatus>,
    drawing: Option<Status>,
    playback: Option<PlaybackStatus>,
    connected: bool,
    apps: Option<AppThemesResult>,
    backends: Vec<String>,
    sddm: Vec<String>,
}

impl FitState {
    fn live(&self) -> Live<'_> {
        Live {
            statuses: &self.statuses,
            drawing: self.drawing.as_ref(),
            playback: self.playback.as_ref(),
            connected: self.connected,
            shelf: home::shelf(&self.name),
            keys: keys_ctx(&self.name),
            apps: self.apps.as_ref(),
            backends: &self.backends,
            sddm: &self.sddm,
            search: None,
        }
    }
}

const LETTERS: usize = 26;
const FIRST_LETTER: u8 = b'a';
const LOW_MODS: &str = "alt";
const HIGH_MODS: &str = "ctrl+alt";
const MOVED_KEY: &str = "Shift+Right click";
const MENU_STATES: [&str; 2] = ["keys-menu", "keys-menu-custom"];

fn state_pop(name: &str) -> Option<u64> {
    MENU_STATES.contains(&name).then(crate::frontend::settings::second_keys_token)
}

const LISTENS: &str = crate::contracts::settings::keys::keybind::FAVOURITE;
const RISES: &str = crate::contracts::settings::keys::keybind::FOLDER_NEXT;

fn keys_ctx(state: &str) -> KeysCtx<'static> {
    match state {
        "keys-listening" => KeysCtx { capture: Some((LISTENS, 0)), ..KeysCtx::default() },
        "keys-rising" => KeysCtx { capture: Some((RISES, 1)), ..KeysCtx::default() },
        "keys-find" => KeysCtx { query: "tag", ..KeysCtx::default() },
        "keys-moved" => {
            KeysCtx { moved: Some((MOVED_KEY, &["keybind-source-steam"])), ..KeysCtx::default() }
        }
        "keys-lit" => KeysCtx { lit: Some(LISTENS), ..KeysCtx::default() },
        _ => KeysCtx::default(),
    }
}

fn key_state(name: &str, bindings: &[(InputAction, &str)]) -> FitState {
    let map = InputMap::from_bindings(
        bindings.iter().map(|(action, binding)| (*action, (*binding).to_string())),
    );
    FitState {
        name: name.to_string(),
        cfg: FakeSettingsSource::default().with_bindings(map),
        weather: None,
        statuses: Vec::new(),
        drawing: None,
        playback: None,
        connected: false,
        apps: None,
        backends: Vec::new(),
        sddm: Vec::new(),
    }
}

fn key_states() -> Vec<FitState> {
    let second: Vec<_> = crate::contracts::picker::KEY_BINDINGS
        .into_iter()
        .enumerate()
        .map(|(at, entry)| {
            let spare = char::from(FIRST_LETTER + (at % LETTERS) as u8);
            let group = if at < LETTERS { LOW_MODS } else { HIGH_MODS };
            (entry.action, format!("{}, {group}+{spare}", entry.action.default_binding()))
        })
        .collect();
    let second: Vec<_> =
        second.iter().map(|(action, binding)| (*action, binding.as_str())).collect();
    let long = [
        (InputAction::Studio, "ctrl+shift+right-click, ctrl+q"),
        (InputAction::SearchMode, "ctrl+shift+middle-click, ctrl+u"),
        (InputAction::Autocomplete, "shift+middle-click, ctrl+alt+pagedown"),
    ];
    let clash = [(InputAction::Playlists, "f")];
    let custom = [(InputAction::NavLeft, "left, q"), (InputAction::NavUp, "up")];
    let vim = [
        (InputAction::NavLeft, "left, h"),
        (InputAction::NavDown, "down, j"),
        (InputAction::NavUp, "up, k"),
        (InputAction::NavRight, "right, l"),
    ];
    let cleared = [(InputAction::Favourite, "none"), (InputAction::Apply, "none")];
    vec![
        key_state("keys-default", &[]),
        key_state("keys-listening", &[]),
        key_state("keys-rising", &[(InputAction::FolderNext, "ctrl+x")]),
        key_state("keys-find", &[]),
        key_state("keys-moved", &cleared),
        key_state("keys-lit", &clash),
        key_state("keys-clash", &clash),
        key_state("keys-custom", &custom),
        key_state("keys-vim", &vim),
        key_state("keys-second", &second),
        key_state("keys-long", &long),
        key_state(MENU_STATES[0], &[]),
        key_state(MENU_STATES[1], &custom),
    ]
}

fn wallpaper_state(
    name: &str,
    cfg: FakeSettingsSource,
    count: usize,
    phase: Option<Phase>,
) -> FitState {
    FitState {
        name: name.to_string(),
        cfg,
        weather: None,
        statuses: outputs(count),
        drawing: phase.map(|phase| pack(phase, PackState::Available)),
        playback: Some(pause_status()),
        connected: false,
        apps: None,
        backends: Vec::new(),
        sddm: Vec::new(),
    }
}

fn wallpaper_states() -> Vec<FitState> {
    use crate::contracts::settings::keys;
    let plain = FakeSettingsSource::default;
    let awww = || plain().with_text(keys::paper::ENGINE, "awww");
    let wipe = || awww().with_text(keys::paper::AWWW_TRANSITION_TYPE, "wipe");
    let sand = || plain().with_text(keys::transition::SHADER, "sand-galaxy");
    let niri = || busy_wallpapers().with_niri(true);
    let crowded = || busy_wallpapers().with_text(keys::paper::ENGINE, "awww").with_niri(true);
    let ready = Some(Phase::Ready);
    let quiet = plain()
        .with_flag(keys::wallpaper::MUTE, true)
        .with_flag(keys::playback::FULLSCREEN, false)
        .with_niri(true)
        .with_flag(keys::niri::OVERVIEW_BACKDROP, false)
        .with_text(keys::plasma::LOCK_SCREEN_MODE, "off");
    let stationary = plain()
        .with_niri(true)
        .with_text(keys::niri::OVERVIEW_MODE, "stationary")
        .with_flag(keys::wallpaper::MUTE, true)
        .with_text(keys::plasma::LOCK_SCREEN_MODE, "static");
    let backdrop_off = plain()
        .with_niri(true)
        .with_flag(keys::niri::OVERVIEW_BACKDROP, false)
        .with_text(keys::plasma::LOCK_SCREEN_MODE, "off");
    let mut missing = wallpaper_state("blocked", plain(), 3, None);
    missing.drawing =
        Some(Status { pack_path: String::new(), ..pack(Phase::Idle, PackState::Missing) });
    let mut absent = wallpaper_state("no-displays", plain(), 0, None);
    absent.connected = true;
    let mut long = wallpaper_state("long-names", plain(), 4, ready);
    long.statuses = long_outputs();
    vec![
        wallpaper_state("default", plain(), 0, None),
        wallpaper_state("busy", busy_wallpapers(), 2, ready),
        wallpaper_state("niri-preparing", niri(), 4, Some(Phase::Preparing)),
        wallpaper_state("awww-paused", awww(), 3, Some(Phase::Paused)),
        wallpaper_state("niri-awww", awww().with_niri(true), 3, ready),
        wallpaper_state("niri-awww-preparing", crowded(), 4, Some(Phase::Preparing)),
        wallpaper_state("niri-sand", sand().with_niri(true), 3, None),
        wallpaper_state(
            "photo-cutout",
            plain().with_text(keys::transition::SHADER, "photo-cutout"),
            3,
            ready,
        ),
        wallpaper_state(
            "transitions-off",
            plain().with_flag(keys::transition::ENABLED, false),
            3,
            ready,
        ),
        wallpaper_state("niri-stationary", stationary, 2, None),
        wallpaper_state("niri-off", backdrop_off, 1, None),
        wallpaper_state("niri-ready", niri(), 3, ready),
        wallpaper_state(
            "niri-backdrop",
            niri().with_flag(keys::niri::OVERVIEW_BACKDROP, true),
            3,
            ready,
        ),
        wallpaper_state("wipe", wipe(), 3, ready),
        wallpaper_state("sand", sand(), 3, ready),
        wallpaper_state("four", plain(), 4, ready),
        wallpaper_state("quiet", quiet, 3, ready),
        wallpaper_state(
            "plasma-static",
            plain().with_text(keys::plasma::LOCK_SCREEN_MODE, "static"),
            3,
            ready,
        ),
        wallpaper_state(
            "plasma-follow",
            plain().with_text(keys::plasma::LOCK_SCREEN_MODE, "follow"),
            3,
            ready,
        ),
        missing,
        absent,
        long,
    ]
}

fn state_named(states: &[FitState], name: &str) -> usize {
    states.iter().position(|state| state.name == name).expect(name)
}

#[test]
#[ignore = "manual offscreen render of the Wallpapers place"]
fn render_wallpapers() {
    use crate::contracts::settings::keys;
    use crate::frontend::settings::category as cat;
    let out = localized();
    let place = crate::frontend::settings::place::WALLPAPERS;
    let states = wallpaper_states();
    let mut heights = Vec::new();
    let mut take = |name: &str, state: &str, open: Option<SectionId>, pop: Option<u64>| {
        let fit = &states[state_named(&states, state)];
        let shot = Shot { place, cfg: &fit.cfg, open, pop, edit: None, weather: None };
        let (view, used) = capture_live(&shot, &fit.live(), Some(&format!("{out}/{name}.png")));
        heights.push(format!("{name}: used {used:.0} of {view:.0}"));
    };
    let section = |category, key| Some(SectionId::new(category, key));
    let play = section(cat::PLAYBACK, "settings-wallpapers-play");
    let motion = section(cat::TRANSITIONS, "settings-section-wallpaper-transitions");
    let drawing = section(cat::TRANSITIONS, DRAWING);
    let displays = section(cat::DISPLAYS, DISPLAYS);
    let lock = section(cat::INTEGRATIONS, "settings-wallpapers-plasma");
    let niri = section(cat::INTEGRATIONS, NIRI);
    let sentence = |path: &str| Some(token_key(path, 0, TokenSide::Sentence));
    let body = |path: &str| Some(token_key(path, 0, TokenSide::Body));
    take("wallpapers", "niri-ready", None, None);
    for name in ["niri-ready", "niri-awww-preparing"] {
        let sections = SourceCtx {
            output_statuses: &states[state_named(&states, name)].statuses,
            drawing: states[state_named(&states, name)].drawing.as_ref(),
            ..source(&states[state_named(&states, name)].cfg, &HashMap::new())
        }
        .place(place)
        .sections;
        for found in sections {
            let prefix = if name == "niri-ready" { "wallpapers" } else { "wallpapers-crowded" };
            take(&format!("{prefix}-{}", found.id.key), name, Some(found.id), None);
        }
    }
    for (name, path) in [
        ("fill", keys::display::FILL_MODE),
        ("backend", keys::paper::ENGINE),
        ("layer", keys::paper::WALLPAPER_LAYER),
        ("family", crate::frontend::settings::SHADER_FAMILY_KEY),
        ("scope", keys::playback::FULLSCREEN_SCOPE),
    ] {
        take(&format!("wallpapers-{name}-menu"), "niri-ready", None, sentence(path));
    }
    take("wallpapers-awww", "wipe", None, None);
    take("wallpapers-awww-open", "wipe", section(cat::PLAYBACK, AWWW), None);
    for (name, path) in
        [("type", keys::paper::AWWW_TRANSITION_TYPE), ("filter", keys::paper::AWWW_FILTER)]
    {
        take(&format!("wallpapers-awww-{name}-menu"), "wipe", None, sentence(path));
    }
    take("wallpapers-sand", "sand", motion, None);
    take("wallpapers-studio", "photo-cutout", motion, None);
    take("wallpapers-trn-off", "transitions-off", None, None);
    take("wallpapers-trn-off-open", "transitions-off", motion, None);
    take("wallpapers-shader-menu", "sand", None, sentence(keys::transition::SHADER));
    take("wallpapers-wheel", "niri-ready", drawing, body(keys::drawing::PAPER_COLOR));
    take("wallpapers-preparing", "niri-preparing", drawing, None);
    take("wallpapers-blocked", "blocked", None, None);
    take("wallpapers-blocked-open", "blocked", drawing, None);
    take(
        "wallpapers-display-more",
        "niri-ready",
        displays,
        Some(crate::frontend::settings::live_key("DP-1")),
    );
    take("wallpapers-placement-menu", "niri-ready", displays, body("display.fillModes.DP-1"));
    for (name, path) in
        [("gpu", keys::performance::GPU_DEVICE), ("scaling", keys::we_render::SCALING)]
    {
        take(&format!("wallpapers-{name}-menu"), "niri-ready", play, body(path));
    }
    for (name, path) in [
        ("quality", keys::transition::SAND_QUALITY),
        ("primary-only", "transition.shaderScopes.sand-bloom"),
    ] {
        take(&format!("wallpapers-{name}-menu"), "sand", motion, body(path));
    }
    take("wallpapers-lock-menu", "niri-ready", None, sentence(keys::plasma::LOCK_SCREEN_MODE));
    take("wallpapers-niri-on", "niri-backdrop", niri, None);
    take("wallpapers-blur-menu", "niri-backdrop", niri, body(keys::niri::BACKDROP_BLUR_STATIC));
    take("wallpapers-dynamic-menu", "plasma-follow", lock, body(keys::plasma::LOCK_SCREEN_DYNAMIC));
    take("wallpapers-detecting-open", "default", displays, None);
    take("wallpapers-no-displays-open", "no-displays", displays, None);
    take("wallpapers-four-open", "four", displays, None);
    take("wallpapers-long-open", "long-names", displays, None);
    take("wallpapers-drawing-open", "niri-ready", drawing, None);
    take("wallpapers-quiet", "quiet", None, None);
    take("wallpapers-plasma-static", "plasma-static", lock, None);
    std::fs::write(format!("{out}/heights.txt"), heights.join("\n")).expect("heights");
}

#[test]
#[ignore = "manual offscreen render of the Keybinds place"]
fn render_keybinds() {
    let out = localized();
    let place = crate::frontend::settings::place::KEYBINDS;
    let states = key_states();
    let mut heights = Vec::new();
    for fit in &states {
        let name = &fit.name;
        let pop = state_pop(name);
        let shot = Shot { place, cfg: &fit.cfg, open: None, pop, edit: None, weather: None };
        let (view, used) = capture_live(&shot, &fit.live(), Some(&format!("{out}/{name}.png")));
        heights.push(format!("{name}: used {used:.0} of {view:.0}"));
    }
    std::fs::write(format!("{out}/heights.txt"), heights.join("\n")).expect("heights");
}

fn layout_cfg(mode: &str) -> FakeSettingsSource {
    use crate::contracts::settings::keys::selector as sel;
    FakeSettingsSource::default().with_text(sel::DISPLAY_MODE, mode)
}

const LAYOUT_NAMES: [&str; 3] = ["Desk", "Laptop", "Streaming"];
const LONG_LAYOUT: &str =
    "The very long saved layout name from the upstairs desk with both displays";

fn layout_state(name: &str, cfg: FakeSettingsSource) -> FitState {
    FitState {
        name: name.to_string(),
        cfg: cfg.with_layouts(&LAYOUT_NAMES, Some(0), 0),
        weather: None,
        statuses: Vec::new(),
        drawing: None,
        playback: None,
        connected: false,
        apps: None,
        backends: Vec::new(),
        sddm: Vec::new(),
    }
}

const CORNERED: [crate::contracts::picker::Mode; 5] = [
    crate::contracts::picker::Mode::Slices,
    crate::contracts::picker::Mode::Hand,
    crate::contracts::picker::Mode::Grid,
    crate::contracts::picker::Mode::Depth,
    crate::contracts::picker::Mode::Collection,
];
const CORNER_STATES: [(skwd_config::corners::CornerStyle, bool); 5] = [
    (skwd_config::corners::CornerStyle::Straight, true),
    (skwd_config::corners::CornerStyle::Round, true),
    (skwd_config::corners::CornerStyle::Round, false),
    (skwd_config::corners::CornerStyle::Chamfered, true),
    (skwd_config::corners::CornerStyle::Chamfered, false),
];

fn layout_states() -> Vec<FitState> {
    use crate::contracts::settings::keys::{launch, selector as sel, video_preview};
    let mut states: Vec<FitState> = crate::contracts::picker::Mode::ALL
        .iter()
        .map(|mode| layout_state(mode.as_key(), layout_cfg(mode.as_key())))
        .collect();
    for mode in CORNERED {
        let keys = mode.corners().expect("corner keys");
        for (style, same) in CORNER_STATES {
            let each = if same { "" } else { "-each" };
            let name = format!("{}-{}{each}", mode.as_key(), style.key());
            let cfg = layout_cfg(mode.as_key())
                .with_text(keys.style, style.key())
                .with_flag(keys.same, same);
            states.push(layout_state(&name, cfg));
        }
    }
    states.push(layout_state("hex-bent", layout_cfg("hex").with_text(sel::HEX_CURVE, "cylinder")));
    states.push(layout_state(
        "wall-cylinder",
        layout_cfg("wall").with_text(sel::GRID_LAYOUT, "cylinder"),
    ));
    states.push(layout_state("still", layout_cfg("slices").with_text(launch::ANIMATION, "none")));
    states.push(layout_state(
        "no-video",
        layout_cfg("slices").with_flag(video_preview::ENABLED, false),
    ));
    let mut named = layout_state("long-name", layout_cfg("sandy"));
    named.cfg = named.cfg.with_layouts(&[LONG_LAYOUT], Some(0), 12);
    states.push(named);
    let mut dirty = layout_state("dirty", layout_cfg("sandy"));
    dirty.cfg = dirty.cfg.with_layouts(&LAYOUT_NAMES, Some(0), 12);
    states.push(dirty);
    let mut lost = layout_state("lost", layout_cfg("slices"));
    lost.cfg = lost.cfg.with_layouts(&LAYOUT_NAMES, None, 0);
    states.push(lost);
    states
}

#[test]
#[ignore = "manual offscreen render of the Layout place"]
fn render_layout() {
    use crate::contracts::settings::keys::{launch, motion, selector as sel, system};
    use crate::frontend::settings::layout_place::{
        ATMOSPHERE, CARDS, CORNERS, FLOW, OPENING, PLACEMENT, RIBBONS, RING, SAND, SHAPE, STYLE,
    };
    use crate::frontend::settings::{LayoutPending, LayoutsUi, PREMADE_KEY, strip_key};
    let out = localized();
    let place = crate::frontend::settings::place::LAYOUT;
    let states = layout_states();
    let mut heights = Vec::new();
    let mut take = |name: &str, state: &str, open: Option<&'static str>, pop: Option<u64>| {
        let fit = &states[state_named(&states, state)];
        let open = open.map(|key| SectionId::new(crate::frontend::settings::category::PICKER, key));
        let shot = Shot { place, cfg: &fit.cfg, open, pop, edit: None, weather: None };
        let (view, used) = capture_live(&shot, &fit.live(), Some(&format!("{out}/{name}.png")));
        heights.push(format!("{name}: used {used:.0} of {view:.0}"));
    };
    for state in ["slices", "hex", "wall", "sandy", "hand", "depth", "collection"] {
        take(&format!("closed-{state}"), state, None, None);
        take(&format!("shape-{state}"), state, Some(SHAPE), None);
    }
    take("style", "slices", Some(STYLE), None);
    take("corners", "slices", Some(CORNERS), None);
    take("corners-same", "slices-round", Some(CORNERS), None);
    take("corners-each", "slices-round-each", Some(CORNERS), None);
    take("corners-cut", "slices-chamfered", Some(CORNERS), None);
    take("corners-cut-each", "slices-chamfered-each", Some(CORNERS), None);
    take("ribbons-cut-each", "hand-chamfered-each", Some(RIBBONS), None);
    take("wall-cut-each", "wall-chamfered-each", Some(SHAPE), None);
    take("ribbons", "hand", Some(RIBBONS), None);
    take("ribbons-straight", "hand-straight", Some(RIBBONS), None);
    take("ribbons-round", "hand-round", Some(RIBBONS), None);
    take("atmosphere", "hand", Some(ATMOSPHERE), None);
    take("sand", "sandy", Some(SAND), None);
    take("flow", "sandy", Some(FLOW), None);
    take("ring", "sandy", Some(RING), None);
    take("hex-bent", "hex-bent", Some(SHAPE), None);
    take("wall-cylinder", "wall-cylinder", Some(SHAPE), None);
    for state in ["slices", "sandy", "depth"] {
        take(&format!("placement-{state}"), state, Some(PLACEMENT), None);
    }
    take("cards", "slices", Some(CARDS), None);
    take("cards-no-video", "no-video", Some(CARDS), None);
    take("opening", "slices", Some(OPENING), None);
    take("opening-still", "still", Some(OPENING), None);
    take("dirty", "dirty", None, None);
    take("long-name", "long-name", None, None);
    let sentence = |path: &str| Some(token_key(path, 0, TokenSide::Sentence));
    for (name, path) in [
        ("mode", sel::DISPLAY_MODE),
        ("premade", PREMADE_KEY),
        ("effect", sel::FLIP_EFFECT),
        ("filter", motion::FILTER_SWAP_SPEED),
        ("monitor", system::MONITOR),
        ("start", sel::START_POSITION),
        ("anim", launch::ANIMATION),
        ("pace", motion::LAUNCH_SPEED),
    ] {
        take(&format!("menu-{name}"), "slices", None, sentence(path));
    }
    take("menu-corner", "slices", None, sentence(skwd_config::corners::SLICES.style));
    take("menu-strip", "slices", None, Some(strip_key()));
    take("menu-strip-dirty", "dirty", None, Some(strip_key()));
    let ui = LayoutsUi { confirm: Some(LayoutPending::Switch(1)), ..LayoutsUi::default() };
    STRIP_UI.with(|cell| cell.set(ui));
    take("menu-confirm", "dirty", None, Some(strip_key()));
    STRIP_UI.with(|cell| cell.set(LayoutsUi { armed: true, ..LayoutsUi::default() }));
    take("menu-armed", "slices", None, Some(strip_key()));
    STRIP_UI.with(|cell| cell.set(LayoutsUi { rejected: true, ..LayoutsUi::default() }));
    take("rejected", "dirty", None, None);
    STRIP_UI.with(|cell| cell.set(LayoutsUi::default()));
    take("lost-menu", "lost", None, Some(strip_key()));
    STRIP_UI.with(|cell| cell.set(LayoutsUi { renaming: true, ..LayoutsUi::default() }));
    take("renaming", "slices", None, None);
    STRIP_UI.with(|cell| cell.set(LayoutsUi::default()));
    std::fs::write(format!("{out}/heights.txt"), heights.join("\n")).expect("heights");
}

fn colour_app(id: &str, enabled: bool, state: &str) -> AppThemeStatus {
    AppThemeStatus {
        id: id.to_string(),
        name: id.to_string(),
        installed: state != "not-installed",
        config_found: true,
        config_path: format!("~/.config/{id}/{id}.conf"),
        output_path: format!("~/.cache/skwd/{id}.conf"),
        enabled,
        state: state.to_string(),
        detail: String::new(),
        can_enable: state != "not-installed",
        can_disable: enabled,
        can_adopt: false,
        template_path: format!("~/.config/skwd/templates/{id}.tmpl"),
        customized: false,
        can_disconnect: enabled,
        can_reconnect: state == "changed",
    }
}

fn every_app() -> AppThemesResult {
    AppThemesResult {
        apps: vec![
            colour_app("btop", true, "changed"),
            colour_app("fish", true, "configured"),
            colour_app("foot", true, "watching"),
            colour_app("ghostty", false, "ready"),
            colour_app("kde", false, "needs-review"),
            colour_app("kitty", true, "reload-sent"),
            colour_app("niri", false, "conflict"),
            colour_app("nvim", true, "interrupted"),
            colour_app("rofi", false, "not-installed"),
            colour_app("tmux", true, "busy"),
        ],
    }
}

fn board_apps() -> AppThemesResult {
    AppThemesResult {
        apps: vec![
            colour_app("Kitty", true, "configured"),
            colour_app("btop", true, "changed"),
            colour_app("Rofi", false, "ready"),
        ],
    }
}

const BOARD_PALETTES: [(&str, &str, [&str; 4]); 6] = [
    ("catppuccin", "Catppuccin", ["#cba6f7", "#89b4fa", "#1e1e2e", "#cdd6f4"]),
    ("dracula", "Dracula", ["#bd93f9", "#ff79c6", "#282a36", "#f8f8f2"]),
    ("nord", "Nord", ["#88c0d0", "#b48ead", "#2e3440", "#eceff4"]),
    ("gruvbox", "Gruvbox", ["#fabd2f", "#83a598", "#282828", "#ebdbb2"]),
    ("everforest", "Everforest", ["#a7c080", "#d699b6", "#2d353b", "#d3c6aa"]),
    ("rose-pine", "Rose Pine", ["#ebbcba", "#c4a7e7", "#191724", "#e0def4"]),
];

fn colour_state(name: &str, cfg: FakeSettingsSource) -> FitState {
    FitState {
        name: name.to_string(),
        cfg,
        weather: None,
        statuses: Vec::new(),
        drawing: None,
        playback: None,
        connected: true,
        apps: Some(board_apps()),
        backends: ["matugen", "wallust", "pywal", "iris", "noctalia", "dms", "caelestia", "end4"]
            .map(str::to_string)
            .to_vec(),
        sddm: Vec::new(),
    }
}

fn colour_states() -> Vec<FitState> {
    use crate::contracts::settings::keys;
    let on = || {
        FakeSettingsSource::default()
            .with_text(keys::theme::POLICY, "wallpaper")
            .with_text(keys::theme::AUTHORITY, "skwd")
            .with_text(keys::lock_screens::MODE, keys::lock_screens::FOLLOW)
    };
    let engine = |name: &str| on().with_text(keys::theme::ENGINE, name);
    let filled = || {
        engine("matugen")
            .with_text(keys::matugen::DEFAULT_CONFIG, "~/.config/matugen/config.toml")
            .with_text(keys::system::EXTERNAL_MATUGEN_COMMAND, "matugen -c %config% image %path%")
            .with_flag(keys::lock_screens::HYPRLOCK, true)
            .with_flag(keys::lock_screens::SWAYLOCK, true)
            .with_flag(keys::noctalia::HOVER_PREVIEW, true)
    };
    let one = || {
        filled()
            .with_array_len(keys::integrations::LIST, 1)
            .with_text("integrations.0.name", "waybar")
            .with_text("integrations.0.template", "~/.config/matugen/waybar.css")
            .with_text("integrations.0.output", "~/.config/waybar/colors.css")
            .with_text("integrations.0.reload", "pkill -SIGUSR2 waybar")
    };
    let two = || {
        one()
            .with_array_len(keys::integrations::LIST, 2)
            .with_text("integrations.1.name", "kitty")
            .with_text("integrations.1.template", "~/.config/matugen/kitty.conf")
            .with_text("integrations.1.output", "~/.config/kitty/colors.conf")
    };
    let three = || {
        two()
            .with_array_len(keys::integrations::LIST, 3)
            .with_text("integrations.2.name", "btop")
            .with_text("integrations.2.template", "~/.config/matugen/btop.theme")
            .with_text("integrations.2.output", "~/.config/btop/themes/skwd.theme")
    };
    let shell = |name: &str| on().with_text(keys::theme::AUTHORITY, name);
    let fixed = || {
        on().with_text(keys::theme::POLICY, "fixed")
            .with_text(keys::theme::STATIC_THEME, "custom")
            .with_text(keys::theme::CUSTOM_COLORS, "#1e1e2e, #89b4fa, #f5c2e7")
            .with_saved_theme("Mine")
            .with_palettes(&BOARD_PALETTES)
    };
    let still = || {
        filled()
            .with_text(keys::lock_screens::MODE, keys::lock_screens::STATIC)
            .with_text(keys::lock_screens::IMAGE, "~/Pictures/lockscreen.jpg")
            .with_text(keys::lock_screens::SDDM_IMAGE, "/var/lib/skwd/sddm.jpg")
    };
    let board = colour_state("board", engine("matugen"));
    let mut crowded = colour_state("every-app", engine("matugen"));
    crowded.apps = Some(every_app());
    let mut loading = colour_state("loading", on());
    loading.apps = None;
    loading.connected = false;
    let mut chooser = colour_state("sddm-chooser", still());
    chooser.sddm = vec![String::from("breeze"), String::from("sugar-candy")];
    let mut lonely = colour_state("no-apps", on());
    lonely.apps = Some(AppThemesResult { apps: Vec::new() });
    vec![
        board,
        crowded,
        colour_state("iris", on()),
        colour_state("matugen", filled()),
        colour_state(
            "wallust",
            engine("wallust").with_text(keys::theme::WALLUST_PALETTE, "dark16"),
        ),
        colour_state("native", engine("native")),
        colour_state("fixed", fixed()),
        colour_state("keep", on().with_text(keys::theme::POLICY, "off")),
        colour_state("noctalia", shell("noctalia")),
        colour_state("dms", shell("dms")),
        colour_state("caelestia", shell("caelestia")),
        colour_state("end4", shell("end4")),
        colour_state("one-output", one()),
        colour_state("two-outputs", two()),
        colour_state("three-outputs", three()),
        colour_state("no-outputs", filled().with_array_len(keys::integrations::LIST, 0)),
        colour_state("static-lock", still()),
        chooser,
        loading,
        lonely,
    ]
}

#[test]
#[ignore = "manual offscreen render of the Colours place"]
fn render_colours() {
    use crate::contracts::settings::keys;
    use crate::frontend::settings::category as cat;
    let out = localized();
    let place = crate::frontend::settings::place::COLOURS;
    let states = colour_states();
    let mut heights = Vec::new();
    let mut take = |name: &str, state: &str, open: Option<SectionId>, pop: Option<u64>| {
        let fit = &states[state_named(&states, state)];
        let shot = Shot { place, cfg: &fit.cfg, open, pop, edit: None, weather: None };
        let (view, used) = capture_live(&shot, &fit.live(), Some(&format!("{out}/{name}.png")));
        heights.push(format!("{name}: used {used:.0} of {view:.0}"));
    };
    let section = |category, key| Some(SectionId::new(category, key));
    let generator = section(cat::THEME, "settings-colours-generator");
    let look = section(cat::THEME, "settings-colours-look");
    let go = section(cat::THEME, "settings-colours-go");
    let custom = section(cat::THEME, "settings-app-themes-custom");
    let shells = section(cat::INTEGRATIONS, "settings-colours-shells");
    let lock = section(cat::INTEGRATIONS, "settings-colours-lock");
    let sentence = |path: &str| Some(token_key(path, 0, TokenSide::Sentence));
    let body = |path: &str| Some(token_key(path, 0, TokenSide::Body));
    take("colours", "board", None, None);
    for (name, state, open) in [
        ("generator", "board", generator),
        ("look", "board", look),
        ("go", "board", go),
        ("go-all", "every-app", go),
        ("custom", "one-output", custom),
        ("custom-two", "two-outputs", custom),
        ("custom-three", "three-outputs", custom),
        ("custom-none", "no-outputs", custom),
        ("shells", "matugen", shells),
        ("lock", "matugen", lock),
        ("lock-static", "static-lock", lock),
        ("lock-chooser", "sddm-chooser", lock),
        ("generator-wallust", "wallust", generator),
        ("generator-fixed", "fixed", generator),
        ("generator-keep", "keep", generator),
        ("generator-shell", "noctalia", generator),
        ("generator-native", "native", generator),
        ("look-fixed", "fixed", look),
        ("look-keep", "keep", look),
        ("look-shell", "noctalia", look),
        ("look-caelestia", "caelestia", look),
        ("go-loading", "loading", go),
        ("go-empty", "no-apps", go),
    ] {
        take(&format!("colours-{name}"), state, open, None);
    }
    for (name, state) in [
        ("fixed", "fixed"),
        ("keep", "keep"),
        ("noctalia", "noctalia"),
        ("wallust", "wallust"),
        ("end4", "end4"),
        ("loading", "loading"),
    ] {
        take(&format!("colours-state-{name}"), state, None, None);
    }
    for (name, state, path) in [
        ("policy", "board", keys::theme::POLICY),
        ("authority", "board", keys::theme::AUTHORITY),
        ("engine", "board", keys::theme::ENGINE),
        ("index", "board", keys::matugen::COLOR_INDEX),
        ("scheme", "board", keys::matugen::SCHEME_TYPE),
        ("finish", "board", keys::theme::STYLE),
        ("finish-iris", "iris", keys::theme::STYLE),
        ("variant", "board", keys::theme::MODE),
        ("imported", "noctalia", keys::theme::MODE),
        ("theme", "fixed", keys::theme::STATIC_THEME),
        ("lock-source", "matugen", keys::lock_screens::MODE),
    ] {
        take(&format!("colours-{name}-menu"), state, None, sentence(path));
    }
    let every = Some(token_key(keys::theme::STATIC_THEME, 1, TokenSide::Sentence));
    take("colours-theme-all-menu", "fixed", None, every);
    take("colours-index-body-menu", "board", generator, body(keys::matugen::COLOR_INDEX));
    take("colours-mode-menu", "matugen", shells, body(keys::noctalia::THEME_MODE));
    take("colours-profile-output-menu", "one-output", custom, body("integrations.0.profile"));
    let details = |id: &str| Some(token_key(&format!("app-theme:{id}"), 0, TokenSide::Body));
    take("colours-app-details", "board", go, details("btop"));
    take("colours-app-details-kitty", "board", go, details("Kitty"));
    take("colours-app-profile", "board", go, body("theme.appProfiles.btop"));
    take("colours-sddm-menu", "sddm-chooser", lock, body("settings-lockscreens-choose-theme"));
    std::fs::write(format!("{out}/heights.txt"), heights.join("\n")).expect("heights");
}

fn fit_states(place: &str) -> Vec<FitState> {
    if place == crate::frontend::settings::place::WALLPAPERS {
        return wallpaper_states();
    }
    if place == crate::frontend::settings::place::LIBRARY {
        return library::states();
    }
    if place == crate::frontend::settings::place::HOME {
        return home::states();
    }
    if place == crate::frontend::settings::place::KEYBINDS {
        return key_states();
    }
    if place == crate::frontend::settings::place::LAYOUT {
        return layout_states();
    }
    if place == crate::frontend::settings::place::COLOURS {
        return colour_states();
    }
    plain_states(place)
        .into_iter()
        .enumerate()
        .map(|(at, (cfg, weather))| FitState {
            name: format!("state {at}"),
            cfg,
            weather,
            statuses: Vec::new(),
            drawing: None,
            playback: None,
            connected: false,
            apps: None,
            backends: Vec::new(),
            sddm: Vec::new(),
        })
        .collect()
}

fn plain_states(place: &str) -> Vec<(FakeSettingsSource, Option<WeatherStatus>)> {
    use crate::contracts::settings::keys;
    use crate::frontend::settings::place::{AUTOMATION, BAR, SOURCES, SYSTEM};
    use crate::frontend::settings::test_source::weather_status;
    match place {
        SYSTEM => vec![(system(true), None), (system(false), None)],
        BAR => vec![
            (FakeSettingsSource::default(), None),
            (busy_bar(), None),
            (quiet_bar(), None),
            (long_model_bar(), None),
            (floats_off_bar(), None),
        ],
        SOURCES => vec![
            (FakeSettingsSource::default(), None),
            (busy_sources(), None),
            (sources(false), None),
            (
                sources(true)
                    .with_text(keys::wallhaven::DEFAULT_SORT, "hot")
                    .with_text(keys::steam::DEFAULT_SORT, "1"),
                None,
            ),
        ],
        AUTOMATION => {
            let quiet = FakeSettingsSource::default()
                .with_flag(keys::general::RANDOM_ROTATE, false)
                .with_flag(keys::schedule::ENABLED, false);
            let mut states = vec![(automation(FIT_COMMANDS), None), (quiet, None)];
            for (source, enabled, status) in [
                (keys::weather::SOURCE_PLAYLISTS, false, None),
                (keys::weather::SOURCE_PLAYLISTS, true, Some(WeatherSource::Playlists)),
                (keys::weather::SOURCE_SEMANTIC, true, Some(WeatherSource::Semantic)),
                (keys::weather::SOURCE_OUTDOORS, true, Some(WeatherSource::Outdoors)),
            ] {
                let cfg = automation(2)
                    .with_flag(keys::weather::ENABLED, enabled)
                    .with_text(keys::weather::SOURCE, source);
                states.push((cfg, status.map(weather_status)));
            }
            states
        }
        _ => Vec::new(),
    }
}

use crate::frontend::settings::layout_place;

const WALLPAPERS: &str = crate::frontend::settings::place::WALLPAPERS;
const BAR: &str = crate::frontend::settings::place::BAR;
const SEARCH_CARD: &str = "settings-tagging-search-card";
const DISPLAYS: &str = "settings-tab-displays";
const AWWW: &str = "settings-section-awww-transition";
const DRAWING: &str = "settings-wallpapers-drawing";
const PAUSE: &str = "settings-playback-pause-title";
const NIRI: &str = "settings-wallpapers-niri";

const COLOURS: &str = crate::frontend::settings::place::COLOURS;
const COLOUR_APPS: &str = "settings-colours-go";
const COLOUR_OUTPUTS: &str = "settings-app-themes-custom";

const LIBRARY: &str = crate::frontend::settings::place::LIBRARY;
const LIBRARY_FOLDERS: &str = "settings-section-folders";
const KEYBINDS: &str = crate::frontend::settings::place::KEYBINDS;
const KEYS_CLOSED: &str = "closed";
const KEYS_SHEET_ID: SectionId = crate::frontend::settings::KEYS_SHEET;
const KEYS_SHEET: &str = "settings-section-controls";
const LIBRARY_PROFILES: &str = "settings-profiles-card";

const LAYOUT: &str = crate::frontend::settings::place::LAYOUT;
const HOME: &str = crate::frontend::settings::place::HOME;
const CHANGED: &str = "settings-home-changed";
const SCROLLS: [Scroll; 83] = [
    Scroll { place: KEYBINDS, state: "keys-long", section: KEYS_CLOSED, most: 41.0 },
    Scroll { place: KEYBINDS, state: "keys-long", section: KEYS_SHEET, most: 41.0 },
    Scroll { place: HOME, state: "home-board", section: CHANGED, most: 56.0 },
    Scroll { place: HOME, state: "home-armed", section: CHANGED, most: 56.0 },
    Scroll { place: HOME, state: "home-watch-down", section: CHANGED, most: 56.0 },
    Scroll { place: HOME, state: "home-eight", section: CHANGED, most: 94.0 },
    Scroll { place: HOME, state: "home-many", section: CHANGED, most: 646.0 },
    Scroll { place: COLOURS, state: "every-app", section: COLOUR_APPS, most: 53.0 },
    Scroll { place: COLOURS, state: "two-outputs", section: COLOUR_OUTPUTS, most: 226.0 },
    Scroll { place: COLOURS, state: "three-outputs", section: COLOUR_OUTPUTS, most: 462.0 },
    Scroll { place: BAR, state: "state 1", section: SEARCH_CARD, most: 32.0 },
    Scroll { place: BAR, state: "state 3", section: SEARCH_CARD, most: 74.0 },
    Scroll { place: BAR, state: "state 4", section: SEARCH_CARD, most: 32.0 },
    Scroll { place: LAYOUT, state: "dirty", section: layout_place::PLACEMENT, most: 63.0 },
    Scroll { place: LAYOUT, state: "dirty", section: layout_place::STYLE, most: 13.0 },
    Scroll { place: LAYOUT, state: "hand", section: layout_place::PLACEMENT, most: 16.0 },
    Scroll { place: LAYOUT, state: "hand-straight", section: layout_place::PLACEMENT, most: 16.0 },
    Scroll { place: LAYOUT, state: "hand-round", section: layout_place::PLACEMENT, most: 16.0 },
    Scroll {
        place: LAYOUT,
        state: "hand-round-each",
        section: layout_place::PLACEMENT,
        most: 16.0,
    },
    Scroll { place: LAYOUT, state: "hand-chamfered", section: layout_place::PLACEMENT, most: 16.0 },
    Scroll {
        place: LAYOUT,
        state: "hand-chamfered-each",
        section: layout_place::PLACEMENT,
        most: 16.0,
    },
    Scroll { place: LAYOUT, state: "long-name", section: layout_place::CARDS, most: 15.0 },
    Scroll { place: LAYOUT, state: "long-name", section: layout_place::PLACEMENT, most: 75.0 },
    Scroll { place: LAYOUT, state: "long-name", section: layout_place::SHAPE, most: 5.0 },
    Scroll { place: LAYOUT, state: "long-name", section: layout_place::STYLE, most: 24.0 },
    Scroll { place: LAYOUT, state: "sandy", section: layout_place::PLACEMENT, most: 63.0 },
    Scroll { place: LAYOUT, state: "sandy", section: layout_place::STYLE, most: 13.0 },
    Scroll { place: WALLPAPERS, state: "long-names", section: DISPLAYS, most: 21.0 },
    Scroll { place: WALLPAPERS, state: "awww-paused", section: DRAWING, most: 152.0 },
    Scroll { place: LIBRARY, state: "armed", section: LIBRARY_PROFILES, most: 76.0 },
    Scroll { place: LIBRARY, state: "filtered", section: LIBRARY_PROFILES, most: 43.0 },
    Scroll { place: LIBRARY, state: "no-match", section: LIBRARY_PROFILES, most: 43.0 },
    Scroll { place: LIBRARY, state: "armed", section: LIBRARY_FOLDERS, most: 41.0 },
    Scroll { place: LIBRARY, state: "everything", section: LIBRARY_PROFILES, most: 58.0 },
    Scroll { place: LIBRARY, state: "everything", section: LIBRARY_FOLDERS, most: 41.0 },
    Scroll { place: LIBRARY, state: "filtered", section: LIBRARY_FOLDERS, most: 41.0 },
    Scroll { place: LIBRARY, state: "four-unread", section: LIBRARY_PROFILES, most: 58.0 },
    Scroll { place: LIBRARY, state: "four-unread", section: LIBRARY_FOLDERS, most: 41.0 },
    Scroll { place: LIBRARY, state: "long-names", section: LIBRARY_PROFILES, most: 58.0 },
    Scroll { place: LIBRARY, state: "long-names", section: LIBRARY_FOLDERS, most: 62.0 },
    Scroll { place: LIBRARY, state: "no-match", section: LIBRARY_FOLDERS, most: 41.0 },
    Scroll { place: LIBRARY, state: "stocked", section: LIBRARY_PROFILES, most: 49.0 },
    Scroll { place: LIBRARY, state: "stocked", section: LIBRARY_FOLDERS, most: 41.0 },
    Scroll { place: LIBRARY, state: "trash-on", section: LIBRARY_PROFILES, most: 58.0 },
    Scroll { place: LIBRARY, state: "trash-on", section: LIBRARY_FOLDERS, most: 32.0 },
    Scroll { place: LIBRARY, state: "undo-draft", section: LIBRARY_PROFILES, most: 49.0 },
    Scroll { place: LIBRARY, state: "undo-draft", section: LIBRARY_FOLDERS, most: 41.0 },
    Scroll { place: LIBRARY, state: "undo-excluded", section: LIBRARY_PROFILES, most: 49.0 },
    Scroll { place: LIBRARY, state: "undo-excluded", section: LIBRARY_FOLDERS, most: 41.0 },
    Scroll { place: LIBRARY, state: "watch-down", section: LIBRARY_PROFILES, most: 46.0 },
    Scroll { place: LIBRARY, state: "watch-down", section: LIBRARY_FOLDERS, most: 20.0 },
    Scroll { place: WALLPAPERS, state: "niri-awww", section: PAUSE, most: 57.0 },
    Scroll { place: WALLPAPERS, state: "niri-awww", section: DISPLAYS, most: 25.0 },
    Scroll { place: WALLPAPERS, state: "niri-awww", section: DRAWING, most: 128.0 },
    Scroll { place: WALLPAPERS, state: "niri-awww", section: NIRI, most: 41.0 },
    Scroll { place: WALLPAPERS, state: "niri-awww-preparing", section: PAUSE, most: 43.0 },
    Scroll { place: WALLPAPERS, state: "niri-awww-preparing", section: DISPLAYS, most: 82.0 },
    Scroll { place: WALLPAPERS, state: "niri-awww-preparing", section: DRAWING, most: 208.0 },
    Scroll { place: WALLPAPERS, state: "niri-awww-preparing", section: NIRI, most: 41.0 },
    Scroll { place: WALLPAPERS, state: "niri-off", section: PAUSE, most: 10.0 },
    Scroll { place: WALLPAPERS, state: "niri-preparing", section: DISPLAYS, most: 35.0 },
    Scroll { place: WALLPAPERS, state: "niri-preparing", section: DRAWING, most: 161.0 },
    Scroll { place: WALLPAPERS, state: "niri-sand", section: PAUSE, most: 10.0 },
    Scroll { place: WALLPAPERS, state: "niri-stationary", section: PAUSE, most: 10.0 },
    Scroll { place: WALLPAPERS, state: "quiet", section: PAUSE, most: 10.0 },
    Scroll { place: WALLPAPERS, state: "blocked", section: DRAWING, most: 25.0 },
    Scroll { place: WALLPAPERS, state: "busy", section: DRAWING, most: 34.0 },
    Scroll { place: WALLPAPERS, state: "default", section: DRAWING, most: 9.0 },
    Scroll { place: WALLPAPERS, state: "four", section: DRAWING, most: 34.0 },
    Scroll { place: WALLPAPERS, state: "long-names", section: DRAWING, most: 34.0 },
    Scroll { place: WALLPAPERS, state: "niri-backdrop", section: DRAWING, most: 81.0 },
    Scroll { place: WALLPAPERS, state: "niri-off", section: DRAWING, most: 56.0 },
    Scroll { place: WALLPAPERS, state: "niri-ready", section: DRAWING, most: 81.0 },
    Scroll { place: WALLPAPERS, state: "niri-sand", section: DRAWING, most: 56.0 },
    Scroll { place: WALLPAPERS, state: "niri-stationary", section: DRAWING, most: 56.0 },
    Scroll { place: WALLPAPERS, state: "no-displays", section: DRAWING, most: 9.0 },
    Scroll { place: WALLPAPERS, state: "photo-cutout", section: DRAWING, most: 34.0 },
    Scroll { place: WALLPAPERS, state: "plasma-follow", section: DRAWING, most: 34.0 },
    Scroll { place: WALLPAPERS, state: "plasma-static", section: DRAWING, most: 34.0 },
    Scroll { place: WALLPAPERS, state: "quiet", section: DRAWING, most: 81.0 },
    Scroll { place: WALLPAPERS, state: "sand", section: DRAWING, most: 34.0 },
    Scroll { place: WALLPAPERS, state: "transitions-off", section: DRAWING, most: 34.0 },
    Scroll { place: WALLPAPERS, state: "wipe", section: DRAWING, most: 81.0 },
];

struct Scroll {
    place: &'static str,
    state: &'static str,
    section: &'static str,
    most: f32,
}

struct Railed {
    tag: &'static str,
    place: &'static str,
    state: String,
    section: &'static str,
    hidden: f32,
    spare: f32,
}

struct Over {
    tag: &'static str,
    place: &'static str,
    state: String,
    section: &'static str,
    by: f32,
}

#[test]
#[ignore = "offscreen layout probe of every composed place in every locale"]
fn places_fit_every_locale() {
    let _renderer = crate::frontend::headless::share(UI_FONT);
    let mut worst = Vec::new();
    let mut over = Vec::new();
    let mut railed = Vec::new();
    let mut table: std::collections::BTreeMap<String, f32> = std::collections::BTreeMap::new();
    let mut ran = 0;
    let only = std::env::var(LANG).ok();
    for language in crate::i18n::LANGUAGES
        .iter()
        .filter(|language| only.as_deref().is_none_or(|tag| tag == language.tag))
    {
        set_language(language.tag);
        iced_wgpu::graphics::text::font_system()
            .write()
            .expect("font system")
            .load_font(crate::shell::ui_font_bytes(language.script).into());
        let only_place = std::env::var(ONLY_PLACE).ok();
        for place in crate::frontend::settings::places::composed_places()
            .filter(|place| only_place.as_deref().is_none_or(|only| only == *place))
        {
            let states = fit_states(place);
            assert!(!states.is_empty(), "{place} has no fit states");
            let mut most = (0.0_f32, String::new());
            for fit in &states {
                let empty = HashMap::new();
                let live = fit.live();
                let ctx = SourceCtx {
                    weather: fit.weather.as_ref(),
                    ..live_source(&fit.cfg, &empty, &live)
                };
                let sections = ctx.place(place).sections;
                let composed = sections.iter().filter(|section| !INTERIM.contains(&section.id.key));
                let opens = std::iter::once(None).chain(composed.map(|section| Some(section.id)));
                for open in opens {
                    let shot = Shot {
                        place,
                        cfg: &fit.cfg,
                        open,
                        pop: state_pop(&fit.name),
                        edit: None,
                        weather: fit.weather.as_ref(),
                    };
                    let Measure { view, used, hidden, .. } =
                        capture_in(SCREEN, &shot, &live, None, &mut []);
                    let key = open.map_or("closed", |id| id.key);
                    if hidden > 0.0 {
                        railed.push(Railed {
                            tag: language.tag,
                            place,
                            state: fit.name.clone(),
                            section: key,
                            hidden,
                            spare: view - FIT_MARGIN - used,
                        });
                    }
                    let name = format!("{} open {key}", fit.name);
                    let slot = table.entry(format!("{place} {name}")).or_default();
                    *slot = slot.max(used);
                    if used > view - FIT_MARGIN && FITTED.contains(&place) {
                        over.push(Over {
                            tag: language.tag,
                            place,
                            state: fit.name.clone(),
                            section: key,
                            by: (used - (view - FIT_MARGIN)).ceil(),
                        });
                    }
                    if used > most.0 {
                        most = (used, name);
                    }
                }
            }
            worst.push(format!("{} {place}: {:.0} ({})", language.tag, most.0, most.1));
            ran += 1;
        }
    }
    assert!(ran > 0, "no place ran");
    set_language("en-US");
    let reach = |scroll: &Scroll, tag: Option<&str>| {
        over.iter()
            .filter(|found| {
                found.place == scroll.place
                    && found.state == scroll.state
                    && found.section == scroll.section
                    && tag.is_none_or(|tag| found.tag == tag)
            })
            .map(|found| found.by)
            .fold(0.0_f32, f32::max)
    };
    if let Ok(out) = std::env::var(OUT) {
        std::fs::write(format!("{out}/fit.txt"), worst.join("\n")).expect("fit report");
        let lines: Vec<_> = SCROLLS
            .iter()
            .map(|scroll| {
                format!(
                    "{} {} {}: worst {} en-US {}",
                    scroll.place,
                    scroll.state,
                    scroll.section,
                    reach(scroll, None),
                    reach(scroll, Some("en-US")),
                )
            })
            .collect();
        std::fs::write(format!("{out}/over.txt"), lines.join("\n")).expect("over report");
        let mut worst_by: std::collections::BTreeMap<String, f32> =
            std::collections::BTreeMap::new();
        for found in &over {
            let slot = worst_by
                .entry(format!("{} {} {}", found.place, found.state, found.section))
                .or_default();
            *slot = slot.max(found.by);
        }
        let rows: Vec<_> = worst_by.iter().map(|(name, by)| format!("{name}: {by}")).collect();
        std::fs::write(format!("{out}/scrolls.txt"), rows.join("\n")).expect("scrolls report");
        let rows: Vec<_> = railed
            .iter()
            .map(|found| {
                format!(
                    "{} {} {} {}: {} spare {}",
                    found.tag, found.place, found.state, found.section, found.hidden, found.spare
                )
            })
            .collect();
        std::fs::write(format!("{out}/rails.txt"), rows.join("\n")).expect("rails report");
        let rows: Vec<_> = table.iter().map(|(name, used)| format!("{name}: {used:.0}")).collect();
        std::fs::write(format!("{out}/table.txt"), rows.join("\n")).expect("table report");
    }
    let unlisted: Vec<_> = over
        .iter()
        .filter(|found| {
            !SCROLLS.iter().any(|scroll| {
                scroll.place == found.place
                    && scroll.state == found.state
                    && scroll.section == found.section
            })
        })
        .map(|found| {
            format!("{} {} {} {}: {}", found.tag, found.place, found.state, found.section, found.by)
        })
        .collect();
    assert!(unlisted.is_empty(), "{unlisted:#?}");
    let listed = |found: &Railed| {
        SCROLLS.iter().any(|scroll| {
            scroll.place == found.place
                && scroll.state == found.state
                && scroll.section == found.section
        })
    };
    let bare: Vec<_> = railed
        .iter()
        .filter(|found| !listed(found))
        .map(|found| {
            format!(
                "{} {} {} {}: {}",
                found.tag, found.place, found.state, found.section, found.hidden
            )
        })
        .collect();
    assert!(bare.is_empty(), "{bare:#?}");
    if only.is_none() && std::env::var(ONLY_PLACE).is_err() {
        let inexact: Vec<_> = SCROLLS
            .iter()
            .filter(|scroll| reach(scroll, None) != scroll.most)
            .map(|scroll| (scroll.state, scroll.section, scroll.most, reach(scroll, None)))
            .collect();
        assert!(inexact.is_empty(), "{inexact:#?}");
    }
}

#[test]
#[ignore = "offscreen layout probe of the snug trim"]
fn snug_sections_need_trim() {
    use crate::frontend::settings::Trim;
    let _renderer = crate::frontend::headless::share(UI_FONT);
    let mut needed: std::collections::BTreeMap<(&str, &str), bool> =
        std::collections::BTreeMap::new();
    UNTRIMMED.set(true);
    for language in crate::i18n::LANGUAGES {
        set_language(language.tag);
        iced_wgpu::graphics::text::font_system()
            .write()
            .expect("font system")
            .load_font(crate::shell::ui_font_bytes(language.script).into());
        for place in crate::frontend::settings::places::composed_places()
            .filter(|place| FITTED.contains(place))
        {
            for fit in fit_states(place) {
                let empty = HashMap::new();
                let live = fit.live();
                let ctx = SourceCtx {
                    weather: fit.weather.as_ref(),
                    ..live_source(&fit.cfg, &empty, &live)
                };
                for section in
                    ctx.place(place).sections.iter().filter(|section| section.trim != Trim::None)
                {
                    let shot = Shot {
                        place,
                        cfg: &fit.cfg,
                        open: Some(section.id),
                        pop: state_pop(&fit.name),
                        edit: None,
                        weather: fit.weather.as_ref(),
                    };
                    let (view, used) = capture_live(&shot, &live, None);
                    *needed.entry((place, section.id.key)).or_default() |= used > view - FIT_MARGIN;
                }
            }
        }
    }
    UNTRIMMED.set(false);
    set_language("en-US");
    let idle: Vec<_> = needed.iter().filter(|(_, need)| !**need).map(|(key, _)| key).collect();
    assert!(idle.is_empty(), "snug without need: {idle:#?}");
}

struct Ids(Vec<String>);

impl Operation for Ids {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn container(&mut self, id: Option<&iced::widget::Id>, _bounds: Rectangle) {
        self.0.extend(id.map(|id| format!("{id:?}")));
    }
}

#[test]
#[ignore = "offscreen layout probe"]
fn extent_counts_hidden_once() {
    use iced::widget::{Space, column, container, scrollable};
    let reading = crate::frontend::settings::settings_reading_id();
    let body = iced::widget::Id::new("probe-body");
    let last = iced::widget::Id::new("probe-last");
    let nested: iced::widget::Scrollable<'_, crate::app::Message> =
        scrollable(Space::new().height(200.0)).height(50.0);
    let rows = column![Space::new().height(100.0), nested, Space::new().height(100.0)];
    let capped = scrollable(rows).id(body.clone()).height(120.0);
    let page = scrollable(container(capped).id(last.clone())).id(reading.clone());
    let mut extent = Extent::new(reading, Some(last), vec![body]);
    crate::frontend::headless::operate(SCREEN, UI_FONT, page.into(), &mut extent);
    assert_eq!(extent.used(), 250.0);
    assert_eq!(extent.heights, [120.0]);
}

#[test]
fn reveal_offsets() {
    assert_eq!(super::reveal_offset(300.0, 100.0, 0.0, 150.0), Some(200.0));
    assert_eq!(super::reveal_offset(300.0, 100.0, 100.0, 300.0), None);
    assert_eq!(super::reveal_offset(300.0, 100.0, 250.0, 300.0), Some(200.0));
}

#[test]
fn row_ids_unique() {
    let (one, two) = (SectionId::new("a", "x"), SectionId::new("a", "y"));
    let owners = [super::section_identity(one), super::section_identity(two)];
    let mut seen = HashSet::new();
    for owner in owners {
        for flat in 0..4 {
            assert!(seen.insert(format!("{:?}", super::row_widget_id(owner, flat))));
        }
    }
    assert_ne!(super::body_scroll_id(one), super::body_scroll_id(two));
}

#[test]
#[ignore = "offscreen layout probe"]
fn echoes_keep_row_ids() {
    let cfg = FakeSettingsSource::default();
    let place = crate::frontend::settings::place::WALLPAPERS;
    let empty = HashMap::new();
    let sections = source(&cfg, &empty).place(place).sections;
    let sound = sections
        .iter()
        .find(|section| section.id.key == "settings-wallpapers-sound")
        .expect("sound");
    assert!(sound.groups.iter().any(|group| !group.echoes.is_empty()));
    let mut ids = Ids(Vec::new());
    let shot =
        Shot { place, cfg: &cfg, open: Some(sound.id), pop: None, edit: None, weather: None };
    capture_in(SCREEN, &shot, &Live::default(), None, &mut [&mut ids]);
    let owner = super::section_identity(sound.id);
    let wanted: Vec<_> = (sound.header_rows()..sound.rows().count())
        .map(|flat| format!("{:?}", super::row_widget_id(owner, flat)))
        .collect();
    let found: Vec<_> = ids.0.iter().filter(|id| id.contains("settings-row")).cloned().collect();
    assert_eq!(found, wanted);
}

#[test]
#[ignore = "offscreen layout probe"]
fn sheet_rows_have_ids() {
    let place = crate::frontend::settings::place::KEYBINDS;
    let fit = &key_states()[0];
    let empty = HashMap::new();
    let live = fit.live();
    let sections = SourceCtx { ..live_source(&fit.cfg, &empty, &live) }.place(place).sections;
    let sheet = sections.iter().find(|section| section.id == KEYS_SHEET_ID).expect("sheet");
    let mut ids = Ids(Vec::new());
    let shot = Shot { place, cfg: &fit.cfg, open: None, pop: None, edit: None, weather: None };
    capture_in(SCREEN, &shot, &live, None, &mut [&mut ids]);
    let owner = super::section_identity(sheet.id);
    let bound =
        |row: &&crate::frontend::settings::Row| matches!(row.control, Control::KeyBinding { .. });
    let wanted: Vec<_> = sheet
        .rows()
        .enumerate()
        .filter(|(_, row)| bound(row))
        .map(|(flat, _)| format!("{:?}", super::row_widget_id(owner, flat)))
        .collect();
    assert!(!wanted.is_empty());
    assert!(wanted.iter().all(|id| ids.0.contains(id)));
}

struct Seen {
    owner: u64,
    scroller: iced::widget::Id,
    rows: Vec<(usize, Rectangle)>,
    view: Option<(Rectangle, Rectangle, f32)>,
}

impl Operation for Seen {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn container(&mut self, id: Option<&iced::widget::Id>, bounds: Rectangle) {
        let tag = format!("settings-row:{}:", self.owner);
        let name = id.map(|id| format!("{id:?}")).unwrap_or_default();
        if let Some((_, rest)) = name.split_once(&tag) {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            self.rows.extend(digits.parse().ok().map(|flat| (flat, bounds)));
        }
    }

    fn scrollable(
        &mut self,
        id: Option<&iced::widget::Id>,
        bounds: Rectangle,
        content: Rectangle,
        translation: Vector,
        _state: &mut dyn operation::Scrollable,
    ) {
        if id == Some(&self.scroller) {
            self.view = Some((bounds, content, translation.y));
        }
    }
}

fn seen_rows(name: &str) -> Seen {
    let place = crate::frontend::settings::place::KEYBINDS;
    let states = key_states();
    let fit = &states[state_named(&states, name)];
    let live = fit.live();
    let owner = super::section_identity(KEYS_SHEET_ID);
    let scroller = super::body_scroll_id(KEYS_SHEET_ID);
    let mut seen = Seen { owner, scroller, rows: Vec::new(), view: None };
    let shot = Shot { place, cfg: &fit.cfg, open: None, pop: None, edit: None, weather: None };
    capture_in(SCREEN, &shot, &live, None, &mut [&mut seen]);
    seen
}

#[test]
#[ignore = "offscreen layout probe"]
fn keys_scroller_reveals() {
    let _renderer = crate::frontend::headless::share(UI_FONT);
    set_language("ru-RU");
    iced_wgpu::graphics::text::font_system()
        .write()
        .expect("font system")
        .load_font(crate::shell::ui_font_bytes(Script::Cyrillic).into());
    let seen = seen_rows("keys-long");
    let (view, content, scrolled) = seen.view.expect("scroller");
    let hidden: Vec<_> =
        seen.rows.iter().filter(|(_, row)| row.y + row.height > view.y + view.height).collect();
    assert!(!hidden.is_empty(), "keys-long scrolls in ru-RU");
    for (flat, row) in &hidden {
        let offset = super::reveal_offset(row.y, content.y, scrolled, view.height);
        assert!(offset.is_some_and(|offset| offset > 0.0), "row {flat} is revealed");
    }
    let first = seen.rows.iter().min_by_key(|(flat, _)| *flat).expect("rows");
    assert_eq!(super::reveal_offset(first.1.y, content.y, scrolled, view.height), None);
    let listening = seen_rows("keys-listening");
    assert!(listening.view.is_some() && !listening.rows.is_empty());
    set_language("en-US");
}

#[test]
#[ignore = "offscreen layout probe"]
fn small_window_keeps_floor() {
    let small = Size::new(1000.0, 520.0);
    let viewport = (small.width, small.height);
    let scale = super::super::appearance::sheet_scale(viewport, 1.0);
    for place in crate::frontend::settings::places::composed_places() {
        let fit = &fit_states(place)[0];
        let empty = HashMap::new();
        let live = fit.live();
        let ctx =
            SourceCtx { weather: fit.weather.as_ref(), ..live_source(&fit.cfg, &empty, &live) };
        for section in ctx.place(place).sections {
            let shot = Shot {
                place,
                cfg: &fit.cfg,
                open: Some(section.id),
                pop: None,
                edit: None,
                weather: fit.weather.as_ref(),
            };
            let bodies = capture_in(small, &shot, &live, None, &mut []).heights;
            let floor = super::BODY.room_floor * scale;
            assert!(
                bodies.iter().all(|height| *height >= floor - 1.0),
                "{place} {} {bodies:?}",
                section.id.key
            );
        }
    }
}

#[test]
#[ignore = "offscreen layout probe of every composed place in every locale"]
fn labels_keep_words() {
    let _renderer = crate::frontend::headless::share(UI_FONT);
    let only = std::env::var(LANG).ok();
    let only_place = std::env::var(ONLY_PLACE).ok();
    let mut squeezed = Vec::new();
    for language in crate::i18n::LANGUAGES
        .iter()
        .filter(|language| only.as_deref().is_none_or(|tag| tag == language.tag))
    {
        set_language(language.tag);
        iced_wgpu::graphics::text::font_system()
            .write()
            .expect("font system")
            .load_font(crate::shell::ui_font_bytes(language.script).into());
        for place in crate::frontend::settings::places::composed_places()
            .filter(|place| only_place.as_deref().is_none_or(|only| only == *place))
        {
            for fit in fit_states(place) {
                let empty = HashMap::new();
                let live = fit.live();
                let ctx = SourceCtx {
                    weather: fit.weather.as_ref(),
                    ..live_source(&fit.cfg, &empty, &live)
                };
                for section in ctx.place(place).sections {
                    let shot = Shot {
                        place,
                        cfg: &fit.cfg,
                        open: Some(section.id),
                        pop: None,
                        edit: None,
                        weather: fit.weather.as_ref(),
                    };
                    FLOORS.with(|floors| floors.borrow_mut().clear());
                    let mut found = Squeezed(Vec::new());
                    capture_in(SCREEN, &shot, &live, None, &mut [&mut found]);
                    squeezed.extend(found.0.into_iter().map(|row| {
                        format!("{} {place} {} {} {row}", language.tag, fit.name, section.id.key)
                    }));
                }
            }
        }
    }
    set_language("en-US");
    assert!(squeezed.is_empty(), "{squeezed:#?}");
}

const LOST_SLACK: f32 = 0.5;

struct Lost(Vec<String>);

impl Probe for Lost {
    fn run(&mut self, size: Size, view: iced::Element<'_, crate::app::Message>) {
        let tree = crate::frontend::headless::laid_tree(size, UI_FONT, view);
        walk_text(&tree, &mut self.0);
    }
}

fn walk_text(tree: &iced::advanced::widget::Tree, found: &mut Vec<String>) {
    use iced::advanced::text::paragraph::Plain;
    use iced::advanced::widget::tree::Tag;
    use iced_wgpu::graphics::text::Paragraph;
    if tree.tag == Tag::of::<Plain<Paragraph>>() {
        found.extend(lost_text(tree.state.downcast_ref::<Plain<Paragraph>>().raw()));
    }
    found.extend(crate::frontend::ui::calm::laid(tree).and_then(lost_text));
    for child in &tree.children {
        walk_text(child, found);
    }
}

fn silent(ch: char) -> bool {
    ch.is_whitespace()
        || matches!(ch, '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}')
}

fn lost_text(paragraph: &iced_wgpu::graphics::text::Paragraph) -> Option<String> {
    use iced::advanced::text::Paragraph as _;
    use iced_wgpu::graphics::text::cosmic_text::BufferLine;
    let buffer = paragraph.buffer();
    let bounds = paragraph.bounds();
    let tall = paragraph.min_bounds().height.min(bounds.height) + LOST_SLACK;
    let mut seen: Vec<Vec<std::ops::Range<usize>>> = vec![Vec::new(); buffer.lines.len()];
    let mut widest = 0.0_f32;
    for run in buffer.layout_runs().filter(|run| run.line_top + run.line_height <= tall) {
        widest = widest.max(run.line_w);
        seen[run.line_i].extend(run.glyphs.iter().map(|glyph| glyph.start..glyph.end));
    }
    let gone: String = buffer
        .lines
        .iter()
        .zip(&seen)
        .flat_map(|(line, ranges)| {
            line.text().char_indices().filter_map(move |(at, ch)| {
                let drawn = ranges.iter().any(|range| range.contains(&at));
                (!silent(ch) && !drawn).then_some(ch)
            })
        })
        .collect();
    let wide = widest > bounds.width + LOST_SLACK;
    let text: Vec<_> = buffer.lines.iter().map(BufferLine::text).collect();
    (wide || !gone.is_empty()).then(|| {
        format!(
            "{:?} widest {widest:.1} of {:.1} tall {:.1} of {:.1} lost {gone:?}",
            text.join(" / "),
            bounds.width,
            paragraph.min_bounds().height,
            bounds.height,
        )
    })
}

#[test]
#[ignore = "offscreen layout probe of every composed place in every locale"]
fn text_keeps_glyphs() {
    let only = std::env::var(LANG).ok();
    let only_place = std::env::var(ONLY_PLACE).ok();
    let _renderer = crate::frontend::headless::share(UI_FONT);
    let mut lost = Lost(Vec::new());
    let mut report = std::collections::BTreeSet::new();
    let mut shots = 0;
    for language in crate::i18n::LANGUAGES
        .iter()
        .filter(|language| only.as_deref().is_none_or(|tag| tag == language.tag))
    {
        set_language(language.tag);
        iced_wgpu::graphics::text::font_system()
            .write()
            .expect("font system")
            .load_font(crate::shell::ui_font_bytes(language.script).into());
        for place in crate::frontend::settings::places::composed_places()
            .filter(|place| only_place.as_deref().is_none_or(|only| only == *place))
        {
            for fit in fit_states(place) {
                let empty = HashMap::new();
                let live = fit.live();
                let ctx = SourceCtx {
                    weather: fit.weather.as_ref(),
                    ..live_source(&fit.cfg, &empty, &live)
                };
                let sections = ctx.place(place).sections;
                let opens =
                    std::iter::once(None).chain(sections.iter().map(|section| Some(section.id)));
                for open in opens {
                    let shot = Shot {
                        place,
                        cfg: &fit.cfg,
                        open,
                        pop: state_pop(&fit.name),
                        edit: None,
                        weather: fit.weather.as_ref(),
                    };
                    capture_in(SCREEN, &shot, &live, None, &mut [&mut lost]);
                    shots += 1;
                    let key = open.map_or("closed", |id| id.key);
                    report.extend(
                        lost.0.drain(..).map(|row| {
                            format!("{} {place} {} {key} {row}", language.tag, fit.name)
                        }),
                    );
                }
            }
        }
    }
    set_language("en-US");
    println!("text_keeps_glyphs: {shots} shots, {} lost", report.len());
    if let Ok(out) = std::env::var(OUT) {
        let rows: Vec<_> = report.iter().cloned().collect();
        std::fs::write(format!("{out}/lost.txt"), rows.join("\n")).expect("lost report");
    }
    assert!(report.is_empty(), "{report:#?}");
}

const WRAPPED_ROWS: [&str; 2] =
    [crate::frontend::settings::place::LAYOUT, crate::frontend::settings::place::LIBRARY];

#[test]
#[ignore = "offscreen layout probe"]
fn closed_rows_keep_page() {
    let _renderer = crate::frontend::headless::share(UI_FONT);
    let floor = super::BODY.room_floor;
    let only = std::env::var(LANG).ok();
    for language in crate::i18n::LANGUAGES
        .iter()
        .filter(|language| only.as_deref().is_none_or(|tag| tag == language.tag))
    {
        set_language(language.tag);
        iced_wgpu::graphics::text::font_system()
            .write()
            .expect("font system")
            .load_font(crate::shell::ui_font_bytes(language.script).into());
        for place in WRAPPED_ROWS {
            for fit in fit_states(place) {
                let empty = HashMap::new();
                let live = fit.live();
                let ctx = SourceCtx {
                    weather: fit.weather.as_ref(),
                    ..live_source(&fit.cfg, &empty, &live)
                };
                for section in ctx.place(place).sections {
                    let shot = Shot {
                        place,
                        cfg: &fit.cfg,
                        open: Some(section.id),
                        pop: None,
                        edit: None,
                        weather: fit.weather.as_ref(),
                    };
                    let measure = capture_in(SCREEN, &shot, &live, None, &mut []);
                    let floored = measure.heights.iter().any(|height| *height <= floor + 1.0);
                    assert!(
                        measure.visible <= measure.view + 1.0 || floored,
                        "{} {place} {} {} {} of {}",
                        language.tag,
                        fit.name,
                        section.id.key,
                        measure.visible,
                        measure.view
                    );
                }
            }
        }
    }
    set_language("en-US");
}

fn shown_labels(
    pieces: Vec<crate::frontend::ui::calm::Piece<'_, crate::app::Message>>,
) -> Vec<String> {
    use crate::frontend::ui::calm::Piece;
    pieces
        .into_iter()
        .filter_map(|piece| match piece {
            Piece::Token { label, .. } => Some(label.into_owned()),
            _ => None,
        })
        .collect()
}

fn blank_values(place: &str, fit: &FitState) -> Vec<String> {
    use crate::frontend::animation::MotionProfile;
    use crate::frontend::settings::Slot;

    let values = seeded(&fit.cfg, place, &fit.live());
    let ctx =
        SourceCtx { weather: fit.weather.as_ref(), ..live_source(&fit.cfg, &values, &fit.live()) };
    let palette = board();
    let folds = HashSet::new();
    let bars = HashMap::new();
    let paint = super::Paint {
        values: &values,
        focus: FocusCtx {
            keyboard_focus: SettingsFocus::Index,
            focused_control: 0,
            focused_choice: None,
            expanded_details: &folds,
            bar_reveals: &bars,
            active_input: None,
            armed: None,
        },
        calm: CalmCtx {
            open: None,
            section: 0,
            busy: 0.0,
            pop: None,
            edit: None,
            instant: true,
            closing: None,
        },
        scale: 1.0,
        palette: &palette,
        motion: MotionProfile::new(1.0, 1.0, 1.0),
        body: SCREEN.width,
        dim: false,
        reading: SCREEN.height,
        owner: 0,
        undivided: false,
        trim: crate::frontend::settings::Trim::None,
    };
    let mut empty = Vec::new();
    let mut check = |what: String, labels: Vec<String>| {
        for label in labels {
            if label.trim().is_empty() || label.ends_with(": ") {
                empty.push(format!("{place} {} {what}: {label:?}", fit.name));
            }
        }
    };
    for section in ctx.place(place).sections {
        let rows: Vec<_> = section.rows().collect();
        for slot in &section.say.tokens {
            match slot {
                Slot::Value { row, blank, .. } => {
                    let Some(row) = rows.get(*row) else { continue };
                    let pieces = super::token_pieces(
                        &row.control,
                        TokenSide::Sentence,
                        blank.as_deref(),
                        paint,
                    );
                    check(
                        format!("{} sentence {}", section.id.key, row.title),
                        shown_labels(pieces),
                    );
                }
                Slot::Checks { blank, .. } => {
                    check(format!("{} checks", section.id.key), vec![blank.clone()]);
                }
            }
        }
        for group in &section.groups {
            for row in &group.rows {
                let caption = row.look.blank.as_deref().or(group.blank.as_deref());
                let caption = caption.filter(|_| matches!(row.control, Control::TextField { .. }));
                let pieces = super::token_pieces(&row.control, TokenSide::Body, caption, paint);
                check(format!("{} body {}", section.id.key, row.title), shown_labels(pieces));
            }
        }
    }
    empty
}

#[test]
fn no_blank_values() {
    set_language("en-US");
    let mut empty = Vec::new();
    for place in crate::frontend::settings::places::composed_places() {
        for fit in fit_states(place) {
            empty.extend(blank_values(place, &fit));
        }
    }
    assert!(empty.is_empty(), "{empty:#?}");
}

struct Placed(Vec<(String, Rectangle)>);

impl Operation for Placed {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn container(&mut self, id: Option<&iced::widget::Id>, bounds: Rectangle) {
        self.0.extend(id.map(|id| (format!("{id:?}"), bounds)));
    }
}

const CORNER_VALUES: [&str; 2] = ["64", "6400000000"];

#[test]
#[ignore = "offscreen layout probe"]
fn corner_tokens_clear_square() {
    use crate::frontend::settings::{Cell, layout_place::CORNERS};
    let _renderer = crate::frontend::headless::share(UI_FONT);
    let keys = skwd_config::corners::SLICES;
    let section = SectionId::new(crate::frontend::settings::category::PICKER, CORNERS);
    let owner = super::section_identity(section);
    let square = super::BODY.corner_box[0] + super::BODY.corner_gap * 2.0;
    for language in crate::i18n::LANGUAGES {
        set_language(language.tag);
        iced_wgpu::graphics::text::font_system()
            .write()
            .expect("font system")
            .load_font(crate::shell::ui_font_bytes(language.script).into());
        for value in CORNER_VALUES {
            let mut cfg =
                layout_cfg("slices").with_text(keys.style, "chamfered").with_flag(keys.same, false);
            for key in keys.values {
                cfg = cfg.with_text(key, value);
            }
            let fit = layout_state("wide-corners", cfg);
            let empty = HashMap::new();
            let sections = source(&fit.cfg, &empty).place(LAYOUT).sections;
            let corners = sections.iter().find(|found| found.id == section).expect("corners");
            let flat = corners
                .rows()
                .position(|row| matches!(row.look.cell, Cell::Corners { .. }))
                .expect("corner row");
            let wanted: Vec<_> = (flat..flat + keys.values.len())
                .map(|at| format!("{:?}", super::row_widget_id(owner, at)))
                .collect();
            let open = Some(section);
            let shot =
                Shot { place: LAYOUT, cfg: &fit.cfg, open, pop: None, edit: None, weather: None };
            let mut placed = Placed(Vec::new());
            capture_in(SCREEN, &shot, &fit.live(), None, &mut [&mut placed]);
            let mut cells: Vec<Rectangle> = placed
                .0
                .iter()
                .filter(|(id, _)| wanted.contains(id))
                .map(|(_, bounds)| *bounds)
                .collect();
            assert_eq!(cells.len(), wanted.len(), "{}", language.tag);
            cells.sort_by(|one, two| one.x.total_cmp(&two.x));
            let (left, right) = cells.split_at(cells.len() / 2);
            let inner = left.iter().map(|cell| cell.x + cell.width).fold(0.0, f32::max);
            let outer = right.iter().map(|cell| cell.x).fold(f32::MAX, f32::min);
            assert!(outer - inner >= square - LOST_SLACK, "{} {value} {cells:?}", language.tag);
        }
    }
    set_language("en-US");
}

#[test]
#[ignore = "manual offscreen render of a place switch"]
fn render_switch() {
    use crate::frontend::settings::place::{AUTOMATION, SYSTEM};
    let out = localized();
    let cfg = system(true);
    for (from, to) in [(SYSTEM, AUTOMATION), (AUTOMATION, SYSTEM)] {
        for at in [0.25_f32, 0.5, 0.75] {
            let values = seeded(&cfg, from, &Live::default());
            let place = source(&cfg, &values).place(from);
            SWITCH.set(Some((PageSnapshot { place, inputs: values }, at)));
            let shot =
                Shot { place: to, cfg: &cfg, open: None, pop: None, edit: None, weather: None };
            capture(&shot, Some(&format!("{out}/{from}-{to}-{at}.png")));
        }
    }
}
