use std::path::Path;

use serde_json::Value;

use crate::contracts::browser::{
    BrowserCollection, BrowserSearchResult, DownloadResponse, SearchPage, WorkshopDetails,
    WorkshopGallery, WorkshopPreview,
};

use super::common::{DecodeResult, envelope, required_array, required_string, root};

const DETAILS_METHOD: &str = wall_proto::rpc::SOURCE_DETAILS;
const ID_FIELD: &str = "id";
const PREVIEWS_FIELD: &str = "previews";
const CREATOR_FIELD: &str = "creator_name";
const PREVIEWS_SHAPE: &str = "Workshop previews";

pub fn decode_browser_search(value: &Value) -> DecodeResult<SearchPage> {
    let object = envelope("browser.search", value)?;
    required_array("browser.search", object, "results")?;
    let result = root::<wall_proto::sources::ListResult>("browser.search", value, "search page")?;
    Ok(SearchPage {
        generation: result.generation,
        last_page: result.last_page,
        current_page: result.current_page,
        next_cursor: result.next_cursor.unwrap_or_default(),
        results: result.results.iter().map(decode_search_result).collect(),
    })
}

pub fn decode_browser_download(value: &Value) -> DecodeResult<DownloadResponse> {
    let object = envelope("browser.download", value)?;
    Ok(match required_string("browser.download", object, "status")?.as_str() {
        "exists" => DownloadResponse::Exists,
        "done" => DownloadResponse::Done,
        "started" => DownloadResponse::Started,
        _ => DownloadResponse::Other,
    })
}

pub fn decode_browser_collections(value: &Value) -> DecodeResult<Vec<BrowserCollection>> {
    let object = envelope("browser.collections", value)?;
    Ok(required_array("browser.collections", object, "collections")?
        .iter()
        .filter_map(|collection| {
            let object = collection.as_object()?;
            Some(BrowserCollection {
                id: object.get("id")?.as_u64()?.to_string(),
                label: object.get("label").and_then(Value::as_str).unwrap_or_default().to_string(),
            })
        })
        .collect())
}

fn decode_search_result(row: &wall_proto::sources::ListItem) -> BrowserSearchResult {
    let thumb_path = row.thumb_path.clone();
    let thumb_ready = !thumb_path.is_empty() && Path::new(&thumb_path).exists();
    BrowserSearchResult {
        id: row.id.clone(),
        full_url: row.full_url.clone(),
        thumb_path,
        title: row.title.clone(),
        resolution: row.resolution.clone(),
        purity: row.purity.clone(),
        file_size: row.file_size,
        category: row.category.clone(),
        duration_secs: row.duration_secs,
        downloaded: row.downloaded,
        attribution: row.attribution.clone(),
        attribution_url: row.attribution_url.clone(),
        track_url: row.track_url.clone(),
        thumb_ready,
        workshop: row.workshop.as_ref().map(|details| WorkshopDetails {
            description: details.description.clone(),
            creator_id: details.creator_id.clone(),
            creator_name: details.creator_name.clone(),
            subscriptions: details.subscriptions,
            votes_up: details.votes_up,
            votes_down: details.votes_down,
            time_created: details.time_created,
            time_updated: details.time_updated,
            previews: decode_workshop_previews(&details.previews),
        }),
    }
}

pub fn decode_workshop_gallery(value: &Value) -> DecodeResult<WorkshopGallery> {
    let object = envelope(DETAILS_METHOD, value)?;
    let id = required_string(DETAILS_METHOD, object, ID_FIELD)?;
    let previews = required_array(DETAILS_METHOD, object, PREVIEWS_FIELD)?;
    let previews = serde_json::from_value::<Vec<wall_proto::sources::WorkshopPreview>>(
        Value::Array(previews.to_vec()),
    )
    .map_err(|_| super::common::DecodeError::InvalidField {
        family: DETAILS_METHOD,
        field: PREVIEWS_FIELD,
        expected: PREVIEWS_SHAPE,
    })?;
    let creator_name =
        object.get(CREATOR_FIELD).and_then(Value::as_str).unwrap_or_default().to_owned();
    Ok(WorkshopGallery { id, previews: decode_workshop_previews(&previews), creator_name })
}

fn decode_workshop_previews(
    previews: &[wall_proto::sources::WorkshopPreview],
) -> Vec<WorkshopPreview> {
    previews
        .iter()
        .map(|preview| match preview {
            wall_proto::sources::WorkshopPreview::Image { url } => {
                WorkshopPreview::Image { url: url.clone() }
            }
            wall_proto::sources::WorkshopPreview::Youtube { id } => {
                WorkshopPreview::Youtube { id: id.clone() }
            }
        })
        .collect()
}
