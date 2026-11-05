use super::*;
use crate::contracts::daemon::{AppThemeStatus, AppThemesResult};
use crate::frontend::settings::Slot;
use crate::frontend::settings::places::drawn;
use crate::frontend::settings::test_source::{FakeSettingsSource, place_source as source, plain};
use crate::frontend::settings::{RuntimeStatus, place, search_settings};

fn sections(cfg: &dyn SettingsSource) -> Vec<Section> {
    crate::i18n::with_language("en-US", || source(cfg).place(place::COLOURS).sections)
}

fn section(cfg: &dyn SettingsSource, key: &str) -> Section {
    sections(cfg).into_iter().find(|section| section.id.key == key).expect("section")
}

fn said(cfg: &dyn SettingsSource, key: &str) -> String {
    plain(&section(cfg, key).say.text)
}

fn apps_section(source: &PlaceSource<'_>) -> Section {
    let found = source.place(place::COLOURS).sections;
    found.into_iter().find(|section| section.id.key == DESTINATIONS).expect("apps section")
}

fn default() -> FakeSettingsSource {
    FakeSettingsSource::default()
}

fn wallpaper() -> FakeSettingsSource {
    default().with_text(theme::POLICY, WALLPAPER).with_text(theme::AUTHORITY, SKWD)
}

fn matugen_on() -> FakeSettingsSource {
    wallpaper().with_text(theme::ENGINE, MATUGEN)
}

fn fixed() -> FakeSettingsSource {
    default().with_text(theme::POLICY, FIXED)
}

fn off() -> FakeSettingsSource {
    default().with_text(theme::POLICY, OFF)
}

fn shell(name: &str) -> FakeSettingsSource {
    wallpaper().with_text(theme::AUTHORITY, name)
}

fn app(id: &str, enabled: bool, state: &str) -> AppThemeStatus {
    AppThemeStatus {
        id: id.into(),
        name: id.into(),
        installed: true,
        config_found: true,
        enabled,
        state: state.into(),
        config_path: String::new(),
        output_path: String::new(),
        detail: String::new(),
        can_enable: true,
        can_disable: enabled,
        can_adopt: false,
        template_path: String::new(),
        customized: false,
        can_disconnect: enabled,
        can_reconnect: false,
    }
}

fn with_apps<T>(
    cfg: &dyn SettingsSource,
    apps: Vec<AppThemeStatus>,
    run: impl Fn(&PlaceSource<'_>) -> T,
) -> T {
    let result = AppThemesResult { apps };
    let runtime = RuntimeStatus { app_themes: Some(&result), ..RuntimeStatus::default() };
    crate::i18n::with_language("en-US", || run(&PlaceSource { runtime, ..source(cfg) }))
}

#[test]
fn board_order() {
    let keys: Vec<_> = sections(&default()).iter().map(|section| section.id.key).collect();
    assert_eq!(keys, [ORIGIN, GENERATOR, LOOK, DESTINATIONS, OUTPUTS, SHELLS, LOCKS]);
}

#[test]
fn every_flow_drawn() {
    for cfg in [
        wallpaper(),
        matugen_on(),
        wallpaper().with_text(theme::ENGINE, "wallust"),
        wallpaper().with_text(theme::ENGINE, NATIVE),
        fixed(),
        off(),
        shell(NOCTALIA),
        shell(DMS),
        shell("caelestia"),
        shell("end4"),
        default().with_text(keys::lock_screens::MODE, keys::lock_screens::STATIC),
        default().with_array_len(keys::integrations::LIST, 2),
    ] {
        for found in sections(&cfg) {
            assert!(drawn(&found), "{}", found.title);
        }
    }
}

#[test]
fn origin_sentences() {
    assert_eq!(said(&wallpaper(), ORIGIN), "Colours [0], made by [1].");
    assert_eq!(said(&fixed(), ORIGIN), "Colours [0].");
    assert_eq!(said(&off(), ORIGIN), "Colours [0].");
    assert!(!section(&off(), ORIGIN).has_body());
}

#[test]
fn generator_flow() {
    let seeded = said(&matugen_on(), GENERATOR);
    assert_eq!(seeded, "[0] makes the palette, seeded from slot [1] at contrast [2].");
    assert_eq!(said(&wallpaper(), GENERATOR), "[0] makes the palette.");
    assert_eq!(said(&fixed(), GENERATOR), "[0] for every wallpaper.");
    assert_eq!(said(&off(), GENERATOR), "Not used while colours are left as they are.");
    assert_eq!(said(&shell(NOCTALIA), GENERATOR), "Noctalia makes the palette from the wallpaper.");
    let palette = section(&fixed(), GENERATOR);
    assert_eq!(palette.title, "Palette");
    assert_eq!(palette.id.key, GENERATOR);
}

#[test]
fn generator_body_always() {
    for cfg in [wallpaper(), fixed(), off(), shell(DMS)] {
        let paths: Vec<_> = section(&cfg, GENERATOR)
            .rows()
            .filter_map(|row| row.control.path().map(str::to_string))
            .collect();
        assert!(paths.iter().any(|path| path == matugen::DEFAULT_CONFIG));
        assert!(paths.iter().any(|path| path == keys::system::EXTERNAL_MATUGEN_COMMAND));
    }
}

#[test]
fn matugen_values_echo() {
    let found = section(&matugen_on(), GENERATOR);
    let heads: Vec<_> = found.head.iter().filter_map(|row| row.control.path()).collect();
    assert_eq!(heads, [theme::ENGINE, matugen::COLOR_INDEX, matugen::CONTRAST]);
    let echoes: Vec<_> = found
        .groups
        .iter()
        .flat_map(|group| &group.echoes)
        .filter_map(|(_, row)| row.control.path())
        .collect();
    assert_eq!(echoes, [matugen::COLOR_INDEX, matugen::CONTRAST]);
}

#[test]
fn look_flow() {
    let plain_look = said(&wallpaper(), LOOK);
    assert_eq!(plain_look, "[0] character with a [1] finish, variant [2].");
    assert_eq!(said(&matugen_on(), LOOK), plain_look);
    assert_eq!(said(&shell(NOCTALIA), LOOK), "Noctalia's palette, imported as the [0] variant.");
    assert_eq!(said(&shell("caelestia"), LOOK), "Caelestia sets its own look.");
    assert_eq!(said(&fixed(), LOOK), "The fixed palette sets its own look.");
    assert_eq!(said(&off(), LOOK), "Not used while colours are left as they are.");
}

#[test]
fn designer_gone_off() {
    let idle = section(&off(), LOOK);
    assert_eq!(idle.foot.as_deref(), Some(tr(NO_DESIGNER)));
    let live = section(&fixed(), LOOK);
    assert!(live.foot.is_none());
    assert!(live.rows().any(|row| {
        matches!(row.control, Control::ActionBtn { id: ActionId::OpenThemeDesigner, .. })
    }));
}

#[test]
fn apps_sentence_counts() {
    let cfg = default();
    let said = |apps| with_apps(&cfg, apps, |source| plain(&apps_section(source).say.text));
    let three = vec![
        app("kitty", true, "applied"),
        app("btop", true, "changed"),
        app("rofi", false, "ready"),
    ];
    assert_eq!(said(three), "2 of 3 apps follow your colours; btop's config changed outside Skwd.");
    assert_eq!(said(vec![app("kitty", true, "applied")]), "1 of 1 apps follows your colours.");
    assert_eq!(
        said(vec![app("kitty", true, "changed"), app("btop", true, "changed")]),
        "2 of 2 apps follow your colours; 2 apps changed outside Skwd."
    );
}

#[test]
fn apps_loading_says() {
    assert_eq!(said(&default(), DESTINATIONS), tr("settings-app-themes-loading"));
}

#[test]
fn apps_split_columns() {
    let apps: Vec<_> = (0..9).map(|at| app(&format!("app{at}"), true, "applied")).collect();
    let cfg = default();
    with_apps(&cfg, apps, |source| {
        let found = apps_section(source);
        let count = |at: usize| {
            found.groups[at]
                .rows
                .iter()
                .filter(|row| matches!(row.control, Control::AppTheme { .. }))
                .count()
        };
        assert_eq!((count(0), count(1)), (APPS_LEFT, 9 - APPS_LEFT));
        assert!(found.groups[1].rows.last().is_some_and(|row| {
            matches!(row.control, Control::ActionBtn { id: ActionId::RefreshAppThemes, .. })
        }));
    });
}

#[test]
fn apps_keep_profiles() {
    let cfg = default();
    with_apps(&cfg, vec![app("kitty", true, "applied")], |source| {
        let found = apps_section(source);
        let rows = &found.groups[0].rows;
        assert!(matches!(rows[0].control, Control::AppTheme { .. }));
        assert_eq!(rows[1].look.cell, Cell::Joined);
        let profile = format!("{}.kitty", theme::APP_PROFILES);
        assert_eq!(rows[1].control.path(), Some(profile.as_str()));
        assert_eq!(rows[0].hint(), "Colours applied");
    });
}

#[test]
fn configured_says_installed() {
    let cfg = default();
    with_apps(&cfg, vec![app("kitty", true, "configured")], |source| {
        let found = apps_section(source);
        assert_eq!(found.groups[0].rows[0].hint(), "Theme configured · Installed");
    });
}

#[test]
fn busy_shows_status() {
    let cfg = default();
    let busy = app("kitty", true, "busy");
    with_apps(&cfg, vec![busy], |source| {
        let found = apps_section(source);
        assert_eq!(found.groups[0].rows[0].hint(), tr("settings-app-themes-busy"));
    });
}

#[test]
fn long_status_whole() {
    let cfg = default();
    with_apps(&cfg, vec![app("kitty", true, "reload-sent")], |source| {
        let found = apps_section(source);
        assert_eq!(found.groups[0].rows[0].hint(), "Colours written · Reload requested");
    });
}

#[test]
fn outputs_say() {
    let none = section(&default(), OUTPUTS);
    assert!(none.rows().any(|row| row.control.path() == Some(keys::features::MATUGEN)));
    assert_eq!(plain(&none.say.text), "No integration renders a template yet.");
    let one = default().with_array_len(keys::integrations::LIST, 1);
    assert_eq!(
        said(&one, OUTPUTS),
        "1 integration renders a template to a file after each colour change."
    );
}

#[test]
fn outputs_one_shape() {
    let one = default().with_array_len(keys::integrations::LIST, 1);
    let found = section(&one, OUTPUTS);
    let kinds: Vec<_> = found.groups.iter().map(|group| group.kind).collect();
    assert_eq!(kinds, [GroupKind::Column, GroupKind::Column, GroupKind::Lines]);
    assert_eq!(found.groups[2].rows.len(), 3);
}

#[test]
fn outputs_headed() {
    let two = default().with_array_len(keys::integrations::LIST, 2);
    let found = section(&two, OUTPUTS);
    assert_eq!(found.groups.len(), 6);
    assert_eq!(found.groups[0].heading.as_deref(), Some("Output 1"));
    assert_eq!(found.groups[3].heading.as_deref(), Some("Output 2"));
    assert_eq!(found.groups[2].rows.len(), 2);
    assert_eq!(found.groups[5].rows.len(), 3);
}

#[test]
fn add_stays_reachable() {
    for count in 0..=3 {
        let cfg = default().with_array_len(keys::integrations::LIST, count);
        let adds = section(&cfg, OUTPUTS)
            .rows()
            .filter(|row| {
                matches!(row.control, Control::ActionBtn { id: ActionId::AddIntegration, .. })
            })
            .count();
        assert_eq!(adds, 1, "{count} outputs");
    }
}

#[test]
fn shells_sentence() {
    assert_eq!(
        said(&default(), SHELLS),
        "Noctalia keeps its own colours; DMS keeps its own colours."
    );
    let sync = default().with_flag(keys::caelestia::WALLPAPER_SYNC, true);
    assert_eq!(
        said(&sync, SHELLS),
        "Noctalia keeps its own colours; DMS keeps its own colours; Caelestia wallpapers sync."
    );
    let found = section(&default(), SHELLS);
    assert_eq!(found.groups[0].rows.len(), 3);
    assert_eq!(found.groups[1].rows.len(), 4);
}

#[test]
fn locks_kept_here() {
    let found = section(&default(), LOCKS);
    let targets = found
        .rows()
        .filter(|row| {
            matches!(row.control, Control::ActionBtn { id: ActionId::SetLockScreen(..), .. })
        })
        .count();
    assert_eq!(targets, 5);
    assert_eq!(plain(&found.say.text), "No lock screen is set up. New ones use [0].");
    let on = default()
        .with_text(keys::lock_screens::MODE, keys::lock_screens::FOLLOW)
        .with_flag(keys::lock_screens::HYPRLOCK, true);
    assert_eq!(said(&on, LOCKS), "Hyprlock uses [0].");
    let still = on.with_text(keys::lock_screens::MODE, keys::lock_screens::STATIC);
    assert_eq!(said(&still, LOCKS), "Hyprlock uses [0]: [1].");
}

#[test]
fn plasma_stays_wallpapers() {
    let paths: Vec<_> = sections(&default())
        .iter()
        .flat_map(Section::rows)
        .filter_map(|row| row.control.path().map(str::to_string))
        .collect();
    assert!(!paths.iter().any(|path| path == keys::plasma::LOCK_SCREEN_MODE));
    assert!(!paths.iter().any(|path| path == keys::effects::AUTO_RECOLOR));
}

#[test]
fn asleep_lands_on_cause() {
    for (cfg, query, cause) in [
        (off(), "Source colour index", theme::POLICY),
        (wallpaper(), "Source colour index", theme::ENGINE),
        (shell(NOCTALIA), "Generator", theme::AUTHORITY),
        (wallpaper(), "Theme", theme::POLICY),
        (default(), "Static image", keys::lock_screens::MODE),
    ] {
        let hits = crate::i18n::with_language("en-US", || search_settings(query, source(&cfg)));
        let hit = hits.iter().find(|hit| hit.title == query).expect(query);
        assert_eq!(hit.tab, place::COLOURS, "{query}");
        let found = sections(&cfg);
        let row = found[hit.section].row(hit.row).and_then(|row| row.control.path());
        assert_eq!(row, Some(cause), "{query}");
    }
}

#[test]
fn every_asleep_row_lands() {
    for cfg in [wallpaper(), matugen_on(), fixed(), off(), shell(DMS)] {
        let src = source(&cfg);
        let found = crate::i18n::with_language("en-US", || asleep(&sections(&cfg), &src));
        assert!(!found.is_empty(), "nothing asleep");
        for sleeper in &found {
            let shown = sections(&cfg)
                .iter()
                .flat_map(Section::rows)
                .any(|row| row.control.path() == Some(sleeper.wakes));
            assert!(shown, "{}", sleeper.wakes);
        }
    }
}

#[test]
fn theme_menu_trims() {
    let colours = ["#000000", "#111111", "#222222", "#333333"];
    let cfg = fixed()
        .with_text(theme::STATIC_THEME, "e")
        .with_text(theme::CUSTOM_COLORS, "#1e1e2e, #89b4fa")
        .with_saved_theme("mine")
        .with_palettes(&[
            ("a", "A", colours),
            ("b", "B", colours),
            ("c", "C", colours),
            ("d", "D", colours),
            ("e", "E", colours),
        ]);
    let found = section(&cfg, GENERATOR);
    let row = found.head.iter().find(|row| row.control.path() == Some(theme::STATIC_THEME));
    let row = row.expect("theme row");
    assert_eq!(row.look.heading.as_deref(), Some(tr(THEME_MENU)));
    let more = row.look.more.as_ref().expect("more");
    assert_eq!(more.short, ["a", "b", "c", "e", "mine", "custom"]);
    assert_eq!(more.label, "All 5 presets…");
    let Control::Dropdown { palettes, .. } = &row.control else { panic!("dropdown") };
    let seeds = palettes.iter().find(|(name, _)| name == CUSTOM_THEME).expect("seeds");
    assert_eq!(seeds.1, ["#1e1e2e", "#89b4fa"]);
}

const PLACEHOLDERS: [&str; 7] = ["empty", "blank", "unset", "n/a", "-", "\u{2014}", "..."];

fn shown(row: &Row, cfg: &dyn SettingsSource) -> Option<String> {
    match &row.control {
        Control::Dropdown { current, .. } | Control::Chips { current, .. } => {
            Some(row.control.current_label().map_or_else(|| current.clone(), str::to_string))
        }
        Control::TextField { path, .. } => {
            Some(Some(cfg.text(path)).filter(|text| !text.is_empty()).or(row.look.blank.clone())?)
        }
        Control::ActionBtn { label, .. } => Some(label.clone()),
        Control::ActionChips { items } => {
            Some(items.iter().map(|(_, label)| label.as_str()).collect())
        }
        Control::Number { .. } | Control::Toggle { .. } | Control::ToggleAction { .. } => {
            Some(row.title.clone())
        }
        _ => None,
    }
}

fn real(text: &str) -> bool {
    let word = text.trim().to_lowercase();
    !word.is_empty() && !PLACEHOLDERS.contains(&word.as_str())
}

#[test]
fn no_blank_values() {
    let outputs = || {
        emptied(matugen_on())
            .with_array_len(keys::integrations::LIST, 2)
            .with_text(keys::lock_screens::MODE, keys::lock_screens::STATIC)
    };
    for language in crate::i18n::LANGUAGES {
        crate::i18n::with_language(language.tag, || {
            for cfg in [
                emptied(wallpaper()),
                outputs(),
                emptied(wallpaper().with_text(theme::ENGINE, "wallust")),
                emptied(fixed()),
                off(),
                shell(NOCTALIA),
            ] {
                let found = source(&cfg).place(place::COLOURS).sections;
                for found in &found {
                    for row in found.rows() {
                        let Some(text) = shown(row, &cfg) else { continue };
                        assert!(real(&text), "{} {} {}", language.tag, found.id.key, row.title);
                    }
                    for slot in &found.say.tokens {
                        let Slot::Value { row, blank, .. } = slot else { continue };
                        let blank = blank.as_deref().is_none_or(real);
                        assert!(blank, "{} {} slot {row}", language.tag, found.id.key);
                    }
                }
            }
        });
    }
}

#[test]
fn defaults_say_words() {
    let blank = |found: &Section, path: &str| {
        found
            .rows()
            .find(|row| row.control.path() == Some(path))
            .and_then(|row| row.look.blank.clone())
    };
    let wallust = section(&emptied(wallpaper().with_text(theme::ENGINE, "wallust")), GENERATOR);
    assert_eq!(blank(&wallust, theme::WALLUST_PALETTE).as_deref(), Some("Auto"));
    let fixed = section(&emptied(fixed()), GENERATOR);
    assert_eq!(blank(&fixed, theme::CUSTOM_COLORS).as_deref(), Some("None"));
    let found = section(&emptied(wallpaper()), GENERATOR);
    assert_eq!(blank(&found, matugen::DEFAULT_CONFIG).as_deref(), Some(DEFAULT_MATUGEN_CONFIG));
    let command = schema::text_default(keys::system::EXTERNAL_MATUGEN_COMMAND);
    assert_eq!(blank(&found, keys::system::EXTERNAL_MATUGEN_COMMAND).as_deref(), command);
}

fn emptied(cfg: FakeSettingsSource) -> FakeSettingsSource {
    cfg.with_text(theme::WALLUST_PALETTE, "")
        .with_text(theme::CUSTOM_COLORS, "")
        .with_text(matugen::DEFAULT_CONFIG, "")
        .with_text(keys::system::EXTERNAL_MATUGEN_COMMAND, "")
}

#[test]
fn locale_keys_complete() {
    for language in crate::i18n::LANGUAGES {
        crate::i18n::with_language(language.tag, || {
            for cfg in [matugen_on(), fixed(), off(), shell(NOCTALIA)] {
                for section in source(&cfg).place(place::COLOURS).sections {
                    assert!(!section.title.is_empty(), "{}", language.tag);
                    assert!(drawn(&section), "{} {}", language.tag, section.id.key);
                    assert!(!section.say.text.is_empty(), "{} {}", language.tag, section.id.key);
                }
            }
        });
    }
}
