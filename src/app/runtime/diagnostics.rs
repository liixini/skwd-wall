pub(crate) const TOAST_MS: u128 = 5000;

pub(crate) fn apply_error_message(kind: &str, detail: &str) -> String {
    let heading = match kind {
        "file_missing" => crate::i18n::tr("status-apply-file-missing"),
        "renderer_unavailable" | "renderer_spawn_failed" => {
            crate::i18n::tr("status-apply-renderer-failed")
        }
        "decode_failed" => crate::i18n::tr("status-apply-decode-failed"),
        "no_outputs" => crate::i18n::tr("status-apply-no-outputs"),
        "bad_request" => crate::i18n::tr("status-apply-invalid-request"),
        _ => crate::i18n::tr("status-apply-failed"),
    };
    let detail = detail.trim();
    if detail.is_empty() || kind == "bad_request" {
        heading.to_string()
    } else {
        crate::i18n::tr_args!("status-apply-detail", heading => heading, detail => detail)
    }
}
