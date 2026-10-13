pub use wall_proto::playlist_source;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum PlaylistKind {
    #[default]
    Curated,
    Smart,
    Other(String),
}

impl PlaylistKind {
    pub fn is_smart(&self) -> bool {
        matches!(self, Self::Smart)
    }
}

impl From<&str> for PlaylistKind {
    fn from(value: &str) -> Self {
        match value {
            "" | "curated" => Self::Curated,
            "smart" => Self::Smart,
            value => Self::Other(value.to_string()),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub kind: PlaylistKind,
    pub source: Option<String>,
    pub order: String,
    pub dwell: i64,
    pub position: i64,
    pub count: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlaylistMember {
    pub key: Option<String>,
    pub kind: Option<String>,
    pub preview: Option<String>,
    pub thumb: Option<String>,
    pub thumb_sm: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlaylistAssignment {
    pub output: String,
    pub id: i64,
}
