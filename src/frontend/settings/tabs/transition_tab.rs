use crate::contracts::settings::{SettingsSource, keys};
use crate::i18n::{tr, tr_args};

use super::super::tables::{SHADER_FAMILIES, SHADER_FAMILY_KEY, shader_family, shaders_in_family};
use super::{Builder, Control};

fn family_label_key(family: &str) -> &'static str {
    match family {
        "fade" => "settings-transitions-family-fade",
        "wipe" => "settings-transitions-family-wipe",
        "warp" => "settings-transitions-family-warp",
        "break" => "settings-transitions-family-break",
        "sand" => "settings-transitions-family-sand",
        _ => "settings-transitions-family-random",
    }
}

fn family_desc_key(family: &str) -> &'static str {
    match family {
        "fade" => "settings-transitions-family-fade-desc",
        "wipe" => "settings-transitions-family-wipe-desc",
        "warp" => "settings-transitions-family-warp-desc",
        "break" => "settings-transitions-family-break-desc",
        "sand" => "settings-transitions-family-sand-desc",
        _ => "settings-transitions-family-random-desc",
    }
}

fn pretty(shader: &str) -> String {
    let mut out = String::with_capacity(shader.len());
    for (idx, word) in shader.split('-').enumerate() {
        if idx > 0 {
            out.push(' ');
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

fn shader_scope_path(shader: &str) -> String {
    format!("{}.{shader}", keys::transition::SHADER_SCOPES)
}

fn shader_scope(cfg: &dyn SettingsSource, shader: &str) -> String {
    let scope = cfg.text(&shader_scope_path(shader));
    if matches!(scope.as_str(), "all" | "primary") {
        return scope;
    }
    if shader.starts_with("sand-") && cfg.text(keys::transition::SAND_SCOPE) == "primary" {
        return String::from("primary");
    }
    String::from("all")
}

pub(super) fn tab_transitions(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    let enabled = cfg.flag_default_true(keys::transition::ENABLED);
    let current = cfg.text(keys::transition::SHADER);
    let current = if current.is_empty() { String::from("random") } else { current };
    let family = shader_family(&current);

    builder.card(
        tr("settings-transitions-transitions-card"),
        tr("settings-transitions-transitions-card-desc"),
    );
    builder.toggle(
        tr("settings-transitions-enable-label"),
        tr("settings-transitions-enable-desc"),
        keys::transition::ENABLED,
    );
    if !enabled {
        return;
    }
    builder.num(
        tr("settings-transitions-duration-label"),
        tr("settings-transitions-duration-desc"),
        keys::transition::DURATION_MS,
        "ms",
    );

    let family_opts: Vec<(String, String)> = SHADER_FAMILIES
        .iter()
        .map(|(key, _, _)| ((*key).to_string(), tr(family_label_key(key)).to_string()))
        .collect();
    builder.row(
        tr("settings-transitions-family-label"),
        tr(family_desc_key(family)),
        Control::Chips {
            path: String::from(SHADER_FAMILY_KEY),
            options: family_opts,
            current: family.to_string(),
            disabled: Vec::new(),
        },
    );
    if family != "random" {
        let options: Vec<(String, String)> = shaders_in_family(family)
            .into_iter()
            .map(|name| (name.to_string(), pretty(name)))
            .collect();
        let count = options.len();
        builder.row(
            tr("settings-transitions-shader-label"),
            &tr_args!("settings-transitions-shader-count-desc", count => count),
            Control::Dropdown {
                palettes: Vec::new(),
                path: keys::transition::SHADER.to_string(),
                options,
                current,
            },
        );
    }

    builder.card(
        tr("settings-transitions-preview-card"),
        tr("settings-transitions-preview-card-desc"),
    );
    builder.toggle(
        tr("settings-transitions-show-preview-label"),
        tr("settings-transitions-show-preview-desc"),
        keys::transition::PREVIEW,
    );
    if cfg.flag_default_true(keys::transition::PREVIEW) {
        builder.num(
            tr("settings-transitions-preview-fps-label"),
            tr("settings-transitions-preview-fps-desc"),
            keys::transition::PREVIEW_FPS,
            "fps",
        );
        builder.row("", "", Control::Preview);
    }

    builder.card(tr("settings-transitions-sand-card"), tr("settings-transitions-sand-card-desc"));
    builder.text_field(
        tr("settings-transitions-primary-label"),
        tr("settings-transitions-primary-desc"),
        keys::transition::SAND_PRIMARY,
        "DP-1",
    );
    for shader in shaders_in_family(family) {
        let path = shader_scope_path(shader);
        builder.dropdown_cur(
            &pretty(shader),
            tr("settings-transitions-scope-desc"),
            &path,
            &[
                ("all", tr("settings-transitions-scope-all")),
                ("primary", tr("settings-transitions-scope-primary")),
            ],
            shader_scope(cfg, shader),
        );
    }
    if family != "sand" {
        return;
    }
    let quality = cfg.text(keys::transition::SAND_QUALITY);
    let quality = if quality.is_empty() { String::from("auto") } else { quality };
    builder.dropdown_cur(
        tr("settings-transitions-quality-label"),
        tr("settings-transitions-quality-desc"),
        keys::transition::SAND_QUALITY,
        &[
            ("auto", tr("settings-transitions-quality-auto")),
            ("full", tr("settings-transitions-quality-full")),
            ("low", tr("settings-transitions-quality-low")),
        ],
        quality,
    );
    builder.num(
        tr("settings-transitions-fps-cap-label"),
        tr("settings-transitions-fps-cap-desc"),
        keys::transition::SAND_FPS,
        "fps",
    );
    builder.toggle(
        tr("settings-transitions-sharp-label"),
        tr("settings-transitions-sharp-desc"),
        keys::transition::SAND_SHARP,
    );
}

mod tests;
