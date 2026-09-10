#![cfg(test)]

use super::*;
use crate::frontend::settings::test_source::FakeSettingsSource;

fn cfg() -> FakeSettingsSource {
    FakeSettingsSource::default()
}

#[test]
fn noctalia_mode_overrides_in_integrations() {
    for mode in ["follow", "keep", "dark", "light", "auto"] {
        let cfg = cfg().with_text(keys::noctalia::THEME_MODE, mode);
        let cards = build_tab("integrations", &cfg, &[], &[], "", &[]);
        let controls: Vec<_> = cards
            .iter()
            .flat_map(|(_, rows)| rows)
            .filter_map(|row| match &row.control {
                Control::Dropdown { path, current, options, .. }
                    if path == keys::noctalia::THEME_MODE =>
                {
                    Some((current, options))
                }
                _ => None,
            })
            .collect();
        assert_eq!(controls.len(), 1);
        assert_eq!(controls[0].0, mode);
        assert_eq!(
            controls[0].1.iter().map(|(value, _)| value.as_str()).collect::<Vec<_>>(),
            ["follow", "keep", "dark", "light", "auto"]
        );
    }
}

#[test]
fn picker_controls_bindings() {
    let cards = build_tab("picker", &cfg(), &[], &[], "", &[]);
    let controls = &cards
        .iter()
        .find(|(card, _)| card.title == "Controls")
        .expect("picker controls section")
        .1;
    let bindings: Vec<_> = controls
        .iter()
        .filter_map(|row| match &row.control {
            Control::KeyBinding { path, default, .. } => Some((path.as_str(), *default)),
            _ => None,
        })
        .collect();
    assert_eq!(bindings.len(), crate::contracts::picker::KEY_BINDINGS.len());
    assert_eq!(bindings[0], (skwd_config::keys::keybind::SELECT, "click"));
}

#[test]
fn theme_dropdown_presets() {
    let cfg = FakeSettingsSource::default()
        .with_text(keys::theme::BACKEND, "static")
        .with_saved_theme("Mine");
    let cards = build_tab("theme", &cfg, &[], &[], "", &[]);
    let options = cards
        .iter()
        .flat_map(|(_, rows)| rows)
        .find_map(|row| match &row.control {
            Control::Dropdown { path, options, .. } if path == "theme.staticTheme" => {
                Some(options.clone())
            }
            _ => None,
        })
        .expect("static theme dropdown exists when the backend is static");
    let values: Vec<&str> = options.iter().map(|(v, _)| v.as_str()).collect();
    assert!(values.contains(&"nord"));
    assert!(values.contains(&"dracula"));
    assert!(values.contains(&"Mine"));
    assert!(values.contains(&"custom"));
}

#[test]
fn theme_page_separation() {
    let cfg = FakeSettingsSource::default()
        .with_text(keys::theme::POLICY, "wallpaper")
        .with_text(keys::theme::AUTHORITY, "skwd")
        .with_text(keys::theme::ENGINE, "wallust");
    let available = [String::from("wallust"), String::from("noctalia")];
    let cards = build_tab("theme", &cfg, &[], &[], "", &available);
    let titles: Vec<_> = cards.iter().map(|(card, _)| card.title).collect();
    assert!(titles.contains(&"Behaviour"));
    assert!(titles.contains(&"Who drives colour"));
    assert!(titles.contains(&"Colour source"));
    assert!(titles.contains(&"Engine options"));

    let paths: Vec<_> = cards
        .iter()
        .flat_map(|(_, rows)| rows)
        .filter_map(|row| match &row.control {
            Control::Dropdown { path, .. } => Some(path.as_str()),
            _ => None,
        })
        .collect();
    assert!(paths.contains(&keys::theme::POLICY));
    assert!(paths.contains(&keys::theme::AUTHORITY));
    assert!(paths.contains(&keys::theme::ENGINE));
    assert!(!paths.contains(&keys::theme::BACKEND));
}

#[test]
fn external_authority_replaces_engine() {
    let cfg = FakeSettingsSource::default()
        .with_text(keys::theme::POLICY, "wallpaper")
        .with_text(keys::theme::AUTHORITY, "noctalia");
    let cards = build_tab("theme", &cfg, &[], &[], "", &[String::from("noctalia")]);
    let titles: Vec<_> = cards.iter().map(|(card, _)| card.title).collect();
    assert!(titles.contains(&"Who drives colour"));
    assert!(titles.contains(&"Imported result"));
    assert!(!titles.contains(&"Colour source"));
}

#[test]
fn matugen_under_theme() {
    fn paths(cards: &[(Card, Vec<Row>)]) -> Vec<&str> {
        cards
            .iter()
            .flat_map(|(_, rows)| rows)
            .filter_map(|row| match &row.control {
                Control::Toggle { path, .. } | Control::TextField { path, .. } => {
                    Some(path.as_str())
                }
                _ => None,
            })
            .collect()
    }

    let cfg = FakeSettingsSource::default().with_array_len(keys::integrations::LIST, 1);
    let theme = build_tab("theme", &cfg, &[], &[], "", &[]);
    let integrations = build_tab("integrations", &cfg, &[], &[], "", &[]);
    let theme_paths = paths(&theme);
    let integration_paths = paths(&integrations);

    assert!(theme_paths.contains(&keys::features::MATUGEN));
    assert!(theme_paths.contains(&keys::matugen::DEFAULT_CONFIG));
    assert!(theme_paths.contains(&keys::system::EXTERNAL_MATUGEN_COMMAND));
    assert!(theme_paths.contains(&"integrations.0.template"));
    assert!(!integration_paths.contains(&keys::features::MATUGEN));
    assert!(!integration_paths.iter().any(|path| path.starts_with("workspace.")));
    assert!(integrations.iter().any(|(card, _)| card.title == "KDE Plasma"));
}

#[test]
fn niri_unified_section() {
    let cfg = FakeSettingsSource::default();
    let mut builder = Builder { cfg: &cfg, cards: Vec::new() };
    theme_tabs::tab_niri(&mut builder, &[String::from("Catppuccin")]);

    let titles: Vec<_> = builder.cards.iter().map(|(card, _)| card.title).collect();
    assert_eq!(titles, ["Niri"]);
    assert_eq!(builder.cards[0].1.len(), 11);
    assert!(matches!(
        &builder.cards[0].1[4].control,
        Control::Code { snippet } if *snippet == NIRI_SNIPPET
    ));
    assert!(matches!(
        &builder.cards[0].1.last().expect("appearance palette").control,
        Control::Dropdown { path, .. } if path == keys::niri::BACKDROP_THEME
    ));
}

#[test]
fn monitor_choices_detected() {
    let cfg = FakeSettingsSource::default().with_text("monitor", "DP-9");
    let cards = build_tab_with_outputs(
        "picker",
        &cfg,
        &[],
        &[],
        "",
        &[],
        &[String::from("HDMI-A-1"), String::from("DP-1"), String::from("DP-1")],
    );
    let options = cards
        .iter()
        .flat_map(|(_, rows)| rows)
        .find_map(|row| match &row.control {
            Control::Chips { path, options, current, .. } if path == "monitor" => {
                assert_eq!(current, "DP-9");
                Some(options.clone())
            }
            _ => None,
        })
        .expect("picker settings expose monitor choices");
    assert_eq!(
        options,
        [
            (String::new(), String::from("Focused monitor")),
            (String::from("DP-1"), String::from("DP-1")),
            (String::from("HDMI-A-1"), String::from("HDMI-A-1")),
            (String::from("DP-9"), String::from("DP-9 (unavailable)")),
        ]
    );
}

#[test]
fn displays_placement_lock() {
    let cfg = FakeSettingsSource::default()
        .with_text("display.fillMode", "fill")
        .with_flag("display.outputLocks.DP-1", true);
    let statuses = [crate::contracts::daemon::OutputStatus {
        name: String::from("DP-1"),
        logical_width: 2560,
        logical_height: 1440,
        current: String::from("/walls/forest.png"),
        kind: crate::contracts::media::MediaKind::Static,
        path: String::from("/walls/forest.png"),
        fill: String::from("fit"),
        ..Default::default()
    }];
    let previews = std::collections::HashMap::from([(
        String::from("DP-1"),
        String::from("/thumbs/forest.webp"),
    )]);
    let cards = build_tab_with_output_statuses(
        "displays",
        &cfg,
        &[],
        &[],
        "",
        &[],
        &[String::from("DP-1")],
        &statuses,
        &previews,
    );
    let rows = &cards[0].1;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title, "DP-1");
    assert!(rows[0].desc.contains("2560 × 1440"));
    let Control::StackBar { id, summary, preview, rows, .. } = &rows[0].control else {
        panic!("display uses the compact stack bar");
    };
    assert_eq!(id, "display:DP-1");
    assert!(summary.contains("forest.png"));
    assert_eq!(preview.as_deref(), Some("/thumbs/forest.webp"));
    assert!(rows[0].title.contains("Placement"));
    assert!(matches!(
        &rows[0].control,
        Control::Chips { path, current, .. }
            if path == "display.fillModes.DP-1" && current == "fit"
    ));
    assert!(matches!(
        &rows[1].control,
        Control::Toggle { path, value: true } if path == "display.outputLocks.DP-1"
    ));
}

#[test]
fn remembered_monitor_offline() {
    let cfg = FakeSettingsSource::default().with_text("display.fillMode", "fill");
    let statuses = [crate::contracts::daemon::OutputStatus {
        name: String::from("DP-3"),
        target: String::from("@monitor:Dell U2723QE @ DP-3"),
        connected: false,
        width: 2560,
        height: 1440,
        current: String::from("/walls/return.png"),
        kind: crate::contracts::media::MediaKind::Static,
        path: String::from("/walls/return.png"),
        ..Default::default()
    }];
    let cards = build_tab_with_output_statuses(
        "displays",
        &cfg,
        &[],
        &[],
        "",
        &[],
        &[],
        &statuses,
        &std::collections::HashMap::new(),
    );
    let row = &cards[0].1[0];
    assert_eq!(row.title, "DP-3");
    assert_eq!(row.desc, "Offline · 2560 × 1440");
    let Control::StackBar { id, summary, .. } = &row.control else {
        panic!("remembered display uses the compact stack bar");
    };
    assert_eq!(id, "display:@monitor:Dell U2723QE @ DP-3");
    assert_eq!(summary, "return.png");
}

#[test]
fn library_watching_surfaces_polling_state_and_controls() {
    let cfg = FakeSettingsSource::default()
        .with_flag(keys::library::POLLING_FALLBACK, true)
        .with_text(keys::library::POLLING_INTERVAL_SECONDS, "45");
    let now_ms =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
            as u64;
    let status = crate::contracts::daemon::LibraryWatchStatus {
        ok: true,
        degraded: true,
        mode: crate::contracts::daemon::LibraryWatchMode::POLLING.to_string(),
        detail: String::from("native watching unavailable for one root"),
        interval_seconds: Some(45),
        entry_budget_per_root: Some(2048),
        last_successful_convergence_unix_ms: Some(now_ms.saturating_sub(65_000)),
        roots: vec![crate::contracts::daemon::LibraryWatchRootStatus {
            path: String::from("/mnt/library"),
            mode: crate::contracts::daemon::LibraryWatchMode::POLLING.to_string(),
            last_successful_convergence_unix_ms: Some(now_ms.saturating_sub(65_000)),
            ..Default::default()
        }],
    };
    let cards = build_tab_with_runtime_status(
        "library",
        &cfg,
        &[],
        &[],
        "",
        &[],
        &[],
        &[],
        &std::collections::HashMap::new(),
        Some(&status),
        None,
    );
    let rows = &cards
        .iter()
        .find(|(card, _)| card.title == "Library watching")
        .expect("library watching section")
        .1;

    assert_eq!(rows[0].title, "Polling fallback active");
    assert!(rows[0].desc.contains("one library folder"));
    assert!(rows[0].desc.contains("2048 entries"));
    assert!(rows[0].desc.contains("every 45 seconds"));
    assert!(rows[0].desc.contains("1 minute ago"));
    assert!(matches!(
        &rows[1].control,
        Control::Toggle { path, value: true } if path == keys::library::POLLING_FALLBACK
    ));
    assert!(matches!(
        &rows[2].control,
        Control::Number { path, unit, .. }
            if path == keys::library::POLLING_INTERVAL_SECONDS && *unit == "s"
    ));
}

#[test]
fn library_watching_explains_recovery_and_unavailable_states() {
    let cfg = FakeSettingsSource::default();
    for (mode, expected_title, expected_help) in [
        (
            crate::contracts::daemon::LibraryWatchMode::RECOVERING,
            "Native watching recovered",
            "hand-off scan is still running",
        ),
        (
            crate::contracts::daemon::LibraryWatchMode::UNAVAILABLE,
            "Library watching unavailable",
            "Enable Polling fallback",
        ),
    ] {
        let status = crate::contracts::daemon::LibraryWatchStatus {
            mode: mode.to_string(),
            ..Default::default()
        };
        let cards = build_tab_with_runtime_status(
            "library",
            &cfg,
            &[],
            &[],
            "",
            &[],
            &[],
            &[],
            &std::collections::HashMap::new(),
            Some(&status),
            None,
        );
        let status_row = &cards
            .iter()
            .find(|(card, _)| card.title == "Library watching")
            .expect("library watching section")
            .1[0];
        assert_eq!(status_row.title, expected_title);
        assert!(status_row.desc.contains(expected_help));
    }
}

#[test]
fn motion_weights_resets() {
    let cards = build_tab("motion", &cfg(), &[], &[], "", &[]);
    let rows = &cards
        .iter()
        .find(|(card, _)| card.title == "Shared speeds")
        .expect("shared motion section")
        .1;
    assert!(matches!(
        rows.as_slice(),
        [Row { control: Control::MotionWeights { weights }, .. }]
            if weights.iter().map(|(_, path, _)| path.as_str()).collect::<Vec<_>>()
                == [keys::motion::FAST_MS, keys::motion::STANDARD_MS, keys::motion::SLOW_MS]
    ));
}

#[test]
fn selector_no_shader_modes() {
    let cards = build_tab("picker", &cfg(), &[], &[], "", &[]);
    for (_, rows) in &cards {
        for row in rows {
            if let Control::Dropdown { path, options, .. } = &row.control {
                assert!(
                    !path.ends_with("displayMode")
                        || options.iter().all(|(v, _)| !v.starts_with("shader:"))
                );
            }
        }
    }
}

#[test]
fn picker_layout_presets() {
    let cards = build_tab("picker", &cfg(), &[], &[], "", &[]);
    assert!(cards.iter().all(|(card, _)| card.title != "Presets"));
    let (_, rows) = cards
        .iter()
        .find(|(card, _)| card.title == "Layout & presets")
        .expect("picker exposes its layout studio");
    assert!(rows.iter().any(|row| row.title == "Display mode"));
    assert!(rows.iter().any(|row| matches!(row.control, Control::Presets { .. })));
    assert!(rows.iter().any(|row| {
        matches!(&row.control, Control::TextField { key, .. } if key == PRESET_NAME_KEY)
    }));
}

#[test]
fn picker_layout_predicate() {
    use crate::frontend::settings::is_picker_layout_section;
    let cards = build_tab("picker", &cfg(), &[], &[], "", &[]);
    let live: Vec<usize> =
        (0..cards.len()).filter(|&section| is_picker_layout_section("picker", section)).collect();
    let studio = cards
        .iter()
        .position(|(card, _)| card.title == "Layout & presets")
        .expect("picker exposes its layout studio");
    assert_eq!(live, [studio], "the live-scene predicate must follow the composed picker tab");
}

#[test]
fn transition_preview_predicate() {
    use crate::frontend::settings::is_transition_preview_section;
    let sand = FakeSettingsSource::default().with_text(keys::transition::SHADER, "sand-helix");
    let cards = build_tab("motion", &sand, &[], &[], "", &[]);
    let titles: Vec<&str> = cards.iter().map(|(card, _)| card.title).collect();
    let live: Vec<usize> = (0..cards.len())
        .filter(|&section| is_transition_preview_section("motion", section))
        .collect();
    let transitions = titles.iter().position(|title| *title == "Wallpaper transitions").unwrap();
    assert_eq!(live, [transitions], "the preview runs only where its stage is visible");
    assert!(!titles.contains(&"Preview"));
    assert!(cards[transitions].1.iter().any(|row| matches!(row.control, Control::Preview)));
}

#[test]
fn wall_studio_mode_led() {
    let config = FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "wall");
    let cards = build_tab("picker", &config, &[], &[], "", &[]);
    let (_, rows) = cards
        .iter()
        .find(|(card, _)| card.title == "Layout & presets")
        .expect("picker exposes its layout studio");
    let paths = rows
        .iter()
        .filter_map(|row| match &row.control {
            Control::Toggle { path, .. }
            | Control::Number { path, .. }
            | Control::Dropdown { path, .. }
            | Control::Chips { path, .. } => Some(path.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    for path in
        [keys::selector::GRID_LAYOUT, keys::selector::GRID_COLUMNS, keys::selector::GRID_ROWS]
    {
        assert!(paths.contains(&path), "missing light-edit control for {path}");
    }
    for hidden in [
        keys::selector::GRID_STAGE_SHEAR_X,
        keys::selector::GRID_STAGE_DEPTH_ANGLE,
        keys::selector::GRID_FLOW_WAVE,
        keys::selector::GRID_SCATTER,
        keys::selector::GRID_SCALE_VARIANCE,
    ] {
        assert!(!paths.contains(&hidden), "engine internal leaked into Folio: {hidden}");
    }
    let mode = rows
        .iter()
        .find(|row| row.title == "Composition · Wall mode")
        .expect("Wall exposes authored modes");
    assert!(matches!(
        &mode.control,
        Control::Chips { options, .. }
            if options.iter().any(|(value, _)| value == "cylinder")
    ));

    let cylinder = FakeSettingsSource::default()
        .with_text(keys::selector::DISPLAY_MODE, "wall")
        .with_text(keys::selector::GRID_LAYOUT, "cylinder");
    let cylinder_cards = build_tab("picker", &cylinder, &[], &[], "", &[]);
    let cylinder_rows = &cylinder_cards
        .iter()
        .find(|(card, _)| card.title == "Layout & presets")
        .expect("picker exposes its layout studio")
        .1;
    assert!(cylinder_rows.iter().any(|row| row.title == "Composition · Curvature"));
    assert!(cylinder_rows.iter().any(|row| row.title == "Composition · Cylinder radius"));
}

#[test]
fn hex_studio_shape_families() {
    let config = FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "hex");
    let cards = build_tab("picker", &config, &[], &[], "", &[]);
    let (_, rows) = cards
        .iter()
        .find(|(card, _)| card.title == "Layout & presets")
        .expect("picker exposes its layout studio");
    let layout = rows
        .iter()
        .find(|row| row.title == "Composition · Geometric mode")
        .expect("Geometric exposes authored layout choices");
    assert!(matches!(
        &layout.control,
        Control::Chips { options, .. }
            if options.iter().map(|(value, _)| value.as_str())
                .eq(["flat", "arc", "wave", "s", "cylinder"])
    ));
    let shape = rows
        .iter()
        .find(|row| row.title == "Composition · Tile family")
        .expect("Geometric exposes geometric card shapes");
    assert!(matches!(
        &shape.control,
        Control::Chips { options, .. }
            if options.iter().map(|(value, _)| value.as_str())
                .eq(["triangle", "hexagon", "diamond", "rhombus"])
    ));
    let paths = rows
        .iter()
        .filter_map(|row| match &row.control {
            Control::Number { path, .. } | Control::Dropdown { path, .. } => Some(path.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    for hidden in [
        keys::selector::HEX_TWIST,
        keys::selector::HEX_SCATTER,
        keys::selector::HEX_STAGE_SHEAR_X,
        keys::selector::HEX_STAGE_DEPTH_ANGLE,
        keys::selector::HEX_LENS,
    ] {
        assert!(!paths.contains(&hidden), "engine internal leaked into Folio: {hidden}");
    }
}

#[test]
fn picker_type_badges() {
    let cards = build_tab("picker", &cfg(), &[], &[], "", &[]);
    let (_, rows) = cards
        .iter()
        .find(|(card, _)| card.title == "Cards & previews")
        .expect("picker exposes card presentation settings");
    assert!(rows.iter().any(|row| {
        matches!(
            &row.control,
            Control::Toggle { path, value }
                if path == keys::selector::SHOW_TYPE_BADGES && *value
        )
    }));
}

#[test]
fn slices_expose_live_edge_tilt() {
    let config = FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "slices");
    let cards = build_tab("picker", &config, &[], &[], "", &[]);
    let (_, rows) = cards
        .iter()
        .find(|(card, _)| card.title == "Layout & presets")
        .expect("picker exposes its layout settings");
    assert!(rows.iter().any(|row| {
        matches!(
            &row.control,
            Control::Number { path, unit, .. }
                if path == keys::selector::SLICE_EDGE_TILT && *unit == "px"
        ) && row.title == "Slice size · Edge tilt"
    }));
}

#[test]
fn filter_search_top_level() {
    let picker = build_tab("picker", &cfg(), &[], &[], "", &[]);
    assert!(picker.iter().all(|(card, _)| card.title != "Filter bar"));

    let filter = build_tab("filter", &cfg(), &[], &[], "", &[]);
    assert_eq!(
        filter.iter().map(|(card, _)| card.title).collect::<Vec<_>>(),
        [
            "Appearance",
            "Buttons",
            "Resolution presets",
            "Defaults",
            "Visibility",
            "Search",
            "Semantic models"
        ]
    );
    assert!(filter.iter().flat_map(|(_, rows)| rows).any(|row| {
        matches!(
            &row.control,
            Control::Dropdown { path, options, current, .. }
                if path == keys::filter_bar::VISUAL_STYLE
                    && current == "match"
                    && options.iter().map(|(key, _)| key.as_str()).collect::<Vec<_>>()
                        == ["match", "slices", "hex", "wall"]
        )
    }));
}

#[test]
fn position_tab_groups_independent_surfaces() {
    let cards = build_tab("position", &cfg(), &[], &[], "", &[]);
    assert_eq!(
        cards.iter().map(|(card, _)| card.title).collect::<Vec<_>>(),
        [
            "Slices picker",
            "Geometric picker",
            "Wall picker",
            "Sandy picker",
            "Filter bar position",
            "Search panel position",
        ]
    );
    assert!(cards.iter().all(|(card, _)| !card.subtitle.is_empty()));
    let paths = cards
        .iter()
        .flat_map(|(_, rows)| rows)
        .filter_map(|row| match &row.control {
            Control::Number { path, .. } => Some(path.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        [
            keys::selector::SLICE_STAGE_X,
            keys::selector::SLICE_STAGE_Y,
            keys::selector::HEX_STAGE_X,
            keys::selector::HEX_STAGE_Y,
            keys::selector::GRID_STAGE_X,
            keys::selector::GRID_STAGE_Y,
            keys::selector::SANDY_STAGE_X,
            keys::selector::SANDY_STAGE_Y,
            keys::filter_bar::OFFSET_X,
            keys::filter_bar::OFFSET_Y,
            keys::selector::TAG_CLOUD_OFFSET_X,
            keys::selector::TAG_CLOUD_OFFSET_Y,
        ]
    );
}

#[test]
fn resolution_preset_items() {
    let root = keys::filter_bar::RESOLUTION_PRESETS;
    let config = FakeSettingsSource::default()
        .with_array_len(root, 2)
        .with_text(&format!("{root}.0.label"), "FHD")
        .with_text(&format!("{root}.0.orientation"), "wide")
        .with_text(&format!("{root}.0.from"), "1920x1080")
        .with_text(&format!("{root}.0.to"), "2559x1439")
        .with_text(&format!("{root}.1.label"), "2K")
        .with_text(&format!("{root}.1.orientation"), "tall")
        .with_text(&format!("{root}.1.from"), "2560x1440")
        .with_text(&format!("{root}.1.to"), "3839x2159");
    let cards = build_tab("filter", &config, &[], &[], "", &[]);
    let rows = &cards
        .iter()
        .find(|(card, _)| card.title == "Resolution presets")
        .expect("resolution preset section")
        .1;
    assert_eq!(rows.len(), 3);
    let Control::Details { id, summary, rows: fields } = &rows[0].control else {
        panic!("resolution preset must use a details item");
    };
    assert_eq!(rows[0].title, "FHD");
    assert_eq!(id, "filterBar.resolutionPresets.0");
    assert_eq!(summary, "WIDE · 1920x1080 - 2559x1439");
    assert_eq!(fields.len(), 5);
    assert!(matches!(
        &fields[0].control,
        Control::TextField { path, .. } if path == "filterBar.resolutionPresets.0.label"
    ));
    assert!(matches!(
        &fields[1].control,
        Control::Dropdown { path, .. } if path == "filterBar.resolutionPresets.0.orientation"
    ));
    assert!(matches!(
        &fields[2].control,
        Control::TextField { path, .. } if path == "filterBar.resolutionPresets.0.from"
    ));
    assert!(matches!(
        &fields[3].control,
        Control::TextField { path, .. } if path == "filterBar.resolutionPresets.0.to"
    ));
    assert!(matches!(
        &fields[4].control,
        Control::ActionBtn { id: ActionId::RemoveResolutionPreset(0), .. }
    ));
    assert!(matches!(
        &rows[2].control,
        Control::ActionBtn { id: ActionId::AddResolutionPreset, .. }
    ));
}

#[test]
fn selector_flip_controls() {
    let cards = build_tab("motion", &cfg(), &[], &[], "", &[]);
    let (_, rows) = cards
        .iter()
        .find(|(card, _)| card.title == "Picker motion")
        .expect("picker motion section");
    for path in [
        keys::selector::FLIP_EFFECT,
        keys::selector::FLIP_SHADER,
        keys::selector::FLIP_BACK_REVEAL,
        keys::selector::FLIP_DURATION_MS,
    ] {
        assert!(rows.iter().any(|row| match &row.control {
            Control::Toggle { path: actual, .. }
            | Control::Dropdown { path: actual, .. }
            | Control::Number { path: actual, .. } => actual == path,
            _ => false,
        }));
    }
    assert!(rows.iter().any(|row| {
        matches!(
            row.control,
            Control::Toggle { ref path, value: true }
                if path == keys::selector::FLIP_SHADER
        )
    }));
    assert!(rows.iter().any(|row| {
        matches!(
            row.control,
            Control::Toggle { ref path, value: true }
                if path == keys::selector::FLIP_BACK_REVEAL
        )
    }));
}

#[test]
fn transition_style_section() {
    let cfg = FakeSettingsSource::default().with_text(keys::transition::SHADER, "sand-helix");
    let cards = build_tab("motion", &cfg, &[], &[], "", &[]);
    assert_eq!(
        cards.iter().map(|(card, _)| card.title).collect::<Vec<_>>(),
        [
            "Shared speeds",
            "Launch",
            "Picker motion",
            "Wallpaper transitions",
            "Transition performance"
        ]
    );
    let (_, rows) = cards
        .iter()
        .find(|(card, _)| card.title == "Wallpaper transitions")
        .expect("wallpaper transitions section");
    assert!(rows.iter().any(|row| row.title == "Family"));
    assert!(rows.iter().any(|row| row.title == "Transition"));
    assert!(rows.iter().any(|row| row.title == "Show preview"));
    assert!(rows.iter().any(|row| matches!(row.control, Control::Preview)));
}

#[test]
fn transition_perf_families() {
    let cfg = FakeSettingsSource::default().with_text(keys::transition::SHADER, "fadecolor");
    let cards = build_tab("motion", &cfg, &[], &[], "", &[]);
    let (_, rows) = cards
        .iter()
        .find(|(card, _)| card.title == "Transition performance")
        .expect("transition performance section");
    assert!(rows.iter().any(|row| row.title == "Fadecolor"));
    assert!(!rows.iter().any(|row| row.title == "Quality"));
}

#[test]
fn performance_power_controls() {
    let cards = build_tab("performance", &cfg(), &[], &[], "", &[]);
    assert_eq!(cards.first().map(|(card, _)| card.title), Some("Power"));
    let rows: Vec<&Row> = cards.iter().flat_map(|(_, rows)| rows).collect();
    for path in [
        keys::performance::BATTERY_SAVER,
        keys::performance::GPU_PREFERENCE,
        keys::performance::BATTERY_FPS,
        keys::performance::BATTERY_VIDEO_IDLE_SECONDS,
        keys::performance::BATTERY_WALLPAPER_PERFORMANCE,
        keys::general::MAX_FPS,
    ] {
        assert!(
            rows.iter().any(|row| match &row.control {
                Control::Toggle { path: actual, .. }
                | Control::Dropdown { path: actual, .. }
                | Control::Number { path: actual, .. } => actual == path,
                _ => false,
            }),
            "missing {path}"
        );
    }
    assert!(rows.iter().any(|row| {
        matches!(
            &row.control,
            Control::Dropdown { path, options, current, .. }
                if path == keys::performance::GPU_PREFERENCE
                    && current == "auto"
                    && options.iter().map(|(key, _)| key.as_str()).collect::<Vec<_>>()
                        == ["auto", "low", "high", "none"]
        )
    }));
    assert!(rows.iter().any(|row| row.title == "Wallpaper Engine scene renderer"));
}

#[test]
fn power_status_battery_saver() {
    let status = |cfg: &FakeSettingsSource| {
        build_tab("performance", cfg, &[], &[], "", &[])
            .into_iter()
            .flat_map(|(_, rows)| rows)
            .find(|row| row.title == "Current power source")
            .map(|row| row.desc)
            .expect("performance tab has a power status row")
    };
    let disabled = FakeSettingsSource::default()
        .with_on_battery(true)
        .with_flag(keys::performance::BATTERY_SAVER, false);
    let enabled = FakeSettingsSource::default()
        .with_on_battery(true)
        .with_flag(keys::performance::BATTERY_SAVER, true);

    assert_eq!(status(&disabled), "Battery (automatic limits are disabled)");
    assert_eq!(status(&enabled), "Battery (automatic limits are active)");
    assert_eq!(
        status(&FakeSettingsSource::default()),
        "External power, desktop, or unavailable (automatic limits are inactive)"
    );
}

#[test]
fn sand_cost_format() {
    assert_eq!(super::selector_tab::compact_count(2_764_800), "2.8M");
    assert_eq!(super::selector_tab::compact_count(460_404), "460K");
    assert_eq!(super::selector_tab::compact_count(28_860), "29K");
    assert_eq!(super::selector_tab::heaviness(2_764_800), "Very heavy");
    assert_eq!(super::selector_tab::heaviness(460_404), "Heavy");
    assert_eq!(super::selector_tab::heaviness(113_880), "Moderate");
    assert_eq!(super::selector_tab::heaviness(28_860), "Light");
}

#[test]
fn folder_dropdown_values() {
    let opts = super::builder::folder_dropdown_options(&[
        String::new(),
        String::from("*"),
        String::from("anime"),
        String::from("nature"),
    ]);
    assert_eq!(opts[0].0, "*");
    assert_eq!(opts[1].0, "");
    assert_eq!(opts[2], (String::from("anime"), String::from("anime")));
    assert_eq!(opts[3], (String::from("nature"), String::from("nature")));
    assert_eq!(opts.len(), 4);
}

#[test]
fn folder_sentinel_labels() {
    let opts = super::builder::folder_dropdown_options(&[String::new(), String::from("*")]);
    let all = crate::i18n::tr("settings-filter-all-folders");
    let main = crate::i18n::tr("settings-filter-main-folder");
    assert_eq!(opts[0], (String::from("*"), all.to_string()));
    assert_eq!(opts[1], (String::new(), main.to_string()));
    assert_ne!(opts[0].1, opts[0].0);
    assert_ne!(opts[1].1, opts[1].0);
}

#[test]
fn static_renderer_choices() {
    assert_eq!(super::super::tables::ENGINES, [("skwd-paper", "Skwd-paper"), ("awww", "awww")]);
}

#[test]
fn tinier_video_engine_is_visible_but_disabled() {
    let cards = build_tab("playback", &cfg(), &[], &[], "", &[]);
    let row = cards
        .iter()
        .flat_map(|(_, rows)| rows)
        .find(|row| {
            matches!(
                &row.control,
                Control::Chips { path, .. } if path == keys::paper::VIDEO_ENGINE
            )
        })
        .expect("playback exposes the video engine choices");
    let Control::Chips { options, disabled, .. } = &row.control else {
        unreachable!();
    };

    assert_eq!(
        options,
        &[
            (String::from("vulkan"), String::from("Vulkan")),
            (String::from("tinier"), String::from("Tinier (work in progress)")),
        ]
    );
    assert_eq!(disabled, &[String::from("tinier")]);
    assert!(row.desc.contains("cannot be selected"));
}

#[test]
fn sources_compact_index() {
    let cfg = cfg()
        .with_flag(keys::features::WALLHAVEN, false)
        .with_flag(keys::sources::UNSPLASH_ENABLED, true);
    let cards = build_tab("sources", &cfg, &[], &[], "", &[]);
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].0.title, "Source providers");

    let expected = [
        ("Wallhaven", "source.wallhaven", "Disabled", 7),
        ("Unsplash", "source.unsplash", "Enabled", 2),
        ("Pexels", "source.pexels", "Disabled", 2),
        ("Bing daily", "source.bing", "Disabled", 2),
        ("Steam Workshop", "source.workshop", "Enabled", 11),
        ("YouTube", "source.youtube", "Disabled", 3),
    ];
    assert_eq!(cards[0].1.len(), expected.len());
    for (row, (title, expected_id, expected_summary, field_count)) in
        cards[0].1.iter().zip(expected)
    {
        assert_eq!(row.title, title);
        let Control::Details { id, summary, rows } = &row.control else {
            panic!("source providers must use reusable details items");
        };
        assert_eq!(id, expected_id);
        assert_eq!(summary, expected_summary);
        assert_eq!(rows.len(), field_count);
    }
}

#[test]
fn theme_options_available() {
    let loading = super::theme_tabs::engine_options(&[], "skwd-iris");
    let loading_ids: Vec<_> = loading.iter().map(|(id, _)| *id).collect();
    assert_eq!(loading_ids, ["skwd-iris"]);

    let detected: Vec<String> =
        ["skwd-iris", "matugen", "wallust", "caelestia", "noctalia", "dms", "end4"]
            .iter()
            .map(|id| (*id).to_string())
            .collect();
    let options = super::theme_tabs::engine_options(&detected, "skwd-iris");
    let ids: Vec<_> = options.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, ["skwd-iris", "matugen", "wallust"]);

    let legacy = super::theme_tabs::engine_options(&detected, "skwd-wallust");
    assert!(legacy.iter().any(|(id, label)| { *id == "skwd-wallust" && label.contains("legacy") }));

    let drivers = super::theme_tabs::authority_options(&detected, "skwd");
    assert_eq!(
        drivers.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        ["skwd", "caelestia", "noctalia", "dms", "end4"]
    );
}

#[test]
fn search_single_index() {
    let cfg =
        FakeSettingsSource::default().with_text(keys::tagging::DEFAULT_SEARCH_MODE, "describe");
    assert!(
        crate::frontend::settings::visible_tabs(&cfg)
            .iter()
            .any(|(key, label)| *key == "filter" && *label == "Filter & Search")
    );
    let cards = build_tab("filter", &cfg, &[], &[], "Tagging 3/10", &[]);
    assert_eq!(cards.len(), 7);
    let cards = &cards[5..];
    assert_eq!(cards[0].0.title, "Search");
    assert_eq!(cards[1].0.title, "Semantic models");

    let expected = [("Search", "search.discovery", "Describe", 2)];
    assert_eq!(cards[0].1.len(), expected.len());
    for (row, (title, expected_id, expected_summary, field_count)) in
        cards[0].1.iter().zip(expected)
    {
        assert_eq!(row.title, title);
        let Control::Details { id, summary, rows } = &row.control else {
            panic!("search and tagging tasks must use reusable details items");
        };
        assert_eq!(id, expected_id);
        assert_eq!(summary, expected_summary);
        assert_eq!(rows.len(), field_count);
    }

    let nested: Vec<_> = cards[0]
        .1
        .iter()
        .flat_map(|row| match &row.control {
            Control::Details { rows, .. } => rows.as_slice(),
            _ => &[],
        })
        .collect();
    assert!(nested.iter().any(|row| row.title == "Describe search"));
    assert!(nested.iter().any(|row| {
        matches!(
            &row.control,
            Control::Dropdown { path, options, current, .. }
                if path == skwd_config::keys::tagging::DEFAULT_SEARCH_MODE
                    && options.iter().map(|(key, _)| key.as_str()).collect::<Vec<_>>()
                        == ["tags", "describe"]
                    && current == "describe"
        )
    }));
}

#[test]
fn model_shelf_packs() {
    let manifest = "/models/pe-core/semantic-pack.json";
    let cfg = FakeSettingsSource::default()
        .with_array_len(keys::semantic::MODELS, 1)
        .with_text("semantic.models.0.name", "PE-Core L14 336")
        .with_text("semantic.models.0.manifest", manifest)
        .with_text(keys::semantic::MANIFEST, manifest)
        .with_text(keys::semantic::INDEX_PROFILE, "multiview");
    let cards = build_tab("filter", &cfg, &[], &[], "", &[]);
    let rows = &cards.iter().find(|(card, _)| card.title == "Semantic models").unwrap().1;
    assert!(rows.iter().any(|row| {
        matches!(row.control, Control::ActionBtn { id: ActionId::ImportSemanticModel, .. })
    }));
    assert!(rows.iter().any(|row| {
        matches!(
            &row.control,
            Control::Dropdown { path, options, current, .. }
                if path == keys::semantic::MANIFEST
                    && current == manifest
                    && options.iter().any(|(value, label)| value == manifest && label == "PE-Core L14 336")
        )
    }));
    assert!(rows.iter().any(|row| {
        matches!(
            &row.control,
            Control::Chips { path, current, .. }
                if path == keys::semantic::INDEX_PROFILE && current == "multiview"
        )
    }));
    assert!(rows.iter().any(|row| {
        matches!(
            &row.control,
            Control::Details { id, summary, .. }
                if id == "semantic.models.0" && summary == manifest
        )
    }));
}

#[test]
fn model_import_progress() {
    let cards =
        build_tab("filter", &FakeSettingsSource::default(), &[], &[], "Validating pack", &[]);
    assert!(cards.iter().find(|(card, _)| card.title == "Semantic models").unwrap().1.iter().any(
        |row| {
            row.title == "Model-pack import"
                && row.desc == "Validating pack"
                && matches!(row.control, Control::Static)
        }
    ));
}

#[test]
fn controls_no_ghosts() {
    let cards = build_tab("picker", &cfg(), &[], &[], "", &[]);
    let (_, rows) =
        cards.iter().find(|(card, _)| card.title == "Controls").expect("picker controls section");
    let mut dump = Vec::new();
    for row in rows {
        let kind = match &row.control {
            Control::KeyBinding { path, default, .. } => format!("binding {path} = {default}"),
            Control::Static => String::from("static"),
            Control::ActionBtn { .. } => String::from("action"),
            other => format!("{other:?}"),
        };
        dump.push(format!("{} | {} | {kind}", row.title, row.desc));
    }
    println!("{}", dump.join("\n"));
    assert_eq!(
        rows.iter().filter(|row| matches!(row.control, Control::KeyBinding { .. })).count(),
        crate::contracts::picker::KEY_BINDINGS.len(),
        "every action is rebindable from this page"
    );
    assert!(
        rows.iter()
            .filter(|row| matches!(row.control, Control::Static))
            .all(|row| !row.desc.trim().is_empty()),
        "a static row must carry its own explanation, not an empty value slot"
    );
    assert!(rows.iter().any(|row| matches!(row.control, Control::ActionBtn { .. })));
}

#[test]
fn performance_lists_detected_devices_and_retains_unavailable_selection() {
    let devices = vec![
        crate::contracts::capabilities::GraphicsDevice {
            id: "uuid:11111111111111111111111111111111".into(),
            name: "Integrated GPU".into(),
        },
        crate::contracts::capabilities::GraphicsDevice {
            id: "uuid:22222222222222222222222222222222".into(),
            name: "Discrete GPU".into(),
        },
    ];
    let cfg = FakeSettingsSource::default()
        .with_devices(devices.clone())
        .with_text(keys::performance::GPU_DEVICE, &devices[1].id);
    let cards = build_tab("performance", &cfg, &[], &[], "", &[]);
    let row = cards.iter().flat_map(|(_, rows)| rows).find(|row| matches!(&row.control, Control::Dropdown { path, .. } if path == keys::performance::GPU_DEVICE)).unwrap();
    let Control::Dropdown { options, current, .. } = &row.control else { unreachable!() };
    assert_eq!(current, &devices[1].id);
    assert_eq!(
        options,
        &vec![
            ("auto".into(), "Automatic".into()),
            (devices[0].id.clone(), devices[0].name.clone()),
            (devices[1].id.clone(), devices[1].name.clone())
        ]
    );
    let cfg =
        FakeSettingsSource::default().with_text(keys::performance::GPU_DEVICE, &devices[1].id);
    let cards = build_tab("performance", &cfg, &[], &[], "", &[]);
    assert!(cards.iter().flat_map(|(_, rows)| rows).any(|row| matches!(&row.control, Control::Dropdown { path, options, current, .. } if path == keys::performance::GPU_DEVICE && current == &devices[1].id && options.iter().any(|(id, label)| id == current && label.contains("unavailable")))));
}

#[test]
fn language_tab_lists_supported_languages_and_system_default() {
    for (saved, expected) in [("auto", "auto"), ("sv", "sv-SE"), ("es-ES", "es-ES")] {
        let cfg = cfg().with_text(keys::general::LANGUAGE, saved);
        assert!(
            crate::frontend::settings::visible_tabs(&cfg).iter().any(|(id, _)| *id == "language")
        );
        let cards = build_tab("language", &cfg, &[], &[], "", &[]);
        assert_eq!(cards.len(), 1);
        let Control::Dropdown { path, options, current, .. } = &cards[0].1[0].control else {
            panic!("language choice missing");
        };
        assert_eq!(path, keys::general::LANGUAGE);
        assert_eq!(current, expected);
        assert_eq!(
            options.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(),
            ["auto", "en-US", "sv-SE", "es-ES"]
        );
        assert_eq!(options[1].1, "English");
        assert_eq!(options[2].1, "Svenska");
        assert_eq!(options[3].1, "Español");
    }
}

#[test]
fn automatic_pause_controls_have_one_playback_owner() {
    let config = cfg();
    let paths = [
        keys::playback::PROCESS_ENABLED,
        keys::playback::PROCESSES,
        keys::playback::FULLSCREEN,
        keys::playback::MAXIMIZED,
        keys::playback::FULLSCREEN_SCOPE,
        keys::playback::RESUME_DELAY,
        keys::paper::IDLE_PAUSE_SECONDS,
    ];
    let cards = build_tab("playback", &config, &[], &[], "", &[]);
    for path in paths {
        assert!(
            cards.iter().flat_map(|(_, rows)| rows).any(|row| match &row.control {
                Control::Toggle { path: actual, .. }
                | Control::Number { path: actual, .. }
                | Control::TextField { path: actual, .. }
                | Control::Dropdown { path: actual, .. } => actual == path,
                _ => false,
            }),
            "missing {path}"
        );
    }
}

#[test]
fn full_width_pause_is_only_exposed_on_niri() {
    for niri in [false, true] {
        let config = cfg().with_niri(niri).with_flag(keys::niri::FULL_WIDTH_PAUSE, true);
        let cards = build_tab("playback", &config, &[], &[], "", &[]);
        let count = cards.iter().flat_map(|(_, rows)| rows).filter(|row| {
            matches!(&row.control, Control::Toggle { path, .. } if path == keys::niri::FULL_WIDTH_PAUSE)
        }).count();
        assert_eq!(count, usize::from(niri));
    }
}
