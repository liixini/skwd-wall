#[derive(Debug, Clone)]
pub enum CardPickerMsg {
    Close,
    Toggle(i64),
    NewInput(String),
    NewSubmit,
}

#[derive(Debug, Clone)]
pub enum PlMsg {
    OpenPicker,
    EditPlaylists,
    Browse(Option<i64>),
    Search(String),
    FilterHelp(bool),
    Select(i64),
    NewInput(String),
    NewSubmit,
    EditName(String),
    EditDwell(String),
    EditSource(String),
    SetProp(i64, String, String),
    ColorPick(i64, i64),
    ToggleFolder(i64, String),
    MoveMember(i64, String, i64),
    RemoveMember(i64, String),
    Delete(i64),
    ToggleOutput(String, i64),
    ToggleAll(i64),
    Stop(i64),
    PlayNow(i64),
}

#[derive(Default)]
pub struct Playlists {
    pub demo: bool,
    pub picker: bool,
    pub browse_pending: bool,
    pub search: String,
    pub filter_help: bool,
    pub lists: Vec<crate::contracts::playlists::Playlist>,
    pub assign: Vec<(String, i64)>,
    pub outputs: Vec<String>,
    pub selected: Option<i64>,
    pub members: Vec<crate::contracts::playlists::PlaylistMember>,
    pub members_for: i64,
    pub new_buf: String,
    pub name_buf: String,
    pub dwell_buf: String,
    pub source_buf: String,
}

impl Playlists {
    pub fn selected_def(&self) -> Option<&crate::contracts::playlists::Playlist> {
        let id = self.selected?;
        self.lists.iter().find(|playlist| playlist.id == id)
    }

    pub fn sync_buffers(&mut self) {
        let values = self.selected_def().map(|playlist| {
            (
                playlist.name.clone(),
                playlist.dwell.to_string(),
                playlist.source.clone().unwrap_or_default(),
            )
        });
        if let Some((name, dwell, source)) = values {
            self.name_buf = name;
            self.dwell_buf = dwell;
            self.source_buf = source;
        }
    }

    pub(crate) fn assigned_id(&self, output: &str) -> Option<i64> {
        self.assign.iter().find(|(name, _)| name == output).map(|(_, id)| *id)
    }

    pub(super) fn active_outputs(&self, id: i64) -> Vec<String> {
        self.assign
            .iter()
            .filter(|(_, assigned_id)| *assigned_id == id)
            .map(|(output, _)| output.clone())
            .collect()
    }

    pub(super) fn any_active(&self) -> bool {
        !self.assign.is_empty()
    }
}

#[derive(Default)]
pub struct CardPicker {
    pub key: String,
    pub name: String,
    pub lists: Vec<crate::contracts::playlists::Playlist>,
    pub member_ids: std::collections::HashSet<i64>,
    pub new_buf: String,
}
