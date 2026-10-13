pub const PRESET_NAME_KEY: &str = crate::contracts::settings::keys::selector::PRESET_DRAFT_NAME;

pub fn workbench_input_id(key: &str) -> iced::widget::Id {
    format!("settings-control:{key}").into()
}

pub fn settings_search_input_id() -> iced::widget::Id {
    "settings-search".into()
}

pub fn weather_review_input_id() -> iced::widget::Id {
    const WEATHER_REVIEW_SEARCH_ID: &str = "settings-weather-review-search";
    WEATHER_REVIEW_SEARCH_ID.into()
}
