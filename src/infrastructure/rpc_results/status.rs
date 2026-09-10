use serde_json::Value;

use crate::contracts::daemon::{
    LibraryWatchRootStatus, LibraryWatchStatus, ProtocolStatus, ScenePropertiesResult,
    StatusResult, TaskCapabilities, TaskListResult, TaskState, TaskStatus,
};

use super::common::{
    DecodeError, DecodeResult, array, envelope, invalid, required_string, required_typed, string,
    strings,
};

pub fn decode_status(value: &Value) -> DecodeResult<StatusResult> {
    let object = envelope("status", value)?;
    let library_watch_present = object.contains_key("library_watch");
    let library_watch = match object.get("library_watch") {
        None | Some(Value::Null) => None,
        Some(value) => Some(map_library_watch(
            serde_json::from_value(value.clone())
                .map_err(|_| invalid("status", "library_watch", "library watcher status"))?,
        )),
    };
    Ok(StatusResult {
        playback: object
            .get("playback")
            .filter(|value| !value.is_null())
            .map(|value| {
                serde_json::from_value(value.clone())
                    .map_err(|_| invalid("status", "playback", "playback status"))
            })
            .transpose()?,
        version: string("status", object, "version")?.unwrap_or_default(),
        protocol: decode_protocol(object)?,
        capabilities: array("status", object, "capabilities")?.map_or_else(Vec::new, strings),
        steam_helper_available: super::common::typed(
            "status",
            object,
            "steam_helper_available",
            "boolean",
        )?,
        library_watch_present,
        library_watch,
    })
}

pub fn decode_task_list(value: &Value) -> DecodeResult<TaskListResult> {
    let object = envelope("task.list", value)?;
    let tasks = array("task.list", object, "tasks")?
        .map(|rows| rows.iter().filter_map(|row| decode_task_status(row).ok()).collect::<Vec<_>>());
    Ok(TaskListResult { tasks })
}

pub fn decode_scene_properties(value: &Value) -> DecodeResult<ScenePropertiesResult> {
    let object = envelope("wall.we_properties", value)?;
    let rows = crate::infrastructure::scene_properties::decode_rows(required_typed::<
        Vec<wall_proto::WeProperty>,
    >(
        "wall.we_properties",
        object,
        "properties",
        "scene property array",
    )?);
    Ok(ScenePropertiesResult {
        we_id: required_string("wall.we_properties", object, "we_id")?,
        rows,
    })
}

pub fn decode_thumbnail_reset(value: &Value) -> DecodeResult<bool> {
    let family = "wall.reset_thumbnail";
    let object = envelope(family, value)?;
    required_typed(family, object, "scheduled", "boolean")
}

pub fn decode_library_watch(value: &Value) -> DecodeResult<LibraryWatchStatus> {
    serde_json::from_value::<wall_proto::LibraryWatchStatus>(value.clone())
        .map(map_library_watch)
        .map_err(|_| invalid("event.library_watch", "result", "library watcher status"))
}

pub fn decode_task_status(value: &Value) -> DecodeResult<TaskStatus> {
    let family = "task.status";
    let object = value.as_object().ok_or(DecodeError::ExpectedObject { family })?;
    let required = |field: &'static str| -> DecodeResult<String> {
        string(family, object, field)?.ok_or_else(|| invalid(family, field, "string"))
    };
    let optional = |field: &'static str| -> DecodeResult<String> {
        Ok(string(family, object, field)?.unwrap_or_default())
    };
    let unsigned = |field: &'static str| -> DecodeResult<u64> {
        object
            .get(field)
            .map(|value| value.as_u64().ok_or_else(|| invalid(family, field, "unsigned integer")))
            .transpose()
            .map(Option::unwrap_or_default)
    };
    let state = match string(family, object, "state")?.as_deref().unwrap_or("running") {
        "running" => TaskState::Running,
        "paused" => TaskState::Paused,
        "completed" => TaskState::Completed,
        "failed" => TaskState::Failed,
        "cancelled" => TaskState::Cancelled,
        other => TaskState::Other(other.to_string()),
    };
    let capabilities = match object.get("capabilities") {
        None => TaskCapabilities::default(),
        Some(value) => decode_task_capabilities(value)?,
    };
    Ok(TaskStatus {
        id: required("id")?,
        kind: required("kind")?,
        label: required("label")?,
        state,
        progress: unsigned("progress")?,
        total: unsigned("total")?,
        detail: optional("detail")?,
        eta: optional("eta")?,
        capabilities,
    })
}

fn decode_protocol(
    object: &serde_json::Map<String, Value>,
) -> DecodeResult<Option<ProtocolStatus>> {
    let Some(value) = object.get("protocol") else { return Ok(None) };
    let family = "status.protocol";
    let protocol = value.as_object().ok_or_else(|| invalid("status", "protocol", "object"))?;
    let name = string(family, protocol, "name")?
        .ok_or_else(|| invalid(family, "name", wall_proto::PROTOCOL_NAME))?;
    if name != wall_proto::PROTOCOL_NAME {
        return Err(invalid(family, "name", wall_proto::PROTOCOL_NAME));
    }
    let version = protocol
        .get("version")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid(family, "version", "positive integer"))?;
    if version != u64::from(wall_proto::PROTOCOL_VERSION) {
        return Err(DecodeError::UnknownVersion { family, version });
    }
    Ok(Some(ProtocolStatus { name, version }))
}

fn decode_task_capabilities(value: &Value) -> DecodeResult<TaskCapabilities> {
    let family = "task.status.capabilities";
    let object = value.as_object().ok_or(DecodeError::ExpectedObject { family })?;
    let flag = |field: &'static str| -> DecodeResult<bool> {
        object
            .get(field)
            .map(|value| value.as_bool().ok_or_else(|| invalid(family, field, "boolean")))
            .transpose()
            .map(Option::unwrap_or_default)
    };
    Ok(TaskCapabilities { pause: flag("pause")?, resume: flag("resume")?, stop: flag("stop")? })
}

fn map_library_watch(status: wall_proto::LibraryWatchStatus) -> LibraryWatchStatus {
    LibraryWatchStatus {
        ok: status.ok,
        degraded: status.degraded,
        mode: status.mode,
        detail: status.detail,
        interval_seconds: status.interval_seconds,
        entry_budget_per_root: status.entry_budget_per_root,
        last_successful_convergence_unix_ms: status.last_successful_convergence_unix_ms,
        roots: status
            .roots
            .into_iter()
            .map(|root| LibraryWatchRootStatus {
                path: root.path,
                mode: root.mode,
                native_error: root.native_error,
                last_completed_sweep_unix_ms: root.last_completed_sweep_unix_ms,
                last_scan_requested_unix_ms: root.last_scan_requested_unix_ms,
                last_successful_convergence_unix_ms: root.last_successful_convergence_unix_ms,
                pending_scans: root.pending_scans,
                last_poll_error: root.last_poll_error,
            })
            .collect(),
    }
}
