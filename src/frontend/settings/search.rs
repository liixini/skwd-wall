use crate::contracts::settings::SettingsSource;
use crate::i18n::tr;

use super::tables::category_label;
use super::{Control, Place, PlaceSource, Row, Section};

const SECTION_MATCH: u32 = 3;
const INSIDE_WORD_PENALTY: u32 = 6;
const RESULT_LIMIT: usize = 7;

#[derive(Debug, Clone)]
pub struct SettingsSearchResult {
    pub tab: String,
    pub tab_label: String,
    pub section: usize,
    pub section_title: String,
    pub row: usize,
    pub title: String,
    pub desc: String,
    pub value: String,
    pub setting: Option<Row>,
    pub home: Option<Section>,
    pub wake: Option<String>,
    pub elsewhere: bool,
    score: u32,
    hidden: bool,
}

impl SettingsSearchResult {
    pub fn live(&self) -> Option<&Row> {
        self.setting.as_ref().filter(|row| self.wake.is_none() && changes_here(&row.control))
    }

    fn move_to(&mut self, place: &Place, section: usize, row: usize) {
        self.tab = place.key.to_string();
        self.tab_label = place.title.to_string();
        self.section_title.clone_from(&place.sections[section].title);
        self.section = section;
        self.row = row;
    }

    pub fn is(&self, other: &Self) -> bool {
        match (&self.setting, &other.setting) {
            (Some(_), Some(row)) => self.same(row),
            (None, None) => {
                self.tab == other.tab
                    && self.section_title == other.section_title
                    && self.title == other.title
            }
            _ => false,
        }
    }

    fn same(&self, row: &Row) -> bool {
        self.setting.as_ref().is_some_and(|setting| match setting.control.path() {
            Some(path) => row.control.path() == Some(path),
            None => row.title == setting.title && row.desc == setting.desc,
        })
    }
}

fn changes_here(control: &Control) -> bool {
    super::tokenable(control)
        || matches!(
            control,
            Control::Toggle { .. }
                | Control::ToggleAction { .. }
                | Control::Colour { .. }
                | Control::Resolution { .. }
                | Control::ActionBtn { .. }
                | Control::ActionChips { .. }
        )
}

#[derive(Debug, Default)]
pub struct SearchHits {
    pub shown: Vec<SettingsSearchResult>,
    pub total: usize,
}

fn origin_label(section: &Section) -> &'static str {
    category_label(section.id.category)
}

#[cfg(test)]
pub fn search_settings(query: &str, source: PlaceSource<'_>) -> Vec<SettingsSearchResult> {
    search_hits(query, source).shown
}

pub fn search_hits(query: &str, source: PlaceSource<'_>) -> SearchHits {
    let places = source.places();
    let mut shown = scan(query, &places, source);
    let total = shown.len();
    shown.truncate(RESULT_LIMIT);
    for result in &mut shown {
        settle(result, &places, source.cfg);
    }
    SearchHits { shown, total }
}

pub fn reread_settings(results: &mut Vec<SettingsSearchResult>, source: PlaceSource<'_>) -> bool {
    let places = source.places();
    let before = results.len();
    results.retain_mut(|result| reread(result, &places, source));
    results.len() == before
}

fn reread(result: &mut SettingsSearchResult, places: &[Place], source: PlaceSource<'_>) -> bool {
    if result.setting.is_none() {
        return true;
    }
    let shown = places.iter().find_map(|place| {
        place.sections.iter().enumerate().find_map(|(at, section)| {
            let row = section.rows().position(|row| result.same(row))?;
            Some((place, at, row))
        })
    });
    if let Some((place, section, row)) = shown {
        result.move_to(place, section, row);
        result.wake = None;
        result.setting = None;
        settle(result, places, source.cfg);
        return true;
    }
    let asleep = places
        .iter()
        .flat_map(|place| source.asleep(place.key, &place.sections))
        .find(|asleep| result.same(&asleep.row));
    let Some(asleep) = asleep else {
        return false;
    };
    let Some(landed) = landing(places, asleep.wakes) else {
        return false;
    };
    result.move_to(landed.place, landed.section, landed.row);
    result.wake = Some(landed.title());
    result.value = control_value(&asleep.row.control, source.cfg);
    result.setting = Some(asleep.row);
    result.home = None;
    true
}

fn settle(result: &mut SettingsSearchResult, places: &[Place], cfg: &dyn SettingsSource) {
    if result.wake.is_some() {
        return;
    }
    let home = places
        .iter()
        .find(|place| place.key == result.tab)
        .and_then(|place| place.sections.get(result.section));
    let Some(home) = home else {
        return;
    };
    result.setting = home.row(result.row).cloned();
    if let Some(row) = &result.setting {
        result.value = control_value(&row.control, cfg);
    }
    result.home = Some(home.clone());
}

fn scan(query: &str, places: &[Place], source: PlaceSource<'_>) -> Vec<SettingsSearchResult> {
    let query = normalized(query);
    let terms: Vec<&str> = query.split_whitespace().collect();
    if terms.is_empty() {
        return Vec::new();
    }
    let cfg = source.cfg;
    let mut results = Vec::new();
    for place in places {
        let tab_label = place.title;
        for (section_index, section) in place.sections.iter().enumerate() {
            let tab_text = normalized(&format!("{tab_label} {}", origin_label(section)));
            if section.rows().next().is_none() {
                results.extend(page_result(
                    &query,
                    &terms,
                    place.key,
                    &tab_text,
                    section_index,
                    section,
                ));
                continue;
            }
            for (row, setting) in section.rows().enumerate() {
                let spot = Spot {
                    tab: place.key,
                    tab_label,
                    tab_text: &tab_text,
                    section: section_index,
                    section_title: &section.title,
                    row,
                    hidden: section.say.parks(row).is_some(),
                    wake: None,
                };
                results.extend(row_result(&query, &terms, &spot, setting, cfg));
            }
        }
        for asleep in source.asleep(place.key, &place.sections) {
            let Some(landed) = landing(places, asleep.wakes) else {
                continue;
            };
            let tab_text = normalized(landed.place.title);
            let wake = landed.title();
            let spot = Spot {
                tab: landed.place.key,
                tab_label: landed.place.title,
                tab_text: &tab_text,
                section: landed.section,
                section_title: &landed.place.sections[landed.section].title,
                row: landed.row,
                hidden: true,
                wake: Some(&wake),
            };
            results.extend(row_result(&query, &terms, &spot, &asleep.row, cfg));
        }
    }
    rank(&mut results);
    results
}

fn rank(results: &mut [SettingsSearchResult]) {
    results.sort_by(|left, right| {
        left.score
            .cmp(&right.score)
            .then_with(|| left.hidden.cmp(&right.hidden))
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.tab_label.cmp(&right.tab_label))
    });
}

struct Spot<'a> {
    tab: &'a str,
    tab_label: &'a str,
    tab_text: &'a str,
    section: usize,
    section_title: &'a str,
    row: usize,
    hidden: bool,
    wake: Option<&'a str>,
}

struct Landing<'a> {
    place: &'a Place,
    section: usize,
    row: usize,
}

impl Landing<'_> {
    fn title(&self) -> String {
        let section = &self.place.sections[self.section];
        section
            .row(self.row)
            .map(|row| row.title.as_str())
            .filter(|title| !title.is_empty())
            .unwrap_or(&section.title)
            .to_string()
    }
}

fn landing<'a>(places: &'a [Place], wakes: &str) -> Option<Landing<'a>> {
    places.iter().find_map(|place| {
        place.sections.iter().enumerate().find_map(|(section, candidate)| {
            let row = candidate.rows().position(|row| row.control.path() == Some(wakes))?;
            Some(Landing { place, section, row })
        })
    })
}

fn row_result(
    query: &str,
    terms: &[&str],
    spot: &Spot<'_>,
    setting: &Row,
    cfg: &dyn SettingsSource,
) -> Option<SettingsSearchResult> {
    if matches!(setting.control, Control::Segment | Control::Preview) {
        return None;
    }
    let section_title = normalized(spot.section_title);
    let tab_text = spot.tab_text;
    let title = normalized(&setting.title);
    let desc = normalized(&setting.desc);
    let control = normalized(&control_search_text(&setting.control));
    let named = format!("{title} {section_title} {tab_text}");
    let mut haystack = format!("{named} {desc} {control}");
    add_aliases(&mut haystack);
    if !terms.iter().all(|term| haystack.contains(term)) {
        return None;
    }
    let score = if title == query {
        0
    } else if title.starts_with(query) {
        1
    } else if title.contains(query) {
        2
    } else if section_title.contains(query) {
        SECTION_MATCH
    } else if tab_text.contains(query) {
        4
    } else {
        5
    };
    let whole_words =
        terms.iter().all(|term| haystack.split_whitespace().any(|word| word.starts_with(term)));
    let score = score + if whole_words { 0 } else { INSIDE_WORD_PENALTY };
    Some(SettingsSearchResult {
        tab: spot.tab.to_string(),
        tab_label: spot.tab_label.to_string(),
        section: spot.section,
        section_title: spot.section_title.to_string(),
        row: spot.row,
        title: setting.look.own_title.clone().unwrap_or_else(|| {
            if setting.title.is_empty() {
                crate::i18n::tr("settings-selector-saved-styles-label").to_string()
            } else {
                setting.title.clone()
            }
        }),
        desc: setting.desc.clone(),
        value: spot.wake.map_or_else(String::new, |_| control_value(&setting.control, cfg)),
        setting: spot.wake.map(|_| setting.clone()),
        home: None,
        wake: spot.wake.map(str::to_string),
        elsewhere: !terms.iter().all(|term| named.contains(term)),
        score,
        hidden: spot.hidden,
    })
}

fn page_result(
    query: &str,
    terms: &[&str],
    tab: &str,
    tab_text: &str,
    section_index: usize,
    section: &Section,
) -> Option<SettingsSearchResult> {
    let section_title = normalized(&section.title);
    let named = format!("{section_title} {tab_text}");
    let mut haystack = format!("{named} {}", normalized(&section.desc));
    add_aliases(&mut haystack);
    if !terms.iter().all(|term| haystack.contains(term)) {
        return None;
    }
    let score = if section_title == query {
        0
    } else if section_title.starts_with(query) {
        1
    } else {
        SECTION_MATCH
    };
    Some(SettingsSearchResult {
        tab: tab.to_string(),
        tab_label: super::place_title(tab).to_string(),
        section: section_index,
        section_title: section.title.clone(),
        row: 0,
        title: section.title.clone(),
        desc: section.desc.clone(),
        value: String::new(),
        setting: None,
        home: None,
        wake: None,
        elsewhere: !terms.iter().all(|term| named.contains(term)),
        score,
        hidden: false,
    })
}

fn normalized(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 8);
    let mut previous_lower = false;
    for character in value.chars() {
        if character.is_uppercase() && previous_lower {
            out.push(' ');
        }
        if character.is_alphanumeric() {
            match character {
                'İ' | 'ı' => out.push('i'),
                _ => out.extend(character.to_lowercase()),
            }
            previous_lower = character.is_lowercase() || character.is_numeric();
        } else {
            if !out.ends_with(' ') {
                out.push(' ');
            }
            previous_lower = false;
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn add_aliases(haystack: &mut String) {
    for (needle, aliases) in [
        (tr("settings-search-alias-fps"), tr("settings-search-alias-fps-terms")),
        (tr("settings-search-alias-battery"), tr("settings-search-alias-battery-terms")),
        (tr("settings-search-alias-gpu"), tr("settings-search-alias-gpu-terms")),
        (tr("settings-search-alias-cpu"), tr("settings-search-alias-cpu-terms")),
        (tr("settings-search-alias-we"), tr("settings-search-alias-we-terms")),
        (tr("settings-search-alias-monitor"), tr("settings-search-alias-monitor-terms")),
        (tr("settings-search-alias-transition"), tr("settings-search-alias-transition-terms")),
        (tr("settings-search-alias-path"), tr("settings-search-alias-path-terms")),
    ] {
        if haystack.contains(needle) {
            haystack.push(' ');
            haystack.push_str(aliases);
        }
    }
}

fn control_search_text(control: &Control) -> String {
    match control {
        Control::Display(control) => format!("{} {}", control.label(), control.summary()),
        Control::Colour { path, hex } => format!("{path} {hex}"),
        Control::Toggle { path, value } => format!(
            "{path} {} {}",
            tr("settings-search-control-toggle"),
            if *value {
                tr("settings-search-control-toggle-on")
            } else {
                tr("settings-search-control-toggle-off")
            }
        ),
        Control::ToggleAction { id, value } => format!(
            "{id:?} {} {}",
            tr("settings-search-control-toggle"),
            if *value {
                tr("settings-search-control-toggle-on")
            } else {
                tr("settings-search-control-toggle-off")
            }
        ),
        Control::Number { key, path, unit } => {
            format!("{key} {path} {unit} {}", tr("settings-search-control-number"))
        }
        Control::TextField { key, path, placeholder, .. } => {
            format!("{key} {path} {placeholder} {}", tr("settings-search-control-text"))
        }
        Control::Resolution { key, path, width_placeholder, height_placeholder } => format!(
            "{key} {path} {width_placeholder} {height_placeholder} {}",
            tr("settings-search-control-number")
        ),
        Control::KeyBinding { key, path, default } => {
            format!("{key} {path} {default} {}", tr("settings-search-control-text"))
        }
        Control::Dropdown { path, options, current, .. } => format!(
            "{path} {current} {} {}",
            options
                .iter()
                .flat_map(|(key, label)| [key.as_str(), label.as_str()])
                .collect::<Vec<_>>()
                .join(" "),
            tr("settings-search-control-dropdown")
        ),
        Control::Chips { path, options, current, .. } => format!(
            "{path} {current} {} {}",
            options
                .iter()
                .flat_map(|(key, label)| [key.as_str(), label.as_str()])
                .collect::<Vec<_>>()
                .join(" "),
            tr("settings-search-control-chips")
        ),
        Control::ActionBtn { label, .. } => {
            format!("{label} {}", tr("settings-search-control-action"))
        }
        Control::ActionChips { items } => format!(
            "{} {}",
            items.iter().map(|(_, label)| label.as_str()).collect::<Vec<_>>().join(" "),
            tr("settings-search-control-action")
        ),
        Control::Details { summary, rows, .. } | Control::StackBar { summary, rows, .. } => {
            format!(
                "{summary} {}",
                rows.iter()
                    .map(|row| format!(
                        "{} {} {}",
                        row.title,
                        row.desc,
                        control_search_text(&row.control)
                    ))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        }
        Control::AppTheme { app, .. } => format!("{} {} {}", app.name, app.state, app.detail),
        Control::Segment => String::new(),
        Control::Static => String::from(tr("settings-search-control-static")),
        Control::Code { snippet } => {
            format!("{snippet} {}", tr("settings-search-control-code"))
        }
        Control::Preview => String::from(tr("settings-search-control-preview")),
    }
}

pub(super) fn control_value(control: &Control, cfg: &dyn SettingsSource) -> String {
    match control {
        Control::Display(control) => format!("{} {}", control.label(), control.summary()),
        Control::Colour { hex, .. } => hex.clone(),
        Control::Toggle { value, .. } => {
            if *value {
                String::from(tr("settings-control-enabled"))
            } else {
                String::from(tr("settings-control-disabled"))
            }
        }
        Control::ToggleAction { value, .. } => String::from(if *value {
            tr("settings-app-themes-on")
        } else {
            tr("settings-app-themes-off")
        }),
        Control::Number { path, unit, .. } => {
            if path == crate::contracts::settings::keys::transition::FPS && cfg.number(path) == 0.0
            {
                return tr("settings-transitions-rate-auto").to_string();
            }
            let value = crate::contracts::picker::format_config_number(cfg.number(path));
            if unit.is_empty() { value } else { format!("{value} {unit}") }
        }
        Control::TextField { path, secret: true, .. } => {
            let raw = cfg.text(path);
            if raw.is_empty() { raw } else { super::masked(&raw) }
        }
        Control::KeyBinding { path, default, .. } => {
            let bound = cfg.text(path);
            if bound.is_empty() { (*default).to_string() } else { bound }
        }
        Control::TextField { path, .. } | Control::Resolution { path, .. } => cfg.text(path),
        Control::Dropdown { options, current, .. } => options
            .iter()
            .find(|(key, _)| key == current)
            .map_or_else(|| current.clone(), |(_, label)| label.clone()),
        Control::Chips { options, current, .. } => options
            .iter()
            .find(|(key, _)| key == current)
            .map_or_else(|| current.clone(), |(_, label)| label.clone()),
        Control::ActionBtn { .. } | Control::ActionChips { .. } => {
            String::from(tr("settings-control-action"))
        }
        Control::Details { summary, .. } | Control::StackBar { summary, .. } => summary.clone(),
        Control::Segment => String::new(),
        Control::Static => String::from(tr("settings-control-status")),
        Control::Code { .. } => String::from(tr("settings-control-code")),
        Control::Preview => String::from(tr("settings-control-preview")),
        Control::AppTheme { app, .. } => app.state.clone(),
    }
}

#[cfg(test)]
#[path = "search_tests.rs"]
mod tests;
