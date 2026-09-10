use serde_json::Value;

use crate::contracts::daemon::{BugReportResult, DiagnosticResult, WeatherResult};

use super::common::{DecodeResult, envelope, required_array, string, strings};

pub fn decode_diagnostic(value: &Value) -> DecodeResult<DiagnosticResult> {
    let object = envelope("diag", value)?;
    Ok(DiagnosticResult { banner: string("diag", object, "banner")? })
}

pub fn decode_weather(value: &Value) -> DecodeResult<WeatherResult> {
    let object = envelope("wall.weather", value)?;
    Ok(WeatherResult { weather: strings(required_array("wall.weather", object, "weather")?) })
}

pub fn decode_bug_report(value: &Value) -> DecodeResult<BugReportResult> {
    let object = envelope("status.bug_report", value)?;
    Ok(BugReportResult { path: string("status.bug_report", object, "path")? })
}
