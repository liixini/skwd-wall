use serde_json::Value;

use super::apply_error_message;
use crate::app::{App, Pending};
use crate::contracts::daemon::{ActivityApplyEvent, ActivityApplyResult, ActivityCompletion};

const APPLY_FAILED: &str = "apply_failed";
const ACTIVITY_SAVED: &str = "effects-activity-saved";
const ACTIVITY_PREPARING: &str = "effects-activity-preparing";

impl App {
    pub(in crate::app) fn call_activity_apply(
        &mut self,
        batch: u64,
        output: &str,
        mut params: Value,
    ) {
        let token = match crate::infrastructure::ipc::apply_token() {
            Ok(token) => token,
            Err(error) => {
                self.finish_activity_batch(batch, ActivityCompletion::Failed(error.to_string()));
                return;
            }
        };
        params[wall_proto::apply_fields::APPLY_TOKEN] = serde_json::json!(token);
        let id = self.call_tracked(
            wall_proto::rpc::WALL_APPLY,
            params,
            Pending::ActivityApply { token: token.clone() },
        );
        if id == 0 {
            self.finish_activity_batch(
                batch,
                ActivityCompletion::Failed(crate::i18n::tr("status-daemon-lost").to_string()),
            );
            return;
        }
        self.daemon.activity_requests.insert(
            token,
            crate::app::state::ActivityApplyRequest { batch, id, output: output.to_string() },
        );
    }

    pub(in crate::app) fn on_activity_apply_response(
        &mut self,
        token: &str,
        result: &ActivityApplyResult,
    ) {
        let Some(request) = self.daemon.activity_requests.get(token) else { return };
        let Some(batch) = self.daemon.activity_applies.get(&request.batch) else { return };
        if result.activity != batch.activity {
            self.complete_activity_request(
                token,
                ActivityCompletion::Failed(
                    crate::i18n::tr("effects-activity-unavailable").to_string(),
                ),
            );
        } else if result.queued {
            if request.batch == self.daemon.next_activity_apply && !batch.failed {
                self.show_toast(
                    crate::i18n::tr_args!(ACTIVITY_PREPARING, activity => batch.name.as_str()),
                );
                self.retick();
            }
        } else {
            self.complete_activity_request(
                token,
                ActivityCompletion::Applied { visible: result.visible },
            );
        }
    }

    pub(in crate::app) fn on_activity_apply_event(&mut self, event: ActivityApplyEvent) {
        let Some(request) = self.daemon.activity_requests.get(&event.token) else { return };
        let Some(batch) = self.daemon.activity_applies.get(&request.batch) else { return };
        if request.id != event.request_id
            || request.output != event.output
            || batch.activity != event.activity
        {
            return;
        }
        if let Some(completion) = event.completion {
            self.complete_activity_request(&event.token, completion);
        }
    }

    pub(in crate::app) fn complete_activity_request(
        &mut self,
        token: &str,
        completion: ActivityCompletion,
    ) {
        let Some(request) = self.daemon.activity_requests.remove(token) else { return };
        self.finish_activity_batch(request.batch, completion);
    }

    pub(in crate::app) fn fail_activity_requests(&mut self, error: &str) {
        let tokens: Vec<_> = self.daemon.activity_requests.keys().cloned().collect();
        for token in tokens {
            self.complete_activity_request(&token, ActivityCompletion::Failed(error.to_string()));
        }
    }

    fn finish_activity_batch(&mut self, batch: u64, completion: ActivityCompletion) {
        let Some(mut apply) = self.daemon.activity_applies.remove(&batch) else { return };
        let latest = batch == self.daemon.next_activity_apply;
        match completion {
            ActivityCompletion::Applied { visible } => apply.visible &= visible,
            ActivityCompletion::Failed(detail) => {
                if !apply.failed && latest {
                    self.show_toast(apply_error_message(APPLY_FAILED, &detail));
                }
                apply.failed = true;
            }
            ActivityCompletion::Cancelled => apply.failed = true,
        }
        apply.pending -= 1;
        if apply.pending > 0 {
            self.daemon.activity_applies.insert(batch, apply);
        } else if latest {
            if !apply.failed && !apply.visible {
                self.show_toast(crate::i18n::tr_args!(ACTIVITY_SAVED, activity => apply.name));
            } else if self.runtime_state.toast.as_ref().is_some_and(|toast| {
                toast.0
                    == crate::i18n::tr_args!(ACTIVITY_PREPARING, activity => apply.name.as_str())
            }) {
                self.runtime_state.toast = None;
            }
        }
        self.retick();
    }
}
