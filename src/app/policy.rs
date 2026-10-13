pub(crate) fn version_mismatch(gui: &str, daemon: &str) -> Option<String> {
    (gui != daemon).then(
        || crate::i18n::tr_args!("status-daemon-version-mismatch", gui => gui, daemon => daemon),
    )
}

pub(crate) fn empty_library_hint(
    total: usize,
    dir: Option<&str>,
    connected: bool,
    ever_connected: bool,
) -> Option<String> {
    if !ever_connected {
        return Some(crate::i18n::tr("status-daemon-connecting").to_string());
    }
    if !connected {
        return Some(crate::i18n::tr("status-daemon-lost").to_string());
    }
    (total == 0).then(|| {
        dir.map_or_else(
            || crate::i18n::tr("status-library-no-folder").to_string(),
            |directory| crate::i18n::tr_args!("status-library-empty", directory => directory),
        )
    })
}

pub(crate) fn needs_list_refresh(have: usize, changed: usize, total: Option<usize>) -> bool {
    match total {
        Some(count) => have != count || changed > 0,
        None => have != changed,
    }
}

pub(crate) fn scene_visibility(
    settings_open: bool,
    tab: &str,
    section: usize,
    hard_overlay: bool,
) -> (bool, bool) {
    let design_mode = settings_open
        && crate::frontend::settings::is_picker_layout_section(tab, section)
        && !hard_overlay;
    let scene_hidden = hard_overlay || (settings_open && !design_mode);
    (design_mode, scene_hidden)
}
