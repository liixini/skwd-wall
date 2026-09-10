use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use crate::domain::library::catalog::{Catalog, Wallpaper, WallpaperColor, WallpaperKind};

#[derive(Debug, Clone, Copy)]
pub struct LibraryPaths<'a> {
    wallpaper_dir: &'a str,
    video_dir: &'a str,
}

impl<'a> LibraryPaths<'a> {
    pub const fn new(wallpaper_dir: &'a str, video_dir: &'a str) -> Self {
        Self { wallpaper_dir, video_dir }
    }

    pub fn renamed_static_path(self, name: &str) -> String {
        format!("{}/{name}", self.wallpaper_dir)
    }
}

pub fn decode_rows(rows: &[wall_proto::WallpaperItem], paths: LibraryPaths<'_>) -> Catalog {
    let mut catalog = Catalog::default();
    catalog.items.reserve(rows.len());

    for row in rows {
        let Some(wallpaper) = decode_row(row, paths) else {
            continue;
        };
        let key = wallpaper.key.clone();

        if let Some(raw) = row.tags.as_deref() {
            let tags = parse_tag_list(raw);
            if !tags.is_empty() {
                catalog.tags.insert(key.clone(), tags);
            }
        }
        if let Some(raw) = row.colors.as_deref()
            && let Ok(value) = serde_json::from_str::<Value>(raw)
        {
            catalog.colors.insert(
                key.clone(),
                WallpaperColor { hue: skwd_config::i64_ref(&value, "hue", 99) },
            );
        }
        if let Some(raw) = row.weather.as_deref()
            && let Ok(Value::Array(values)) = serde_json::from_str::<Value>(raw)
        {
            let weather =
                values.into_iter().filter_map(|value| value.as_str().map(String::from)).collect();
            catalog.weather.insert(key.clone(), weather);
        }
        if row.favourite == Some(1) {
            catalog.favourites.insert(key);
        }
        catalog.items.push(wallpaper);
    }

    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    for tag in catalog.tags.values().flatten() {
        *tag_counts.entry(tag.clone()).or_default() += 1;
    }
    catalog.tag_vocab = tag_counts.keys().cloned().collect();
    catalog.tag_counts = tag_counts;
    catalog
}

pub fn decode_cached(data: &Value, paths: LibraryPaths<'_>) -> Option<Wallpaper> {
    let row = wall_proto::WallpaperItem::deserialize(data).ok()?;
    decode_row(&row, paths)
}

fn decode_row(row: &wall_proto::WallpaperItem, paths: LibraryPaths<'_>) -> Option<Wallpaper> {
    let name = sanitize_name(row.name.as_deref().filter(|name| !name.is_empty())?);
    if name.is_empty() {
        return None;
    }

    let key =
        row.key.as_deref().filter(|key| !key.is_empty()).map_or_else(|| name.clone(), String::from);
    let raw_kind = row.kind.as_deref().unwrap_or(wall_proto::kind::STATIC);
    if raw_kind == wall_proto::kind::SHADER {
        return None;
    }
    let thumb = row.thumb.clone().unwrap_or_default();
    if thumb.is_empty() {
        return None;
    }

    let kind = match raw_kind {
        wall_proto::kind::VIDEO => WallpaperKind::Video,
        wall_proto::kind::WE => WallpaperKind::We,
        _ => WallpaperKind::Static,
    };
    let we_id = row.we_id.clone().unwrap_or_default();
    let video_file = row.video_file.clone().unwrap_or_default();
    let path = match raw_kind {
        wall_proto::kind::STATIC => paths.renamed_static_path(&name),
        wall_proto::kind::VIDEO if video_file.is_empty() => {
            format!("{}/{name}", paths.video_dir)
        }
        wall_proto::kind::VIDEO => video_file.clone(),
        _ => String::new(),
    };

    Some(Wallpaper {
        key,
        name,
        kind,
        thumb,
        thumbnail_generated: row.thumbnail_generated,
        path,
        preview: row.preview.clone().unwrap_or_default(),
        we_id,
        video_file,
        mtime: row.mtime.unwrap_or(0),
        hue: row.hue.unwrap_or(99),
        sat: row.sat.unwrap_or(0),
        richness: row.richness.unwrap_or(0),
        apply_count: row.apply_count.unwrap_or(0),
        last_applied: row.last_applied.unwrap_or(0),
        width: row.width.unwrap_or(0),
        height: row.height.unwrap_or(0),
        filesize: row.filesize.unwrap_or(0),
        duration_ms: row.duration_ms.unwrap_or(0),
    })
}

fn sanitize_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut previous_slash = false;
    for ch in name.chars() {
        if ch == '/' {
            if previous_slash {
                continue;
            }
            previous_slash = true;
        } else {
            previous_slash = false;
        }
        out.push(ch);
    }
    while out.ends_with('/') {
        out.pop();
    }
    out
}

fn parse_tag_list(raw: &str) -> Vec<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Vec::new();
    }
    if raw.starts_with('[')
        && let Ok(Value::Array(values)) = serde_json::from_str::<Value>(raw)
    {
        return values
            .into_iter()
            .filter_map(|value| value.as_str().map(|tag| tag.trim().to_string()))
            .filter(|tag| !tag.is_empty())
            .collect();
    }
    raw.split(',').map(str::trim).filter(|tag| !tag.is_empty()).map(String::from).collect()
}
