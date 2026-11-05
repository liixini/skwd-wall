use skwd_palette::studio::Tuning;

use super::*;
use crate::domain::studio::Mode;
use crate::frontend::colour_studio::controls::{Hit, StudioMsg};
use crate::frontend::colour_studio::tokens::INTRO_END;

const TICK: f32 = 1.0 / 60.0;

pub fn wallpaper(key: &str) -> Wallpaper {
    Wallpaper {
        key: key.into(),
        label: "Red ribbon".into(),
        image: iced::widget::image::Handle::from_path("/w/red.webp"),
        current: true,
        aspect: None,
    }
}

pub fn studio(source: &str) -> ColourStudio {
    let model = Studio::new(Source::parse(source), Mode::Dark, Tuning::default());
    let mut studio = ColourStudio::new(model, wallpaper("red"), Vec::new());
    studio.sync();
    studio
}

fn idle(studio: &mut ColourStudio) {
    let mut left = INTRO_END * 2.0;
    while studio.animating() && left > 0.0 {
        studio.tick(TICK);
        left -= TICK;
    }
    assert!(!studio.animating());
}

fn settle(studio: &mut ColourStudio) {
    let mut left = INTRO_END * 2.0;
    while left > 0.0 {
        studio.tick(TICK);
        left -= TICK;
    }
}

#[test]
fn idle_after_intro() {
    let mut studio = studio("theme:nord");
    assert!(studio.animating());
    assert!(studio.intro_age().is_some());
    settle(&mut studio);
    assert!(!studio.animating());
    assert!(studio.intro_age().is_none());
}

#[test]
fn first_copy_unswapped() {
    let studio = studio("theme:nord");
    let view = &studio.swaps.values[Row::View.index()];
    assert!(view.outgoing(studio.now).is_none());
    assert!(!view.now.is_empty());
}

#[test]
fn theme_pick_crossfades() {
    let mut studio = studio("theme:nord");
    settle(&mut studio);
    studio.update(&StudioMsg::Pick("theme:graphite".into()));
    assert!(studio.animating());
    assert!(studio.scenes.previous(studio.now).is_some());
    assert!(studio.panes[Group::Themes.index()].target() > 0.0);
}

#[test]
fn dots_follow_available_palettes() {
    let studio = studio("theme:nord");
    assert!(studio.dots_for(&Source::Theme("nord".into())).is_some());
    assert!(studio.dots_for(&Source::Material("content".into())).is_none());
}

#[test]
fn changes_flash_the_popover() {
    let mut studio = studio("theme:nord");
    settle(&mut studio);
    assert!(!studio.pop_open());
    studio.update(&StudioMsg::Step(Row::Warmth, 1, false));
    assert!(studio.pop_open());
    idle(&mut studio);
    assert!(!studio.pop_open());
}

#[test]
fn hover_holds_the_popover() {
    let mut studio = studio("theme:nord");
    settle(&mut studio);
    studio.update(&StudioMsg::Hover(Some(Hit::Tab(Row::Colour))));
    assert_eq!(studio.row, Row::Colour);
    settle(&mut studio);
    assert!(studio.pop_open());
    studio.update(&StudioMsg::Hover(None));
    idle(&mut studio);
    assert!(!studio.pop_open());
}

#[test]
fn english_tabs_match_design() {
    if std::env::var_os("SKWD_STUDIO_FONT_TEST").is_none() {
        let root = tempfile::tempdir().expect("font fixture");
        let config = root.path().join("fonts.conf");
        std::fs::write(&config, "<fontconfig></fontconfig>").expect("empty font config");
        let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args([
                "--exact",
                "frontend::colour_studio::state::tests::english_tabs_match_design",
                "--nocapture",
            ])
            .env("SKWD_STUDIO_FONT_TEST", "1")
            .env("SKWD_WALL_LOCALE", "en-US")
            .env("FONTCONFIG_FILE", config)
            .output()
            .expect("isolated studio test");
        assert!(
            output.status.success(),
            "{} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    iced_wgpu::graphics::text::font_system()
        .write()
        .expect("font system")
        .load_font(crate::shell::ui_font_bytes(crate::i18n::Script::Latin).into());
    let studio = studio("theme:nord");
    assert_eq!(
        studio.tabs,
        layout::tabs([0.0; Row::ALL.len()], crate::frontend::colour_studio::draw::header_limit())
    );
    let compare = studio.tabs[Row::Compare.index()];
    let actions = crate::frontend::colour_studio::draw::actions(&studio);
    let first = actions.first().expect("actions");
    assert!(first.plate.left >= compare.x + compare.width);
}

#[test]
fn view_change_ends_undo() {
    let mut studio = studio("theme:nord");
    studio.mark_applied();
    studio.update(&StudioMsg::Step(Row::View, 1, false));
    assert!(!studio.undoable());
}

#[test]
fn edits_end_undo() {
    let mut studio = studio("theme:nord");
    studio.mark_applied();
    assert!(studio.undoable());
    studio.update(&StudioMsg::Step(Row::Warmth, 1, false));
    assert!(!studio.undoable());
}

#[test]
fn equal_palette_needs_apply() {
    let mut studio = studio("tonal");
    let palette = skwd_palette::studio::from_theme(&skwd_palette::preset("nord").expect("nord"));
    let variant = |key: &str| crate::domain::studio::Variant {
        key: key.into(),
        dark: palette,
        light: palette,
        shape: skwd_palette::studio::Shape::Plain,
    };
    studio.model.snapshot = Some(crate::domain::studio::Snapshot {
        methods: vec![variant("tonal"), variant("pigment")],
        ..Default::default()
    });
    studio.sync();
    studio.mark_applied();
    assert!(studio.undoable());
    studio.update(&StudioMsg::Pick("pigment".into()));
    assert_eq!(studio.palette(), Some(palette));
    assert!(!studio.undoable());
}

fn pick_theme(studio: &mut ColourStudio, index: usize) {
    studio.model.source = Source::Theme(skwd_palette::PRESETS[index].0.into());
    studio.sync();
}

#[test]
fn fades_skip_queued_middles() {
    let mut studio = studio("theme:nord");
    settle(&mut studio);
    pick_theme(&mut studio, 1);
    let second = studio.scenes.shown().cloned().expect("second");
    pick_theme(&mut studio, 2);
    pick_theme(&mut studio, 3);
    assert_eq!(studio.scenes.shown().map(|scene| scene.palette), Some(second.palette));
    settle(&mut studio);
    let last = studio.palette().expect("last");
    assert_eq!(studio.scenes.shown().map(|scene| scene.palette), Some(last));
    assert!(studio.scenes.previous(studio.now).is_none());
}

#[test]
fn live_colours_start_intro() {
    let model = Studio::new(Source::parse("tonal"), Mode::Dark, Tuning::default());
    let mut studio = ColourStudio::new(model, wallpaper("red"), Vec::new());
    studio.model.live =
        Some(skwd_palette::studio::from_theme(&skwd_palette::preset("nord").expect("nord")));
    studio.sync();
    assert!(studio.palette().is_none());
    assert!(studio.intro_age().is_some());
}
