#![cfg(test)]

use super::{pretty, shader_scope, shader_scope_path};
use crate::contracts::settings::{keys, transitions};
use crate::frontend::settings::tables::{
    SHADER_FAMILIES, SHADERS, family_default, shader_family, shaders_in_family,
};
use crate::frontend::settings::test_source::FakeSettingsSource;

#[test]
fn shaders_match_transition_catalog() {
    use std::collections::HashSet;
    let gui: HashSet<&str> = SHADERS.iter().copied().collect();
    let catalog: HashSet<&str> = transitions::TRANSITIONS.iter().map(|spec| spec.key).collect();
    assert_eq!(gui, catalog);
}

#[test]
fn shader_family_coverage() {
    let keys: Vec<&str> = SHADER_FAMILIES.iter().map(|(key, _, _)| *key).collect();
    for shader in SHADERS {
        let family = shader_family(shader);
        assert!(keys.contains(&family), "{shader} -> {family}");
    }
    let total: usize = keys.iter().map(|key| shaders_in_family(key).len()).sum();
    assert_eq!(total, SHADERS.len());
}

#[test]
fn family_sizes_balanced() {
    for (key, label, _) in SHADER_FAMILIES {
        let count = shaders_in_family(key).len();
        assert!(count > 0, "{label}");
        assert!(count < SHADERS.len() / 2, "{label} holds {count}");
    }
}

#[test]
fn family_default_keeps_fit() {
    assert_eq!(family_default("sand-helix", "sand"), "sand-helix");
    assert_eq!(family_default("random", "random"), "random");
    let moved = family_default("sand-helix", "warp");
    assert_eq!(shader_family(&moved), "warp");
    assert!(SHADERS.contains(&moved.as_str()), "{moved}");
}

#[test]
fn shader_names_read_as_words() {
    assert_eq!(pretty("sand-helix"), "Sand Helix");
    assert_eq!(pretty("polka-dots-curtain"), "Polka Dots Curtain");
    assert_eq!(pretty("iris"), "Iris");
}

#[test]
fn shader_order_pins_random_first() {
    assert_eq!(SHADERS[0], "random");
    assert!(SHADERS.contains(&"sand-tornado"));
}

#[test]
fn studio_lists_kept_presets() {
    let mut studio = shaders_in_family("studio");
    let mut expected: Vec<&str> = super::PRESET_LABELS.iter().map(|(id, _)| *id).collect();
    studio.sort_unstable();
    expected.sort_unstable();
    assert_eq!(studio, expected);
    assert!(studio.iter().all(|shader| transitions::preset_duration_ms(shader).is_some()));
    for (id, label) in super::PRESET_LABELS {
        assert_eq!(pretty(id), crate::i18n::tr(label));
        assert_ne!(crate::i18n::tr(label), label);
    }
}

#[test]
fn drawn_is_not_offered() {
    assert!(SHADERS.iter().all(|shader| !shader.starts_with("drawn")));
}

#[test]
fn preset_timing_controls_duration() {
    for custom in [false, true] {
        let config = FakeSettingsSource::default()
            .with_text(keys::transition::SHADER, paper_control::transitions::PHOTO_CUTOUT)
            .with_flag(keys::transition::PRESET_TIMING, !custom);
        let mut builder = super::Builder { cfg: &config, cards: Vec::new() };
        super::tab_transitions(&mut builder);
        let controls: Vec<&super::Control> = builder
            .cards
            .iter()
            .flat_map(|(_, rows)| rows.iter().map(|row| &row.control))
            .collect();
        assert!(controls.iter().any(|control| matches!(
            control, super::Control::Toggle { path, .. }
                if path == keys::transition::PRESET_TIMING
        )));
        assert_eq!(
            controls.iter().any(|control| matches!(
                control, super::Control::Number { path, .. }
                    if path == keys::transition::DURATION_MS
            )),
            custom
        );
    }
}

#[test]
fn shader_scope_paths_distinct() {
    assert_eq!(shader_scope_path("fade"), "transition.shaderScopes.fade");
    assert_ne!(shader_scope_path("fade"), shader_scope_path("glitch"));
}

#[test]
fn shader_scope_migration() {
    let config = FakeSettingsSource::default()
        .with_text(keys::transition::SAND_SCOPE, "primary")
        .with_text(&shader_scope_path("sand-donut"), "all")
        .with_text(&shader_scope_path("fade"), "primary");

    assert_eq!(shader_scope(&config, "sand-helix"), "primary");
    assert_eq!(shader_scope(&config, "sand-donut"), "all");
    assert_eq!(shader_scope(&config, "fade"), "primary");
    assert_eq!(shader_scope(&config, "glitch"), "all");
}

#[test]
fn every_shader_family_has_one_transition_rate_beside_duration() {
    for shader in ["random", "crossfade", "sand-helix", "glitch"] {
        let config = FakeSettingsSource::default().with_text(keys::transition::SHADER, shader);
        let mut builder = super::Builder { cfg: &config, cards: Vec::new() };
        super::tab_transitions(&mut builder);
        let paths: Vec<&str> = builder
            .cards
            .iter()
            .flat_map(|(_, rows)| rows.iter())
            .filter_map(|row| {
                if let super::Control::Number { path, .. } = &row.control {
                    Some(path.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(paths.iter().filter(|&&path| path == keys::transition::FPS).count(), 1);
        let duration =
            paths.iter().position(|&path| path == keys::transition::DURATION_MS).unwrap();
        assert_eq!(paths[duration + 1], keys::transition::FPS);
        assert!(!paths.contains(&keys::transition::SAND_FPS));
    }
}

#[test]
fn drawing_survives_transitions_off() {
    let config = FakeSettingsSource::default().with_flag(keys::transition::ENABLED, false);
    let cards = super::super::build_tab_with_runtime_status(
        super::super::super::tables::category::TRANSITIONS,
        &config,
        &[],
        &[],
        "",
        &[],
        super::super::RuntimeStatus::default(),
    );
    let rows = &cards
        .iter()
        .find(|(card, _)| card.title == crate::i18n::tr("settings-drawing-card"))
        .unwrap()
        .1;
    assert!(rows.iter().any(|row| {
        matches!(&row.control, super::Control::Toggle { path, .. }
            if path == keys::drawing::ENABLED)
    }));
    let colours: Vec<_> = rows
        .iter()
        .filter_map(|row| match &row.control {
            super::Control::Colour { path, .. } => Some(path.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(colours, [keys::drawing::PAPER_COLOR, keys::drawing::INK_COLOR]);
}

#[test]
fn drawing_preparation_states() {
    use crate::contracts::settings::drawing::{PackState, Phase, Status};

    let config = FakeSettingsSource::default();
    let mut status =
        Status { pack: PackState::Available, key: String::from("still:one"), ..Status::default() };
    let mut builder = super::Builder { cfg: &config, cards: Vec::new() };
    super::tab_drawing(&mut builder, Some(&status), Some("still:one"), 0, false);
    let rows = &builder.cards[0].1;
    assert!(rows.iter().any(|row| matches!(
        row.control,
        super::Control::ActionBtn { id: super::ActionId::PrepareDrawing, .. }
    )));
    assert!(!rows.iter().any(|row| matches!(
        row.control,
        super::Control::ActionBtn { id: super::ActionId::PreviewDrawing, .. }
    )));

    status.phase = Phase::Checking;
    status.pack = PackState::Missing;
    builder.cards.clear();
    super::tab_drawing(&mut builder, Some(&status), Some("still:one"), 0, false);
    let rows = &builder.cards[0].1;
    assert!(rows.iter().any(|row| row.desc == crate::i18n::tr("settings-drawing-pack-checking")));
    assert!(rows.iter().any(|row| matches!(
        row.control,
        super::Control::ActionBtn { id: super::ActionId::CancelDrawing, .. }
    )));
    assert!(!rows.iter().any(|row| matches!(
        row.control,
        super::Control::ActionBtn { id: super::ActionId::PauseDrawing, .. }
    )));

    status.phase = Phase::Preparing;
    status.pack = PackState::Available;
    builder.cards.clear();
    super::tab_drawing(&mut builder, Some(&status), Some("still:one"), 0, false);
    let rows = &builder.cards[0].1;
    assert!(rows.iter().any(|row| matches!(
        row.control,
        super::Control::ActionBtn { id: super::ActionId::CancelDrawing, .. }
    )));
    assert!(!rows.iter().any(|row| matches!(
        row.control,
        super::Control::ActionBtn { id: super::ActionId::PrepareDrawing, .. }
    )));

    status.phase = Phase::Paused;
    builder.cards.clear();
    super::tab_drawing(&mut builder, Some(&status), Some("still:one"), 0, false);
    let rows = &builder.cards[0].1;
    assert!(rows.iter().any(|row| matches!(
        row.control,
        super::Control::ActionBtn { id: super::ActionId::ResumeDrawing, .. }
    )));

    status.phase = Phase::Ready;
    status.prepared = true;
    status.preview_path = Some(String::from("/cache/ink.png"));
    builder.cards.clear();
    super::tab_drawing(&mut builder, Some(&status), Some("still:one"), 0, false);
    let rows = &builder.cards[0].1;
    assert!(rows.iter().any(|row| matches!(
        row.control,
        super::Control::ActionBtn { id: super::ActionId::PreviewDrawing, .. }
    )));
    assert!(rows.iter().any(|row| matches!(&row.control, super::Control::StackBar { preview: Some(path), .. } if path == "/cache/ink.png")));
}

#[test]
fn drawing_cache_keeps_pack() {
    use crate::contracts::settings::drawing::{PackState, Status};

    let config = FakeSettingsSource::default();
    let mut builder = super::Builder { cfg: &config, cards: Vec::new() };
    super::tab_drawing(&mut builder, Some(&Status::default()), None, 0, false);
    let rows = &builder.cards[0].1;
    assert!(rows.iter().any(|row| matches!(
        row.control,
        super::Control::ActionBtn { id: super::ActionId::ChooseDrawingPack, .. }
    )));
    assert!(!rows.iter().any(|row| matches!(
        row.control,
        super::Control::ActionBtn { id: super::ActionId::DetachDrawingPack, .. }
    )));

    builder.cards.clear();
    let status = Status { pack: PackState::Available, cache_bytes: 1024, ..Status::default() };
    super::tab_drawing(&mut builder, Some(&status), None, 0, false);
    let rows = &builder.cards[0].1;
    assert!(rows.iter().any(|row| matches!(
        row.control,
        super::Control::ActionBtn { id: super::ActionId::DetachDrawingPack, .. }
    )));
    assert!(rows.iter().any(|row| matches!(
        row.control,
        super::Control::ActionBtn { id: super::ActionId::ClearDrawingCache, .. }
    )));
}

#[test]
fn drawing_setup_first() {
    let config = FakeSettingsSource::default();
    let mut builder = super::Builder { cfg: &config, cards: Vec::new() };
    super::tab_drawing(
        &mut builder,
        Some(&crate::contracts::settings::drawing::Status::default()),
        None,
        0,
        false,
    );
    let rows = &builder.cards[0].1;
    assert_eq!(rows[0].title, crate::i18n::tr("settings-drawing-pack-label"));
    let choose = rows
        .iter()
        .position(|row| {
            matches!(
                row.control,
                super::Control::ActionBtn { id: super::ActionId::ChooseDrawingPack, .. }
            )
        })
        .unwrap();
    let enable = rows
        .iter()
        .position(|row| row.title == crate::i18n::tr("settings-drawing-enable-label"))
        .unwrap();
    assert!(choose < enable);
}

#[test]
fn drawing_stages_localized() {
    for stage in
        ["checking", "ready", "starting", "source", "lineart", "depth", "subjects", "complete"]
    {
        assert_ne!(super::drawing_stage(stage), stage);
    }
    assert!(!super::drawing_stage("unknown").is_empty());
}

#[test]
fn drawing_other_job_named() {
    use crate::contracts::settings::drawing::{PackState, Phase, Status};

    let config = FakeSettingsSource::default();
    let status = Status {
        pack: PackState::Available,
        key: String::from("still:two"),
        active_key: String::from("still:one"),
        phase: Phase::Preparing,
        ..Status::default()
    };
    let mut builder = super::Builder { cfg: &config, cards: Vec::new() };
    super::tab_drawing(&mut builder, Some(&status), Some("still:two"), 0, false);
    let rows = &builder.cards[0].1;
    assert!(
        rows.iter().any(|row| {
            row.desc == crate::i18n::tr("settings-drawing-selection-other-preparing")
        })
    );
    assert!(!rows.iter().any(|row| matches!(
        row.control,
        super::Control::ActionBtn { id: super::ActionId::PrepareDrawing, .. }
    )));
}

#[test]
fn drawing_view_scope_bounded() {
    use crate::contracts::settings::drawing::{PackState, Status};

    let config =
        FakeSettingsSource::default().with_text(keys::drawing::SCOPE, keys::drawing::SCOPE_VIEW);
    let status = Status { pack: PackState::Available, ..Status::default() };
    let mut builder = super::Builder { cfg: &config, cards: Vec::new() };
    super::tab_drawing(&mut builder, Some(&status), None, 3, false);
    assert!(builder.cards[0].1.iter().any(|row| matches!(
        row.control,
        super::Control::ActionBtn { id: super::ActionId::PrepareDrawing, .. }
    )));

    builder.cards.clear();
    super::tab_drawing(
        &mut builder,
        Some(&status),
        None,
        crate::contracts::settings::drawing::MAX_PREPARE_KEYS + 1,
        false,
    );
    assert!(!builder.cards[0].1.iter().any(|row| matches!(
        row.control,
        super::Control::ActionBtn { id: super::ActionId::PrepareDrawing, .. }
    )));
}

fn memory_menu(saved: &str) -> (Vec<(String, String)>, String) {
    let config = FakeSettingsSource::default().with_text(keys::drawing::MEMORY_LIMIT_MIB, saved);
    let mut builder = super::Builder { cfg: &config, cards: Vec::new() };
    builder.card(crate::i18n::text("settings-drawing-memory-label"), "");
    super::drawing_runtime(&mut builder);
    let row = builder
        .cards
        .iter()
        .flat_map(|card| &card.1)
        .find(|row| row.control.path() == Some(keys::drawing::MEMORY_LIMIT_MIB))
        .unwrap();
    match &row.control {
        super::Control::Dropdown { options, current, .. } => (options.clone(), current.clone()),
        _ => unreachable!(),
    }
}

#[test]
fn memory_parses_like_deck() {
    let (base, _) = memory_menu("8192");
    for saved in [" 8192", "08192", "8192 "] {
        let (options, current) = memory_menu(saved);
        assert_eq!((options.len(), current.as_str()), (base.len(), "8192"));
    }
    let (options, current) = memory_menu("3000");
    assert_eq!(current, "3000");
    assert_eq!(options.len(), base.len() + 1);
    assert_eq!(memory_menu("lots").1, keys::drawing::MEMORY_LIMIT_AUTO);
}

#[test]
fn memory_shows_clamped() {
    let (options, current) = memory_menu("100");
    assert_eq!(current, "512");
    assert_eq!(options.last(), Some(&(String::from("512"), String::from("512 MiB"))));
    assert_eq!(memory_menu("9999999").1, "65536");
}

#[test]
fn drawing_pack_install_hint() {
    use crate::contracts::settings::drawing::{PackState, Phase, Status};

    let config = FakeSettingsSource::default();
    let install = crate::i18n::tr(super::DRAWING_PACK_INSTALL);
    for (status, shown) in [
        (Some(Status::default()), true),
        (Some(Status { phase: Phase::Checking, ..Status::default() }), false),
        (Some(Status { pack: PackState::Invalid, ..Status::default() }), false),
        (None, false),
    ] {
        let mut builder = super::Builder { cfg: &config, cards: Vec::new() };
        super::tab_drawing(&mut builder, status.as_ref(), None, 0, false);
        let pack = &builder.cards[0].1[0];
        assert_eq!(pack.desc == install, shown, "{status:?}");
    }
}

#[test]
fn drawing_ink_choices() {
    for (saved, current) in [("", "plain"), ("rainbow", "rainbow"), ("gold", "plain")] {
        let config = FakeSettingsSource::default().with_text(keys::drawing::INK, saved);
        let mut builder = super::Builder { cfg: &config, cards: Vec::new() };
        super::tab_drawing(&mut builder, None, None, 0, false);
        let ink = builder.cards[0]
            .1
            .iter()
            .find_map(|row| match &row.control {
                super::Control::Dropdown { path, options, current, .. }
                    if path == keys::drawing::INK =>
                {
                    Some((options.len(), current.clone()))
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(ink, (2, current.to_string()), "{saved}");
    }
}
