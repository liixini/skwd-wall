#[cfg(test)]
#[path = "home_tests.rs"]
mod tests;

use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use crate::contracts::picker::{KEY_BINDINGS, format_config_number};
use crate::contracts::settings::schema::{self, DefaultValue, ValueKind};
use crate::contracts::settings::{Overview, SettingChange, SettingsSource, Undo, keys};
use crate::domain::input::{InputMap, Trigger, parse_binding};
use crate::frontend::ui::calm::sentinel;
use crate::i18n::{self, tr, tr_args};

use super::folio::default_text;
use super::folio::number_editor::{duration_caption, number_caption};
use super::places::{Choice, PlaceSource, SUMMARY_JOINER, bind, column, marked};
use super::tabs::library_watch_label;
use super::{
    ActionId, Control, Form, Group, GroupKind, Note, Place, Row, RuntimeStatus, Say, Section,
    SectionId, Slot, Trim, automation, library, masked, place, place_title, wallpapers,
};

pub const LEAD: SectionId = SectionId::new(place::HOME, "settings-place-home");

struct Face {
    title: &'static str,
    desc: &'static str,
}

const SOUND: Face = Face { title: wallpapers::SOUND, desc: "settings-home-sound-desc" };
const COLOURS: Face = Face { title: "settings-tab-theme", desc: "settings-home-colours-desc" };
const ROTATION: Face =
    Face { title: "settings-section-random-rotation", desc: "settings-home-rotation-desc" };
const TRANSITIONS: Face = Face {
    title: "settings-section-wallpaper-transitions",
    desc: "settings-home-transitions-desc",
};
const CHANGED: &str = "settings-home-changed";

const SEARCH: &str = "settings-home-search";
const DISPLAYS: &str = "settings-home-displays";
const DISPLAYS_SAY: &str = "settings-home-displays-say";
const NO_DISPLAYS: &str = "settings-home-displays-none";
const NO_DISPLAYS_SAY: &str = "settings-home-displays-none-say";
const WATCHING_SAY: &str = "settings-home-watching";
const MODEL_SAY: &str = "settings-bar-model-status";
const MODEL_READY: &str = "settings-home-model-ready";
const MODEL_MISSING: &str = "settings-home-model-missing";
const MODEL_OFF: &str = "settings-home-model-off";
const FROM_SAY: &str = "settings-home-colours-from";
const MORE_SAY: &str = "settings-home-colours-say";
const SOUND_SAY: &str = "settings-home-sound-say";
const ROTATION_SAY: &str = "settings-home-rotation-say";
const ROTATION_OFF: &str = "settings-home-rotation-off";
const TRANSITIONS_SAY: &str = "settings-home-transitions-say";
const TRANSITIONS_OFF: &str = "settings-home-transitions-off";
const CHANGED_DESC: &str = "settings-home-changed-desc";
const CHANGED_SAY: &str = "settings-home-changed-say";
const CHANGED_NONE: &str = "settings-home-changed-none";
const REVIEW: &str = "settings-home-review";
const RESET: &str = "settings-home-reset";
const RESET_ALL: &str = "settings-home-reset-all";
const RESET_SCOPE: &str = "settings-home-reset-scope";
const BADGE: &str = "settings-home-badge";
const ITEMS: &str = "settings-home-items";
const AUTOMATIC: &str = "settings-home-automatic";
const NO_VALUE: &str = "settings-control-none";
const TOGGLE_OFF: &str = "settings-toggle-off";
const TOGGLE_ON: &str = "settings-toggle-on";

const LINK_ARG: &str = "link";
const COUNT_ARG: &str = "count";
const STATE_ARG: &str = "state";
const ENGINE_ARG: &str = "engine";
const STATUS_ARG: &str = "status";
const DUCKS: Choice = Choice { arg: "duck", values: ["no", "yes"] };
const SECONDS: &str = "s";
const SAME_NUMBER: f64 = 1e-9;
const PATHS_PREFIX: &str = "paths.";
const STAMP_FLAGS: usize = 3;

const PERSONAL: [&str; 11] = [
    keys::general::LOCALE,
    keys::schedule::LATITUDE,
    keys::schedule::LONGITUDE,
    keys::steam::USERNAME,
    keys::wallhaven::USERNAME,
    keys::steam::API_KEY,
    keys::wallhaven::API_KEY,
    keys::sources::PEXELS_API_KEY,
    keys::sources::UNSPLASH_ACCESS_KEY,
    keys::library::FOLDERS,
    keys::library::EXCLUDED_FOLDERS,
];
const HIDDEN: [&str; 2] = [keys::steam::USERNAME, keys::wallhaven::USERNAME];
const LISTS: [(&str, &str); 3] = [
    (library::FOLDERS, keys::library::FOLDERS),
    (library::EXCLUDED, keys::library::EXCLUDED_FOLDERS),
    (library::PROFILES, keys::library::PROFILES),
];

fn stamp(source: &PlaceSource<'_>) -> u64 {
    let cfg = source.cfg;
    let flags: [bool; STAMP_FLAGS] =
        [cfg.default_semantic_model_available(), cfg.is_niri(), cfg.on_battery_power()];
    let mut hasher = DefaultHasher::new();
    (i18n::active_tag(), source.themes, source.folders, source.backends, flags).hash(&mut hasher);
    hasher.finish()
}

pub(super) fn overview(source: &PlaceSource<'_>) -> Arc<Overview> {
    source.cfg.overview(stamp(source), &|| scan(source))
}

#[cfg(test)]
pub fn setting_changes(source: &PlaceSource<'_>) -> Vec<SettingChange> {
    overview(source).changes.clone()
}

pub fn fresh_changes(source: &PlaceSource<'_>) -> Vec<SettingChange> {
    scan(source).changes
}

pub(super) fn badge(key: &str, overview: &Overview) -> Option<String> {
    if key == place::HOME {
        let count = overview.changes.len();
        return (count > 0).then(|| tr_args!(BADGE, count => count));
    }
    let count = overview.count(key);
    (count > 0).then(|| count.to_string())
}

pub(super) fn compose(source: &PlaceSource<'_>) -> Vec<Section> {
    let cfg = source.cfg;
    let found = overview(source);
    let colours = source.place(place::COLOURS);
    let mut wallpapers = source.place(place::WALLPAPERS);
    let mut automation = source.place(place::AUTOMATION);
    let mut sections = vec![lead(source, &colours)];
    sections.extend(owned(&mut wallpapers, wallpapers::SOUND).map(|section| sound(section, cfg)));
    sections.extend(palette(&colours));
    sections.extend(owned(&mut automation, automation::ROTATION).map(rotation));
    sections.extend(owned(&mut wallpapers, wallpapers::MOTION).map(transitions));
    sections.push(changed(&found));
    sections
}

fn owned(owner: &mut Place, key: &str) -> Option<Section> {
    let at = owner.sections.iter().position(|section| section.id.key == key)?;
    Some(owner.sections.swap_remove(at))
}

fn rehomed(section: Section, face: &Face) -> Section {
    Section {
        id: SectionId::new(place::HOME, face.title),
        title: tr(face.title).to_string(),
        desc: tr(face.desc).to_string(),
        ..section
    }
}

fn restated(mut section: Section, say: Option<Say>) -> Section {
    if let Some(say) = say {
        section.say = Say { text: say.text, tokens: say.tokens, ..section.say };
    }
    section
}

fn sound(section: Section, cfg: &dyn SettingsSource) -> Section {
    let section = rehomed(section, &SOUND);
    if cfg.flag(keys::wallpaper::MUTE) {
        return section;
    }
    let duck = DUCKS.pick(cfg.flag(keys::playback::MUTE_ON_OTHER_AUDIO));
    let say = bind(&section, SOUND_SAY, &[wallpapers::VOLUME_SLOT], &[duck]);
    restated(section, say)
}

fn rotation(section: Section) -> Section {
    let mut section = rehomed(section, &ROTATION);
    if section.switched_off() {
        section.say.text = tr(ROTATION_OFF).to_string();
        return section;
    }
    let say = bind(&section, ROTATION_SAY, &[automation::INTERVAL_SLOT], &[]);
    restated(section, say)
}

fn transitions(section: Section) -> Section {
    let mut section = Section { trim: Trim::Rows, ..rehomed(section, &TRANSITIONS) };
    if section.switched_off() {
        section.say.text = tr(TRANSITIONS_OFF).to_string();
        return section;
    }
    let timed = section.rows().any(|row| row.control.path() == Some(keys::transition::DURATION_MS));
    if !timed {
        return section;
    }
    let mut section = lowered(section, keys::transition::DURATION_MS);
    let say = bind(&section, TRANSITIONS_SAY, &[wallpapers::DURATION_SLOT], &[]);
    section.say.parked.clear();
    restated(section, say)
}

fn lowered(mut section: Section, kept: &str) -> Section {
    let head = std::mem::take(&mut section.head);
    let (stay, moved): (Vec<Row>, Vec<Row>) = head.into_iter().partition(|row| {
        matches!(row.control, Control::ActionBtn { .. }) || row.control.path() == Some(kept)
    });
    section.head = stay;
    match section.groups.first_mut() {
        Some(group) => {
            group.rows = moved.into_iter().chain(std::mem::take(&mut group.rows)).collect();
        }
        None => section.groups.push(column(None, moved)),
    }
    section
}

fn leaf_at<'p>(owner: &'p Place, path: &str) -> Option<&'p Row> {
    let rows = owner.sections.iter().flat_map(Section::rows);
    rows.flat_map(leaves).find(|row| row.control.path() == Some(path))
}

fn link(label: &str, place: &'static str, setting: Option<&'static str>) -> Row {
    let id = ActionId::OpenPlace { place, setting };
    Row::of(label.to_string(), String::new(), Control::ActionBtn { id, label: label.to_string() })
}

fn linked(row: usize, form: Form) -> Slot {
    Slot::Value { row, blank: None, form }
}

fn mark(slot: u32) -> String {
    sentinel(slot).map(String::from).unwrap_or_default()
}

fn palette(colours: &Place) -> Option<Section> {
    let policy = leaf_at(colours, keys::theme::POLICY)?.clone();
    let name = place_title(place::COLOURS);
    let text = tr_args!(MORE_SAY, policy => mark(0), place => mark(1));
    let more = link(name, place::COLOURS, Some(keys::theme::POLICY));
    Some(Section {
        head: vec![policy, more],
        say: Say {
            text,
            tokens: vec![linked(0, Form::Plain), linked(1, Form::Mention)],
            ..Say::default()
        },
        ..Section::new(
            SectionId::new(place::HOME, COLOURS.title),
            tr(COLOURS.title),
            tr(COLOURS.desc),
        )
    })
}

fn said(key: &str, pairs: &[(&'static str, &str)]) -> String {
    let mut args = marked();
    for (arg, value) in pairs {
        args.set(*arg, (*value).to_string());
    }
    i18n::format(key, &args)
}

fn lead(source: &PlaceSource<'_>, colours: &Place) -> Section {
    let cfg = source.cfg;
    let shown = source.runtime.statuses.iter().filter(|status| !status.current.is_empty()).count();
    let (displays, displays_say) = if shown == 0 {
        (tr(NO_DISPLAYS).to_string(), NO_DISPLAYS_SAY)
    } else {
        (tr_args!(DISPLAYS, count => shown), DISPLAYS_SAY)
    };
    let search = tr(SEARCH).to_string();
    let name = place_title(place::COLOURS);
    let head = vec![
        Row::of(
            search.clone(),
            String::new(),
            Control::ActionBtn { id: ActionId::OpenSearch, label: search },
        ),
        link(&displays, place::WALLPAPERS, None),
        link(name, place::COLOURS, None),
    ];
    let analysis = source.analysis.trim();
    let model = model(cfg);
    let status = if analysis.is_empty() { model.as_str() } else { analysis };
    let engine = leaf_at(colours, keys::theme::ENGINE)
        .map_or_else(|| tr(NO_VALUE).to_string(), |row| shown_value(row, cfg));
    let mut args = marked();
    args.set(LINK_ARG, mark(0));
    args.set(COUNT_ARG, shown);
    let text = [
        i18n::format(displays_say, &args),
        said(WATCHING_SAY, &[(STATE_ARG, &library_watch_label(source.runtime.library_watch))]),
        said(MODEL_SAY, &[(STATUS_ARG, status)]),
        said(FROM_SAY, &[(LINK_ARG, &mark(1)), (ENGINE_ARG, &engine)]),
    ]
    .concat();
    Section {
        head,
        say: Say {
            text,
            tokens: vec![linked(1, Form::Link), linked(2, Form::Link)],
            ..Say::default()
        },
        ..Section::new(LEAD, "", "")
    }
}

fn model(cfg: &dyn SettingsSource) -> String {
    let key = if !cfg.flag(keys::semantic::ENABLED) {
        MODEL_OFF
    } else if cfg.default_semantic_model_available() {
        MODEL_READY
    } else {
        MODEL_MISSING
    };
    tr(key).to_string()
}

fn changed(overview: &Overview) -> Section {
    let count = overview.changes.len();
    let text = if count == 0 {
        tr(CHANGED_NONE).to_string()
    } else {
        tr_args!(CHANGED_SAY, count => count)
    };
    let mut groups = Vec::new();
    if count > 0 {
        let rows = overview.changes.iter().map(reset_row).collect();
        let label = tr(RESET_ALL).to_string();
        let all = Row::of(
            label.clone(),
            tr(RESET_SCOPE).to_string(),
            Control::ActionBtn { id: ActionId::ResetEverything, label },
        );
        groups.push(Group::new(GroupKind::Spread, None, rows));
        groups.push(Group::new(GroupKind::Foot, None, vec![all]));
    }
    Section {
        say: Say { text, ..Say::default() },
        groups,
        opens: Some(tr(REVIEW)),
        undivided: true,
        min_columns: 2,
        ..Section::new(SectionId::new(place::HOME, CHANGED), tr(CHANGED), tr(CHANGED_DESC))
    }
}

fn reset_row(change: &SettingChange) -> Row {
    let control = Control::ActionBtn {
        id: ActionId::ResetSetting(Box::new(change.clone())),
        label: tr(RESET).to_string(),
    };
    let mut row = Row::of(change.title.clone(), String::new(), control);
    row.look.note = Note::Own(change.spot.clone());
    row
}

fn scan(source: &PlaceSource<'_>) -> Overview {
    let source = PlaceSource { analysis: "", runtime: RuntimeStatus::default(), ..*source };
    let cfg = source.cfg;
    let map = cfg.bindings();
    let mut changes = Vec::new();
    let mut seen = HashSet::new();
    for shown in source.places() {
        let sleepers = source.asleep(shown.key, &shown.sections);
        let sections = shown.sections.iter();
        let awake =
            sections.flat_map(|section| section.rows().map(|row| (row, section.title.as_str())));
        let woken =
            sleepers.iter().map(|asleep| (&asleep.row, spot_of(&shown.sections, asleep.wakes)));
        for (row, section) in awake.chain(woken) {
            for row in leaves(row) {
                let Some(path) = row.control.path().filter(|path| seen.insert(path.to_string()))
                else {
                    continue;
                };
                changes.extend(change(row, &map, cfg).map(|found| SettingChange {
                    place: shown.key,
                    spot: spot_label(shown.title, section),
                    title: if row.title.is_empty() { section } else { &row.title }.to_string(),
                    path: path.to_string(),
                    was: found.was,
                    now: found.now,
                    undo: found.undo,
                    spared: spared(row, path),
                }));
            }
        }
        changes.extend(lists(&shown, cfg));
    }
    Overview { changes }
}

fn lists(shown: &Place, cfg: &dyn SettingsSource) -> Vec<SettingChange> {
    let owner = shown.key == place::LIBRARY;
    let found = LISTS.iter().filter(|_| owner).filter_map(|(section, path)| {
        let count = cfg.array_len(path);
        let title = &shown.sections.iter().find(|candidate| candidate.id.key == *section)?.title;
        (count > 0).then(|| SettingChange {
            place: shown.key,
            spot: shown.title.to_string(),
            title: title.clone(),
            path: (*path).to_string(),
            was: tr(NO_VALUE).to_string(),
            now: tr_args!(ITEMS, count => count),
            undo: Undo::Clear,
            spared: true,
        })
    });
    found.collect()
}

fn spared(row: &Row, path: &str) -> bool {
    matches!(row.control, Control::TextField { .. })
        || PERSONAL.contains(&path)
        || LISTS.iter().any(|(_, list)| *list == path)
        || path.starts_with(PATHS_PREFIX)
}

fn spot_of<'s>(sections: &'s [Section], wakes: &str) -> &'s str {
    let found =
        sections.iter().find(|section| section.rows().any(|row| row.control.path() == Some(wakes)));
    found.map_or("", |section| section.title.as_str())
}

fn spot_label(place: &str, section: &str) -> String {
    if section.is_empty() {
        return place.to_string();
    }
    format!("{place}{SUMMARY_JOINER}{section}")
}

fn leaves(row: &Row) -> Vec<&Row> {
    match &row.control {
        Control::Details { rows, .. } | Control::StackBar { rows, .. } => {
            rows.iter().flat_map(leaves).collect()
        }
        _ => vec![row],
    }
}

struct Found {
    was: String,
    now: String,
    undo: Undo,
}

fn change(row: &Row, map: &InputMap, cfg: &dyn SettingsSource) -> Option<Found> {
    if let Control::KeyBinding { path, default, .. } = &row.control {
        return binding(path, default, map);
    }
    let path = row.control.path()?;
    let now = shown_value(row, cfg);
    let now = if HIDDEN.contains(&path) { masked(&now) } else { now };
    match default_of(path)? {
        DefaultValue::Boolean(default) => (cfg.flag(path) != default).then(|| Found {
            was: toggle_word(default),
            now,
            undo: Undo::Toggle(default),
        }),
        DefaultValue::Number(default) => differs(cfg.number(path), &[default]).then(|| Found {
            was: number_word(row, default),
            now,
            undo: number_undo(row, default),
        }),
        DefaultValue::ResponsiveNumber { regular, compact } => {
            differs(cfg.number(path), &[regular, compact]).then(|| Found {
                was: number_word(row, regular),
                now,
                undo: Undo::Forget,
            })
        }
        DefaultValue::Text(default) => (cfg.text(path) != default).then(|| Found {
            was: text_word(row, default),
            now,
            undo: text_undo(row, default),
        }),
        DefaultValue::EmptyArray => None,
    }
}

fn default_of(path: &str) -> Option<DefaultValue> {
    if let Some(spec) = schema::find(path) {
        return Some(spec.default);
    }
    if let Some(on) = schema::boolean_default(path) {
        return Some(DefaultValue::Boolean(on));
    }
    if let Some(text) = schema::text_default(path) {
        return Some(DefaultValue::Text(text));
    }
    (schema::value_kind(path) == Some(ValueKind::Text)).then_some(DefaultValue::Text(""))
}

fn differs(value: f64, defaults: &[f64]) -> bool {
    defaults.iter().all(|default| (value - default).abs() > SAME_NUMBER)
}

fn binding(path: &str, default: &str, map: &InputMap) -> Option<Found> {
    let action = KEY_BINDINGS.into_iter().find(|entry| entry.path == path)?.action;
    let defaults = parse_binding(default).unwrap_or_default();
    let triggers: &[Trigger] = map.triggers(action);
    (triggers != defaults.as_slice()).then(|| Found {
        was: default_text(&defaults),
        now: default_text(triggers),
        undo: Undo::Binding,
    })
}

fn toggle_word(on: bool) -> String {
    tr(if on { TOGGLE_ON } else { TOGGLE_OFF }).to_string()
}

fn number_word(row: &Row, value: f64) -> String {
    let raw = format_config_number(value);
    if let Control::Number { key, unit, .. } = &row.control {
        return caption(key, &raw, unit);
    }
    text_word(row, &raw)
}

fn caption(key: &str, raw: &str, unit: &str) -> String {
    let timed = (unit == SECONDS).then(|| duration_caption(raw)).flatten();
    timed.unwrap_or_else(|| number_caption(key, raw, unit))
}

fn text_word(row: &Row, value: &str) -> String {
    let listed = match &row.control {
        Control::Dropdown { options, .. } | Control::Chips { options, .. } => {
            options.iter().find(|(key, _)| key == value).map(|(_, label)| label.clone())
        }
        _ => None,
    };
    let menu = matches!(row.control, Control::Dropdown { .. } | Control::Chips { .. });
    let word = listed.unwrap_or_else(|| value.to_string());
    if word.is_empty() && menu {
        return tr(AUTOMATIC).to_string();
    }
    if word.is_empty() { blank_word(row) } else { word }
}

fn blank_word(row: &Row) -> String {
    let placeholder = match &row.control {
        Control::TextField { placeholder, .. } => *placeholder,
        _ => "",
    };
    row.look
        .blank
        .clone()
        .filter(|blank| !blank.is_empty())
        .or_else(|| (!placeholder.is_empty()).then(|| placeholder.to_string()))
        .unwrap_or_else(|| tr(NO_VALUE).to_string())
}

fn shown_value(row: &Row, cfg: &dyn SettingsSource) -> String {
    let text = match &row.control {
        Control::Toggle { value, .. } => toggle_word(*value),
        Control::Number { key, path, unit } => {
            caption(key, &format_config_number(cfg.number(path)), unit)
        }
        control => super::search::control_value(control, cfg),
    };
    if text.is_empty() { blank_word(row) } else { text }
}

fn number_undo(row: &Row, default: f64) -> Undo {
    let raw = format_config_number(default);
    match &row.control {
        Control::Number { key, .. } => Undo::Input { key: key.clone(), raw },
        _ => Undo::Pick(raw),
    }
}

fn text_undo(row: &Row, default: &str) -> Undo {
    match &row.control {
        Control::TextField { key, .. } | Control::Resolution { key, .. } => {
            Undo::Input { key: key.clone(), raw: default.to_string() }
        }
        _ => Undo::Pick(default.to_string()),
    }
}
