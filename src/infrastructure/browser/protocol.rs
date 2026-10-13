use serde_json::{Value, json};

use crate::contracts::browser::{
    ApplyKind, ApplyTarget, CatalogSearch, DownloadRequest, DownloadStatus, DownloadUpdate,
    PreviewRequest, SearchRequest, Source, SteamSearch, WallhavenSearch,
};

const STEAM_NSFW_TAGS: [&str; 6] =
    ["Mature", "Questionable", "NSFW", "Partial Nudity", "Nudity", "Gore"];
const STEAM_UNSUPPORTED_TYPES: [&str; 2] = ["Web", "Application"];
const STEAM_RESOLUTION_TAGS: [(&str, &str); 3] = [
    ("2560 x 1080", "Ultrawide 2560 x 1080"),
    ("3440 x 1440", "Ultrawide 3440 x 1440"),
    ("3840 x 1080", "Dual 3840 x 1080"),
];

pub struct RpcCall {
    pub method: &'static str,
    pub params: Value,
}

pub fn encode_search(request: &SearchRequest) -> RpcCall {
    match request {
        SearchRequest::Wallhaven(request) => encode_wallhaven_search(request),
        SearchRequest::Steam(request) => encode_steam_search(request),
        SearchRequest::Catalog(request) => encode_catalog_search(request),
    }
}

fn encode_wallhaven_search(request: &WallhavenSearch) -> RpcCall {
    let categories = bool_bits(request.categories);
    let purity = bool_bits(request.purity);
    RpcCall {
        method: "wallhaven.search",
        params: json!({
            "query": request.query,
            "categories": categories,
            "sorting": request.sorting,
            "order": "desc",
            "purity": purity,
            "topRange": request.top_range,
            "atleast": request.atleast,
            "atmost": request.atmost,
            "resolutions": request.resolutions,
            "ratios": request.ratios,
            "collection": request.collection,
            "colors": wallhaven_color(request.color_hue),
            "page": request.page,
        }),
    }
}

fn encode_steam_search(request: &SteamSearch) -> RpcCall {
    let resolution = STEAM_RESOLUTION_TAGS
        .iter()
        .find_map(|(choice, tag)| (request.resolution == *choice).then_some(*tag))
        .unwrap_or(&request.resolution);
    let tags: Vec<&str> = [request.request_type.as_str(), request.category.as_str(), resolution]
        .into_iter()
        .filter(|tag| !tag.is_empty())
        .collect();
    let excluded_tags: Vec<&str> = STEAM_UNSUPPORTED_TYPES
        .iter()
        .copied()
        .chain(
            (!request.allow_nsfw)
                .then_some(STEAM_NSFW_TAGS.as_slice())
                .into_iter()
                .flatten()
                .copied(),
        )
        .collect();
    RpcCall {
        method: "steam.search",
        params: json!({
            "query": request.query,
            "query_type": request.query_type,
            "days": request.trend_days,
            "tags": tags,
            "excluded_tags": excluded_tags,
            "page": request.page,
        }),
    }
}

fn encode_catalog_search(request: &CatalogSearch) -> RpcCall {
    let mut params = json!({
        "source": request.source.key(),
        "query": request.query,
        "page": request.page,
    });
    match request.source {
        Source::Unsplash => {
            params["order_by"] = json!(request.order_by);
            params["orientation"] = json!(request.orientation);
            params["color"] = json!(request.color);
            params["content_filter"] = json!(request.content_filter);
        }
        Source::Pexels => {
            params["orientation"] = json!(request.orientation);
            params["size"] = json!(request.size);
            params["color"] = json!(request.color);
        }
        Source::Youtube => params["max_duration"] = json!(request.max_duration),
        Source::Wallhaven | Source::Steam | Source::Bing => {}
    }
    RpcCall { method: "source.list", params }
}

pub fn encode_download(request: &DownloadRequest) -> RpcCall {
    match request {
        DownloadRequest::Steam { id } => {
            RpcCall { method: "steam.download", params: json!({ "id": id }) }
        }
        DownloadRequest::Wallhaven { id, full_url } => RpcCall {
            method: "wallhaven.download",
            params: json!({ "id": id, "full_url": full_url }),
        },
        DownloadRequest::Catalog { source, id, full_url, attribution, track_url, clip } => {
            let mut params = json!({
                "source": source.key(),
                "id": id,
                "full_url": full_url,
                "attribution": attribution,
                "track_url": track_url,
            });
            if let Some(clip) = clip {
                params["start"] = json!(clip.start_secs);
                if let Some(duration) = clip.duration_secs {
                    params["dur"] = json!(duration);
                }
            }
            RpcCall { method: "source.download", params }
        }
    }
}

pub fn encode_preview(request: &PreviewRequest) -> RpcCall {
    match request.source {
        Source::Steam => RpcCall {
            method: "steam.preview",
            params: json!({ "id": request.id, "full_url": request.full_url }),
        },
        Source::Wallhaven => RpcCall {
            method: "wallhaven.preview",
            params: json!({ "id": request.id, "full_url": request.full_url }),
        },
        source => RpcCall {
            method: "source.preview",
            params: json!({
                "source": source.key(),
                "id": request.id,
                "full_url": request.full_url,
            }),
        },
    }
}

pub fn encode_apply(target: &ApplyTarget) -> Value {
    match target {
        ApplyTarget::Path { kind, path } => json!({
            "type": apply_kind(*kind),
            "path": path,
        }),
        ApplyTarget::WallpaperEngine { id } => json!({
            "type": wall_proto::kind::WE,
            "we_id": id,
        }),
    }
}

#[cfg(test)]
pub fn decode_preview_ready_path(result: &Value) -> Option<String> {
    if result.get("status").and_then(Value::as_str) != Some("ready") {
        return None;
    }
    result.get("path").and_then(Value::as_str).filter(|path| !path.is_empty()).map(str::to_string)
}

pub fn decode_download_event(data: &Value) -> Option<DownloadUpdate> {
    let event = serde_json::from_value::<wall_proto::DownloadEvent>(data.clone()).ok()?;
    if event.id.is_empty() {
        return None;
    }
    let status = match event.status.as_str() {
        wall_proto::dl_status::QUEUED => DownloadStatus::Queued,
        wall_proto::dl_status::DOWNLOADING => DownloadStatus::Downloading,
        wall_proto::dl_status::DONE => DownloadStatus::Done,
        wall_proto::dl_status::ERROR | wall_proto::dl_status::AUTH_ERROR => DownloadStatus::Failed,
        _ => DownloadStatus::Other,
    };
    Some(DownloadUpdate {
        id: event.id,
        status,
        path: event.path,
        progress: event.progress.map(|progress| progress as f32),
        message: event.message.or(event.error).filter(|message| !message.is_empty()),
    })
}

fn bool_bits(values: [bool; 3]) -> String {
    values.into_iter().map(|value| if value { '1' } else { '0' }).collect()
}

fn wallhaven_color(hue: Option<u8>) -> &'static str {
    const COLORS: [&str; 13] = [
        "cc0000", "ff6600", "999900", "669900", "336600", "66cccc", "0099cc", "0066cc", "333399",
        "663399", "993399", "ea4c88", "999999",
    ];
    hue.and_then(|index| COLORS.get(usize::from(index))).copied().unwrap_or("")
}

fn apply_kind(kind: ApplyKind) -> &'static str {
    match kind {
        ApplyKind::Static => wall_proto::kind::STATIC,
        ApplyKind::Video => wall_proto::kind::VIDEO,
        ApplyKind::WallpaperEngine => wall_proto::kind::WE,
    }
}
