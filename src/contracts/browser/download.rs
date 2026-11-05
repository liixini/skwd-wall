use super::{ApplyKind, Source};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clip {
    pub start_secs: u64,
    pub duration_secs: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadRequest {
    Steam {
        id: String,
    },
    Wallhaven {
        id: String,
        full_url: String,
    },
    Catalog {
        source: Source,
        id: String,
        full_url: String,
        attribution: String,
        track_url: String,
        clip: Option<Clip>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewRequest {
    pub source: Source,
    pub id: String,
    pub full_url: String,
    pub refresh: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Done,
    Failed,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DownloadUpdate {
    pub id: String,
    pub status: DownloadStatus,
    pub path: Option<String>,
    pub progress: Option<f32>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyTarget {
    Path { kind: ApplyKind, path: String },
    WallpaperEngine { id: String },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DownloadResponse {
    Exists,
    Done,
    Started,
    #[default]
    Other,
}
