use iced::Task;

use crate::app::state::arrange::displays::OutputRefresh;
use crate::app::{App, Message, Pending};
use crate::frontend::arrange::ArrangeMsg;

const ERROR: &str = "arrange-display-error";

pub(super) fn defer(app: &mut App, action: &ArrangeMsg) -> bool {
    let Some(session) = app.panels.arrange.as_mut() else { return false };
    if session.outputs.action.as_ref().is_some_and(|pending| same_action(pending, action)) {
        return true;
    }
    if matches!(action, ArrangeMsg::Pointer(_)) {
        return false;
    }
    let previous = std::mem::take(&mut session.outputs.request);
    app.daemon.pending.remove(&previous);
    session.outputs = OutputRefresh::default();
    if session.preview_config.is_some() || session.gesturing() {
        return false;
    }
    if !matches!(
        action,
        ArrangeMsg::OpenDisplays
            | ArrangeMsg::ReviewDisplay(_)
            | ArrangeMsg::ApplyDisplay
            | ArrangeMsg::Done
            | ArrangeMsg::ResolveConflict(_)
    ) {
        return false;
    }
    let request = app.call_tracked(
        wall_proto::rpc::WALL_OUTPUTS,
        serde_json::json!({}),
        Pending::ArrangeOutputs,
    );
    if request == 0 {
        failed(app, crate::i18n::tr("status-daemon-lost").to_string());
        return true;
    }
    let session = app.panels.arrange.as_mut().unwrap();
    session.outputs.request = request;
    session.outputs.action = Some(action.clone());
    session.outputs.document = Some(app.config.document().clone());
    true
}

pub(in crate::app) fn received(app: &mut App, result: crate::contracts::daemon::OutputsResult) {
    app.on_outputs(result);
    if let Some(session) = app.panels.arrange.as_mut() {
        session.outputs.ready = true;
    }
}

pub(in crate::app) fn failed(app: &mut App, error: String) {
    if let Some(session) = app.panels.arrange.as_mut() {
        session.outputs = OutputRefresh::default();
        app.show_toast(crate::i18n::tr_args!(ERROR, error => error));
    }
}

pub(in crate::app) fn resume(app: &mut App) -> Task<Message> {
    let Some(session) = app.panels.arrange.as_mut() else { return Task::none() };
    if !session.outputs.ready {
        return Task::none();
    }
    let refresh = std::mem::take(&mut session.outputs);
    if refresh.document.as_ref() != Some(app.config.document()) {
        return Task::none();
    }
    refresh.action.map_or_else(Task::none, |action| super::core::dispatch(app, action))
}

fn same_action(pending: &ArrangeMsg, action: &ArrangeMsg) -> bool {
    match (pending, action) {
        (ArrangeMsg::OpenDisplays, ArrangeMsg::OpenDisplays)
        | (ArrangeMsg::ApplyDisplay, ArrangeMsg::ApplyDisplay)
        | (ArrangeMsg::Done, ArrangeMsg::Done) => true,
        (ArrangeMsg::ReviewDisplay(previous), ArrangeMsg::ReviewDisplay(next)) => previous == next,
        (ArrangeMsg::ResolveConflict(previous), ArrangeMsg::ResolveConflict(next)) => {
            previous == next
        }
        _ => false,
    }
}
