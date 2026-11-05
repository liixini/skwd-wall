#[cfg(test)]
#[path = "keybinds_tests.rs"]
mod tests;

use crate::contracts::picker::{KEY_BINDINGS, KeyBindingGroup};
use crate::domain::input::{InputAction, InputMap, NavKeys, Trigger, slot_triggers};
use crate::i18n::{text, tr};

use super::category::APP;
use super::places::{PlaceSource, take};
use super::system::CONTROLS;
use super::tabs::{take_card, take_control_paths};
use super::{ActionId, Control, Form, Parked, Row, Say, Section, SectionId, Slot};

pub const KEYS_SHEET: SectionId = SectionId::new(APP, CONTROLS);

pub(super) const GROUPS_PER_COLUMN: usize = 2;
pub(super) const SHEET_GROUPS: [KeyBindingGroup; 6] = [
    KeyBindingGroup::Wallpaper,
    KeyBindingGroup::Folders,
    KeyBindingGroup::Filters,
    KeyBindingGroup::Panels,
    KeyBindingGroup::Navigation,
    KeyBindingGroup::Downloads,
];
const NAV_ORDER: [InputAction; 4] =
    [InputAction::NavLeft, InputAction::NavRight, InputAction::NavUp, InputAction::NavDown];
const NO_KEY: &str = "-";
const NAV_JOINER: &str = " ";
const FIND_ID: &str = "settings-keys-find";
const SECOND_TOKEN: &str = "settings-keys-second";

const DESC: &str = "settings-keybinds-controls-card-desc";
const UNDO: &str = "settings-paths-undo-action";
const CONFLICT: &str = "settings-keybinds-conflict-tag";

#[derive(Clone, Copy, Default)]
pub struct KeysCtx<'a> {
    pub capture: Option<(&'a str, usize)>,
    pub query: &'a str,
    pub moved: Option<(&'a str, &'a [&'static str])>,
    pub same: Option<&'a str>,
    pub lit: Option<&'a str>,
}

pub fn keys_find_id() -> iced::widget::Id {
    iced::widget::Id::new(FIND_ID)
}

pub fn second_keys_token() -> u64 {
    super::token_key(SECOND_TOKEN, 0, super::TokenSide::Body)
}

pub fn second_keys_row(control: &Control) -> bool {
    matches!(control, Control::ActionChips { items }
        if matches!(items.first(), Some((ActionId::SetNavKeys(_), _))))
}

pub fn links_control(control: &Control) -> bool {
    matches!(control, Control::ActionChips { items }
        if matches!(items.first(), Some((ActionId::LightKey(_), _))))
}

pub(super) fn compose(source: &PlaceSource<'_>) -> Vec<Section> {
    let mut cards = source.cards(APP);
    let mut pool = take_card(&mut cards, text(CONTROLS));
    let mut rows = take(&mut pool, |control| matches!(control, Control::ActionChips { .. }));
    rows.extend(
        take(&mut pool, |control| matches!(control, Control::ActionBtn { .. })).into_iter().map(
            |row| match row.control {
                Control::ActionBtn { id, .. } => {
                    let control = Control::ActionBtn { id, label: row.title.clone() };
                    Row { control, ..row }
                }
                control => Row { control, ..row },
            },
        ),
    );
    for group in SHEET_GROUPS {
        let (title, desc) = (tr(group.title_key()), tr(group.description_key()));
        rows.push(Row::of(title.to_string(), desc.to_string(), Control::Segment));
        for entry in KEY_BINDINGS.into_iter().filter(|entry| entry.group == group) {
            rows.extend(take_control_paths(&mut pool, &[entry.path]));
        }
    }
    rows.extend(links_row(&source.cfg.bindings()));
    rows.extend(take(&mut pool, |control| matches!(control, Control::Static)));
    let tokens =
        (0..rows.len()).map(|row| Slot::Value { row, blank: None, form: Form::Plain }).collect();
    let parked = rows
        .iter()
        .enumerate()
        .filter(|(_, row)| matches!(row.control, Control::Static))
        .map(|(row, _)| Parked { row, cause: 0 })
        .collect();
    vec![Section {
        head: rows,
        say: Say { tokens, parked, ..Say::default() },
        ..Section::new(KEYS_SHEET, tr(CONTROLS), tr(DESC))
    }]
}

pub fn undo_row() -> Row {
    let label = tr(UNDO).to_string();
    Row::of(label.clone(), String::new(), Control::ActionBtn { id: ActionId::UndoKeyMove, label })
}

fn links_row(map: &InputMap) -> Option<Row> {
    let (first, second) = *map.conflicts().first()?;
    let link = |action| {
        KEY_BINDINGS
            .into_iter()
            .find(|entry| entry.action == action)
            .map(|entry| (ActionId::LightKey(entry.path), tr(entry.title_key).to_string()))
    };
    let items = vec![link(first)?, link(second)?];
    Some(Row::of(tr(CONFLICT).to_string(), String::new(), Control::ActionChips { items }))
}

pub fn conflict_links(section: &Section, at: usize) -> bool {
    section.row(at).is_some_and(|row| links_control(&row.control))
}

pub(super) fn second_keys(map: &InputMap) -> Option<NavKeys> {
    NavKeys::ALL.into_iter().find(|keys| {
        keys.second_keys().into_iter().all(|(action, key)| {
            Trigger::parse(key).is_some_and(|trigger| {
                slot_triggers(map.triggers(action), 1) == std::slice::from_ref(&trigger)
            })
        })
    })
}

pub(super) fn second_summary(map: &InputMap) -> String {
    NAV_ORDER
        .map(|action| {
            let shown = slot_triggers(map.triggers(action), 1);
            if shown.is_empty() {
                NO_KEY.to_string()
            } else {
                crate::domain::input::binding_label(shown)
            }
        })
        .join(NAV_JOINER)
}

pub(super) fn matches_query(query: &str, title: &str, labels: &[String]) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty()
        || std::iter::once(title)
            .chain(labels.iter().map(String::as_str))
            .any(|text| text.to_lowercase().contains(&query))
}

pub(super) fn shared_key(map: &InputMap) -> Option<Trigger> {
    let (first, second) = *map.conflicts().first()?;
    let other = map.triggers(second);
    map.triggers(first).iter().find(|trigger| other.contains(trigger)).cloned()
}
