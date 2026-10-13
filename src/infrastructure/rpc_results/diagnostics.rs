use serde_json::Value;

use crate::contracts::daemon::{
    BugReportResult, DiagnosticResult, WeatherCandidates, WeatherStatus,
};

use super::common::{DecodeError, DecodeResult, envelope, required_array, string, strings};

pub fn decode_diagnostic(value: &Value) -> DecodeResult<DiagnosticResult> {
    let object = envelope("diag", value)?;
    Ok(DiagnosticResult { banner: string("diag", object, "banner")? })
}

pub fn decode_weather(value: &Value) -> DecodeResult<WeatherStatus> {
    const STATUS_FIELD: &str = "status";
    const STATUS_TYPE: &str = "weather status";
    const WEATHER_FIELD: &str = "weather";
    let family = wall_proto::rpc::WALL_WEATHER;
    let weather = strings(required_array(family, envelope(family, value)?, WEATHER_FIELD)?);
    let mut compatible = value.clone();
    compatible[WEATHER_FIELD] = serde_json::json!(weather);
    serde_json::from_value(compatible).map_err(|_| DecodeError::InvalidField {
        family,
        field: STATUS_FIELD,
        expected: STATUS_TYPE,
    })
}

pub fn decode_weather_candidates(value: &Value) -> DecodeResult<WeatherCandidates> {
    const CATEGORY_FIELD: &str = "category";
    const CANDIDATES_TYPE: &str = "weather candidates";
    let family = wall_proto::rpc::WALL_WEATHER_CANDIDATES;
    let object = envelope(family, value)?;
    if !object.get(CATEGORY_FIELD).is_some_and(Value::is_string) {
        return Err(DecodeError::InvalidField {
            family,
            field: CATEGORY_FIELD,
            expected: CANDIDATES_TYPE,
        });
    }
    serde_json::from_value(value.clone()).map_err(|_| DecodeError::InvalidField {
        family,
        field: CATEGORY_FIELD,
        expected: CANDIDATES_TYPE,
    })
}

pub fn decode_bug_report(value: &Value) -> DecodeResult<BugReportResult> {
    let object = envelope("status.bug_report", value)?;
    Ok(BugReportResult { path: string("status.bug_report", object, "path")? })
}
