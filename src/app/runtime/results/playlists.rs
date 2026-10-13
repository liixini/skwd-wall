use serde_json::json;

#[allow(clippy::wildcard_imports)]
use crate::app::*;

impl App {
    pub(super) fn on_pl_list(&mut self, result: crate::contracts::daemon::PlaylistListResult) {
        if let Some(rows) = &result.playlists {
            self.daemon.weather_playlists = rows.clone();
        }
        if self.panels.playlists.as_ref().is_some_and(|playlists| playlists.demo) {
            return;
        }
        let mut fetch = None;
        if let Some(pl) = self.panels.playlists.as_mut() {
            if let Some(rows) = result.playlists {
                pl.lists = rows;
            }
            if let Some(assigns) = result.assignments {
                pl.assign = assigns.into_iter().map(|asg| (asg.output, asg.id)).collect();
            }
            let valid = pl.selected.is_some_and(|id| pl.lists.iter().any(|row| row.id == id));
            if !valid && !pl.picker {
                let first = pl.lists.first().map(|row| row.id);
                pl.selected = first;
                pl.sync_buffers();
                fetch = first;
            }
        }
        if let Some(id) = fetch {
            self.refresh_playlist_members(id);
        }
        self.retick();
    }

    pub(super) fn on_weather_playlists(
        &mut self,
        result: crate::contracts::daemon::PlaylistListResult,
    ) {
        if let Some(rows) = result.playlists {
            self.daemon.weather_playlists = rows;
            self.retick();
        }
    }

    pub(super) fn on_pl_members(
        &mut self,
        result: crate::contracts::daemon::PlaylistMembersResult,
    ) {
        if self.panels.playlists.as_ref().is_some_and(|playlists| playlists.demo) {
            return;
        }
        let mut filt = None;
        let mut close_picker = false;
        if let Some(pl) = self.panels.playlists.as_mut() {
            if result.id != pl.selected || result.members.is_none() {
                return;
            }
            if let Some(rows) = result.members {
                pl.members = rows;
            }
            if let Some(id) = result.id {
                pl.members_for = id;
            }
            if pl.selected == Some(pl.members_for) && (!pl.picker || pl.browse_pending) {
                close_picker = pl.picker;
                let id = pl.members_for;
                let name = pl.lists.iter().find(|row| row.id == id).map_or_else(
                    || crate::i18n::tr("playlists-unnamed").to_string(),
                    |row| row.name.clone(),
                );
                let keys: std::collections::HashSet<String> =
                    pl.members.iter().filter_map(|mem| mem.key.clone()).collect();
                filt = Some((id, name, keys));
            }
        }
        if close_picker {
            self.panels.playlists = None;
        }
        if let Some(flt) = filt {
            self.library_session.playlist_filter = Some(flt);
            self.refilter();
        }
        self.retick();
    }

    pub(super) fn on_pl_outputs(
        &mut self,
        result: crate::contracts::daemon::PlaylistOutputsResult,
    ) {
        if let Some(pl) = self.panels.playlists.as_mut()
            && let Some(outputs) = result.outputs
        {
            pl.outputs = outputs;
        }
        self.retick();
    }

    pub(super) fn on_card_picker_list(
        &mut self,
        result: crate::contracts::daemon::PlaylistListResult,
    ) {
        if let Some(cp) = self.panels.card_picker.as_mut()
            && let Some(rows) = result.playlists
        {
            cp.lists = rows;
        }
        self.retick();
    }

    pub(super) fn on_card_picker_members(
        &mut self,
        result: crate::contracts::daemon::CardPickerMembershipsResult,
    ) {
        if let Some(cp) = self.panels.card_picker.as_mut()
            && let Some(ids) = result.ids
        {
            cp.member_ids = ids.into_iter().collect();
        }
        self.retick();
    }

    pub(super) fn on_card_picker_create(
        &mut self,
        result: crate::contracts::daemon::CardPickerCreateResult,
    ) {
        let key = self.panels.card_picker.as_ref().map(|cp| cp.key.clone());
        if let (Some(id), Some(key)) = (result.id, key.clone()) {
            self.daemon.client.call("playlist.add", json!({ "id": id, "key": key }));
        }
        self.call_tracked("playlist.list", json!({}), Pending::CardPickerList);
        if let Some(key) = key {
            self.call_tracked(
                "playlist.memberships",
                json!({ "key": key }),
                Pending::CardPickerMembers,
            );
        }
        self.retick();
    }
}
