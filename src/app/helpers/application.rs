#[allow(clippy::wildcard_imports)]
use super::super::*;

impl App {
    pub(in crate::app) fn unsubscribe_available(&self) -> bool {
        self.config.steam_uses_client()
            && self.daemon.connected
            && self.daemon.steam_client_available == Some(true)
    }

    pub(in crate::app) fn refresh_unsubscribe_status(&mut self) {
        self.daemon.steam_client_available = None;
        self.daemon.steam_refresh_needed = false;
        if self.daemon.connected && self.config.steam_uses_client() {
            if self.daemon.pending.values().any(|kind| matches!(kind, Pending::Status)) {
                self.daemon.steam_refresh_needed = true;
            } else {
                self.call_tracked(wall_proto::rpc::STATUS, serde_json::json!({}), Pending::Status);
            }
        }
        self.scene.touch();
        self.retick();
    }

    pub(in crate::app) fn call_tracked(
        &mut self,
        method: &str,
        params: serde_json::Value,
        kind: Pending,
    ) -> u64 {
        let id = self.daemon.client.call(method, params);
        if id > 0 {
            self.daemon.pending.insert(id, kind);
        }
        id
    }

    pub(in crate::app) fn show_toast(&mut self, message: impl Into<String>) {
        self.runtime_state.toast = Some((message.into(), std::time::Instant::now()));
    }

    #[allow(clippy::unused_self)]
    pub(in crate::app) fn invalidate_settings(&mut self) {}

    pub(in crate::app) fn close_settings(&mut self) {
        if !self.panels.settings.open {
            return;
        }
        self.panels.settings.open = false;
        self.panels.settings.input_edit = None;
        self.panels.settings.keybind_capture = None;
        self.panels.settings.search_open = false;
        self.panels.settings.search_query.clear();
        self.panels.settings.search_results.clear();
        self.panels.settings.focused_choice = None;
        self.panels.settings.armed = None;
        self.panels.settings.sddm_themes.clear();
        self.panels.settings.folder_undo = None;
        crate::app::update::settings_policy::flush_staged(self);
        self.release_settings_preview();
        self.retick();
        self.invalidate_settings();
    }

    pub(in crate::app) fn selected_settings_preview_path(&self) -> &str {
        self.library_session
            .filtered
            .get(self.scene.current)
            .and_then(|&source_index| {
                self.library_session.library.catalog().items.get(source_index as usize)
            })
            .map_or("", |item| item.thumb.as_str())
    }

    pub(in crate::app) fn sync_settings_preview(&mut self) -> iced::Task<Message> {
        if !self.panels.settings.open {
            self.release_settings_preview();
            return iced::Task::none();
        }
        if crate::frontend::settings::is_picker_layout_section(
            &self.panels.settings.tab,
            self.panels.settings.section,
        ) {
            self.release_settings_preview();
            return iced::Task::none();
        }
        let desired = self.selected_settings_preview_path().to_string();
        self.panels.settings.preview_desired_path.clone_from(&desired);
        if desired.is_empty() {
            self.release_settings_preview();
            return iced::Task::none();
        }
        if self.panels.settings.preview_allocated_path == desired
            || !self.panels.settings.preview_in_flight_path.is_empty()
        {
            return iced::Task::none();
        }
        self.panels.settings.preview_in_flight_path.clone_from(&desired);
        iced_runtime::image::allocate(iced::widget::image::Handle::from_path(&desired))
            .map(move |result| Message::SettingsPreviewAllocated(desired.clone(), result.ok()))
    }

    pub(in crate::app) fn release_settings_preview(&mut self) {
        self.panels.settings.preview_desired_path.clear();
        self.panels.settings.preview_in_flight_path.clear();
        self.panels.settings.preview_allocated_path.clear();
        self.panels.settings.preview_allocation = None;
    }

    pub(in crate::app) fn finish_settings_preview(
        &mut self,
        path: &str,
        allocation: Option<iced_runtime::image::Allocation>,
    ) {
        if self.panels.settings.preview_in_flight_path == path {
            self.panels.settings.preview_in_flight_path.clear();
        }
        if self.panels.settings.preview_desired_path == path {
            self.panels.settings.preview_allocated_path = path.to_string();
            self.panels.settings.preview_allocation = allocation;
        }
    }
}
