use crate::contracts::settings::SettingsSource;
use crate::i18n::tr;

use super::{Control, build_tab, visible_tabs};

const SECTION_MATCH: u32 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsSearchResult {
    pub tab: String,
    pub tab_label: String,
    pub section: usize,
    pub section_title: String,
    pub row: usize,
    pub title: String,
    pub desc: String,
    pub value: String,
    score: u32,
}

pub fn search_settings(
    query: &str,
    cfg: &dyn SettingsSource,
    themes: &[String],
    folders: &[String],
    analysis: &str,
    backends: &[String],
    apps: Option<&crate::contracts::daemon::AppThemesResult>,
) -> Vec<SettingsSearchResult> {
    let query = normalized(query);
    let terms: Vec<&str> = query.split_whitespace().collect();
    if terms.is_empty() {
        return Vec::new();
    }
    let mut results = Vec::new();
    for (tab, tab_label) in visible_tabs(cfg) {
        let cards = if apps.is_some() {
            super::build_tab_with_runtime_status(
                tab,
                cfg,
                themes,
                folders,
                analysis,
                backends,
                super::RuntimeStatus { app_themes: apps, ..super::RuntimeStatus::default() },
            )
        } else {
            build_tab(tab, cfg, themes, folders, analysis, backends)
        };
        for (section, (card, rows)) in cards.into_iter().enumerate() {
            if rows.is_empty() {
                results.extend(page_result(&query, &terms, tab, tab_label, section, &card));
                continue;
            }
            for (row, setting) in rows.into_iter().enumerate() {
                if matches!(setting.control, Control::Segment) {
                    continue;
                }
                let title = normalized(&setting.title);
                let section_title = normalized(card.title);
                let tab_text = normalized(tab_label);
                let desc = normalized(&setting.desc);
                let control = normalized(&control_search_text(&setting.control));
                let mut haystack = format!("{title} {section_title} {tab_text} {desc} {control}");
                add_aliases(&mut haystack);
                if !terms.iter().all(|term| haystack.contains(term)) {
                    continue;
                }
                let score = if title == query {
                    0
                } else if title.starts_with(&query) {
                    1
                } else if title.contains(&query) {
                    2
                } else if section_title.contains(&query) {
                    SECTION_MATCH
                } else if tab_text.contains(&query) {
                    4
                } else {
                    5
                };
                results.push(SettingsSearchResult {
                    tab: tab.to_string(),
                    tab_label: tab_label.to_string(),
                    section,
                    section_title: card.title.to_string(),
                    row,
                    title: if setting.title.is_empty() {
                        crate::i18n::tr("settings-selector-saved-styles-label").to_string()
                    } else {
                        setting.title
                    },
                    desc: setting.desc,
                    value: control_value(&setting.control, cfg),
                    score,
                });
            }
        }
    }
    results.sort_by(|left, right| {
        left.score
            .cmp(&right.score)
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.tab_label.cmp(&right.tab_label))
    });
    results.truncate(6);
    results
}

fn page_result(
    query: &str,
    terms: &[&str],
    tab: &str,
    tab_label: &str,
    section: usize,
    card: &super::Card,
) -> Option<SettingsSearchResult> {
    let section_title = normalized(card.title);
    let mut haystack =
        format!("{section_title} {} {}", normalized(tab_label), normalized(card.subtitle));
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
        tab_label: tab_label.to_string(),
        section,
        section_title: card.title.to_string(),
        row: 0,
        title: card.title.to_string(),
        desc: card.subtitle.to_string(),
        value: String::new(),
        score,
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
        Control::TextField { key, path, placeholder } => {
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
        Control::MotionWeights { weights } => weights
            .iter()
            .map(|(label, path, _)| {
                format!("{label} {path} {}", tr("settings-search-control-motion"))
            })
            .collect::<Vec<_>>()
            .join(" "),
        Control::ActionBtn { label, .. } => {
            format!("{label} {}", tr("settings-search-control-action"))
        }
        Control::ActionChips { items } => format!(
            "{} {}",
            items.iter().map(|(_, label)| label.as_str()).collect::<Vec<_>>().join(" "),
            tr("settings-search-control-action")
        ),
        Control::Presets { mode, items } => format!(
            "{mode} {} {}",
            items.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>().join(" "),
            tr("settings-search-control-presets")
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

fn control_value(control: &Control, cfg: &dyn SettingsSource) -> String {
    match control {
        Control::Display(control) => format!("{} {}", control.label(), control.summary()),
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
        Control::TextField { path, .. }
        | Control::Resolution { path, .. }
        | Control::KeyBinding { path, .. } => cfg.text(path),
        Control::Dropdown { options, current, .. } => options
            .iter()
            .find(|(key, _)| key == current)
            .map_or_else(|| current.clone(), |(_, label)| label.clone()),
        Control::Chips { options, current, .. } => options
            .iter()
            .find(|(key, _)| key == current)
            .map_or_else(|| current.clone(), |(_, label)| label.clone()),
        Control::MotionWeights { weights } => weights
            .iter()
            .map(|(label, path, _)| format!("{label} {} ms", cfg.number(path)))
            .collect::<Vec<_>>()
            .join(" · "),
        Control::ActionBtn { .. } | Control::ActionChips { .. } => {
            String::from(tr("settings-control-action"))
        }
        Control::Presets { items, .. } => items
            .iter()
            .find(|(_, active)| *active)
            .map_or_else(|| String::from(tr("settings-control-none")), |(name, _)| name.clone()),
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
