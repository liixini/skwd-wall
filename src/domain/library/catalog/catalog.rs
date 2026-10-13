use std::collections::{HashMap, HashSet};

const WE_PREFIX: &str = "we:";

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
    pub thumbnail_generated: bool,
    pub steam_subscribed: bool,
    pub steam_confirmation_required: bool,
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
    pub fn engine_id(&self) -> &str {
        if self.we_id.is_empty() {
            self.key.strip_prefix(WE_PREFIX).unwrap_or("")
        } else {
            &self.we_id
        }
    }

    pub fn backdrop_source(&self) -> String {
        if self.kind == WallpaperKind::We {
            let id = self.engine_id();
            if !id.is_empty() {
                return format!("we:{id}");
            }
        }
        self.path.clone()
    }

    pub fn is_tall(&self) -> bool {
        self.height > self.width
            && self.width > 0
            && matches!(self.effective_kind(), WallpaperKind::Static | WallpaperKind::Video)
    }

    pub fn effective_kind(&self) -> WallpaperKind {
        if self.kind == WallpaperKind::We && !self.video_file.is_empty() {
            WallpaperKind::Video
        } else {
            self.kind
        }
    }

    pub fn folder_path(&self) -> &str {
        if self.kind == WallpaperKind::We {
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
    pub hidden: HashSet<String>,
    pub profile_hidden: HashSet<String>,
    pub profile_excluded: HashSet<String>,
}

impl Catalog {
    pub fn is_favourite(&self, wallpaper: &Wallpaper) -> bool {
        self.favourites.contains(&wallpaper.key)
    }

    pub fn is_hidden(&self, wallpaper: &Wallpaper) -> bool {
        self.hidden.contains(&wallpaper.key) || self.profile_hidden.contains(&wallpaper.key)
    }

    pub fn has_hidden(&self) -> bool {
        !self.hidden.is_empty() || !self.profile_hidden.is_empty()
    }

    pub fn is_shown(&self, wallpaper: &Wallpaper, reveal: bool) -> bool {
        !self.profile_excluded.contains(&wallpaper.key) && (reveal || !self.is_hidden(wallpaper))
    }

    pub fn tags_for(&self, wallpaper: &Wallpaper) -> &[String] {
        self.tags.get(&wallpaper.key).map_or(&[], Vec::as_slice)
    }

    pub fn available_folders(&self, roots: &[String], reveal: Option<bool>) -> Vec<String> {
        let seen = self.populated_folders(roots, reveal);
        let all = reveal.map(|_| self.populated_folders(roots, None));
        let mut folders = Vec::new();
        for root in roots {
            let hidden_root =
                all.as_ref().is_some_and(|all| all.contains(root) && !seen.contains(root));
            if folders.contains(root) || hidden_root {
                continue;
            }
            folders.push(root.clone());
            let mut descendants: Vec<_> = seen
                .iter()
                .filter(|folder| {
                    std::path::Path::new(folder).starts_with(root)
                        && *folder != root
                        && !roots.contains(folder)
                        && !roots.iter().any(|nested| {
                            nested != root
                                && std::path::Path::new(nested).starts_with(root)
                                && std::path::Path::new(folder).starts_with(nested)
                        })
                })
                .cloned()
                .collect();
            descendants.sort();
            folders.extend(descendants);
        }
        folders
    }
}

impl Catalog {
    fn populated_folders(&self, roots: &[String], reveal: Option<bool>) -> HashSet<String> {
        let mut seen = HashSet::new();
        for wallpaper in &self.items {
            if wallpaper.kind == WallpaperKind::We
                || reveal.is_some_and(|reveal| !self.is_shown(wallpaper, reveal))
            {
                continue;
            }
            let path = std::path::Path::new(&wallpaper.path);
            let Some(root) =
                roots.iter().filter(|root| path.starts_with(root)).max_by_key(|root| root.len())
            else {
                continue;
            };
            let Some(parent) = path.parent() else { continue };
            for folder in parent.ancestors().take_while(|folder| folder.starts_with(root)) {
                seen.insert(folder.to_string_lossy().into_owned());
            }
        }
        seen
    }
}

fn stem(name: &str) -> &str {
    match name.rfind('.') {
        Some(dot) if dot > 0 => &name[..dot],
        _ => name,
    }
}
