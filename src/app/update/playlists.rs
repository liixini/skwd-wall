use iced::Task;
use serde_json::json;

#[allow(clippy::wildcard_imports)]
use super::super::*;

use crate::frontend::playlists::PlMsg;

pub(super) fn update(app: &mut App, msg: PlMsg) -> Task<Message> {
    match msg {
        PlMsg::OpenPicker => {
            if app.menu_capturing() {
                return Task::none();
            }
            app.panels.playlists = Some(crate::frontend::playlists::Playlists {
                picker: true,
                selected: app.library_session.playlist_filter.as_ref().map(|(id, _, _)| *id),
                ..Default::default()
            });
            app.refresh_playlists();
            app.retick();
            Task::none()
        }
        PlMsg::EditPlaylists => {
            if let Some(pl) = app.panels.playlists.as_mut() {
                pl.picker = false;
                pl.browse_pending = false;
                pl.selected = pl.selected.or_else(|| pl.lists.first().map(|row| row.id));
                pl.sync_buffers();
            }
            if let Some(id) = app.panels.playlists.as_ref().and_then(|pl| pl.selected) {
                app.refresh_playlist_members(id);
            }
            app.call_tracked("wall.outputs", json!({}), Pending::PlOutputs);
            Task::none()
        }
        PlMsg::Browse(Some(id)) => {
            if let Some(pl) = app.panels.playlists.as_mut()
                && pl.lists.iter().any(|row| row.id == id)
            {
                pl.selected = Some(id);
                pl.browse_pending = true;
                app.refresh_playlist_members(id);
            }
            Task::none()
        }
        PlMsg::Browse(None) => {
            app.panels.playlists = None;
            app.library_session.playlist_filter = None;
            app.change_filters(|_| {});
            app.retick();
            Task::none()
        }
        PlMsg::Search(value) => {
            if let Some(pl) = app.panels.playlists.as_mut() {
                pl.search = value;
            }
            Task::none()
        }
        PlMsg::FilterHelp(open) => {
            if let Some(pl) = app.panels.playlists.as_mut() {
                pl.filter_help = open;
            }
            Task::none()
        }
        PlMsg::Select(id) => pl_select(app, id),
        PlMsg::NewInput(text) => pl_new_input(app, text),
        PlMsg::NewSubmit => pl_new_submit(app),
        PlMsg::EditName(text) => pl_edit(app, "name", text, |_| true, |pl| &mut pl.name_buf),
        PlMsg::EditDwell(text) => pl_edit(
            app,
            "dwell",
            text,
            |val| val.trim().parse::<i64>().is_ok(),
            |pl| &mut pl.dwell_buf,
        ),
        PlMsg::EditSource(text) => pl_edit(app, "source", text, |_| true, |pl| &mut pl.source_buf),
        PlMsg::SetProp(id, field, value) => {
            app.playlist_set(id, &field, &value);
            Task::none()
        }
        PlMsg::ColorPick(id, bucket) => pl_color_pick(app, id, bucket),
        PlMsg::ToggleFolder(id, folder) => pl_folder_toggle(app, id, &folder),
        PlMsg::MoveMember(id, key, delta) => pl_member_move(app, id, &key, delta),
        PlMsg::RemoveMember(id, key) => pl_member_remove(app, id, &key),
        PlMsg::Delete(id) => pl_delete(app, id),
        PlMsg::ToggleOutput(output, id) => pl_toggle_output(app, &output, id),
        PlMsg::ToggleAll(id) => pl_toggle_all(app, id),
        PlMsg::Stop(id) => pl_stop(app, id),
        PlMsg::PlayNow(id) => pl_play_now(app, id),
    }
}

fn pl_member_move(app: &mut App, id: i64, key: &str, delta: i64) -> Task<Message> {
    app.daemon.client.call("playlist.move", json!({ "id": id, "key": key, "delta": delta }));
    app.refresh_playlist_members(id);
    Task::none()
}

fn pl_member_remove(app: &mut App, id: i64, key: &str) -> Task<Message> {
    app.daemon.client.call("playlist.remove", json!({ "id": id, "key": key }));
    app.refresh_playlist_members(id);
    app.refresh_playlists();
    Task::none()
}

pub(super) fn pl_select(app: &mut App, id: i64) -> Task<Message> {
    if let Some(pl) = app.panels.playlists.as_mut() {
        pl.selected = Some(id);
        pl.sync_buffers();
    }
    app.refresh_playlist_members(id);
    Task::none()
}

pub(super) fn pl_new_input(app: &mut App, text: String) -> Task<Message> {
    if let Some(pl) = app.panels.playlists.as_mut() {
        pl.new_buf = text;
    }
    Task::none()
}

pub(super) fn pl_new_submit(app: &mut App) -> Task<Message> {
    let name =
        app.panels.playlists.as_ref().map(|pl| pl.new_buf.trim().to_string()).unwrap_or_default();
    if name.is_empty() {
        return Task::none();
    }
    app.daemon.client.call("playlist.create", json!({ "name": name }));
    if let Some(pl) = app.panels.playlists.as_mut() {
        pl.new_buf.clear();
        pl.selected = None;
    }
    app.refresh_playlists();
    Task::none()
}

pub(super) fn pl_edit(
    app: &mut App,
    field: &str,
    text: String,
    accept: impl Fn(&str) -> bool,
    buf: impl FnOnce(&mut crate::frontend::playlists::Playlists) -> &mut String,
) -> Task<Message> {
    let id = app.panels.playlists.as_ref().and_then(|pl| pl.selected);
    if let Some(id) = id
        && accept(&text)
    {
        app.playlist_set(id, field, &text);
    }
    if let Some(pl) = app.panels.playlists.as_mut() {
        *buf(pl) = text;
    }
    Task::none()
}

pub(super) fn pl_color_pick(app: &mut App, id: i64, bucket: i64) -> Task<Message> {
    edit_source(app, id, |source| crate::frontend::playlists::toggle_color_clause(source, bucket))
}

fn pl_folder_toggle(app: &mut App, id: i64, folder: &str) -> Task<Message> {
    edit_source(app, id, |source| crate::frontend::playlists::toggle_folder_clause(source, folder))
}

fn edit_source(app: &mut App, id: i64, edit: impl FnOnce(&str) -> String) -> Task<Message> {
    let cur = app.panels.playlists.as_ref().map(|pl| pl.source_buf.clone()).unwrap_or_default();
    let next = edit(&cur);
    app.playlist_set(id, "source", &next);
    if let Some(pl) = app.panels.playlists.as_mut() {
        pl.source_buf = next;
    }
    Task::none()
}

pub(super) fn pl_delete(app: &mut App, id: i64) -> Task<Message> {
    app.daemon.client.call("playlist.delete", json!({ "id": id }));
    if let Some(pl) = app.panels.playlists.as_mut()
        && pl.selected == Some(id)
    {
        pl.selected = None;
        pl.members.clear();
    }
    app.refresh_playlists();
    Task::none()
}

pub(super) fn pl_toggle_output(app: &mut App, output: &str, id: i64) -> Task<Message> {
    let plan = app.panels.playlists.as_ref().map(|pl| {
        (pl.assigned_id("*") == Some(id), pl.assigned_id(output) == Some(id), pl.outputs.clone())
    });
    let Some((wildcard_on, own_on, outputs)) = plan else {
        return Task::none();
    };
    if wildcard_on {
        for out in outputs.iter().filter(|name| name.as_str() != output) {
            app.daemon.client.call("playlist.assign", json!({ "output": out, "id": id }));
        }
        app.daemon.client.call("playlist.assign", json!({ "output": "*", "id": 0 }));
    } else {
        let to = if own_on { 0 } else { id };
        app.daemon.client.call("playlist.assign", json!({ "output": output, "id": to }));
    }
    app.refresh_playlists();
    Task::none()
}

pub(super) fn pl_toggle_all(app: &mut App, id: i64) -> Task<Message> {
    let plan = app.panels.playlists.as_ref().map(|pl| {
        let all_on = pl.assigned_id("*") == Some(id);
        let explicit: Vec<String> =
            pl.outputs.iter().filter(|out| pl.assigned_id(out) == Some(id)).cloned().collect();
        (all_on, explicit)
    });
    let Some((all_on, explicit)) = plan else {
        return Task::none();
    };
    for out in &explicit {
        app.daemon.client.call("playlist.assign", json!({ "output": out, "id": 0 }));
    }
    let wild = if all_on { 0 } else { id };
    app.daemon.client.call("playlist.assign", json!({ "output": "*", "id": wild }));
    app.refresh_playlists();
    Task::none()
}

pub(super) fn pl_stop(app: &mut App, id: i64) -> Task<Message> {
    let params = if id > 0 { json!({ "id": id }) } else { json!({}) };
    app.daemon.client.call("playlist.stop", params);
    app.refresh_playlists();
    Task::none()
}

pub(super) fn pl_play_now(app: &mut App, id: i64) -> Task<Message> {
    let outs: Vec<String> = app
        .panels
        .playlists
        .as_ref()
        .map(|pl| {
            pl.assign.iter().filter(|(_, asg)| *asg == id).map(|(out, _)| out.clone()).collect()
        })
        .unwrap_or_default();
    if outs.is_empty() {
        app.daemon.client.call("playlist.assign", json!({ "output": "*", "id": id }));
        app.daemon.client.call("wall.playlist.next", json!({ "output": "*" }));
    } else {
        for out in outs {
            app.daemon.client.call("wall.playlist.next", json!({ "output": out }));
        }
    }
    app.refresh_playlists();
    Task::none()
}

pub(super) fn card_picker_update(
    app: &mut App,
    msg: crate::frontend::playlists::CardPickerMsg,
) -> Task<Message> {
    use crate::frontend::playlists::CardPickerMsg;
    match msg {
        CardPickerMsg::Close => {
            app.panels.card_picker = None;
            app.retick();
            Task::none()
        }
        CardPickerMsg::Toggle(id) => card_picker_toggle(app, id),
        CardPickerMsg::NewInput(text) => card_picker_new_input(app, text),
        CardPickerMsg::NewSubmit => card_picker_new_submit(app),
    }
}

pub(super) fn card_picker_toggle(app: &mut App, id: i64) -> Task<Message> {
    let Some(key) = app.panels.card_picker.as_ref().map(|cp| cp.key.clone()) else {
        return Task::none();
    };
    app.daemon.client.call("playlist.toggle", json!({ "id": id, "key": key }));
    app.refresh_card_memberships();
    if app.panels.playlists.as_ref().and_then(|pl| pl.selected) == Some(id) {
        app.refresh_playlist_members(id);
    }
    Task::none()
}

pub(super) fn card_picker_new_input(app: &mut App, text: String) -> Task<Message> {
    if let Some(cp) = app.panels.card_picker.as_mut() {
        cp.new_buf = text;
    }
    Task::none()
}

pub(super) fn card_picker_new_submit(app: &mut App) -> Task<Message> {
    let name = app.panels.card_picker.as_ref().map(|cp| {
        let typed = cp.new_buf.trim();
        if typed.is_empty() {
            crate::i18n::tr_args!(
                "playlists-generated-name",
                number => (cp.lists.len() + 1).to_string()
            )
        } else {
            typed.to_string()
        }
    });
    let Some(name) = name else {
        return Task::none();
    };
    if let Some(cp) = app.panels.card_picker.as_mut() {
        cp.new_buf.clear();
    }
    app.call_tracked("playlist.create", json!({ "name": name }), Pending::CardPickerCreate);
    Task::none()
}

pub(super) fn open_playlists(app: &mut App) -> Task<Message> {
    if app.panels.playlists.is_some() {
        return close_playlists(app);
    }
    if app.menu_capturing() {
        return Task::none();
    }
    app.chrome.pane_scrolls.remove("pl.list");
    app.chrome.pane_scrolls.remove("pl.contents");
    app.panels.playlists = Some(crate::frontend::playlists::Playlists::default());
    app.refresh_playlists();
    app.call_tracked("wall.outputs", json!({}), Pending::PlOutputs);
    app.retick();
    Task::none()
}

pub(super) fn close_playlists(app: &mut App) -> Task<Message> {
    let picker = app.panels.playlists.take().is_some_and(|pl| pl.picker);
    if !picker && app.library_session.playlist_filter.take().is_some() {
        app.change_filters(|_| {});
    }
    app.retick();
    Task::none()
}
