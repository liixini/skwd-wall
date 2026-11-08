#[cfg(test)]
#[path = "bar_tests.rs"]
mod tests;

use crate::contracts::settings::{SHOW_SORT_PREFIX, SHOW_TYPE_PREFIX, SettingsSource, keys};
use crate::i18n::{text, tr, tr_args};

use super::category::PICKER;
use super::places::{
    Bind, Choice, LIST_KEY, PlaceSource, bind, checks, column, heads, indexed, listed, looked,
    marked, noted, park, rows, spoken, take, unfold,
};
use super::tabs::{take_card, take_control_paths};
use super::{ActionId, Cell, Control, Group, GroupKind, Note, Row, Say, Section, SectionId};

use keys::filter_bar as bar;
use keys::semantic;

pub(super) const FILTER: &str = "settings-bar-filter";
pub(super) const SIZES: &str = "settings-bar-sizes";
pub(super) const START: &str = "settings-bar-start";
pub(super) const SEARCH: &str = "settings-tagging-search-card";
pub(super) const SECTIONS: [&str; 4] = [FILTER, SIZES, START, SEARCH];
pub(super) const CARDS: [&str; 8] = [
    "settings-section-tag-panel",
    "settings-section-appearance",
    "settings-section-buttons",
    "settings-section-resolution-presets",
    "settings-section-defaults",
    "settings-section-visibility",
    "settings-section-search-tagging",
    "settings-section-semantic-models",
];

const FILTER_SAY: &str = "settings-bar-filter-say";
const BUTTON_COUNT: &str = "settings-bar-button-count";
const SORT_COUNT: &str = "settings-bar-sort-count";
const OFFERS: &str = "settings-bar-offers";
const FLOATS_HINT: &str = "settings-bar-floats-hint";
const BEHAVES: &str = "settings-bar-behaves";
const BUTTONS: &str = "settings-bar-buttons";
const SORTS: &str = "settings-filter-sort-options-card";
const PINNED_HINT: &str = "settings-bar-pinned-hint";
const PLACED: &str = "settings-bar-placed";
const ARRANGE: &str = "arrange-title";
const SIZES_SAY: &str = "settings-bar-sizes-say";
const SIZES_NONE: &str = "settings-bar-sizes-none";
const UNNAMED: &str = "settings-bar-unnamed";
const FROM: &str = "settings-bar-from";
const UP_TO: &str = "settings-bar-up-to";
const ADD_HINT: &str = "settings-bar-add-hint";
const START_SAY: &str = "settings-bar-start-say";
const STICKY_HINT: &str = "settings-bar-sticky-hint";
const SEARCH_SAY: &str = "settings-bar-search-say";
const SEARCH_OFF: &str = "settings-bar-search-off";
const MODEL_STATUS: &str = "settings-bar-model-status";
const SEARCHING: &str = "settings-bar-searching";
const ACTIVE: &str = "settings-tagging-model-active-label";
const UNUSED: &str = "settings-bar-unused";
const NUMBERED: &str = "settings-tagging-model-numbered";
const PANEL: &str = "settings-section-tag-panel";
const NAME_HINT: &str = "settings-bar-name-hint";
const MANUAL_HINT: &str = "settings-bar-manual-hint";
const FORGET: &str = "settings-tagging-model-forget-action";
const FORGET_HINT: &str = "settings-bar-forget-hint";
const DELETE_NAMED: &str = "settings-bar-delete-named";
const FORGET_NAMED: &str = "settings-bar-forget-named";
const DELETE_HINT: &str = "settings-bar-delete-hint";
const INDEX_NOTES: [&str; 2] =
    ["settings-tagging-describe-info-desc", "settings-tagging-model-index-desc"];

const OTHER_BUTTONS: [&str; 12] = [
    bar::SHOW_FOLDER,
    bar::SHOW_FAVOURITES,
    bar::SHOW_RANDOM,
    bar::SHOW_COLORS,
    bar::SHOW_THEME,
    bar::SHOW_TAG_CLOUD,
    bar::SHOW_RESOLUTION,
    bar::SHOW_ORIENT,
    bar::SHOW_PLAYLISTS,
    bar::SHOW_COLOUR_STUDIO,
    bar::SHOW_WIDGETS,
    bar::SHOW_DOWNLOAD,
];
const ORIENTATIONS: [(&str, &str); 2] = [
    (ORIENTED.values[0], "settings-filter-orientation-horizontal"),
    (ORIENTED.values[1], "settings-filter-orientation-vertical"),
];
const MODES: [(&str, &str); 2] = [
    (MODE.values[0], "settings-tagging-mode-tags"),
    (MODE.values[1], "settings-tagging-mode-describe"),
];
const VIEWS: [(&str, &str); 2] = [
    (VIEWED.values[0], "settings-tagging-index-profile-full"),
    (VIEWED.values[1], "settings-tagging-index-profile-multiview"),
];
const CUSTOM_BAND: &str = "custom";
const BAND_LABEL: &str = "label";
const BAND_FROM: &str = "from";
pub(super) const BAND_TO: &str = "to";
const BAND_ORIENTATION: &str = "orientation";
const MODEL_NAME: &str = "name";
const MODEL_MANIFEST: &str = "manifest";

const BUTTONS_ARG: &str = "buttons";
const SORTS_ARG: &str = "sorts";
const STATUS_ARG: &str = "status";
const STYLE_SLOT: Bind<'_> = Bind::new("style", bar::VISUAL_STYLE);
const FOLDER_SLOT: Bind<'_> = Bind::new("folder", bar::DEFAULT_FOLDER);
const MODEL_SLOT: Bind<'_> = Bind::new("model", semantic::MANIFEST);
const ORIENTED: Choice = Choice { arg: "orientation", values: ["horizontal", "vertical"] };
const PINNED: Choice = Choice { arg: "pinned", values: ["no", "yes"] };
const STICKY: Choice = Choice { arg: "sticky", values: ["no", "yes"] };
const MODE: Choice = Choice { arg: "mode", values: ["tags", "describe"] };
const VIEWED: Choice = Choice { arg: "views", values: ["full", "multiview"] };

pub(super) fn compose(source: &PlaceSource<'_>) -> Vec<Section> {
    let cfg = source.cfg;
    let mut cards = source.cards(PICKER);
    let mut pool = Vec::new();
    for card in CARDS {
        pool.extend(take_card(&mut cards, text(card)));
    }
    let mut pool = unfold(unfold(pool, |_| true), |_| true);
    vec![
        filter(cfg, &mut pool),
        sizes(cfg, &mut pool),
        start(cfg, &mut pool),
        search(source, &mut pool),
    ]
}

fn count(key: &'static str, members: &[Row]) -> String {
    let shown = members.iter().filter(|row| row.control.switched_on() == Some(true)).count();
    tr_args!(key, shown => shown, all => members.len())
}

fn prefixed(prefix: &'static str) -> impl Fn(&Control) -> bool {
    move |control| control.path().is_some_and(|path| path.starts_with(prefix))
}

fn action(pool: &mut Vec<Row>, wanted: impl Fn(&ActionId) -> bool) -> Vec<Row> {
    take(pool, |control| matches!(control, Control::ActionBtn { id, .. } if wanted(id)))
}

fn filter(cfg: &dyn SettingsSource, pool: &mut Vec<Row>) -> Section {
    let head = heads(pool, &[STYLE_SLOT]);
    let mut buttons = take(pool, prefixed(SHOW_TYPE_PREFIX));
    buttons.extend(rows(pool, &OTHER_BUTTONS.map(|path| (path, None))));
    let sorts = take(pool, prefixed(SHOW_SORT_PREFIX));
    let counts = [count(BUTTON_COUNT, &buttons), count(SORT_COUNT, &sorts)];
    let mut offers = checks(tr(BUTTONS), String::new(), buttons);
    offers.extend(checks(tr(SORTS), String::new(), sorts));
    offers.extend(rows(pool, &[(bar::BUTTON_ORDER, None), (bar::HIDDEN_BUTTONS, None)]));
    offers.extend(rows(
        pool,
        &[(bar::SHOW_ARRANGE, Some(FLOATS_HINT)), (bar::SHOW_SWATCH, Some(FLOATS_HINT))],
    ));
    let mut behaves = looked(rows(pool, &[(bar::ORIENTATION, None)]), Cell::Buttons(&ORIENTATIONS));
    behaves.extend(rows(pool, &[(keys::general::FILTER_BAR_ALWAYS_VISIBLE, Some(PINNED_HINT))]));
    for mut arrange in action(pool, |id| *id == ActionId::ArrangePicker) {
        arrange.title = tr(ARRANGE).to_string();
        behaves.extend(noted(vec![arrange], None));
    }
    let mut panel = rows(pool, &[(keys::selector::TAG_CLOUD_WIDTH, None)]);
    for row in &mut panel {
        row.title = tr(PANEL).to_string();
    }
    panel.extend(looked(rows(pool, &[(keys::selector::TAG_CLOUD_ROWS, None)]), Cell::Beside));
    behaves.extend(panel);
    let section = Section {
        head,
        groups: vec![column(Some(OFFERS), offers), column(Some(BEHAVES), behaves)],
        foot: Some(tr(PLACED).to_string()),
        ..Section::new(SectionId::new(PICKER, FILTER), tr(FILTER), tr("settings-bar-filter-desc"))
    };
    let vertical = cfg.text(bar::ORIENTATION) == ORIENTED.values[1];
    let args = [
        (BUTTONS_ARG, counts[0].as_str()),
        (SORTS_ARG, counts[1].as_str()),
        ORIENTED.pick(vertical),
        PINNED.pick(cfg.flag(keys::general::FILTER_BAR_ALWAYS_VISIBLE)),
    ];
    spoken(section, |section| bind(section, FILTER_SAY, &[STYLE_SLOT], &args))
}

fn band(pool: &mut Vec<Row>, at: usize) -> Vec<Row> {
    let path = |leaf: &str| indexed(bar::RESOLUTION_PRESETS, at, leaf);
    let mut band = take_control_paths(pool, &[&path(BAND_LABEL)]);
    for (leaf, title) in [(BAND_FROM, FROM), (BAND_TO, UP_TO)] {
        for mut limit in take_control_paths(pool, &[&path(leaf)]) {
            limit.title = tr(title).to_string();
            band.push(limit);
        }
    }
    for mut side in take_control_paths(pool, &[&path(BAND_ORIENTATION)]) {
        if let Control::Chips { current, options, .. } = &mut side.control
            && current.is_empty()
        {
            current.clone_from(&options[0].0);
        }
        band.push(side);
    }
    band.extend(action(pool, |id| *id == ActionId::RemoveResolutionPreset(at as u16)));
    noted(band, None)
}

fn sizes(cfg: &dyn SettingsSource, pool: &mut Vec<Row>) -> Section {
    let count = cfg.array_len(bar::RESOLUTION_PRESETS);
    let names: Vec<String> = (0..count)
        .map(|at| cfg.text(&indexed(bar::RESOLUTION_PRESETS, at, BAND_LABEL)))
        .map(|name| if name.trim().is_empty() { tr(UNNAMED).to_string() } else { name })
        .collect();
    let bands = (0..count).flat_map(|at| band(pool, at)).collect();
    let mut add = take(pool, |control| matches!(control, Control::ActionChips { .. }));
    for row in &mut add {
        if let Control::ActionChips { items } = &mut row.control {
            items.retain(|(id, label)| {
                !matches!(id, ActionId::CreateResolutionPresetBand(band) if *band != CUSTOM_BAND)
                    || !names.contains(label)
            });
        }
    }
    let add = looked(noted(add, Some(ADD_HINT)), Cell::Menu);
    let lines =
        Group { blank: Some(tr(UNNAMED).to_string()), ..Group::new(GroupKind::Lines, None, bands) };
    let section = Section {
        groups: vec![lines, column(None, add)],
        ..Section::new(SectionId::new(PICKER, SIZES), tr(SIZES), tr("settings-bar-sizes-desc"))
    };
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let text = if names.is_empty() {
        tr(SIZES_NONE).to_string()
    } else {
        tr_args!(SIZES_SAY, bands => listed(LIST_KEY, &names))
    };
    spoken(section, |_| Some(Say { text, ..Say::default() }))
}

fn start(cfg: &dyn SettingsSource, pool: &mut Vec<Row>) -> Section {
    let section = Section {
        head: heads(pool, &[FOLDER_SLOT]),
        groups: vec![column(None, rows(pool, &[(bar::STICKY, Some(STICKY_HINT))]))],
        ..Section::new(SectionId::new(PICKER, START), tr(START), tr("settings-bar-start-desc"))
    };
    let sticky = STICKY.pick(cfg.flag(bar::STICKY));
    spoken(section, |section| bind(section, START_SAY, &[FOLDER_SLOT], &[sticky]))
}

fn model_rows(pool: &mut Vec<Row>, at: usize) -> Vec<Row> {
    let path = |leaf: &str| indexed(semantic::MODELS, at, leaf);
    take_control_paths(pool, &[&path(MODEL_NAME), &path(MODEL_MANIFEST)])
}

fn named(mut rows: Vec<Row>, key: &'static str, model: &str) -> Vec<Row> {
    for row in &mut rows {
        if let Control::ActionBtn { label, .. } = &mut row.control {
            *label = tr_args!(key, name => model);
        }
    }
    rows
}

fn active(cfg: &dyn SettingsSource, pool: &mut Vec<Row>) -> Vec<Row> {
    let selected = cfg.text(semantic::MANIFEST);
    let manifest = |at: usize| cfg.text(&indexed(semantic::MODELS, at, MODEL_MANIFEST));
    let count = cfg.array_len(semantic::MODELS);
    let current = (0..count).find(|at| !selected.is_empty() && manifest(*at) == selected);
    let mut chosen = Vec::new();
    let mut others = Vec::new();
    let name = |at: usize| {
        let name = cfg.text(&indexed(semantic::MODELS, at, MODEL_NAME));
        if name.trim().is_empty() { tr_args!(NUMBERED, index => at + 1) } else { name }
    };
    for at in 0..count {
        let index = at as u16;
        let mut entry = noted(model_rows(pool, at), None);
        if Some(at) == current {
            if let Some(field) = entry.first_mut() {
                field.look.note = Note::Own(tr(NAME_HINT).to_string());
            }
            chosen.extend(entry);
            action(pool, |id| *id == ActionId::DeleteSemanticModel(index));
            for mut forget in action(pool, |id| *id == ActionId::RemoveSemanticModel(index)) {
                forget.title = tr(FORGET).to_string();
                forget.look.note = Note::Own(tr(FORGET_HINT).to_string());
                chosen.push(forget);
            }
        } else {
            let model = name(at);
            let delete = action(pool, |id| *id == ActionId::DeleteSemanticModel(index));
            let mut removal = named(delete, DELETE_NAMED, &model);
            let forget = action(pool, |id| *id == ActionId::RemoveSemanticModel(index));
            removal.extend(looked(named(forget, FORGET_NAMED, &model), Cell::Joined));
            if let Some(lead) = removal.first_mut() {
                lead.title = model;
            }
            others.extend(noted(removal, Some(UNUSED)));
        }
    }
    let delete = action(pool, |id| *id == ActionId::DeleteActiveSemanticModel);
    chosen.extend(noted(delete, Some(DELETE_HINT)));
    chosen.extend(others);
    chosen
}

fn search(source: &PlaceSource<'_>, pool: &mut Vec<Row>) -> Section {
    let cfg = source.cfg;
    let master = take_control_paths(pool, &[semantic::ENABLED]).pop();
    let head = heads(pool, &[MODEL_SLOT]);
    let mut searching =
        looked(rows(pool, &[(keys::tagging::DEFAULT_SEARCH_MODE, None)]), Cell::Buttons(&MODES));
    searching.extend(looked(rows(pool, &[(semantic::INDEX_PROFILE, None)]), Cell::Buttons(&VIEWS)));
    searching.extend(noted(action(pool, |id| *id == ActionId::ImportSemanticModel), None));
    searching
        .extend(noted(action(pool, |id| *id == ActionId::AddSemanticModel), Some(MANUAL_HINT)));
    let chosen = active(cfg, pool);
    let section = Section {
        master,
        head,
        groups: vec![column(Some(SEARCHING), searching), column(Some(ACTIVE), chosen)],
        foot: Some(INDEX_NOTES.map(tr).join(" ")),
        ..Section::new(SectionId::new(PICKER, SEARCH), tr(SEARCH), tr("settings-bar-search-desc"))
    };
    let describe = MODE.pick(cfg.text(keys::tagging::DEFAULT_SEARCH_MODE) == MODE.values[1]);
    let multiview = VIEWED.pick(cfg.text(semantic::INDEX_PROFILE) == VIEWED.values[1]);
    let mut section = spoken(section, |section| {
        if section.switched_off() {
            return Some(Say { text: tr_args!(SEARCH_OFF, mode => describe.1), ..Say::default() });
        }
        bind(section, SEARCH_SAY, &[MODEL_SLOT], &[describe, multiview])
    });
    if section.switched_off() {
        park(&mut section, semantic::ENABLED, |_| true);
    }
    if !source.analysis.trim().is_empty() {
        let mut args = marked();
        args.set(STATUS_ARG, source.analysis.trim());
        section.say.text.push_str(&crate::i18n::format(MODEL_STATUS, &args));
    }
    section
}
