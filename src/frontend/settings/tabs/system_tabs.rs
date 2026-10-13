#[allow(clippy::wildcard_imports)]
use super::*;
use crate::i18n::{settings_keybind_conflict, tr, tr_args};

const SECONDS_PER_MINUTE: u64 = 60;
const WEATHER_PLAYLIST_SOURCE: &str = "playlists";
const WEATHER_SEMANTIC_SOURCE: &str = "semantic";
const WEATHER_ADVANCED: &str = "weather.advanced";
const OUTDOOR_CONTEXT_VALUE: &str = "settings-outdoors-context-value";
const SECONDS_UNIT: &str = "s";
const WEATHER_SEMANTIC_ACTIVE: &str = "settings-weather-state-semantic-active";
const WEATHER_SEMANTIC_EMPTY: &str = "settings-weather-state-semantic-empty";
const WEATHER_SEMANTIC_UNMAPPED: &str = "settings-weather-state-semantic-unmapped";
const WEATHER_APPROVED_ACTIVE: &str = "settings-weather-state-approved-active";
const WEATHER_APPROVED_EMPTY: &str = "settings-weather-state-approved-empty";
const WEATHER_CATEGORIES: [(&str, &str, &str); 7] = [
    ("clear", keys::weather::CLEAR, "schedule-weather-clear"),
    ("cloudy", keys::weather::CLOUDY, "schedule-weather-cloudy"),
    ("rainy", keys::weather::RAINY, "schedule-weather-rainy"),
    ("snowy", keys::weather::SNOWY, "schedule-weather-snowy"),
    ("stormy", keys::weather::STORMY, "schedule-weather-stormy"),
    ("foggy", keys::weather::FOGGY, "schedule-weather-foggy"),
    ("windy", keys::weather::WINDY, "schedule-weather-windy"),
];
const WEATHER_QUERIES: [(&str, &str); 7] = [
    (keys::weather::SEMANTIC_CLEAR, "schedule-weather-clear"),
    (keys::weather::SEMANTIC_CLOUDY, "schedule-weather-cloudy"),
    (keys::weather::SEMANTIC_RAINY, "schedule-weather-rainy"),
    (keys::weather::SEMANTIC_SNOWY, "schedule-weather-snowy"),
    (keys::weather::SEMANTIC_STORMY, "schedule-weather-stormy"),
    (keys::weather::SEMANTIC_FOGGY, "schedule-weather-foggy"),
    (keys::weather::SEMANTIC_WINDY, "schedule-weather-windy"),
];
const WEATHER_STATES: [(crate::contracts::daemon::WeatherState, &str); 10] = [
    (crate::contracts::daemon::WeatherState::Disabled, "settings-weather-state-disabled"),
    (crate::contracts::daemon::WeatherState::Loading, "settings-weather-state-loading"),
    (crate::contracts::daemon::WeatherState::NoLocation, "settings-weather-state-no-location"),
    (crate::contracts::daemon::WeatherState::Unavailable, "settings-weather-state-unavailable"),
    (crate::contracts::daemon::WeatherState::Unmapped, "settings-weather-state-unmapped"),
    (crate::contracts::daemon::WeatherState::Missing, "settings-weather-state-missing"),
    (crate::contracts::daemon::WeatherState::Empty, "settings-weather-state-empty"),
    (
        crate::contracts::daemon::WeatherState::SearchUnavailable,
        "settings-weather-state-search-unavailable",
    ),
    (crate::contracts::daemon::WeatherState::Active, "settings-weather-state-active"),
    (crate::contracts::daemon::WeatherState::Paused, "settings-weather-state-paused"),
];
const OUTDOOR_STATES: [(crate::contracts::daemon::WeatherState, &str); 2] = [
    (crate::contracts::daemon::WeatherState::Active, "settings-weather-state-loading"),
    (crate::contracts::daemon::WeatherState::Unavailable, "settings-outdoors-unavailable"),
];
const OUTDOOR_MATCH_LABELS: [(crate::contracts::daemon::OutdoorMatch, &str); 3] = [
    (crate::contracts::daemon::OutdoorMatch::Library, "settings-outdoors-match-library"),
    (crate::contracts::daemon::OutdoorMatch::Starter, "settings-outdoors-match-starter"),
    (crate::contracts::daemon::OutdoorMatch::Ambient, "settings-outdoors-match-ambient"),
];
const WEATHER_MISSING_ID: &str = "settings-weather-missing-id";
const WEATHER_SUNNY: &str = "sunny";
const WEATHER_SUNNY_LABEL: &str = "schedule-weather-sunny";
const WEATHER_COUNT_VALUE: &str = "settings-weather-count-value";
const WEATHER_FETCHED_AGO: &str = "settings-weather-fetched-ago";
const OUTPUT_SEPARATOR: &str = ", ";

pub(crate) fn weather_category_label(category: &str) -> &'static str {
    WEATHER_CATEGORIES
        .iter()
        .find(|(key, _, _)| *key == category)
        .map_or(tr("settings-weather-review-label"), |(_, _, label)| tr(label))
}

pub(crate) fn weather_status_key(
    status: crate::contracts::daemon::WeatherState,
    source: crate::contracts::daemon::WeatherSource,
    selection: crate::contracts::daemon::WeatherSelection,
) -> &'static str {
    match (source, selection, status) {
        (
            crate::contracts::daemon::WeatherSource::Semantic,
            crate::contracts::daemon::WeatherSelection::Approved,
            crate::contracts::daemon::WeatherState::Active,
        ) => return WEATHER_APPROVED_ACTIVE,
        (
            crate::contracts::daemon::WeatherSource::Semantic,
            crate::contracts::daemon::WeatherSelection::Approved,
            crate::contracts::daemon::WeatherState::Empty,
        ) => return WEATHER_APPROVED_EMPTY,
        (
            crate::contracts::daemon::WeatherSource::Semantic,
            _,
            crate::contracts::daemon::WeatherState::Active,
        ) => return WEATHER_SEMANTIC_ACTIVE,
        (
            crate::contracts::daemon::WeatherSource::Semantic,
            _,
            crate::contracts::daemon::WeatherState::Empty,
        ) => return WEATHER_SEMANTIC_EMPTY,
        (
            crate::contracts::daemon::WeatherSource::Semantic,
            _,
            crate::contracts::daemon::WeatherState::Unmapped,
        ) => return WEATHER_SEMANTIC_UNMAPPED,
        (crate::contracts::daemon::WeatherSource::Outdoors, _, state) => {
            if let Some((_, key)) = OUTDOOR_STATES.iter().find(|(kind, _)| *kind == state) {
                return key;
            }
        }
        _ => {}
    }
    WEATHER_STATES
        .iter()
        .find(|(state, _)| *state == status)
        .map_or(WEATHER_STATES[0].1, |(_, label)| *label)
}

fn outdoor_match_key(kind: crate::contracts::daemon::OutdoorMatch) -> &'static str {
    OUTDOOR_MATCH_LABELS
        .iter()
        .find(|(candidate, _)| *candidate == kind)
        .map(|(_, key)| *key)
        .expect("outdoor match label")
}

pub(super) fn tab_general(
    builder: &mut Builder<'_>,
    themes: &[String],
    detected_outputs: &[String],
) {
    builder.card(tr("settings-general-general-card"), "");
    let configured = builder.cfg.text(keys::system::MONITOR);
    let mut outputs = detected_outputs.to_vec();
    outputs.sort();
    outputs.dedup();
    let mut choices = vec![(String::new(), String::from(tr("settings-general-monitor-focused")))];
    choices.extend(outputs.iter().cloned().map(|output| (output.clone(), output)));
    if !configured.is_empty() && !outputs.contains(&configured) {
        choices.push((
            configured.clone(),
            tr_args!("settings-general-monitor-unavailable", output => &configured),
        ));
    }
    builder.dynamic_chips(
        tr("settings-general-monitor-label"),
        tr("settings-general-monitor-desc"),
        keys::system::MONITOR,
        choices,
    );
    builder.toggle(
        tr("settings-general-apply-picker-monitor-label"),
        tr("settings-general-apply-picker-monitor-desc"),
        keys::general::APPLY_ON_PICKER_MONITOR,
    );
    builder.num_setting(
        tr("settings-general-ui-scale-label"),
        tr("settings-general-ui-scale-desc"),
        crate::contracts::settings::schema::setting::general::UI_SCALE,
        "x",
    );
    builder.card(tr("settings-general-features-card"), tr("settings-general-features-card-desc"));
    builder.toggle(
        tr("settings-general-feature-steam-label"),
        tr("settings-general-feature-steam-desc"),
        keys::features::STEAM,
    );
    builder.toggle(
        tr("settings-general-feature-wallhaven-label"),
        tr("settings-general-feature-wallhaven-desc"),
        keys::features::WALLHAVEN,
    );
    builder.card(tr("settings-general-audio-card"), tr("settings-general-audio-card-desc"));
    builder.toggle(
        tr("settings-general-mute-label"),
        tr("settings-general-mute-desc"),
        keys::wallpaper::MUTE,
    );
    builder.num(
        tr("settings-general-volume-label"),
        tr("settings-general-volume-desc"),
        keys::wallpaper::VOLUME,
        "%",
    );
    builder.toggle(
        tr("settings-general-other-audio-label"),
        tr("settings-general-other-audio-desc"),
        keys::playback::MUTE_ON_OTHER_AUDIO,
    );
    builder.card(
        tr("settings-general-auto-recolour-card"),
        tr("settings-general-auto-recolour-card-desc"),
    );
    builder.toggle(
        tr("settings-general-auto-recolour-label"),
        tr("settings-general-auto-recolour-desc"),
        keys::effects::AUTO_RECOLOR,
    );
    builder.theme_dropdown(
        tr("settings-general-recolour-theme-label"),
        tr("settings-general-recolour-theme-desc"),
        keys::effects::AUTO_THEME,
        themes,
    );
    builder.card(tr("settings-general-behaviour-card"), "");
    builder.toggle(
        tr("settings-general-close-on-selection-label"),
        tr("settings-general-close-on-selection-desc"),
        keys::general::CLOSE_ON_SELECTION,
    );
    builder.toggle(
        tr("settings-general-hover-selects-label"),
        tr("settings-general-hover-selects-desc"),
        keys::general::HOVER_SELECTS,
    );
    builder.toggle(
        tr("settings-general-notify-label"),
        tr("settings-general-notify-desc"),
        keys::general::NOTIFY_ON_WALLPAPER_CHANGE,
    );
    builder.toggle(
        tr("settings-general-restore-label"),
        tr("settings-general-restore-desc"),
        keys::system::RESTORE_ON_STARTUP,
    );
    builder.card(tr("settings-general-random-card"), "");
    builder.toggle(
        tr("settings-general-rotate-label"),
        tr("settings-general-rotate-desc"),
        keys::general::RANDOM_ROTATE,
    );
    builder.num(
        tr("settings-general-interval-label"),
        tr("settings-general-interval-desc"),
        keys::general::RANDOM_INTERVAL,
        "s",
    );
    builder.toggle(
        tr("settings-general-include-images-label"),
        tr("settings-general-include-images-desc"),
        keys::general::RANDOM_INCLUDE_STATIC,
    );
    builder.toggle(
        tr("settings-general-include-video-label"),
        tr("settings-general-include-video-desc"),
        keys::general::RANDOM_INCLUDE_VIDEO,
    );
    builder.toggle(
        tr("settings-general-include-we-label"),
        tr("settings-general-include-we-desc"),
        keys::general::RANDOM_INCLUDE_WE,
    );
    builder.toggle(
        tr("settings-general-favourites-only-label"),
        tr("settings-general-favourites-only-desc"),
        keys::general::RANDOM_INCLUDE_FAVOURITES,
    );
}

pub(super) fn tab_schedule(builder: &mut Builder<'_>) {
    builder.card(tr("settings-schedule-schedule-card"), tr("settings-schedule-schedule-card-desc"));
    builder.toggle(
        tr("settings-schedule-enable-label"),
        tr("settings-schedule-enable-desc"),
        keys::schedule::ENABLED,
    );
    builder.toggle(
        tr("settings-schedule-apply-on-start-label"),
        tr("settings-schedule-apply-on-start-desc"),
        keys::schedule::APPLY_ON_START,
    );
    builder.action(
        tr("settings-schedule-editor-label"),
        tr("settings-schedule-editor-desc"),
        ActionId::OpenScheduleEditor,
        tr("settings-schedule-editor-action"),
    );
    builder.card(tr("settings-schedule-location-card"), tr("settings-schedule-location-card-desc"));
    builder.text_field(
        tr("settings-filter-location-label"),
        tr("settings-filter-location-desc"),
        keys::general::LOCALE,
        tr("settings-filter-location-placeholder"),
    );
    builder.text_field(
        tr("settings-schedule-latitude-label"),
        tr("settings-schedule-latitude-desc"),
        keys::schedule::LATITUDE,
        "59.33",
    );
    builder.text_field(
        tr("settings-schedule-longitude-label"),
        tr("settings-schedule-longitude-desc"),
        keys::schedule::LONGITUDE,
        "18.06",
    );
}

pub(super) fn tab_weather(
    builder: &mut Builder<'_>,
    weather: Option<&crate::contracts::daemon::WeatherStatus>,
    playlists: &[crate::contracts::playlists::Playlist],
    pending: bool,
) {
    builder.card(tr("settings-weather-card"), tr("settings-weather-card-desc"));
    let active = weather.is_some_and(|status| {
        status.enabled && status.source == crate::contracts::daemon::WeatherSource::Outdoors
    }) || (builder.cfg.flag(keys::weather::ENABLED)
        && builder.cfg.text(keys::weather::SOURCE) == keys::weather::SOURCE_OUTDOORS);
    builder.action(
        tr("settings-outdoors-follow-label"),
        tr(if pending {
            "settings-outdoors-pending-desc"
        } else {
            "settings-outdoors-follow-desc"
        }),
        ActionId::FollowOutdoors(!active),
        tr(if pending {
            "settings-outdoors-pending-action"
        } else if active {
            "settings-outdoors-stop-action"
        } else {
            "settings-outdoors-follow-action"
        }),
    );
    builder.dynamic_dropdown(
        tr("settings-outdoors-transition-label"),
        tr("settings-outdoors-transition-desc"),
        keys::weather::OUTDOORS_TRANSITION_SHADER,
        super::super::tables::SHADERS
            .iter()
            .map(|shader| (shader.to_string(), super::transition_tab::pretty(shader)))
            .collect(),
    );
    builder.toggle(
        tr("settings-outdoors-retheme-label"),
        tr("settings-outdoors-retheme-desc"),
        keys::weather::OUTDOORS_RETHEME,
    );
    if builder.cfg.text(keys::general::LOCALE).trim().is_empty()
        && builder.cfg.text(keys::schedule::LATITUDE).trim().is_empty()
        && builder.cfg.text(keys::schedule::LONGITUDE).trim().is_empty()
    {
        builder.info(tr("settings-outdoors-location-label"), tr("settings-outdoors-add-city"));
    }
    if let Some(status) =
        weather.filter(|status| status.source == crate::contracts::daemon::WeatherSource::Outdoors)
    {
        outdoor_status(builder, status, pending);
    } else if active {
        builder.info(tr("settings-weather-status-label"), tr("settings-weather-state-loading"));
    }
    builder.details(
        tr("settings-outdoors-advanced-label"),
        tr("settings-outdoors-advanced-desc"),
        WEATHER_ADVANCED.to_string(),
        tr("settings-outdoors-advanced-summary").to_string(),
        |builder| weather_advanced(builder, weather, playlists),
    );
}

fn outdoor_status(
    builder: &mut Builder<'_>,
    status: &crate::contracts::daemon::WeatherStatus,
    pending: bool,
) {
    use crate::contracts::daemon::{LightPhase, OutdoorSeason, WeatherState};

    let Some(outdoors) = status.outdoors.as_ref() else {
        builder.info(
            tr("settings-weather-status-label"),
            tr(weather_status_key(status.status, status.source, status.selection)),
        );
        return;
    };
    let season = tr(match outdoors.season {
        OutdoorSeason::Neutral => "settings-outdoors-season-neutral",
        OutdoorSeason::Spring => "settings-outdoors-season-spring",
        OutdoorSeason::Summer => "settings-outdoors-season-summer",
        OutdoorSeason::Autumn => "settings-outdoors-season-autumn",
        OutdoorSeason::Winter => "settings-outdoors-season-winter",
    });
    let light = tr(match outdoors.light {
        LightPhase::Dawn => "settings-outdoors-light-dawn",
        LightPhase::Day => "settings-outdoors-light-day",
        LightPhase::Dusk => "settings-outdoors-light-dusk",
        LightPhase::Night => "settings-outdoors-light-night",
    });
    let condition =
        status.weather.first().map_or(tr("settings-outdoors-weather-unknown"), |category| {
            if category == WEATHER_SUNNY {
                tr(WEATHER_SUNNY_LABEL)
            } else {
                WEATHER_CATEGORIES
                    .iter()
                    .find(|(key, _, _)| *key == category)
                    .map_or(tr("settings-outdoors-weather-unknown"), |(_, _, label)| tr(label))
            }
        });
    builder.info(
        tr("settings-outdoors-context-label"),
        &tr_args!(OUTDOOR_CONTEXT_VALUE, season => season, weather => condition, light => light),
    );
    builder.info(
        tr("settings-weather-status-label"),
        tr(match status.status {
            WeatherState::NoLocation => "settings-outdoors-no-location",
            WeatherState::Loading if outdoors.stale => "settings-outdoors-loading-held",
            WeatherState::Loading => "settings-weather-state-loading",
            WeatherState::Unavailable if outdoors.stale => "settings-outdoors-unavailable-held",
            WeatherState::Unavailable => "settings-outdoors-unavailable",
            WeatherState::Disabled => "settings-weather-state-disabled",
            _ if outdoors.stale => "settings-outdoors-stale",
            _ => outdoor_match_key(outdoors.match_kind),
        }),
    );
    if !outdoors.location.is_empty() {
        builder.info(tr("settings-outdoors-location-label"), &outdoors.location);
    }
    for output in &outdoors.outputs {
        outdoor_output(builder, output, outdoors, pending);
    }
}

fn outdoor_output(
    builder: &mut Builder<'_>,
    output: &crate::contracts::daemon::OutdoorOutput,
    outdoors: &crate::contracts::daemon::OutdoorStatus,
    pending: bool,
) {
    let held = output.held;
    let mut actions = Vec::new();
    if held {
        actions.push((
            ActionId::OutdoorResume(output.output.clone()),
            tr("settings-outdoors-resume-action"),
        ));
    } else {
        if !outdoors.stale {
            actions.push((
                ActionId::OutdoorNext(output.output.clone()),
                tr("settings-outdoors-next-action"),
            ));
        }
        actions.extend([
            (
                ActionId::OutdoorExclude { output: output.output.clone(), key: output.key.clone() },
                tr("settings-outdoors-exclude-action"),
            ),
            (ActionId::OutdoorPin(output.output.clone()), tr("settings-outdoors-pin-action")),
        ]);
    }
    let controls = vec![Row {
        title: tr("settings-outdoors-display-actions").to_string(),
        desc: tr(if pending {
            "settings-outdoors-pending-desc"
        } else if held {
            "settings-outdoors-held-desc"
        } else {
            "settings-outdoors-actions-desc"
        })
        .to_string(),
        control: if pending || output.key.is_empty() {
            Control::Static
        } else {
            Control::ActionChips {
                items: actions.into_iter().map(|(id, label)| (id, label.to_string())).collect(),
            }
        },
    }];
    builder.row(
        &output.output,
        tr(if held {
            "settings-outdoors-held-label"
        } else {
            outdoor_match_key(outdoors.match_kind)
        }),
        Control::StackBar {
            id: format!("outdoors:{}", output.output),
            summary: output.name.clone(),
            preview: output.preview.clone(),
            rows: controls,
        },
    );
}

fn weather_advanced(
    builder: &mut Builder<'_>,
    weather: Option<&crate::contracts::daemon::WeatherStatus>,
    playlists: &[crate::contracts::playlists::Playlist],
) {
    if builder.cfg.text(keys::weather::SOURCE) != keys::weather::SOURCE_OUTDOORS {
        builder.toggle(
            tr("settings-weather-enable-label"),
            tr("settings-weather-enable-desc"),
            keys::weather::ENABLED,
        );
        builder.dropdown(
            tr("settings-weather-source-label"),
            tr("settings-weather-source-desc"),
            keys::weather::SOURCE,
            &[
                (WEATHER_PLAYLIST_SOURCE, tr("settings-weather-source-playlists")),
                (WEATHER_SEMANTIC_SOURCE, tr("settings-weather-source-semantic")),
            ],
        );
    }
    if builder.cfg.text(keys::weather::SOURCE) == WEATHER_SEMANTIC_SOURCE {
        builder.dropdown(
            tr("settings-weather-selection-label"),
            tr("settings-weather-selection-desc"),
            keys::weather::SEMANTIC_SELECTION,
            &[
                (keys::weather::SELECTION_AUTOMATIC, tr("settings-weather-selection-automatic")),
                (keys::weather::SELECTION_APPROVED, tr("settings-weather-selection-approved")),
            ],
        );
        for (index, (path, label)) in WEATHER_QUERIES.into_iter().enumerate() {
            builder.text_field(tr(label), tr("settings-weather-query-desc"), path, "");
            builder.action(
                tr("settings-weather-review-label"),
                tr("settings-weather-review-desc"),
                ActionId::ReviewWeather(WEATHER_CATEGORIES[index].0),
                tr("settings-weather-review-action"),
            );
        }
        builder.num(
            tr("settings-weather-max-results-label"),
            tr("settings-weather-max-results-desc"),
            keys::weather::SEMANTIC_MAX_RESULTS,
            "",
        );
        builder.num(
            tr("settings-weather-dwell-label"),
            tr("settings-weather-dwell-desc"),
            keys::weather::SEMANTIC_DWELL_SECONDS,
            SECONDS_UNIT,
        );
    } else if builder.cfg.text(keys::weather::SOURCE) == WEATHER_PLAYLIST_SOURCE {
        let mut options = vec![(String::new(), tr("settings-weather-unmapped").to_string())];
        options.extend(
            playlists
                .iter()
                .filter(|playlist| {
                    matches!(
                        &playlist.kind,
                        crate::contracts::playlists::PlaylistKind::Curated
                            | crate::contracts::playlists::PlaylistKind::Smart
                    )
                })
                .map(|playlist| {
                    (playlist.id.to_string(), format!("{} ({})", playlist.name, playlist.id))
                }),
        );
        for (_, path, label) in WEATHER_CATEGORIES {
            let saved = builder.cfg.text(path);
            let mut choices = options.clone();
            if !saved.is_empty() && !choices.iter().any(|(id, _)| id == &saved) {
                choices.push((saved.clone(), tr_args!(WEATHER_MISSING_ID, id => saved)));
            }
            builder.dynamic_dropdown(tr(label), tr("settings-weather-map-desc"), path, choices);
        }
    }
    builder.action(
        tr("settings-weather-refresh-label"),
        tr("settings-weather-refresh-desc"),
        ActionId::RefreshWeather,
        tr("settings-weather-refresh-action"),
    );
    if let Some(weather) = weather {
        weather_advanced_status(builder, weather);
    }
}

fn weather_advanced_status(
    builder: &mut Builder<'_>,
    weather: &crate::contracts::daemon::WeatherStatus,
) {
    if let Some(credits) = weather.outdoors.as_ref().and_then(|outdoors| outdoors.credits.as_ref())
    {
        builder.action(
            tr("settings-outdoors-credits-label"),
            tr("settings-outdoors-credits-desc"),
            ActionId::OutdoorCredits(credits.clone()),
            tr("settings-outdoors-credits-action"),
        );
    }
    builder.info(
        tr("settings-weather-status-label"),
        tr(weather_status_key(weather.status, weather.source, weather.selection)),
    );
    if !weather.weather.is_empty() {
        let sunny = weather.weather.iter().any(|category| category == WEATHER_SUNNY);
        let conditions = weather
            .weather
            .iter()
            .filter(|category| !(sunny && category.as_str() == WEATHER_CATEGORIES[0].0))
            .map(|category| {
                if category == WEATHER_SUNNY {
                    return tr(WEATHER_SUNNY_LABEL);
                }
                WEATHER_CATEGORIES
                    .iter()
                    .find(|(key, _, _)| *key == category)
                    .map_or(category.as_str(), |(_, _, label)| tr(label))
            })
            .collect::<Vec<_>>()
            .join(OUTPUT_SEPARATOR);
        builder.info(tr("settings-weather-current-label"), &conditions);
    }
    if let Some(query) = weather.query.as_deref() {
        builder.info(tr("settings-weather-query-label"), query);
        builder.info(
            tr("settings-weather-count-label"),
            &tr_args!(WEATHER_COUNT_VALUE, count => weather.resolved_count),
        );
    } else if let Some(name) = weather.playlist_name.as_deref() {
        builder.info(tr("settings-weather-playlist-label"), name);
        builder.info(
            tr("settings-weather-count-label"),
            &tr_args!(WEATHER_COUNT_VALUE, count => weather.resolved_count),
        );
    } else if weather.source == crate::contracts::daemon::WeatherSource::Semantic
        && weather.selection == crate::contracts::daemon::WeatherSelection::Approved
    {
        builder.info(
            tr("settings-weather-count-label"),
            &tr_args!(WEATHER_COUNT_VALUE, count => weather.resolved_count),
        );
    }
    if let Some(fetched_at) = weather.fetched_at {
        let age = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |now| now.as_secs().saturating_sub(fetched_at));
        builder.info(
            tr("settings-weather-fetched-label"),
            &tr_args!(WEATHER_FETCHED_AGO, minutes => age / SECONDS_PER_MINUTE),
        );
    }
    if !weather.paused_outputs.is_empty() {
        builder.info(
            tr("settings-weather-paused-label"),
            &weather.paused_outputs.join(OUTPUT_SEPARATOR),
        );
    }
    if weather.resume_available
        && weather.source != crate::contracts::daemon::WeatherSource::Outdoors
    {
        builder.action(
            tr("settings-weather-resume-label"),
            tr("settings-weather-resume-desc"),
            ActionId::ResumeWeather,
            tr("settings-weather-resume-action"),
        );
    }
}

pub(super) fn tab_performance(builder: &mut Builder<'_>) {
    let battery_saver = builder.cfg.flag(keys::performance::BATTERY_SAVER);
    let power_source = match (builder.cfg.on_battery_power(), battery_saver) {
        (true, true) => tr("settings-performance-power-battery-limited"),
        (true, false) => tr("settings-performance-power-battery-unlimited"),
        (false, _) => tr("settings-performance-power-external"),
    };
    builder.card(tr("settings-performance-power-card"), tr("settings-performance-power-card-desc"));
    builder.info(tr("settings-performance-power-source-label"), power_source);
    let devices = builder.cfg.graphics_devices();
    let mut device_options =
        vec![(String::from("auto"), tr("settings-performance-device-auto").to_string())];
    for device in &devices {
        let name = if devices.iter().filter(|candidate| candidate.name == device.name).count() > 1 {
            format!("{} ({})", device.name, device.id)
        } else {
            device.name.clone()
        };
        device_options.push((device.id.clone(), name));
    }
    let current = builder.cfg.text(keys::performance::GPU_DEVICE);
    if !current.is_empty()
        && current != "auto"
        && !devices.iter().any(|device| device.id == current)
    {
        device_options.push((current, tr("settings-performance-device-unavailable").to_string()));
    }
    let options = device_options
        .iter()
        .map(|(value, label)| (value.as_str(), label.as_str()))
        .collect::<Vec<_>>();
    builder.dropdown(
        tr("settings-performance-device-label"),
        tr("settings-performance-device-desc"),
        keys::performance::GPU_DEVICE,
        &options,
    );
    builder.toggle(
        tr("settings-performance-battery-saver-label"),
        tr("settings-performance-battery-saver-desc"),
        keys::performance::BATTERY_SAVER,
    );
    builder.dropdown(
        tr("settings-performance-gpu-label"),
        tr("settings-performance-gpu-desc"),
        keys::performance::GPU_PREFERENCE,
        &[
            ("auto", tr("settings-performance-gpu-auto")),
            ("low", tr("settings-performance-gpu-low")),
            ("high", tr("settings-performance-gpu-high")),
            ("none", tr("settings-performance-gpu-none")),
        ],
    );
    builder.num(
        tr("settings-performance-battery-fps-label"),
        tr("settings-performance-battery-fps-desc"),
        keys::performance::BATTERY_FPS,
        "fps",
    );
    builder.num(
        tr("settings-performance-battery-idle-label"),
        tr("settings-performance-battery-idle-desc"),
        keys::performance::BATTERY_VIDEO_IDLE_SECONDS,
        "s",
    );
    builder.toggle(
        tr("settings-performance-battery-wallpaper-label"),
        tr("settings-performance-battery-wallpaper-desc"),
        keys::performance::BATTERY_WALLPAPER_PERFORMANCE,
    );
    builder.card(
        tr("settings-performance-rendering-card"),
        tr("settings-performance-rendering-card-desc"),
    );
    builder.info(
        tr("settings-performance-we-renderer-label"),
        tr("settings-performance-we-renderer-value"),
    );
    builder.num(
        tr("settings-performance-max-fps-label"),
        tr("settings-performance-max-fps-desc"),
        keys::general::MAX_FPS,
        "fps",
    );
    builder.num(
        tr("settings-performance-preview-fps-label"),
        tr("settings-performance-preview-fps-desc"),
        keys::transition::PREVIEW_FPS,
        "fps",
    );
    builder.toggle(
        tr("settings-performance-hover-previews-label"),
        tr("settings-performance-hover-previews-desc"),
        keys::video_preview::ENABLED,
    );
    builder.card(
        tr("settings-performance-image-opt-card"),
        tr("settings-performance-image-opt-card-desc"),
    );
    builder.toggle(
        tr("settings-performance-auto-optimize-label"),
        tr("settings-performance-auto-optimize-desc"),
        keys::performance::AUTO_OPTIMIZE_IMAGES,
    );
    builder.dropdown(
        tr("settings-performance-quality-label"),
        tr("settings-performance-quality-desc"),
        keys::performance::IMAGE_OPTIMIZE_PRESET,
        &[
            ("light", tr("settings-performance-quality-light")),
            ("balanced", tr("settings-performance-quality-balanced")),
            ("quality", tr("settings-performance-quality-quality")),
        ],
    );
    builder.dropdown(
        tr("settings-performance-resolution-label"),
        tr("settings-performance-resolution-desc"),
        keys::performance::IMAGE_OPTIMIZE_RESOLUTION,
        &[("1080p", "1080p"), ("2k", "2K"), ("4k", "4K")],
    );
    builder.action(
        tr("settings-performance-optimize-all-label"),
        tr("settings-performance-optimize-all-desc"),
        ActionId::OptimizeImages,
        tr("settings-performance-optimize-action"),
    );
    builder.card(tr("settings-performance-trash-card"), tr("settings-performance-trash-card-desc"));
    builder.num(
        tr("settings-performance-retention-label"),
        tr("settings-performance-retention-desc"),
        keys::performance::IMAGE_TRASH_DAYS,
        tr("settings-performance-retention-unit"),
    );
    builder.toggle(
        tr("settings-performance-auto-delete-label"),
        tr("settings-performance-auto-delete-desc"),
        keys::performance::AUTO_DELETE_IMAGE_TRASH,
    );
    builder.card(tr("settings-performance-thumbs-card"), "");
    builder.num(
        tr("settings-performance-thumb-jobs-label"),
        tr("settings-performance-thumb-jobs-desc"),
        keys::performance::MAX_THUMB_JOBS,
        "",
    );
    builder.action(
        tr("settings-performance-capture-we-label"),
        tr("settings-performance-capture-we-desc"),
        ActionId::CaptureWeThumbnails,
        tr("settings-performance-capture-we-action"),
    );
    builder.action(
        tr("settings-performance-clear-cache-label"),
        tr("settings-performance-clear-cache-desc"),
        ActionId::ClearCache,
        tr("settings-performance-clear-cache-action"),
    );
    builder.action(
        tr("settings-performance-recompute-label"),
        tr("settings-performance-recompute-desc"),
        ActionId::RecomputeColors,
        tr("settings-performance-recompute-action"),
    );
    builder.card(tr("settings-performance-bug-report-label"), "");
    builder.action(
        tr("settings-performance-bug-report-label"),
        tr("settings-performance-bug-report-desc"),
        ActionId::GenerateBugReport,
        tr("settings-performance-bug-report-action"),
    );
}

pub(super) fn tab_postprocessing(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    builder.card(tr("settings-postprocessing-behaviour-card"), "");
    builder.toggle(
        tr("settings-postprocessing-pick-only-label"),
        tr("settings-postprocessing-pick-only-desc"),
        keys::system::PICK_ONLY_MODE,
    );
    builder.toggle(
        tr("settings-postprocessing-on-restore-label"),
        tr("settings-postprocessing-on-restore-desc"),
        keys::system::POST_PROCESS_ON_RESTORE,
    );
    builder.card(
        tr("settings-postprocessing-commands-card"),
        tr("settings-postprocessing-commands-card-desc"),
    );
    let post_types = POST_TYPES.map(|key| {
        let label = if key == crate::contracts::settings::wallpaper_kind::STATIC {
            "settings-postprocessing-type-image"
        } else if key == crate::contracts::settings::wallpaper_kind::VIDEO {
            "settings-postprocessing-type-video"
        } else if key == crate::contracts::settings::wallpaper_kind::WE {
            "settings-postprocessing-type-we"
        } else {
            "settings-postprocessing-type-all"
        };
        (key, tr(label))
    });
    let entries = cfg.array_len(keys::post_processing::LIST);
    for idx in 0..entries {
        builder.chips(
            tr("settings-postprocessing-runs-for-label"),
            "",
            &format!("postProcessing.{idx}.type"),
            &post_types,
        );
        let key = format!("postProcessing.{idx}.command");
        builder.row(
            &tr_args!("settings-postprocessing-command-label", index => idx + 1),
            "",
            Control::TextField {
                key: key.clone(),
                path: key,
                placeholder: tr("settings-postprocessing-command-placeholder"),
            },
        );
        builder.action(
            "",
            tr("settings-postprocessing-remove-desc"),
            ActionId::RemovePostCommand(idx as u16),
            tr("settings-postprocessing-remove-action"),
        );
    }
    builder.action(
        tr("settings-postprocessing-add-label"),
        tr("settings-postprocessing-add-desc"),
        ActionId::AddPostCommand,
        tr("settings-postprocessing-add-action"),
    );
}

pub(super) fn tab_keybinds(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    let bindings = cfg.bindings();
    for group in crate::contracts::picker::KeyBindingGroup::ALL {
        builder.card(tr(group.title_key()), tr(group.description_key()));
        if group == crate::contracts::picker::KeyBindingGroup::Navigation {
            let chips = crate::domain::input::NavKeys::ALL
                .map(|keys| (ActionId::SetNavKeys(keys), tr(keys.label_key())));
            builder.action_chips(
                tr("settings-keybinds-second-keys-label"),
                tr("settings-keybinds-second-keys-desc"),
                &chips,
            );
        }
        for descriptor in
            crate::contracts::picker::KEY_BINDINGS.into_iter().filter(|entry| entry.group == group)
        {
            builder.key_binding(
                tr(descriptor.title_key),
                descriptor.path,
                descriptor.action.default_binding(),
            );
        }
    }
    let conflicts = bindings.conflicts();
    if !conflicts.is_empty() {
        builder.card(
            tr("settings-keybinds-conflicts-card"),
            tr("settings-keybinds-conflicts-card-desc"),
        );
    }
    for (first, second) in conflicts {
        let title = |action| {
            crate::contracts::picker::KEY_BINDINGS
                .into_iter()
                .find(|entry| entry.action == action)
                .map_or("", |entry| tr(entry.title_key))
        };
        let msg = settings_keybind_conflict(title(first), title(second), &bindings.label(first));
        builder.row(tr("settings-keybinds-conflict-label"), &msg, Control::Static);
    }
    builder.card(tr("settings-keybinds-fixed-card"), tr("settings-keybinds-fixed-card-desc"));
    builder.row("Escape", tr("settings-keybinds-fixed-escape-desc"), Control::Static);
    builder.row(
        tr("settings-keybinds-mouse-scroll-label"),
        tr("settings-keybinds-mouse-scroll-desc"),
        Control::Static,
    );
    builder.row(
        tr("settings-keybinds-mouse-hover-label"),
        tr("settings-keybinds-mouse-hover-desc"),
        Control::Static,
    );
    builder.card(tr("settings-keybinds-reset-card"), tr("settings-keybinds-reset-desc"));
    builder.action(
        tr("settings-keybinds-reset-label"),
        tr("settings-keybinds-reset-desc"),
        ActionId::ResetKeybinds,
        tr("settings-keybinds-reset-action"),
    );
}

pub(super) fn tab_language(builder: &mut Builder<'_>) {
    builder.card(tr("settings-language-card"), tr("settings-language-card-desc"));
    let current = crate::i18n::language_choice(&builder.cfg.text(keys::general::LANGUAGE));
    let mut options = vec![("auto", tr("settings-language-system"))];
    options
        .extend(crate::i18n::LANGUAGES.iter().map(|language| (language.tag, tr(language.label))));
    builder.dropdown_cur(
        tr("settings-language-choice-label"),
        tr("settings-language-choice-desc"),
        keys::general::LANGUAGE,
        &options,
        current.to_string(),
    );
}
