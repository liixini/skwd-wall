use serde_json::{Value, json};

use super::*;
use crate::contracts::browser::DownloadResponse;
use crate::contracts::daemon::TaskState;
use crate::contracts::media::MediaKind;
use crate::contracts::playlists::PlaylistKind;
use crate::infrastructure::library::LibraryPaths;

fn envelope_contract<T>(valid: Value, decode: impl Fn(&Value) -> DecodeResult<T>) {
    assert!(decode(&valid).is_ok());
    let mut compatible = valid.clone();
    let object = compatible.as_object_mut().expect("fixture object");
    object.insert("schema_version".into(), json!(1));
    object.insert("future".into(), json!({"nested": true}));
    assert!(decode(&compatible).is_ok());

    let mut unknown = valid;
    unknown.as_object_mut().expect("fixture object").insert("schema_version".into(), json!(2));
    assert!(matches!(decode(&unknown), Err(DecodeError::UnknownVersion { version: 2, .. })));
    assert!(matches!(decode(&json!([])), Err(DecodeError::ExpectedObject { .. })));
}

#[test]
fn presentation_boundaries_validate_envelopes_and_shapes() {
    envelope_contract(json!({"outputs": []}), decode_outputs);
    envelope_contract(json!({"outputs": []}), decode_audio_outputs);
    envelope_contract(json!({}), decode_theme_backends);
    envelope_contract(json!({"colors": []}), decode_theme_preview);
    envelope_contract(
        json!({"backend": "skwd-iris", "backends": [], "previews": []}),
        decode_theme_previews,
    );

    assert!(decode_outputs(&json!({})).is_err());
    assert!(decode_audio_outputs(&json!({})).is_err());
    assert!(decode_outputs(&json!({"outputs": "invalid"})).is_err());
    assert!(decode_audio_outputs(&json!({"outputs": {}})).is_err());
    assert!(decode_theme_backends(&json!({"backends": false})).is_err());
    assert!(decode_theme_preview(&json!({"colors": {}})).is_err());
    assert!(decode_theme_preview(&json!({})).is_err());
    assert!(decode_theme_previews(&json!({"previews": "invalid"})).is_err());
    assert!(decode_theme_previews(&json!({"backend": "skwd-iris"})).is_err());

    let outputs = decode_outputs(&json!({
        "outputs": [
            {"name": "DP-1", "type": "video", "future": true},
            {"name": "DP-2", "type": "future-media"}
        ],
        "future": 7
    }))
    .unwrap();
    assert_eq!(outputs.outputs[0].name, "DP-1");
    assert_eq!(outputs.outputs[0].kind, MediaKind::Video);
    assert_eq!(outputs.outputs[1].kind, MediaKind::Other("future-media".into()));
    let preview = decode_theme_previews(&json!({
        "backend": "matugen",
        "backends": ["matugen", 7],
        "previews": [{
            "backend": "matugen",
            "key": "primary",
            "value": "#ffffff",
            "label": "Primary",
            "palette": {"primary": "#ffffff", "future": "ignored"},
            "future": true
        }],
        "future": true
    }))
    .unwrap();
    assert_eq!(preview.backends, ["matugen"]);
    assert_eq!(preview.previews[0].palette.primary.as_deref(), Some("#ffffff"));
}

#[test]
fn playlist_boundaries_map_consumer_types() {
    envelope_contract(json!({}), decode_playlist_list);
    envelope_contract(json!({}), decode_playlist_members);
    envelope_contract(json!({}), decode_playlist_outputs);
    envelope_contract(json!({}), decode_card_picker_memberships);
    envelope_contract(json!({}), decode_card_picker_create);

    assert!(decode_playlist_list(&json!({"playlists": false})).is_err());
    assert!(decode_playlist_members(&json!({"members": {}})).is_err());
    assert!(decode_playlist_outputs(&json!({"outputs": "invalid"})).is_err());
    assert!(decode_card_picker_memberships(&json!({"ids": {}})).is_err());
    assert!(decode_card_picker_create(&json!({"id": "invalid"})).is_err());

    let result = decode_playlist_list(&json!({
        "playlists": [
            {"id": 1, "kind": "smart", "future": true},
            {"id": 2, "kind": "future-kind"}
        ],
        "assign": [{"output": "DP-1", "id": 1, "future": true}],
        "future": true
    }))
    .unwrap();
    let playlists = result.playlists.unwrap();
    assert_eq!(playlists[0].kind, PlaylistKind::Smart);
    assert_eq!(playlists[1].kind, PlaylistKind::Other("future-kind".into()));
    assert_eq!(result.assignments.unwrap()[0].output, "DP-1");

    let members = decode_playlist_members(&json!({
        "id": 1,
        "members": [{"key": "wall", "type": "video", "future": true}],
        "future": true
    }))
    .unwrap();
    assert_eq!(members.members.unwrap()[0].key.as_deref(), Some("wall"));
}

#[test]
fn effects_boundaries_return_domain_definitions() {
    envelope_contract(json!({"output": ""}), decode_effect_operation);
    envelope_contract(json!({}), decode_effects_list);

    assert!(decode_effect_operation(&json!({})).is_err());
    assert!(decode_effect_operation(&json!({"output": 7})).is_err());
    assert!(decode_effects_list(&json!({"effects": {}})).is_err());

    let result = decode_effects_list(&json!({
        "effects": [{
            "id": "theme",
            "future": true,
            "params": [{
                "id": "theme",
                "type": "dropdown",
                "options": ["Nord", {"mode": "Skwd", "future": true}]
            }]
        }],
        "future": true
    }))
    .unwrap();
    assert_eq!(result.definitions.unwrap()[0].id, "theme");
    assert_eq!(result.theme_options.unwrap(), ["Nord", "Skwd"]);
}

#[test]
fn diagnostic_boundaries_preserve_partial_and_future_fields() {
    envelope_contract(json!({}), decode_diagnostic);
    envelope_contract(json!({"weather": []}), decode_weather);
    envelope_contract(json!({}), decode_bug_report);

    assert!(decode_weather(&json!({})).is_err());
    assert!(decode_diagnostic(&json!({"banner": 7})).is_err());
    assert!(decode_weather(&json!({"weather": {}})).is_err());
    assert!(decode_bug_report(&json!({"path": false})).is_err());

    assert_eq!(decode_weather(&json!({"weather": ["rain", 7]})).unwrap().weather, ["rain"]);
}

#[test]
fn browser_boundaries_validate_remaining_result_envelopes() {
    envelope_contract(json!({"results": []}), decode_browser_search);
    envelope_contract(json!({"collections": []}), decode_browser_collections);
    envelope_contract(json!({"status": "future"}), decode_browser_download);

    assert!(decode_browser_search(&json!({})).is_err());
    assert!(decode_browser_collections(&json!({})).is_err());
    assert!(decode_browser_download(&json!({})).is_err());
    assert!(decode_browser_search(&json!({"results": "invalid"})).is_err());
    assert!(decode_browser_collections(&json!({"collections": {}})).is_err());
    assert!(decode_browser_download(&json!({"status": 7})).is_err());

    let page = decode_browser_search(&json!({
        "generation": 9,
        "results": [{"id": "wall", "future": true}],
        "future": true
    }))
    .unwrap();
    assert_eq!(page.generation, Some(9));
    assert_eq!(page.results[0].id, "wall");
    assert_eq!(
        decode_browser_download(&json!({"status": "exists", "future": true})).unwrap(),
        DownloadResponse::Exists
    );
}

#[test]
fn status_boundaries_validate_runtime_shapes() {
    envelope_contract(json!({"scheduled": true}), decode_thumbnail_reset);
    assert!(decode_thumbnail_reset(&json!({"scheduled": true})).unwrap());
    assert!(!decode_thumbnail_reset(&json!({"scheduled": false})).unwrap());
    assert!(decode_thumbnail_reset(&json!({})).is_err());
    assert!(decode_thumbnail_reset(&json!({"scheduled": "false"})).is_err());
    envelope_contract(json!({}), decode_status);
    envelope_contract(json!({}), decode_task_list);
    envelope_contract(json!({"we_id": "", "properties": []}), decode_scene_properties);

    assert!(decode_status(&json!({"capabilities": {}})).is_err());
    assert!(decode_task_list(&json!({"tasks": "invalid"})).is_err());
    assert!(decode_scene_properties(&json!({"properties": {}})).is_err());
    assert!(decode_scene_properties(&json!({"we_id": "scene"})).is_err());
    assert!(decode_scene_properties(&json!({"properties": []})).is_err());

    let status = decode_status(&json!({
        "version": "1.2.3",
        "protocol": {"name": "skwd-wall", "version": 1, "future": true},
        "capabilities": ["picker-session", 7],
        "library_watch": {"mode": "native", "future": true},
        "future": true
    }))
    .unwrap();
    assert!(status.advertises("picker-session"));
    assert_eq!(status.protocol.unwrap().version, 1);
    assert_eq!(status.library_watch.unwrap().mode, "native");

    assert!(matches!(
        decode_status(&json!({
            "version": "1.2.3",
            "protocol": {"name": "skwd-wall", "version": 2},
            "capabilities": ["picker-session"]
        })),
        Err(DecodeError::UnknownVersion { family: "status.protocol", version: 2 })
    ));
    assert!(matches!(
        decode_status(&json!({
            "protocol": {"name": "other", "version": 1}
        })),
        Err(DecodeError::InvalidField { family: "status.protocol", field: "name", .. })
    ));
    let cleared_watch = decode_status(&json!({"library_watch": null})).unwrap();
    assert!(cleared_watch.library_watch_present);
    assert!(cleared_watch.library_watch.is_none());

    let scene = decode_scene_properties(&json!({
        "we_id": "scene",
        "properties": [],
        "future": true
    }))
    .unwrap();
    assert_eq!(scene.we_id, "scene");
    assert!(
        decode_scene_properties(&json!({
            "we_id": "scene",
            "properties": [
                {"name": "glow", "kind": "bool", "value": true},
                7
            ]
        }))
        .is_err()
    );
}

#[test]
fn library_boundary_rejects_atomic_shape_errors_and_accepts_future_fields() {
    let paths = LibraryPaths::new("/wallpapers", "/videos");
    assert!(decode_library_list(&json!([]), paths).is_err());
    assert!(decode_library_list(&json!({}), paths).is_err());
    assert!(decode_library_list(&json!({"wallpapers": false}), paths).is_err());
    assert!(matches!(
        decode_library_list(&json!({"schema_version": 2, "wallpapers": []}), paths,),
        Err(DecodeError::UnknownVersion { version: 2, .. })
    ));
    let catalog = decode_library_list(
        &json!({
            "schema_version": 1,
            "wallpapers": [{
                "name": "future.png",
                "type": "static",
                "thumb": "/thumb/future.webp",
                "future": {"nested": true}
            }],
            "future": true
        }),
        paths,
    )
    .unwrap();
    assert_eq!(catalog.items.len(), 1);
    assert_eq!(catalog.items[0].name, "future.png");
}

#[test]
fn task_boundary_keeps_valid_rows_and_unknown_states() {
    let result = decode_task_list(&json!({
        "tasks": [
            {
                "id": "scan",
                "kind": "scan",
                "label": "Scanning",
                "state": "running"
            },
            {"id": 7, "kind": "broken", "label": "Broken"},
            {
                "id": "queued",
                "kind": "index",
                "label": "Queued",
                "state": "waiting_for_network",
                "future": true
            }
        ]
    }))
    .unwrap();
    let tasks = result.tasks.unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].state, TaskState::Running);
    assert_eq!(tasks[1].state, TaskState::Other("waiting_for_network".into()));
    assert!(tasks[1].state.is_active());

    assert!(decode_task_status(&json!({"id": "missing-fields"})).is_err());
    assert!(
        decode_task_status(&json!({
            "id": "scan",
            "kind": "scan",
            "label": "Scanning",
            "capabilities": {"pause": "yes"}
        }))
        .is_err()
    );
}

#[test]
fn status_decodes_optional_steam_helper_availability() {
    assert_eq!(decode_status(&json!({})).unwrap().steam_helper_available, None);
    assert_eq!(
        decode_status(&json!({"steam_helper_available":false})).unwrap().steam_helper_available,
        Some(false)
    );
    assert!(decode_status(&json!({"steam_helper_available":"no"})).is_err());
}
