use log::warn;
use serde_json::{Value, json};

use crate::infrastructure::ipc::IpcMsg;

#[allow(clippy::wildcard_imports)]
use super::super::*;

impl App {
    pub(in crate::app) fn handle_ipc(&mut self, msg: IpcMsg) -> bool {
        match msg {
            IpcMsg::Connected { list_id } => {
                self.ipc_connected(list_id);
                true
            }
            IpcMsg::Disconnected => {
                self.ipc_disconnected();
                true
            }
            IpcMsg::Response { id, result, error } => self.ipc_response(id, result, error),
            IpcMsg::Event { name, data } => {
                self.on_event(&name, &data);
                true
            }
        }
    }

    fn ipc_connected(&mut self, list_id: u64) {
        self.daemon.connected = true;
        self.daemon.ever_connected = true;
        if list_id > 0 {
            self.daemon.pending.insert(list_id, Pending::List);
        }
        self.call_tracked("effects.list", json!({}), Pending::EffectThemes);
        self.call_tracked("wall.outputs", json!({}), Pending::Outputs);
        self.call_tracked("status", json!({}), Pending::Status);
        if self.library_session.filters.weather_active {
            self.call_tracked("wall.weather", json!({}), Pending::Weather);
        }
        self.call_tracked("task.list", json!({}), Pending::TaskList);
        if self.source_browser.browser.is_some() {
            self.run_browser_search(false);
        }
        if self.panels.audio.is_some() {
            self.call_tracked("wall.outputs", json!({}), Pending::AudioOutputs);
        }
        if self.theme.audition_open {
            self.request_theme_previews();
        }
        crate::infrastructure::observability::log_startup_checkpoint("ipc_connected");
    }

    fn ipc_disconnected(&mut self) {
        self.library_session.list_dirty = true;
        let dropped: Vec<Pending> = self.daemon.pending.drain().map(|(_, kind)| kind).collect();
        for kind in dropped {
            self.rpc_error(kind, crate::i18n::tr("status-daemon-lost").to_string());
        }
        self.theme.job_pending = None;
        self.theme.audition_pending_backend = None;
        if self.daemon.connected {
            self.daemon.connected = false;
            self.show_toast(crate::i18n::tr("status-daemon-lost"));
        }
    }

    fn ipc_response(
        &mut self,
        id: u64,
        result: Option<Value>,
        error: Option<wall_proto::ErrorInfo>,
    ) -> bool {
        let Some(kind) = self.daemon.pending.remove(&id) else {
            return false;
        };
        let quiet = matches!(kind, Pending::Diag);
        if let Some(error) = error {
            let message = wall_proto::redact_sensitive(&error.message);
            warn!("rpc error for {kind:?}: {message} (code {})", error.code);
            self.rpc_error(kind, message);
            return !quiet;
        }
        let Some(result) = result else {
            return !quiet;
        };
        self.on_result(kind, &result);
        !quiet
    }
}
