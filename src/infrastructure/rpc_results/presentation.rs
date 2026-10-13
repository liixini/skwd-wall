use serde_json::Value;

use super::common::{
    DecodeResult, array, envelope, required_array, required_string, required_typed, strings,
};
use crate::contracts::daemon::{
    ActivitiesResult, Activity, ActivityApplyEvent, ActivityApplyResult, ActivityCompletion,
    AudioOutputsResult, OutputStatus, OutputsResult, ThemeAuditionPreview, ThemeBackendsResult,
    ThemePreviewResult, ThemePreviewsResult,
};

pub fn decode_activities(value: &Value) -> DecodeResult<ActivitiesResult> {
    const FIELD: &str = "result";
    const EXPECTED: &str = "activities";
    let object = envelope(wall_proto::rpc::WALL_ACTIVITIES, value)?;
    let decoded: wall_proto::ActivitiesResult =
        serde_json::from_value(Value::Object(object.clone())).map_err(|_| {
            super::common::invalid(wall_proto::rpc::WALL_ACTIVITIES, FIELD, EXPECTED)
        })?;
    Ok(ActivitiesResult {
        supported: decoded.supported,
        current: decoded.current,
        activities: decoded
            .activities
            .into_iter()
            .map(|activity| Activity {
                id: activity.id,
                name: activity.name,
                current: activity.current,
            })
            .collect(),
    })
}

pub fn decode_activity_apply(value: &Value) -> DecodeResult<ActivityApplyResult> {
    const FIELD: &str = "result";
    const EXPECTED: &str = "activity apply";
    let object = envelope(wall_proto::rpc::WALL_APPLY, value)?;
    serde_json::from_value(Value::Object(object.clone()))
        .map_err(|_| super::common::invalid(wall_proto::rpc::WALL_APPLY, FIELD, EXPECTED))
}

pub fn decode_activity_event(value: &Value) -> DecodeResult<Option<ActivityApplyEvent>> {
    const FIELD: &str = "result";
    const EXPECTED: &str = "activity apply event";
    let object = envelope(wall_proto::ev::APPLY_RESULT, value)?;
    let payload: wall_proto::ev::ApplyResult =
        serde_json::from_value(Value::Object(object.clone()))
            .map_err(|_| super::common::invalid(wall_proto::ev::APPLY_RESULT, FIELD, EXPECTED))?;
    let Some(activity) = payload.activity else { return Ok(None) };
    let token = payload.apply_token.unwrap_or_default();
    let ok = required_typed::<bool>(
        wall_proto::ev::APPLY_RESULT,
        object,
        wall_proto::apply_fields::OK,
        EXPECTED,
    )?;
    let visible = payload.visible.ok_or_else(|| {
        super::common::invalid(
            wall_proto::ev::APPLY_RESULT,
            wall_proto::apply_fields::VISIBLE,
            EXPECTED,
        )
    })?;
    let completion = if payload.queued == Some(true) {
        None
    } else if payload.cancelled == Some(true) || payload.superseded == Some(true) {
        Some(ActivityCompletion::Cancelled)
    } else if ok {
        Some(ActivityCompletion::Applied { visible })
    } else {
        Some(ActivityCompletion::Failed(payload.detail.unwrap_or_default()))
    };
    Ok(Some(ActivityApplyEvent {
        token,
        request_id: payload.request_id,
        activity,
        output: payload.output,
        completion,
    }))
}
use crate::contracts::media::MediaKind;

pub fn decode_outputs(value: &Value) -> DecodeResult<OutputsResult> {
    let object = envelope("wall.outputs", value)?;
    let outputs = required_typed::<Vec<wall_proto::OutputStatus>>(
        "wall.outputs",
        object,
        "outputs",
        "output status array",
    )?
    .into_iter()
    .map(map_output)
    .collect();
    Ok(OutputsResult { outputs })
}

pub fn decode_audio_outputs(value: &Value) -> DecodeResult<AudioOutputsResult> {
    let object = envelope("audio.outputs", value)?;
    let outputs = required_typed::<Vec<wall_proto::OutputStatus>>(
        "audio.outputs",
        object,
        "outputs",
        "output status array",
    )?
    .into_iter()
    .map(map_output)
    .collect();
    Ok(AudioOutputsResult { outputs })
}

pub fn decode_theme_backends(value: &Value) -> DecodeResult<ThemeBackendsResult> {
    let object = envelope("theme.backends", value)?;
    Ok(ThemeBackendsResult { backends: array("theme.backends", object, "backends")?.map(strings) })
}

pub fn decode_theme_preview(value: &Value) -> DecodeResult<ThemePreviewResult> {
    let object = envelope("theme.preview", value)?;
    let palette = object.get("palette").and_then(|value| {
        value.is_object().then(|| crate::infrastructure::theme::decode_palette(value))
    });
    Ok(ThemePreviewResult {
        colors: strings(required_array("theme.preview", object, "colors")?),
        palette,
    })
}

pub fn decode_theme_previews(value: &Value) -> DecodeResult<ThemePreviewsResult> {
    let object = envelope("theme.previews", value)?;
    let previews = required_array("theme.previews", object, "previews")?
        .iter()
        .filter_map(decode_audition_preview)
        .collect();
    Ok(ThemePreviewsResult {
        backend: Some(required_string("theme.previews", object, "backend")?),
        backends: strings(required_array("theme.previews", object, "backends")?),
        previews,
    })
}

fn decode_audition_preview(value: &Value) -> Option<ThemeAuditionPreview> {
    let object = value.as_object()?;
    let text = |field| object.get(field).and_then(Value::as_str).map(str::to_string);
    let palette = object.get("palette")?;
    if !palette.is_object() {
        return None;
    }
    let palette = crate::infrastructure::theme::decode_palette(palette);
    Some(ThemeAuditionPreview {
        backend: text("backend")?,
        key: text("key")?,
        value: text("value")?,
        label: text("label")?,
        palette,
    })
}

fn map_output(output: wall_proto::OutputStatus) -> OutputStatus {
    OutputStatus {
        name: output.name,
        target: output.target,
        connected: output.connected,
        width: output.width,
        height: output.height,
        logical_width: output.logical_width,
        logical_height: output.logical_height,
        current: output.current,
        kind: MediaKind::from_key(&output.kind),
        path: output.path,
        we_id: output.we_id,
        mute: output.mute,
        volume: output.volume,
        fill: output.fill,
        audio_shared: output.audio_shared,
        paused: output.paused,
        manual_paused: output.manual_paused,
    }
}

pub fn decode_current_theme(
    value: &serde_json::Value,
) -> super::common::DecodeResult<crate::contracts::daemon::CurrentTheme> {
    #[derive(serde::Deserialize)]
    struct Current {
        key: String,
        name: String,
        thumb: String,
        palette: serde_json::Value,
        scheme: Option<serde_json::Value>,
        dark: bool,
    }
    let mut current: Current = serde_json::from_value(value.clone())
        .map_err(|_| super::common::invalid("theme.current", "result", "current palette"))?;
    if let Some(scheme) = current.scheme {
        current.palette["_scheme"] = scheme;
        current.palette["_schemeVersion"] = serde_json::json!(1);
    }
    let palette =
        crate::infrastructure::theme::decode_candidate_variant(&current.palette, current.dark);
    Ok(crate::contracts::daemon::CurrentTheme {
        key: current.key,
        name: current.name,
        thumb: current.thumb,
        palette,
        dark: current.dark,
    })
}

pub fn decode_running_processes(
    value: &serde_json::Value,
) -> super::common::DecodeResult<Vec<String>> {
    let object = super::common::envelope("playback.processes", value)?;
    Ok(super::common::array("playback.processes", object, "processes")?
        .map_or_else(Vec::new, super::common::strings))
}
