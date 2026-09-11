use std::cmp::Ordering;

use crate::domain::library::catalog::{Catalog, Wallpaper, WallpaperKind};
use crate::domain::library::search::NumericQuery;

#[derive(Debug, Clone, PartialEq)]
pub struct Filters {
    pub color: i64,
    pub kind: String,
    pub folder: String,
    pub show_hidden_folders: bool,
    pub tags: Vec<String>,
    pub numeric: NumericQuery,
    pub tags_match_any: bool,
    pub favourites_only: bool,
    pub weather_active: bool,
    pub current_weather: Vec<String>,
    pub sort: String,
    pub orient: String,
    pub resolution: String,
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            color: -1,
            kind: String::new(),
            folder: String::from("*"),
            show_hidden_folders: true,
            tags: Vec::new(),
            numeric: NumericQuery::default(),
            tags_match_any: false,
            favourites_only: false,
            weather_active: false,
            current_weather: Vec::new(),
            sort: String::from("color"),
            orient: String::new(),
            resolution: String::new(),
        }
    }
}

impl Filters {
    pub fn folder_visible(&self, folder: &str) -> bool {
        self.show_hidden_folders || !folder.split('/').any(|part| part.starts_with('.'))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionPreset {
    pub label: String,
    pub orientation: String,
    pub from_width: i64,
    pub from_height: i64,
    pub to_width: Option<i64>,
    pub to_height: Option<i64>,
}

impl ResolutionPreset {
    pub fn key(&self) -> String {
        let orientation =
            resolution_orientation(&self.orientation, self.from_width, self.from_height);
        let (from_width, from_height) =
            oriented_resolution(orientation, self.from_width, self.from_height);
        let from = format!("{from_width}x{from_height}");
        match (self.to_width, self.to_height) {
            (Some(width), Some(height)) => {
                let (width, height) = oriented_resolution(orientation, width, height);
                format!("{}:{from}..{width}x{height}", orientation.key())
            }
            _ => format!("{}:{from}..", orientation.key()),
        }
    }

    pub fn matches_shape(&self, shape: &str) -> bool {
        match shape {
            "landscape" => self.orientation == "wide",
            "portrait" => self.orientation == "tall",
            _ => true,
        }
    }
}

pub fn filter_sort(catalog: &Catalog, filters: &Filters) -> Vec<u32> {
    let mut out: Vec<u32> = Vec::with_capacity(catalog.items.len());
    for (idx, item) in catalog.items.iter().enumerate() {
        if keep(catalog, filters, item) {
            out.push(idx as u32);
        }
    }
    if filters.weather_active
        && !filters.current_weather.is_empty()
        && out.iter().any(|&idx| weather_matches(catalog, filters, &catalog.items[idx as usize]))
    {
        out.retain(|&idx| weather_matches(catalog, filters, &catalog.items[idx as usize]));
    }
    sort_indices(catalog, filters, &mut out);
    out
}

fn keep(catalog: &Catalog, filters: &Filters, item: &Wallpaper) -> bool {
    folder_ok(filters, item)
        && (filters.kind.is_empty() || item.effective_kind().as_str() == filters.kind)
        && color_ok(filters, item)
        && (!filters.favourites_only || catalog.is_favourite(item))
        && orient_ok(filters, item)
        && resolution_ok(filters, item)
        && tags_ok(catalog, filters, item)
        && filters.numeric.matches(item)
}

fn resolution_ok(filters: &Filters, item: &Wallpaper) -> bool {
    let Some((orientation, (from_width, from_height), to)) =
        parse_resolution_range(&filters.resolution)
    else {
        return filters.resolution.is_empty();
    };
    if !orientation.matches(item.width, item.height) {
        return false;
    }
    let (width, height) = oriented_resolution(orientation, item.width, item.height);
    let (from_width, from_height) = oriented_resolution(orientation, from_width, from_height);
    let above_minimum = width >= from_width && height >= from_height;
    match to.map(|(width, height)| oriented_resolution(orientation, width, height)) {
        Some((to_width, to_height)) => above_minimum && width <= to_width && height <= to_height,
        None => above_minimum,
    }
}

pub fn parse_resolution(value: &str) -> Option<(i64, i64)> {
    let (width, height) = value.split_once(['x', 'X', '×'])?;
    let width = width.trim().parse::<i64>().ok()?;
    let height = height.trim().parse::<i64>().ok()?;
    (width > 0 && height > 0).then_some((width, height))
}

fn parse_resolution_range(
    value: &str,
) -> Option<(ResolutionOrientation, (i64, i64), Option<(i64, i64)>)> {
    let (configured_orientation, value) = match value.split_once(':') {
        Some(("wide", range)) => (Some(ResolutionOrientation::Wide), range),
        Some(("tall", range)) => (Some(ResolutionOrientation::Tall), range),
        Some(_) => return None,
        None => (None, value),
    };
    let Some((from, to)) = value.split_once("..") else {
        let exact = parse_resolution(value)?;
        let orientation =
            configured_orientation.unwrap_or_else(|| resolution_orientation("", exact.0, exact.1));
        return Some((orientation, exact, Some(exact)));
    };
    let from = parse_resolution(from)?;
    let orientation =
        configured_orientation.unwrap_or_else(|| resolution_orientation("", from.0, from.1));
    let to = if to.trim().is_empty() { None } else { Some(parse_resolution(to)?) };
    let valid = to.is_none_or(|to| {
        let from = oriented_resolution(orientation, from.0, from.1);
        let to = oriented_resolution(orientation, to.0, to.1);
        from.0 <= to.0 && from.1 <= to.1
    });
    valid.then_some((orientation, from, to))
}

#[derive(Clone, Copy)]
enum ResolutionOrientation {
    Wide,
    Tall,
}

impl ResolutionOrientation {
    const fn key(self) -> &'static str {
        match self {
            Self::Wide => "wide",
            Self::Tall => "tall",
        }
    }

    const fn matches(self, width: i64, height: i64) -> bool {
        match self {
            Self::Wide => width >= height,
            Self::Tall => height > width,
        }
    }
}

fn resolution_orientation(configured: &str, width: i64, height: i64) -> ResolutionOrientation {
    match configured {
        "tall" => ResolutionOrientation::Tall,
        "wide" => ResolutionOrientation::Wide,
        _ if height > width => ResolutionOrientation::Tall,
        _ => ResolutionOrientation::Wide,
    }
}

fn oriented_resolution(orientation: ResolutionOrientation, width: i64, height: i64) -> (i64, i64) {
    match orientation {
        ResolutionOrientation::Wide => (width.max(height), width.min(height)),
        ResolutionOrientation::Tall => (width.min(height), width.max(height)),
    }
}

fn orient_ok(filters: &Filters, item: &Wallpaper) -> bool {
    match filters.orient.as_str() {
        "landscape" => item.width > 0 && item.width >= item.height,
        "portrait" => item.height > 0 && item.height > item.width,
        _ => true,
    }
}

fn folder_ok(filters: &Filters, item: &Wallpaper) -> bool {
    let directory = item.name.rsplit_once('/').map_or("", |(folder, _)| folder);
    if item.kind != WallpaperKind::We && !filters.folder_visible(directory) {
        return false;
    }
    if filters.folder == "*" {
        return true;
    }
    let fp = item.folder_path();
    if filters.folder.is_empty() {
        return fp.is_empty();
    }
    fp == filters.folder
        || fp.strip_prefix(filters.folder.as_str()).is_some_and(|rest| rest.starts_with('/'))
}

fn color_ok(filters: &Filters, item: &Wallpaper) -> bool {
    if filters.color == -1 {
        return true;
    }
    item.hue == filters.color
}

fn tags_ok(catalog: &Catalog, filters: &Filters, item: &Wallpaper) -> bool {
    if filters.tags.is_empty() {
        return true;
    }
    let item_tags = catalog.tags.get(&item.key).map_or(&[][..], Vec::as_slice);
    let mut has_positive = false;
    let mut any_matched = false;
    let mut all_matched = true;
    for raw in &filters.tags {
        let Some(excluded) = raw.strip_prefix('-') else {
            has_positive = true;
            if item_tags.iter().any(|tag| tag == raw) {
                any_matched = true;
            } else {
                all_matched = false;
            }
            continue;
        };
        if !excluded.is_empty() && item_tags.iter().any(|tag| tag == excluded) {
            return false;
        }
    }
    if !has_positive {
        return true;
    }
    if item_tags.is_empty() {
        return false;
    }
    if filters.tags_match_any { any_matched } else { all_matched }
}

fn weather_matches(catalog: &Catalog, filters: &Filters, item: &Wallpaper) -> bool {
    match catalog.weather.get(&item.key) {
        Some(wp) if !wp.is_empty() => filters.current_weather.iter().any(|cur| wp.contains(cur)),
        _ => false,
    }
}

fn sort_indices(catalog: &Catalog, filters: &Filters, indices: &mut [u32]) {
    let items = &catalog.items;
    match filters.sort.as_str() {
        "date" => sort_with(items, indices, cmp_date),
        "recent" => sort_with(items, indices, cmp_recent),
        "pop" => sort_with(items, indices, cmp_pop),
        "richness" => sort_with(items, indices, cmp_richness),
        "minimalist" => sort_with(items, indices, cmp_minimalist),
        "res" => sort_with(items, indices, cmp_res),
        _ => sort_with(items, indices, cmp_color),
    }
}

fn sort_with(
    items: &[Wallpaper],
    indices: &mut [u32],
    compare: impl Fn(&Wallpaper, &Wallpaper) -> Ordering,
) {
    indices.sort_by(|&lhs, &rhs| compare(&items[lhs as usize], &items[rhs as usize]));
}

fn cmp_for(sort: &str) -> fn(&Wallpaper, &Wallpaper) -> Ordering {
    match sort {
        "date" => cmp_date,
        "recent" => cmp_recent,
        "pop" => cmp_pop,
        "richness" => cmp_richness,
        "minimalist" => cmp_minimalist,
        "res" => cmp_res,
        _ => cmp_color,
    }
}

fn cmp_date(ia: &Wallpaper, ib: &Wallpaper) -> Ordering {
    ib.mtime.cmp(&ia.mtime)
}

fn cmp_recent(ia: &Wallpaper, ib: &Wallpaper) -> Ordering {
    ib.last_applied.cmp(&ia.last_applied).then(ib.mtime.cmp(&ia.mtime))
}

fn cmp_pop(ia: &Wallpaper, ib: &Wallpaper) -> Ordering {
    let pa = ia.sat as f64 - ia.richness as f64 / 15.0;
    let pb = ib.sat as f64 - ib.richness as f64 / 15.0;
    pb.partial_cmp(&pa).unwrap_or(Ordering::Equal)
}

fn cmp_richness(ia: &Wallpaper, ib: &Wallpaper) -> Ordering {
    ib.richness.cmp(&ia.richness).then(ib.sat.cmp(&ia.sat))
}

fn cmp_minimalist(ia: &Wallpaper, ib: &Wallpaper) -> Ordering {
    ia.richness.cmp(&ib.richness).then(ib.sat.cmp(&ia.sat))
}

fn cmp_res(ia: &Wallpaper, ib: &Wallpaper) -> Ordering {
    let area_a = ia.width * ia.height;
    let area_b = ib.width * ib.height;
    area_b.cmp(&area_a).then(ib.width.cmp(&ia.width)).then(ib.mtime.cmp(&ia.mtime))
}

fn cmp_color(ia: &Wallpaper, ib: &Wallpaper) -> Ordering {
    let ha = if ia.hue == 99 { 100 } else { ia.hue };
    let hb = if ib.hue == 99 { 100 } else { ib.hue };
    ha.cmp(&hb).then(ib.sat.cmp(&ia.sat))
}

pub fn insert_index(catalog: &Catalog, filters: &Filters, filtered: &[u32], new_idx: u32) -> usize {
    let items = &catalog.items;
    let item = &items[new_idx as usize];
    let cmp = cmp_for(filters.sort.as_str());
    for (pos, &ex_idx) in filtered.iter().enumerate() {
        if cmp(item, &items[ex_idx as usize]) == Ordering::Less {
            return pos;
        }
    }
    filtered.len()
}
