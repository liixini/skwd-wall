#[cfg(test)]
#[path = "places_tests.rs"]
mod tests;

use crate::contracts::settings::SettingsSource;
use crate::frontend::ui::calm::{VALUE_CLOSE, VALUE_OPEN};
use crate::i18n::tr;

use super::place;
use super::tables::{LEGACY_ROUTES, canonical_category, category};
use super::{
    Card, Cell, Control, Form, Group, GroupKind, Look, Note, Parked, Place, Row, RuntimeStatus,
    Say, Section, SectionId, Slot, build_tab_with_runtime_status,
};

const MAX_GROUPS: usize = 2;
pub(super) const SUMMARY_JOINER: &str = " · ";
const LEAD_KEY: &str = "settings-section-lead";
const LEAD_SETTING: &str = "setting";
const LEAD_VALUE: &str = "value";
const VALUE_ARG: &str = "val";
const VALUE_END_ARG: &str = "end";
pub(super) const LIST_KEY: &str = "settings-list";
const LIST_COUNT: &str = "count";
const LIST_COUNTS: [&str; LIST_NAMES.len() + 1] = ["none", "one", "two", "three", "four"];
const LIST_MANY: &str = "many";
const LIST_NAMES: [&str; 4] = ["first", "second", "third", "fourth"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cut {
    Whole,
    Bar,
    Layout,
}

type Compose = fn(&PlaceSource<'_>) -> Vec<Section>;

type Sleepers = fn(&[Section], &PlaceSource<'_>) -> Vec<Asleep>;

struct Composed {
    place: &'static str,
    compose: Compose,
    asleep: Option<Sleepers>,
}

const COMPOSED: [Composed; 10] = [
    Composed { place: place::HOME, compose: super::home::compose, asleep: None },
    Composed {
        place: place::LAYOUT,
        compose: super::layout_place::compose,
        asleep: Some(super::layout_place::asleep),
    },
    Composed { place: place::BAR, compose: super::bar::compose, asleep: None },
    Composed {
        place: place::WALLPAPERS,
        compose: super::wallpapers::compose,
        asleep: Some(super::wallpapers::asleep),
    },
    Composed {
        place: place::COLOURS,
        compose: super::colours::compose,
        asleep: Some(super::colours::asleep),
    },
    Composed { place: place::LIBRARY, compose: super::library::compose, asleep: None },
    Composed { place: place::SOURCES, compose: super::sources::compose, asleep: None },
    Composed { place: place::AUTOMATION, compose: super::automation::compose, asleep: None },
    Composed { place: place::KEYBINDS, compose: super::keybinds::compose, asleep: None },
    Composed { place: place::SYSTEM, compose: super::system::compose, asleep: None },
];

const STRIPPED: [&str; 1] = [place::COLOURS];

struct PlaceSpec {
    key: &'static str,
    title: &'static str,
    note: &'static str,
    index_note: &'static str,
    sources: &'static [(&'static str, Cut)],
}

const PLACES: [PlaceSpec; 10] = [
    PlaceSpec {
        key: place::HOME,
        title: "settings-place-home",
        note: "settings-place-note-home",
        index_note: "settings-place-index-home",
        sources: &[],
    },
    PlaceSpec {
        key: place::LAYOUT,
        title: "settings-place-layout",
        note: "settings-place-index-layout",
        index_note: "settings-place-index-layout",
        sources: &[(category::PICKER, Cut::Layout)],
    },
    PlaceSpec {
        key: place::BAR,
        title: "settings-place-bar",
        note: "settings-place-note-bar",
        index_note: "settings-place-index-bar",
        sources: &[(category::PICKER, Cut::Bar)],
    },
    PlaceSpec {
        key: place::WALLPAPERS,
        title: "settings-place-wallpapers",
        note: "settings-place-note-wallpapers",
        index_note: "settings-tab-note-displays",
        sources: &[
            (category::DISPLAYS, Cut::Whole),
            (category::PLAYBACK, Cut::Whole),
            (category::TRANSITIONS, Cut::Whole),
        ],
    },
    PlaceSpec {
        key: place::COLOURS,
        title: "settings-tab-theme",
        note: "settings-place-note-colours",
        index_note: "settings-tab-note-theme",
        sources: &[(category::THEME, Cut::Whole), (category::INTEGRATIONS, Cut::Whole)],
    },
    PlaceSpec {
        key: place::LIBRARY,
        title: "settings-tab-library",
        note: "settings-place-note-library",
        index_note: "settings-place-index-library",
        sources: &[(category::LIBRARY, Cut::Whole)],
    },
    PlaceSpec {
        key: place::SOURCES,
        title: "settings-tab-sources",
        note: "settings-tab-note-sources",
        index_note: "settings-place-index-sources",
        sources: &[(category::SOURCES, Cut::Whole)],
    },
    PlaceSpec {
        key: place::AUTOMATION,
        title: "settings-tab-automation",
        note: "settings-place-note-automation",
        index_note: "settings-tab-note-automation",
        sources: &[(category::AUTOMATION, Cut::Whole)],
    },
    PlaceSpec {
        key: place::KEYBINDS,
        title: "settings-place-keybinds",
        note: "settings-place-note-keybinds",
        index_note: "settings-place-index-keybinds",
        sources: &[],
    },
    PlaceSpec {
        key: place::SYSTEM,
        title: "settings-place-system",
        note: "settings-place-note-system",
        index_note: "settings-place-index-system",
        sources: &[(category::APP, Cut::Whole)],
    },
];

pub const PLACE_COUNT: usize = PLACES.len();

pub fn place_keys() -> [&'static str; PLACE_COUNT] {
    PLACES.map(|spec| spec.key)
}

pub fn place_title(key: &str) -> &'static str {
    tr(spec(key).title)
}

#[cfg(test)]
pub(super) fn composed_places() -> impl Iterator<Item = &'static str> {
    COMPOSED.iter().map(|composed| composed.place)
}

fn composer(key: &str) -> Option<&'static Composed> {
    COMPOSED.iter().find(|composed| composed.place == key)
}

fn spec(key: &str) -> &'static PlaceSpec {
    PLACES.iter().find(|spec| spec.key == key).unwrap_or(&PLACES[0])
}

fn takes(cut: Cut, key: &str) -> bool {
    let bar = super::bar::CARDS.contains(&key) || super::bar::SECTIONS.contains(&key);
    match cut {
        Cut::Whole => true,
        Cut::Bar => bar,
        Cut::Layout => !bar,
    }
}

pub fn canonical_place(tab: &str) -> Option<&'static str> {
    if let Some(spec) = PLACES.iter().find(|spec| spec.key == tab) {
        return Some(spec.key);
    }
    let route = legacy_section(tab);
    let category = canonical_category(tab.to_string());
    PLACES
        .iter()
        .find(|spec| {
            spec.sources.iter().any(|(source, cut)| {
                *source == category && route.is_none_or(|key| takes(*cut, key))
            })
        })
        .map(|spec| spec.key)
}

pub fn legacy_section(tab: &str) -> Option<&'static str> {
    LEGACY_ROUTES.iter().find(|(old, _, _)| *old == tab).map(|(_, _, key)| *key)
}

#[derive(Clone, Copy)]
pub struct PlaceSource<'a> {
    pub cfg: &'a dyn SettingsSource,
    pub themes: &'a [String],
    pub folders: &'a [String],
    pub analysis: &'a str,
    pub backends: &'a [String],
    pub runtime: RuntimeStatus<'a>,
}

impl PlaceSource<'_> {
    pub fn cards(&self, category: &str) -> Vec<(Card, Vec<Row>)> {
        build_tab_with_runtime_status(
            category,
            self.cfg,
            self.themes,
            self.folders,
            self.analysis,
            self.backends,
            self.runtime,
        )
    }

    pub fn place(&self, key: &str) -> Place {
        let spec = spec(key);
        Place {
            key: spec.key,
            title: tr(spec.title),
            note: tr(spec.note),
            index_note: tr(spec.index_note),
            strip: STRIPPED.contains(&spec.key),
            sections: composer(spec.key).map_or_else(Vec::new, |composed| {
                (composed.compose)(self).into_iter().map(kept).collect()
            }),
        }
    }

    pub fn places(&self) -> Vec<Place> {
        PLACES
            .iter()
            .filter(|spec| spec.key != place::HOME)
            .map(|spec| self.place(spec.key))
            .collect()
    }

    pub(super) fn asleep(&self, key: &str, sections: &[Section]) -> Vec<Asleep> {
        let sleepers = composer(key).and_then(|composed| composed.asleep);
        sleepers.map_or_else(Vec::new, |sleepers| sleepers(sections, self))
    }
}

pub(super) struct Asleep {
    pub row: Row,
    pub wakes: &'static str,
}

fn groups(rows: Vec<Row>) -> Vec<Group> {
    let mut groups = vec![Group::new(GroupKind::Column, None, Vec::new())];
    for row in rows {
        let room = groups.len() < MAX_GROUPS;
        let current = groups.last_mut().expect("one group");
        if !matches!(row.control, Control::Segment) {
            current.rows.push(row);
        } else if current.rows.is_empty() && current.heading.is_none() {
            current.heading = Some(row.title);
        } else if room {
            groups.push(Group::new(GroupKind::Column, Some(row.title), Vec::new()));
        } else {
            current.rows.push(row);
        }
    }
    groups
}

pub fn tokenable(control: &Control) -> bool {
    match control {
        Control::Number { .. } | Control::Chips { .. } | Control::TextField { .. } => true,
        Control::Dropdown { palettes, .. } => palettes.is_empty(),
        _ => false,
    }
}

fn summary<'a>(titles: impl Iterator<Item = &'a str>) -> String {
    titles.filter(|title| !title.is_empty()).collect::<Vec<_>>().join(SUMMARY_JOINER)
}

fn say(section: &Section) -> Say {
    let titles =
        section.rows().enumerate().filter(|(_, row)| !matches!(row.control, Control::Segment));
    let lead = section.rows().find(|row| tokenable(&row.control)).and_then(|row| {
        let path = row.control.path()?;
        bind(section, LEAD_KEY, &[Bind::new(LEAD_VALUE, path)], &[(LEAD_SETTING, &row.title)])
    });
    let Some(mut say) = lead else {
        return Say { text: summary(titles.map(|(_, row)| row.title.as_str())), ..Say::default() };
    };
    let rest = summary(titles.filter(|(at, _)| !say.binds(*at)).map(|(_, row)| row.title.as_str()));
    if !rest.is_empty() {
        say.text = format!("{}{SUMMARY_JOINER}{rest}", say.text);
    }
    say
}

pub(super) fn legacy(id: SectionId, card: &Card, rows: Vec<Row>) -> Section {
    wrap(id, card.title, card.subtitle, rows)
}

fn wrap(id: SectionId, title: &str, desc: &str, rows: Vec<Row>) -> Section {
    let mut section = Section { groups: groups(rows), ..Section::new(id, title, desc) };
    section.say = say(&section);
    section
}

fn fallback(section: Section) -> Section {
    let (id, title, desc) = (section.id, section.title.clone(), section.desc.clone());
    wrap(id, &title, &desc, section.into_rows().collect())
}

fn kept(section: Section) -> Section {
    if drawn(&section) { section } else { fallback(section) }
}

pub(super) fn spoken(mut section: Section, say: impl FnOnce(&Section) -> Option<Say>) -> Section {
    let Some(say) = say(&section) else {
        return fallback(section);
    };
    section.say = say;
    section
}

#[derive(Clone, Copy)]
pub(super) struct Bind<'a> {
    pub arg: &'static str,
    pub path: &'a str,
    pub blank: Option<&'static str>,
    pub form: Form,
}

impl<'a> Bind<'a> {
    pub(super) const fn new(arg: &'static str, path: &'a str) -> Self {
        Self { arg, path, blank: None, form: Form::Plain }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Checks<'a> {
    pub arg: &'static str,
    pub paths: &'a [&'a str],
    pub heading: &'static str,
    pub blank: &'static str,
}

#[derive(Clone, Copy)]
pub(super) enum Spec<'a> {
    Value(Bind<'a>),
    Checks(Checks<'a>),
}

impl<'a> From<Bind<'a>> for Spec<'a> {
    fn from(bind: Bind<'a>) -> Self {
        Self::Value(bind)
    }
}

impl<'a> Spec<'a> {
    pub(super) fn arg(&self) -> &'static str {
        match self {
            Self::Value(bind) => bind.arg,
            Self::Checks(checks) => checks.arg,
        }
    }

    pub(super) fn paths(&self) -> &[&'a str] {
        match self {
            Self::Value(bind) => std::slice::from_ref(&bind.path),
            Self::Checks(checks) => checks.paths,
        }
    }
}

pub(super) const NO_SLOTS: [Bind<'static>; 0] = [];

pub(super) fn bind<'a>(
    section: &Section,
    key: &'static str,
    slots: &[impl Copy + Into<Spec<'a>>],
    args: &[(&'static str, &str)],
) -> Option<Say> {
    let mut fluent = marked();
    let mut tokens = Vec::new();
    let mut marks = Vec::new();
    let find = |path: &str| section.rows().position(|row| row.control.path() == Some(path));
    for (at, spec) in (0..).zip(slots) {
        let spec: Spec<'a> = (*spec).into();
        let mark = crate::frontend::ui::calm::sentinel(at)?;
        tokens.push(match spec {
            Spec::Value(bind) => Slot::Value {
                row: find(bind.path)?,
                blank: bind.blank.map(|key| tr(key).to_string()),
                form: bind.form,
            },
            Spec::Checks(checks) => Slot::Checks {
                rows: checks.paths.iter().map(|path| find(path)).collect::<Option<_>>()?,
                heading: tr(checks.heading).to_string(),
                blank: tr(checks.blank).to_string(),
            },
        });
        fluent.set(spec.arg(), mark.to_string());
        marks.push(mark);
    }
    for (arg, value) in args {
        fluent.set(*arg, *value);
    }
    let text = crate::i18n::format(key, &fluent);
    (spans_close(&text) && marks.iter().all(|mark| text.matches(*mark).count() == 1))
        .then_some(Say { text, tokens, ..Say::default() })
}

fn spans_close(text: &str) -> bool {
    let mut open = false;
    for ch in text.chars() {
        match ch {
            VALUE_OPEN if open => return false,
            VALUE_CLOSE if !open => return false,
            VALUE_OPEN | VALUE_CLOSE => open = !open,
            _ if open && crate::frontend::ui::calm::is_sentinel(ch) => return false,
            _ => {}
        }
    }
    !open
}

pub(super) fn marked() -> fluent::FluentArgs<'static> {
    let mut args = fluent::FluentArgs::new();
    args.set(VALUE_ARG, VALUE_OPEN.to_string());
    args.set(VALUE_END_ARG, VALUE_CLOSE.to_string());
    args
}

pub(super) struct Choice {
    pub arg: &'static str,
    pub values: [&'static str; 2],
}

impl Choice {
    pub(super) fn pick(&self, on: bool) -> (&'static str, &'static str) {
        (self.arg, self.values[usize::from(on)])
    }
}

pub(super) fn switched(
    section: &Section,
    off: &'static str,
    on: impl FnOnce() -> Option<Say>,
) -> Option<Say> {
    if !section.switched_off() {
        return on();
    }
    Some(Say { text: tr(off).to_string(), ..Say::default() })
}

pub(super) fn note(row: &Row) -> String {
    if row.title.is_empty() || row.desc.is_empty() {
        return format!("{}{}", row.title, row.desc);
    }
    lead(&row.title, &row.desc)
}

pub(super) fn lead(setting: &str, value: &str) -> String {
    crate::i18n::tr_args!(LEAD_KEY, setting => setting, value => value)
}

pub(super) fn indexed(base: &str, at: usize, leaf: &str) -> String {
    format!("{base}.{at}.{leaf}")
}

pub(super) fn column(heading: Option<&'static str>, rows: Vec<Row>) -> Group {
    Group::new(GroupKind::Column, heading.map(|key| tr(key).to_string()), rows)
}

pub(super) fn lone(id: SectionId, desc: &str, head: Vec<Row>) -> Section {
    Section { head, ..Section::new(id, tr(id.key), desc) }
}

pub(super) fn rows(pool: &mut Vec<Row>, table: &[(&str, Option<&'static str>)]) -> Vec<Row> {
    table
        .iter()
        .flat_map(|(path, hint)| noted(super::tabs::take_control_paths(pool, &[path]), *hint))
        .collect()
}

pub(super) fn noted(rows: Vec<Row>, hint: Option<&'static str>) -> Vec<Row> {
    let note = hint.map_or(Note::Quiet, |key| Note::Own(tr(key).to_string()));
    rows.into_iter()
        .map(|row| Row { look: Look { note: note.clone(), ..row.look }, ..row })
        .collect()
}

pub(super) fn heads<'a>(pool: &mut Vec<Row>, slots: &[impl Copy + Into<Spec<'a>>]) -> Vec<Row> {
    let paths: Vec<_> = slots
        .iter()
        .flat_map(|slot| {
            let spec: Spec<'a> = (*slot).into();
            spec.paths().iter().map(|path| (*path, None)).collect::<Vec<_>>()
        })
        .collect();
    rows(pool, &paths)
}

pub(super) fn looked(rows: Vec<Row>, cell: Cell) -> Vec<Row> {
    rows.into_iter().map(|row| Row { look: Look { cell, ..row.look }, ..row }).collect()
}

pub(super) fn checks(title: &str, caption: String, members: Vec<Row>) -> Vec<Row> {
    let lead = Row {
        look: Look { cell: Cell::Checks, note: Note::Quiet, ..Look::default() },
        ..Row::of(title.to_string(), caption, Control::Static)
    };
    std::iter::once(lead).chain(looked(noted(members, None), Cell::Joined)).collect()
}

pub(super) fn take(rows: &mut Vec<Row>, wanted: impl Fn(&Control) -> bool) -> Vec<Row> {
    let (taken, kept) = std::mem::take(rows).into_iter().partition(|row| wanted(&row.control));
    *rows = kept;
    taken
}

pub(super) fn unfold(rows: Vec<Row>, wanted: impl Fn(&str) -> bool) -> Vec<Row> {
    let mut flat = Vec::new();
    for row in rows {
        match row.control {
            Control::Details { id, rows, .. } if wanted(&id) => flat.extend(rows),
            control => flat.push(Row { control, ..row }),
        }
    }
    flat
}

pub(super) fn listed(key: &'static str, items: &[&str]) -> String {
    let mut args = fluent::FluentArgs::new();
    args.set(LIST_COUNT, LIST_COUNTS.get(items.len()).copied().unwrap_or(LIST_MANY));
    for (name, item) in LIST_NAMES.iter().zip(items) {
        args.set(*name, *item);
    }
    crate::i18n::format(key, &args)
}

pub(super) fn park(section: &mut Section, cause: &str, parked: impl Fn(&Row) -> bool) {
    let Some(cause) = section.rows().position(|row| row.control.path() == Some(cause)) else {
        return;
    };
    let lead = usize::from(section.master.is_some());
    let rows =
        section.head.iter().enumerate().filter(|(at, row)| lead + at != cause && parked(row));
    section.say.parked = rows.map(|(at, _)| Parked { row: lead + at, cause }).collect();
}

fn reachable(section: &Section, row: usize) -> bool {
    section.say.parks(row).is_some_and(|cause| {
        cause != row
            && ((cause == 0 && section.master.is_some())
                || section.say.binds(cause)
                || cause >= section.header_rows())
    })
}

pub fn drawn(section: &Section) -> bool {
    let lead = usize::from(section.master.is_some());
    section.master.as_ref().is_none_or(|row| row.control.switched_on().is_some())
        && section.head.iter().enumerate().all(|(at, row)| {
            matches!(row.control, Control::ActionBtn { .. })
                || section.say.binds(lead + at)
                || reachable(section, lead + at)
        })
}
