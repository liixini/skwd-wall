use iced::Task;
use serde_json::json;

use crate::app::{App, Message, Pending};
use crate::frontend::settings::WeatherReviewTab;

pub(in crate::app) const VISIBLE_LIMIT: usize = 80;
pub(in crate::app) const TAB_COUNT: usize = crate::frontend::settings::WEATHER_REVIEW_TAB_COUNT;
const KEYBOARD_FOCUS: &str = "settings-workbench-keyboard-focus";

pub(in crate::app) fn query_key(category: &str) -> Option<&'static str> {
    use skwd_config::keys::weather;
    Some(match category {
        wall_proto::weather_category::CLEAR => weather::SEMANTIC_CLEAR,
        wall_proto::weather_category::CLOUDY => weather::SEMANTIC_CLOUDY,
        wall_proto::weather_category::RAINY => weather::SEMANTIC_RAINY,
        wall_proto::weather_category::SNOWY => weather::SEMANTIC_SNOWY,
        wall_proto::weather_category::STORMY => weather::SEMANTIC_STORMY,
        wall_proto::weather_category::FOGGY => weather::SEMANTIC_FOGGY,
        wall_proto::weather_category::WINDY => weather::SEMANTIC_WINDY,
        _ => return None,
    })
}

pub(super) fn open(app: &mut App, category: &'static str) -> Task<Message> {
    let Some(query_key) = query_key(category) else { return Task::none() };
    let review = &mut app.panels.settings;
    review.weather_review_category = Some(category);
    review.weather_review_tab = WeatherReviewTab::Suggested;
    review.weather_review_query.clear();
    review.weather_review_candidates.clear();
    review.weather_review_error = None;
    review.weather_review_loading = true;
    review.weather_review_focus = 0;
    review.weather_review_generation = review.weather_review_generation.wrapping_add(1);
    let generation = review.weather_review_generation;
    let query = app.config.str_path(query_key).trim().to_owned();
    let request = app.call_tracked(
        wall_proto::rpc::WALL_WEATHER_CANDIDATES,
        json!({wall_proto::rpc_params::CATEGORY: category}),
        Pending::WeatherCandidates { category: category.to_owned(), query, generation },
    );
    if request == 0 {
        app.panels.settings.weather_review_loading = false;
        app.panels.settings.weather_review_error =
            Some(crate::i18n::tr("settings-weather-review-offline").to_owned());
    }
    app.retick();
    iced::widget::operation::focus(crate::frontend::settings::weather_review_input_id())
}

pub(super) fn close(app: &mut App) {
    app.panels.settings.weather_review_category = None;
    app.panels.settings.weather_review_candidates.clear();
    app.panels.settings.weather_review_error = None;
    app.panels.settings.weather_review_loading = false;
    app.panels.settings.weather_review_generation =
        app.panels.settings.weather_review_generation.wrapping_add(1);
    app.retick();
}

pub(in crate::app) fn approved(app: &App, category: &str) -> Vec<String> {
    skwd_config::keys::weather::approved_key(category).map_or_else(Vec::new, |path| {
        app.config
            .array_slice(path)
            .iter()
            .filter_map(|value| value.as_str().map(str::to_owned))
            .collect()
    })
}

pub(in crate::app) fn visible_keys(app: &App) -> Vec<String> {
    let review = &app.panels.settings;
    let Some(category) = review.weather_review_category else { return Vec::new() };
    let approved = approved(app, category);
    let keys = match review.weather_review_tab {
        WeatherReviewTab::Suggested => review.weather_review_candidates.clone(),
        WeatherReviewTab::Approved => approved,
        WeatherReviewTab::Library => app
            .library_session
            .library
            .catalog()
            .items
            .iter()
            .map(|item| item.key.clone())
            .collect(),
    };
    let search = review.weather_review_query.trim().to_lowercase();
    let names = app
        .library_session
        .library
        .catalog()
        .items
        .iter()
        .map(|item| (item.key.as_str(), item.name.as_str()))
        .collect::<std::collections::HashMap<_, _>>();
    keys.into_iter()
        .filter(|key| {
            search.is_empty()
                || key.to_lowercase().contains(&search)
                || names.get(key.as_str()).is_some_and(|name| name.to_lowercase().contains(&search))
        })
        .take(if review.weather_review_tab == WeatherReviewTab::Library {
            VISIBLE_LIMIT
        } else {
            usize::MAX
        })
        .collect()
}

pub(super) fn toggle(app: &mut App, key: &str) {
    let Some(category) = app.panels.settings.weather_review_category else { return };
    let Some(path) = skwd_config::keys::weather::approved_key(category) else { return };
    let available = app.library_session.library.catalog().items.iter().any(|item| item.key == key);
    let mut keys = approved(app, category);
    if keys.iter().any(|candidate| candidate == key) {
        keys.retain(|candidate| candidate != key);
    } else if available {
        keys.push(key.to_owned());
    } else {
        return;
    }
    super::settings_policy::save_value(app, path, &json!(keys));
    app.retick();
}

pub(super) fn key(app: &mut App, key: crate::frontend::settings::SettingsKey) -> Task<Message> {
    use crate::frontend::settings::SettingsKey;
    if app.panels.settings.weather_review_category.is_none() {
        return Task::none();
    }
    let total = visible_keys(app).len();
    match key {
        SettingsKey::Cancel => close(app),
        SettingsKey::FocusNext { backwards } | SettingsKey::CategoryNext { backwards } => {
            let current = app.panels.settings.weather_review_focus;
            app.panels.settings.weather_review_focus = if backwards {
                current.checked_sub(1).unwrap_or(total + TAB_COUNT - 1)
            } else {
                (current + 1) % (total + TAB_COUNT)
            };
            app.retick();
            return iced::widget::operation::focus(iced::widget::Id::new(KEYBOARD_FOCUS));
        }
        SettingsKey::Up | SettingsKey::Previous => {
            app.panels.settings.weather_review_focus =
                app.panels.settings.weather_review_focus.saturating_sub(1);
            app.retick();
        }
        SettingsKey::Down | SettingsKey::Next => {
            app.panels.settings.weather_review_focus =
                (app.panels.settings.weather_review_focus + 1).min(total + TAB_COUNT - 1);
            app.retick();
        }
        SettingsKey::Activate => {
            let focused = app.panels.settings.weather_review_focus;
            match focused {
                0 => app.panels.settings.weather_review_tab = WeatherReviewTab::Suggested,
                1 => app.panels.settings.weather_review_tab = WeatherReviewTab::Approved,
                2 => app.panels.settings.weather_review_tab = WeatherReviewTab::Library,
                _ => {
                    if let Some(key) = visible_keys(app).get(focused - TAB_COUNT) {
                        toggle(app, key);
                    }
                }
            }
            app.retick();
        }
        SettingsKey::Search => {}
    }
    Task::none()
}
