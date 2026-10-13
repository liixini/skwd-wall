use log::{info, warn};

#[allow(clippy::wildcard_imports)]
use crate::app::*;

impl App {
    pub(in crate::app) fn on_weather(&mut self, result: crate::contracts::daemon::WeatherStatus) {
        use crate::contracts::daemon::{WeatherSelection, WeatherSource, WeatherState};
        use skwd_config::keys::{semantic, weather};

        let source = self.config.str_path(weather::SOURCE);
        let location_key = self.outdoor_location_key();
        let source_matches = match result.source {
            WeatherSource::Playlists => source == weather::SOURCE_PLAYLISTS,
            WeatherSource::Semantic => source == weather::SOURCE_SEMANTIC,
            WeatherSource::Outdoors => source == weather::SOURCE_OUTDOORS,
        };
        if !source_matches
            || result.enabled != self.config.flag_default_config(weather::ENABLED)
            || (result.source == WeatherSource::Outdoors
                && result.location_key.as_deref() != Some(location_key.as_str()))
            || (result.source == WeatherSource::Semantic
                && result.selection
                    != if self.config.str_path(weather::SEMANTIC_SELECTION)
                        == weather::SELECTION_APPROVED
                    {
                        WeatherSelection::Approved
                    } else {
                        WeatherSelection::Automatic
                    })
            || (result.source == WeatherSource::Semantic
                && result.selection == WeatherSelection::Automatic
                && matches!(result.status, WeatherState::Active | WeatherState::Paused)
                && !self.config.flag_default_true(semantic::ENABLED))
        {
            return;
        }
        if result.source == WeatherSource::Outdoors
            && self.daemon.weather.as_ref().is_some_and(|old| {
                old.source == WeatherSource::Outdoors
                    && old.location_key == result.location_key
                    && old.outdoors.as_ref().is_some_and(|old| {
                        result
                            .outdoors
                            .as_ref()
                            .is_some_and(|next| next.generation < old.generation)
                    })
            })
        {
            return;
        }
        if let Some(query) = result.query.as_deref().filter(|_| {
            result.source == WeatherSource::Semantic
                && result.selection == WeatherSelection::Automatic
        }) {
            let key = match result.category.as_deref() {
                Some(wall_proto::weather_category::CLEAR) => Some(weather::SEMANTIC_CLEAR),
                Some(wall_proto::weather_category::CLOUDY) => Some(weather::SEMANTIC_CLOUDY),
                Some(wall_proto::weather_category::RAINY) => Some(weather::SEMANTIC_RAINY),
                Some(wall_proto::weather_category::SNOWY) => Some(weather::SEMANTIC_SNOWY),
                Some(wall_proto::weather_category::STORMY) => Some(weather::SEMANTIC_STORMY),
                Some(wall_proto::weather_category::FOGGY) => Some(weather::SEMANTIC_FOGGY),
                Some(wall_proto::weather_category::WINDY) => Some(weather::SEMANTIC_WINDY),
                _ => None,
            };
            if key.is_none_or(|key| self.config.str_path(key).trim() != query) {
                return;
            }
        }
        if result.source == WeatherSource::Semantic
            && result.selection == WeatherSelection::Approved
        {
            let Some(path) = result.category.as_deref().and_then(weather::approved_key) else {
                return;
            };
            let approved = self.config.array_slice(path);
            if result
                .member_keys
                .iter()
                .any(|key| !approved.iter().any(|entry| entry.as_str() == Some(key.as_str())))
            {
                return;
            }
        }
        self.daemon.weather = Some(result);
        self.retick();
    }

    pub(in crate::app) fn on_weather_candidates(
        &mut self,
        category: &str,
        query: &str,
        generation: u64,
        result: crate::contracts::daemon::WeatherCandidates,
    ) {
        let review = &mut self.panels.settings;
        if review.weather_review_category != Some(category)
            || review.weather_review_generation != generation
            || result.category != category
            || result.query.as_deref() != (!query.is_empty()).then_some(query)
            || crate::app::update::weather_review::query_key(category)
                .is_none_or(|path| self.config.str_path(path).trim() != query)
        {
            return;
        }
        review.weather_review_candidates = result.member_keys;
        review.weather_review_error = result.error;
        review.weather_review_loading = false;
        self.retick();
    }

    pub(super) fn on_bug_report(&mut self, result: crate::contracts::daemon::BugReportResult) {
        match result.path {
            Some(path) => {
                info!("bug report saved: {path}");
                self.show_toast(crate::i18n::tr_args!("status-bug-report-saved", path => &path));
            }
            None => self.show_toast(crate::i18n::tr("status-bug-report-failed")),
        }
        self.scene.touch();
    }

    pub(super) fn on_diagnostic(&mut self, result: crate::contracts::daemon::DiagnosticResult) {
        if let Some(banner) = result.banner {
            self.daemon.diagnostic = banner;
        }
    }

    pub(super) fn on_status(&mut self, result: crate::contracts::daemon::StatusResult) {
        self.daemon.steam_helper_available = result.steam_helper_available;
        if self.daemon.steam_refresh_needed {
            self.refresh_unsubscribe_status();
        } else {
            self.daemon.steam_client_available = result.steam_client_available;
        }
        self.scene.touch();
        self.retick();
        let picker_session = result.advertises("picker-session");
        if let Some(playback) = result.playback {
            self.daemon.playback = playback;
        }
        if result.library_watch_present {
            self.daemon.library_watch = result.library_watch;
        }
        if let Some(warning) = version_mismatch(env!("CARGO_PKG_VERSION"), &result.version) {
            warn!("{warning}");
            self.show_toast(warning);
            self.scene.touch();
        }
        if picker_session {
            self.daemon.client.call("picker.session.begin", serde_json::json!({}));
        }
    }

    pub(super) fn on_task_list(&mut self, result: crate::contracts::daemon::TaskListResult) {
        if let Some(tasks) = result.tasks {
            self.daemon.tasks.replace(tasks);
            self.chrome.bar.cache.clear();
        }
    }
}
