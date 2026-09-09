use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum WallpaperKind {
    #[default]
    Static,
    Video,
    We,
}

impl WallpaperKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Video => "video",
            Self::We => "we",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Wallpaper {
    pub key: String,
    pub name: String,
    pub kind: WallpaperKind,
    pub thumb: String,
    pub path: String,
    pub preview: String,
    pub we_id: String,
    pub video_file: String,
    pub mtime: i64,
    pub hue: i64,
    pub sat: i64,
    pub richness: i64,
    pub apply_count: i64,
    pub last_applied: i64,
    pub width: i64,
    pub height: i64,
    pub filesize: i64,
    pub duration_ms: i64,
}

impl Wallpaper {
    pub fn backdrop_source(&self) -> String {
        if self.kind == WallpaperKind::We {
            let id = if self.we_id.is_empty() {
                self.key.strip_prefix("we:").unwrap_or("")
            } else {
                &self.we_id
            };
            if !id.is_empty() {
                return format!("we:{id}");
            }
        }
        self.path.clone()
    }

    pub fn effective_kind(&self) -> WallpaperKind {
        if self.kind == WallpaperKind::We && !self.video_file.is_empty() {
            WallpaperKind::Video
        } else {
            self.kind
        }
    }

    pub fn folder_path(&self) -> &str {
        if self.kind != WallpaperKind::Static {
            return "";
        }
        match self.name.rfind('/') {
            Some(idx) => &self.name[..idx],
            None => "",
        }
    }

    pub fn dedup_key(&self) -> &str {
        if self.we_id.is_empty() { &self.name } else { &self.we_id }
    }

    pub fn applied_key(&self) -> String {
        if self.we_id.is_empty() { stem(&self.name).to_string() } else { self.we_id.clone() }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WallpaperColor {
    pub hue: i64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Catalog {
    pub items: Vec<Wallpaper>,
    pub tags: HashMap<String, Vec<String>>,
    pub tag_vocab: HashSet<String>,
    pub tag_counts: HashMap<String, usize>,
    pub colors: HashMap<String, WallpaperColor>,
    pub weather: HashMap<String, Vec<String>>,
    pub favourites: HashSet<String>,
}

impl Catalog {
    pub fn is_favourite(&self, wallpaper: &Wallpaper) -> bool {
        self.favourites.contains(&wallpaper.key)
    }

    pub fn tags_for(&self, wallpaper: &Wallpaper) -> &[String] {
        self.tags.get(&wallpaper.key).map_or(&[], Vec::as_slice)
    }

    pub fn available_folders(&self) -> Vec<String> {
        let mut seen = HashSet::new();
        for wallpaper in &self.items {
            let folder = wallpaper.folder_path();
            if folder.is_empty() {
                continue;
            }
            let mut prefix = String::new();
            for segment in folder.split('/') {
                if segment.is_empty() {
                    continue;
                }
                if !prefix.is_empty() {
                    prefix.push('/');
                }
                prefix.push_str(segment);
                seen.insert(prefix.clone());
            }
        }
        let mut folders: Vec<String> = seen.into_iter().collect();
        folders.sort();
        folders
    }
}

fn stem(name: &str) -> &str {
    match name.rfind('.') {
        Some(dot) if dot > 0 => &name[..dot],
        _ => name,
    }
}
