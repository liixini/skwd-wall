use iced::Color;

use crate::contracts::browser::{
    BrowserSearchResult, CatalogSearch, Clip, DownloadRequest, DownloadStatus, DownloadUpdate,
    PreviewRequest, SearchRequest, SteamSearch, WallhavenSearch,
};
pub use crate::contracts::browser::{Source, SourceAvailability, SourceUnavailableReason};
use crate::i18n::{tr, tr_args};

#[derive(Debug, Clone)]
pub enum BrowserMsg {
    SwitchSource(Source),
    SearchInput(String),
    SearchSubmit,
    SetSort(String),
    SteamFilter(String, String),
    CatalogFilter(String, String),
    SetTopRange(String),
    SetAtleast(String),
    SetAtmost(String),
    SetRatios(String),
    SetMaxDuration(String),
    ToggleResExact,
    SetCollection(String),
    ClipStart(String),
    ClipLen(String),
    WallInput(crate::frontend::ui::BrowserWallInput),
    TogglePurity(u8),
    ToggleCategory(u8),
    SetColor(i64),
    Download(String),
    Apply(String),
    OpenPreview(usize),
    ClosePreview,
}

#[derive(Debug, Clone)]
pub enum BrowserIntent {
    Update(BrowserMsg),
    Close,
    Capture,
}

#[derive(Debug, Clone)]
pub struct BrowserItem {
    pub id: String,
    pub full_url: String,
    pub thumb_path: String,
    pub title: String,
    pub resolution: String,
    pub purity: String,
    pub file_size: u64,
    pub category: String,
    pub thumb_ready: bool,
    pub downloaded: bool,
    pub downloading: bool,
    pub queued: bool,
    pub progress: f32,
    pub phase: String,
    pub duration_secs: u64,
    pub downloaded_path: Option<String>,
    pub preview_path: Option<String>,
    pub thumb_failed: bool,
    pub attribution: String,
    pub attribution_url: String,
    pub track_url: String,
}

impl BrowserItem {
    pub fn apply_download_update(&mut self, update: &DownloadUpdate) {
        match update.status {
            DownloadStatus::Done => {
                self.downloaded = true;
                self.downloading = false;
                self.queued = false;
                self.downloaded_path.clone_from(&update.path);
            }
            DownloadStatus::Failed => {
                self.downloading = false;
                self.queued = false;
                self.progress = 0.0;
                self.phase.clear();
            }
            DownloadStatus::Downloading => {
                self.downloading = true;
                self.queued = false;
                if let Some(progress) = update.progress {
                    self.progress = progress;
                }
                self.phase = update.message.clone().unwrap_or_default();
            }
            DownloadStatus::Queued => {
                self.downloading = true;
                self.queued = true;
                self.progress = 0.0;
                self.phase = update
                    .message
                    .clone()
                    .unwrap_or_else(|| crate::i18n::tr("browser-queued").to_string());
            }
            DownloadStatus::Other => {}
        }
    }
}

pub(super) const PURITY_SKETCHY: Color = Color { r: 1.0, g: 0.8, b: 0.0, a: 0.2 };
pub(super) const PURITY_NSFW: Color = Color { r: 1.0, g: 0.3, b: 0.3, a: 0.2 };
pub(super) const PURITY_SFW: Color = Color { r: 0.3, g: 0.8, b: 0.3, a: 0.2 };
pub(super) const ON_MEDIA: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 0.85 };

pub const SORT_KEYS: [&str; 7] =
    ["toplist", "hot", "date_added", "relevance", "views", "favorites", "random"];

pub const STEAM_SORTS: [&str; 5] = ["3", "0", "1", "21", "9"];

pub const STEAM_TREND_DAYS: [&str; 3] = ["1", "3", "7"];

pub const STEAM_TYPES: [&str; 3] = ["", "Scene", "Video"];

pub const STEAM_RESOLUTIONS: [&str; 7] =
    ["", "1920 x 1080", "2560 x 1440", "3840 x 2160", "2560 x 1080", "3440 x 1440", "3840 x 1080"];

pub const STEAM_CATEGORIES: [&str; 20] = [
    "",
    "Abstract",
    "Animal",
    "Anime",
    "CGI",
    "Cyberpunk",
    "Fantasy",
    "Game",
    "Girls",
    "Guys",
    "Landscape",
    "Medieval",
    "Music",
    "Nature",
    "Pixel art",
    "Relaxing",
    "Retro",
    "Sci-Fi",
    "Technology",
    "Vehicle",
];

pub const TOP_RANGES: [(&str, &str); 7] = [
    ("1d", "Day"),
    ("3d", "3 Days"),
    ("1w", "Week"),
    ("1M", "Month"),
    ("3M", "3 Months"),
    ("6M", "6 Months"),
    ("1y", "Year"),
];

pub const RESOLUTIONS: [(&str, &str); 6] = [
    ("", "Any"),
    ("1920x1080", "1080p"),
    ("2560x1440", "2K"),
    ("3840x2160", "4K"),
    ("5120x2880", "5K"),
    ("7680x4320", "8K"),
];

pub const DURATIONS: [(&str, &str); 5] = [
    ("", "Any"),
    ("300", "\u{2264}5m"),
    ("600", "\u{2264}10m"),
    ("1800", "\u{2264}30m"),
    ("3600", "\u{2264}1h"),
];

pub const PHOTO_ORIENTATIONS: [(&str, &str); 4] =
    [("", "Any"), ("landscape", "Landscape"), ("portrait", "Portrait"), ("square", "Square")];

pub const UNSPLASH_ORDERS: [(&str, &str); 2] = [("relevant", "Relevant"), ("latest", "Latest")];

pub const UNSPLASH_SAFETY: [(&str, &str); 2] = [("low", "Standard"), ("high", "Strict")];

pub const UNSPLASH_COLORS: [(&str, &str); 12] = [
    ("", "Any colour"),
    ("black_and_white", "B&W"),
    ("black", "Black"),
    ("white", "White"),
    ("yellow", "Yellow"),
    ("orange", "Orange"),
    ("red", "Red"),
    ("purple", "Purple"),
    ("magenta", "Magenta"),
    ("green", "Green"),
    ("teal", "Teal"),
    ("blue", "Blue"),
];

pub const PEXELS_SIZES: [(&str, &str); 4] =
    [("", "Any size"), ("small", "4 MP+"), ("medium", "12 MP+"), ("large", "24 MP+")];

pub const PEXELS_COLORS: [(&str, &str); 14] = [
    ("", "Any colour"),
    ("red", "Red"),
    ("orange", "Orange"),
    ("yellow", "Yellow"),
    ("green", "Green"),
    ("turquoise", "Turquoise"),
    ("blue", "Blue"),
    ("violet", "Violet"),
    ("pink", "Pink"),
    ("brown", "Brown"),
    ("black", "Black"),
    ("gray", "Gray"),
    ("white", "White"),
    ("#808080", "Neutral"),
];

pub const RATIOS: [(&str, &str); 6] = [
    ("", "Any"),
    ("16x9", "16:9"),
    ("16x10", "16:10"),
    ("21x9", "21:9"),
    ("32x9", "32:9"),
    ("4x3", "4:3"),
];

pub fn progress_label(verb: &str, item: &BrowserItem) -> String {
    let pct = (item.progress.clamp(0.0, 1.0) * 100.0).round() as u32;
    if item.progress <= 0.0 {
        return format!("{verb}\u{2026}");
    }
    if item.phase.is_empty() {
        format!("{verb} {pct}%")
    } else {
        format!("{verb} {pct}% \u{00b7} {}", download_phase(&item.phase))
    }
}

pub fn download_phase(phase: &str) -> String {
    let key = match phase {
        "video" => Some("browser-phase-video"),
        "audio" => Some("browser-phase-audio"),
        "merging" => Some("browser-phase-merging"),
        "clipping" => Some("browser-phase-clipping"),
        "fetching" => Some("browser-phase-fetching"),
        "queued" => Some("browser-phase-queued"),
        _ => None,
    };
    if let Some(key) = key {
        return tr(key).to_string();
    }
    if let Some(count) =
        phase.strip_prefix("queued - ").and_then(|rest| rest.strip_suffix(" ahead"))
    {
        return tr_args!("browser-phase-queued-ahead", count => count);
    }
    phase.to_string()
}

pub fn fmt_clock(secs: u64) -> String {
    if secs >= 3600 {
        format!("{}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
    } else {
        format!("{}:{:02}", secs / 60, secs % 60)
    }
}

pub fn parse_clock(raw: &str) -> Option<u64> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let mut total: u64 = 0;
    for part in raw.split(':') {
        let seg = part.trim();
        if seg.is_empty() || !seg.chars().all(|ch| ch.is_ascii_digit()) {
            return None;
        }
        total = total.checked_mul(60)?.checked_add(seg.parse::<u64>().ok()?)?;
    }
    Some(total)
}

pub fn is_selected(list: &str, key: &str) -> bool {
    !key.is_empty() && list.split(',').any(|item| item == key)
}

pub fn toggle_selected(list: &mut String, key: &str) {
    if key.is_empty() {
        list.clear();
        return;
    }
    let mut parts: Vec<&str> = list.split(',').filter(|item| !item.is_empty()).collect();
    if let Some(pos) = parts.iter().position(|item| *item == key) {
        parts.remove(pos);
    } else {
        parts.push(key);
    }
    *list = parts.join(",");
}

pub struct Browser {
    pub source: Source,
    pub request: BrowserRequestState,
    pub session: BrowserSessionState,
    pub view: BrowserViewState,
}

pub struct BrowserRequestState {
    pub query: String,
    pub sorting: String,
    pub nsfw: bool,
    pub wallhaven: WallhavenRequestState,
    pub steam: SteamRequestState,
    pub catalog: CatalogRequestState,
}

pub struct WallhavenRequestState {
    pub general: bool,
    pub anime: bool,
    pub people: bool,
    pub sfw: bool,
    pub sketchy: bool,
    pub color: i64,
    pub top_range: String,
    pub atleast: String,
    pub atmost: String,
    pub resolutions: String,
    pub res_exact: bool,
    pub ratios: String,
    pub collections: Vec<(String, String)>,
    pub collection: String,
}

pub struct CatalogRequestState {
    pub clip_start: String,
    pub clip_len: String,
    pub max_duration: String,
    pub unsplash: UnsplashRequestState,
    pub pexels: PexelsRequestState,
}

pub struct UnsplashRequestState {
    pub order_by: String,
    pub orientation: String,
    pub color: String,
    pub content_filter: String,
}

pub struct PexelsRequestState {
    pub orientation: String,
    pub size: String,
    pub color: String,
}

pub struct SteamRequestState {
    pub kind: String,
    pub resolution: String,
    pub category: String,
    pub trend_days: String,
}

pub struct BrowserSessionState {
    pub submitted_search: Option<SearchRequest>,
    pub search_generation: u64,
    pub page: u32,
    pub last_page: u32,
    pub next_cursor: String,
    pub loading: bool,
    pub items: Vec<BrowserItem>,
    pub hover: Option<usize>,
    pub preview: Option<usize>,
    pub error: Option<String>,
    pub page_failed: bool,
    pub pending_apply: Option<String>,
}

pub struct BrowserViewState {
    pub thumb_fades: std::collections::HashMap<String, crate::frontend::animation::Spring>,
    pub hover_fades: std::collections::HashMap<usize, crate::frontend::animation::Spring>,
    pub bar_items: std::cell::RefCell<
        Option<(
            u64,
            std::rc::Rc<Vec<(crate::frontend::ui::BarItem, crate::frontend::ui::BrowserAct)>>,
        )>,
    >,
}

impl Browser {
    pub fn new(source: Source) -> Self {
        let sorting = match source {
            Source::Steam => "3",
            Source::Wallhaven => "toplist",
            _ => "",
        };
        Self {
            source,
            request: BrowserRequestState {
                query: String::new(),
                sorting: sorting.into(),
                nsfw: false,
                wallhaven: WallhavenRequestState {
                    general: true,
                    anime: true,
                    people: true,
                    sfw: true,
                    sketchy: false,
                    color: -1,
                    top_range: "1M".into(),
                    atleast: String::new(),
                    atmost: String::new(),
                    resolutions: String::new(),
                    res_exact: false,
                    ratios: String::new(),
                    collections: Vec::new(),
                    collection: String::new(),
                },
                steam: SteamRequestState {
                    kind: String::new(),
                    resolution: String::new(),
                    category: String::new(),
                    trend_days: "7".into(),
                },
                catalog: CatalogRequestState {
                    clip_start: String::new(),
                    clip_len: String::new(),
                    max_duration: String::new(),
                    unsplash: UnsplashRequestState {
                        order_by: "relevant".into(),
                        orientation: String::new(),
                        color: String::new(),
                        content_filter: "high".into(),
                    },
                    pexels: PexelsRequestState {
                        orientation: String::new(),
                        size: String::new(),
                        color: String::new(),
                    },
                },
            },
            session: BrowserSessionState {
                submitted_search: None,
                search_generation: 0,
                page: 1,
                last_page: 1,
                next_cursor: String::new(),
                loading: false,
                items: Vec::new(),
                hover: None,
                preview: None,
                error: None,
                page_failed: false,
                pending_apply: None,
            },
            view: BrowserViewState {
                thumb_fades: std::collections::HashMap::new(),
                hover_fades: std::collections::HashMap::new(),
                bar_items: std::cell::RefCell::new(None),
            },
        }
    }

    pub fn is_steam(&self) -> bool {
        self.source == Source::Steam
    }

    pub fn search_request(&self, page: u32, _after: &str) -> SearchRequest {
        match self.source {
            Source::Wallhaven => SearchRequest::Wallhaven(WallhavenSearch {
                query: self.request.query.clone(),
                categories: [
                    self.request.wallhaven.general,
                    self.request.wallhaven.anime,
                    self.request.wallhaven.people,
                ],
                sorting: self.request.sorting.clone(),
                purity: [
                    self.request.wallhaven.sfw,
                    self.request.wallhaven.sketchy,
                    self.request.nsfw,
                ],
                top_range: self.request.wallhaven.top_range.clone(),
                atleast: if self.request.wallhaven.res_exact {
                    String::new()
                } else {
                    self.request.wallhaven.atleast.clone()
                },
                atmost: if self.request.wallhaven.res_exact {
                    String::new()
                } else {
                    self.request.wallhaven.atmost.clone()
                },
                resolutions: if self.request.wallhaven.res_exact {
                    self.request.wallhaven.resolutions.clone()
                } else {
                    String::new()
                },
                ratios: self.request.wallhaven.ratios.clone(),
                collection: self.request.wallhaven.collection.clone(),
                color_hue: u8::try_from(self.request.wallhaven.color).ok().filter(|hue| *hue < 13),
                page,
            }),
            Source::Steam => SearchRequest::Steam(SteamSearch {
                query: self.request.query.clone(),
                query_type: self.request.sorting.parse::<u32>().unwrap_or(3),
                trend_days: self.request.steam.trend_days.parse::<u32>().unwrap_or(7).clamp(1, 7),
                request_type: self.request.steam.kind.clone(),
                category: self.request.steam.category.clone(),
                resolution: self.request.steam.resolution.clone(),
                allow_nsfw: self.request.nsfw,
                page,
            }),
            source => SearchRequest::Catalog(CatalogSearch {
                source,
                query: self.request.query.clone(),
                page,
                max_duration: self.request.catalog.max_duration.parse::<u64>().unwrap_or(0),
                order_by: self.request.catalog.unsplash.order_by.clone(),
                orientation: match source {
                    Source::Unsplash => self.request.catalog.unsplash.orientation.clone(),
                    Source::Pexels => self.request.catalog.pexels.orientation.clone(),
                    _ => String::new(),
                },
                size: self.request.catalog.pexels.size.clone(),
                color: match source {
                    Source::Unsplash => self.request.catalog.unsplash.color.clone(),
                    Source::Pexels => self.request.catalog.pexels.color.clone(),
                    _ => String::new(),
                },
                content_filter: self.request.catalog.unsplash.content_filter.clone(),
            }),
        }
    }

    pub fn download_request(&self, item: &BrowserItem) -> DownloadRequest {
        match self.source {
            Source::Steam => DownloadRequest::Steam { id: item.id.clone() },
            Source::Wallhaven => {
                DownloadRequest::Wallhaven { id: item.id.clone(), full_url: item.full_url.clone() }
            }
            source => DownloadRequest::Catalog {
                source,
                id: item.id.clone(),
                full_url: item.full_url.clone(),
                attribution: item.attribution.clone(),
                track_url: item.track_url.clone(),
                clip: (source == Source::Youtube).then(|| Clip {
                    start_secs: parse_clock(&self.request.catalog.clip_start).unwrap_or(0),
                    duration_secs: parse_clock(&self.request.catalog.clip_len),
                }),
            },
        }
    }

    pub fn preview_request(&self, item: &BrowserItem) -> PreviewRequest {
        PreviewRequest { source: self.source, id: item.id.clone(), full_url: item.full_url.clone() }
    }

    pub fn item_mut(&mut self, id: &str) -> Option<&mut BrowserItem> {
        self.session.items.iter_mut().find(|it| it.id == id)
    }

    pub fn mark_pending_download(&mut self, id: &str) {
        self.session.pending_apply = Some(id.to_string());
        if let Some(it) = self.item_mut(id) {
            it.downloading = true;
        }
    }

    pub fn preview_loading(&self) -> bool {
        self.session
            .preview
            .and_then(|idx| self.session.items.get(idx))
            .is_some_and(|it| it.preview_path.as_deref().unwrap_or("").is_empty())
    }

    pub fn thumbs_pending(&self) -> bool {
        self.session.items.iter().any(|it| !it.thumb_ready && !it.thumb_failed)
    }

    pub fn transfer_busy(&self) -> bool {
        self.session.pending_apply.is_some() || self.session.items.iter().any(|it| it.downloading)
    }

    pub fn busy(&self) -> bool {
        self.session.loading
            || self.thumbs_pending()
            || self.preview_loading()
            || self.transfer_busy()
    }
}

pub(super) fn fmt_size(bytes: u64) -> String {
    if bytes >= 1 << 20 {
        format!("{:.1} MB", bytes as f64 / (1u64 << 20) as f64)
    } else if bytes >= 1 << 10 {
        format!("{} KB", bytes / (1 << 10))
    } else {
        String::new()
    }
}

impl From<BrowserSearchResult> for BrowserItem {
    fn from(result: BrowserSearchResult) -> Self {
        Self {
            id: result.id,
            full_url: result.full_url,
            thumb_path: result.thumb_path,
            title: result.title,
            resolution: result.resolution,
            purity: result.purity,
            file_size: result.file_size,
            category: result.category,
            thumb_ready: result.thumb_ready,
            downloaded: result.downloaded,
            downloading: false,
            queued: false,
            progress: 0.0,
            phase: String::new(),
            duration_secs: result.duration_secs,
            downloaded_path: None,
            preview_path: None,
            thumb_failed: false,
            attribution: result.attribution,
            attribution_url: result.attribution_url,
            track_url: result.track_url,
        }
    }
}
