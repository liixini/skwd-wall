use log::{info, warn};

use crate::app::runtime::doctor_summary;
#[allow(clippy::wildcard_imports)]
use crate::app::*;

impl App {
    pub(in crate::app) fn on_weather(&mut self, result: crate::contracts::daemon::WeatherResult) {
        let now = result.weather;
        if now.is_empty() {
            info!("weather match: no current weather resolved (offline or no location set)");
            self.change_filters(|flt| flt.weather_active = false);
            return;
        }
        info!("weather match: current weather is {now:?}");
        self.change_filters(|flt| {
            flt.weather_active = true;
            flt.current_weather = now;
        });
    }

    pub(super) fn on_doctor(&mut self, result: crate::contracts::daemon::DoctorResult) {
        let summary = doctor_summary(&result.checks);
        info!("doctor: {summary}");
        for check in result.checks {
            info!("doctor: [{}] {}: {}", check.status, check.check, check.detail);
        }
        self.show_toast(summary);
        self.scene.touch();
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
