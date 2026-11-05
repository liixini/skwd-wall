const YOUTUBE_POSTER_BASE: &str = "https://i.ytimg.com/vi/";
const YOUTUBE_POSTER_FILE: &str = "hqdefault.jpg";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkshopDetails {
    pub description: String,
    pub creator_id: String,
    pub creator_name: String,
    pub subscriptions: Option<u64>,
    pub votes_up: Option<u32>,
    pub votes_down: Option<u32>,
    pub time_created: u64,
    pub time_updated: u64,
    pub previews: Vec<WorkshopPreview>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkshopPreview {
    Image { url: String },
    Youtube { id: String },
}

impl WorkshopPreview {
    pub fn poster_url(&self) -> std::borrow::Cow<'_, str> {
        match self {
            Self::Image { url } => std::borrow::Cow::Borrowed(url),
            Self::Youtube { id } => {
                std::borrow::Cow::Owned(format!("{YOUTUBE_POSTER_BASE}{id}/{YOUTUBE_POSTER_FILE}"))
            }
        }
    }

    pub const fn is_video(&self) -> bool {
        matches!(self, Self::Youtube { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PreviewIdentity {
    pub source: super::Source,
    pub id: String,
    pub url: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkshopGallery {
    pub id: String,
    pub previews: Vec<WorkshopPreview>,
    pub creator_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkshopLink {
    Item(String),
    Creator(String),
    Video(String),
}
