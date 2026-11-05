use super::Source;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WallhavenSearch {
    pub query: String,
    pub categories: [bool; 3],
    pub sorting: String,
    pub purity: [bool; 3],
    pub top_range: String,
    pub atleast: String,
    pub atmost: String,
    pub resolutions: String,
    pub ratios: String,
    pub collection: String,
    pub color_hue: Option<u8>,
    pub page: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamSearch {
    pub query: String,
    pub query_type: u32,
    pub trend_days: u32,
    pub request_type: String,
    pub tags: Vec<String>,
    pub excluded_tags: Vec<String>,
    pub resolution: String,
    pub allow_nsfw: bool,
    pub page: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogSearch {
    pub source: Source,
    pub query: String,
    pub page: u32,
    pub max_duration: u64,
    pub order_by: String,
    pub orientation: String,
    pub size: String,
    pub color: String,
    pub content_filter: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchRequest {
    Wallhaven(WallhavenSearch),
    Steam(SteamSearch),
    Catalog(CatalogSearch),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BrowserSearchResult {
    pub id: String,
    pub full_url: String,
    pub thumb_path: String,
    pub title: String,
    pub resolution: String,
    pub purity: String,
    pub file_size: u64,
    pub category: String,
    pub duration_secs: u64,
    pub downloaded: bool,
    pub attribution: String,
    pub attribution_url: String,
    pub track_url: String,
    pub thumb_ready: bool,
    pub workshop: Option<super::WorkshopDetails>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchPage {
    pub generation: Option<u64>,
    pub current_page: u32,
    pub last_page: u32,
    pub next_cursor: String,
    pub results: Vec<BrowserSearchResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserCollection {
    pub id: String,
    pub label: String,
}
