use skwd_palette::studio::Area;

use super::*;
use crate::domain::studio::Pixels;
use crate::frontend::colour_studio::state::tests::studio;
use crate::frontend::colour_studio::state::{Decoded, Load};

fn pixels() -> Decoded {
    let (width, height) = (32, 20);
    let mut rgba = Vec::with_capacity(width * height * 4);
    for index in 0..width * height {
        let red = if index % width < width / 2 { 230 } else { 20 };
        rgba.extend_from_slice(&[red, 60, 90, 255]);
    }
    let grey = iced::widget::image::Handle::from_rgba(1, 1, vec![0, 0, 0, 255]);
    Decoded { pixels: Pixels { rgba, width, height, aspect: 1.6 }, grey }
}

fn shown(left: f32, top: f32) -> Area {
    Area { left, top, right: left, bottom: top }
}

#[test]
fn snaps_to_neutral() {
    assert_eq!(snap(Knob::Warmth, 0.02), 0.0);
    assert_eq!(snap(Knob::Warmth, 0.37), 0.37);
    assert_eq!(snap(Knob::Colour, 9.0), 1.6);
}

#[test]
fn keys_stop_at_neutral() {
    let mut studio = studio("theme:nord");
    studio.model.tuning.warmth = 0.03;
    studio.update(&StudioMsg::Step(Row::Warmth, -1, false));
    assert_eq!(studio.model.tuning.warmth, 0.0);
    studio.update(&StudioMsg::Step(Row::Warmth, -1, false));
    assert_eq!(studio.model.tuning.warmth, -0.05);
    studio.update(&StudioMsg::Step(Row::Warmth, 1, true));
    assert_eq!(studio.model.tuning.warmth, -0.04);
}

#[test]
fn wheel_steps_percent() {
    let mut studio = studio("theme:nord");
    studio.update(&StudioMsg::Notch(Knob::Contrast, 3));
    assert_eq!(studio.model.tuning.contrast, 0.03);
    assert!(studio.quiet());
    studio.update(&StudioMsg::Reset(Knob::Contrast));
    assert_eq!(studio.model.tuning.contrast, 0.0);
}

#[test]
fn rows_wrap_around() {
    assert_eq!(Row::View.cycled(-1), Row::Compare);
    assert_eq!(Row::Compare.cycled(1), Row::View);
}

#[test]
fn options_pick_directly() {
    let mut studio = studio("theme:nord");
    studio.update(&StudioMsg::Choose(Row::View, 2));
    assert_eq!(studio.view, View::Chat);
    studio.update(&StudioMsg::Choose(Row::FineTune, 0));
    assert_eq!(studio.model.role, TUNABLE[0]);
    assert_eq!(studio.row, Row::FineTune);
}

#[test]
fn shade_targets_role() {
    let mut studio = studio("theme:nord");
    studio.update(&StudioMsg::Step(Row::FineTune, 1, false));
    studio.update(&StudioMsg::Shade(2));
    assert_eq!(studio.model.tuning.shades[studio.model.role.index()], 2);
    assert_eq!(studio.row, Row::Shade);
}

#[test]
fn area_starts_picking() {
    let mut studio = studio("theme:nord");
    studio.update(&StudioMsg::Pick(area().key()));
    assert!(studio.picking.is_some());
    studio.update(&StudioMsg::CancelPick);
    assert!(studio.picking.is_none());
    assert_eq!(studio.model.source, Source::Theme("nord".into()));
}

#[test]
fn release_commits_the_box() {
    let mut studio = studio("theme:nord");
    studio.set_decoded(Load::Ready(pixels()));
    studio.update(&StudioMsg::StartPick);
    studio.update(&StudioMsg::Drag(shown(0.1, 0.1), Phase::Down));
    assert_eq!(studio.model.source, area());
    studio.update(&StudioMsg::Drag(shown(0.4, 0.6), Phase::Drag));
    assert!(studio.model.area.is_some());
    let effect = studio.update(&StudioMsg::Drag(shown(0.4, 0.6), Phase::Up));
    assert!(matches!(effect, Effect::Area(_)));
    assert!(studio.picking.is_none());
    assert!(studio.saved_area().is_some());
    assert!(studio.palette().is_some());
}

#[test]
fn tiny_boxes_are_dropped() {
    let mut studio = studio("theme:nord");
    studio.set_decoded(Load::Ready(pixels()));
    studio.update(&StudioMsg::StartPick);
    studio.update(&StudioMsg::Drag(shown(0.5, 0.5), Phase::Down));
    studio.update(&StudioMsg::Drag(shown(0.51, 0.51), Phase::Drag));
    let effect = studio.update(&StudioMsg::Drag(shown(0.51, 0.51), Phase::Up));
    assert_eq!(effect, Effect::None);
    assert!(studio.picking.is_some());
    assert_eq!(studio.model.source, Source::Theme("nord".into()));
}

#[test]
fn clear_area_goes_back() {
    let mut studio = studio("theme:nord");
    studio.set_decoded(Load::Ready(pixels()));
    studio.update(&StudioMsg::StartPick);
    studio.update(&StudioMsg::Drag(shown(0.1, 0.1), Phase::Down));
    studio.update(&StudioMsg::Drag(shown(0.5, 0.5), Phase::Drag));
    studio.update(&StudioMsg::Drag(shown(0.5, 0.5), Phase::Up));
    studio.update(&StudioMsg::Pick(area().key()));
    assert!(studio.picking.is_some());
    studio.update(&StudioMsg::ClearArea);
    assert!(studio.saved_area().is_none());
    assert_eq!(studio.model.source, Source::Method("tonal".into()));
}

#[test]
fn save_uses_the_placeholder() {
    let mut studio = studio("theme:nord");
    studio.update(&StudioMsg::SaveAs);
    assert!(studio.naming.is_some());
    let effect = studio.update(&StudioMsg::Save);
    assert_eq!(effect, Effect::Persist);
    assert_eq!(studio.saved.len(), 1);
    assert_eq!(studio.saved[0].name, "Red ribbon, nord");
}

#[test]
fn typed_names_win() {
    let mut studio = studio("theme:nord");
    studio.update(&StudioMsg::SaveAs);
    studio.update(&StudioMsg::Name(Edit::Insert("Dusk".into())));
    studio.update(&StudioMsg::Name(Edit::Insert("x".into())));
    studio.update(&StudioMsg::Name(Edit::Erase));
    studio.update(&StudioMsg::Save);
    assert_eq!(studio.saved[0].name, "Dusk");
}

#[test]
fn page_keys_walk_order() {
    let mut studio = studio("theme:nord");
    studio.update(&StudioMsg::SchemeStep(1));
    assert_eq!(studio.model.source, Source::Theme("dracula".into()));
    studio.update(&StudioMsg::SchemeStep(-1));
    assert_eq!(studio.model.source, Source::Theme("nord".into()));
    assert_eq!(studio.dir, -1.0);
}

#[test]
fn mode_switches_family_twin() {
    let mut studio = studio("theme:catppuccin-frappe");
    studio.update(&StudioMsg::Mode(crate::domain::studio::ModeChoice::Light));
    assert_eq!(studio.model.source, Source::Theme("catppuccin-latte".into()));
    assert_eq!(studio.dir, 1.0);
}

#[test]
fn time_picks_twin() {
    use crate::domain::studio::{Mode, ModeChoice};

    let mut studio = studio("theme:catppuccin-frappe");
    studio.set_time_mode(Mode::Light);
    studio.update(&StudioMsg::Mode(ModeChoice::ByTime));
    assert_eq!(studio.model.mode_choice, ModeChoice::ByTime);
    assert_eq!(studio.model.mode, Mode::Light);
    assert_eq!(studio.model.source, Source::Theme("catppuccin-latte".into()));
    studio.set_time_mode(Mode::Dark);
    assert_eq!(studio.model.mode, Mode::Dark);
    assert_eq!(studio.model.source, Source::Theme("catppuccin-frappe".into()));
    studio.update(&StudioMsg::Mode(ModeChoice::Light));
    studio.set_time_mode(Mode::Dark);
    assert_eq!(studio.model.mode, Mode::Light);
}

#[test]
fn presses_skip_intro() {
    let mut studio = studio("theme:nord");
    studio.update(&StudioMsg::Hover(None));
    assert!(studio.intro);
    studio.update(&StudioMsg::SkipIntro);
    assert!(!studio.intro);
}

#[test]
fn compare_remounts_tags() {
    let mut studio = studio("theme:nord");
    studio.update(&StudioMsg::Compare);
    assert!(!studio.model.compare);
    studio.tick(1.0);
    studio.update(&StudioMsg::Compare);
    assert!(studio.model.compare);
    assert_eq!(studio.mounts.tags, studio.now);
}

#[test]
fn pane_scroll_stays_put() {
    let mut studio = studio("theme:nord");
    studio.update(&StudioMsg::Scroll(Group::Themes, 200.0));
    assert_eq!(studio.panes[Group::Themes.index()].target(), 200.0);
}

#[test]
fn stale_area_reply_ignored() {
    let mut studio = studio("theme:nord");
    studio.set_decoded(Load::Ready(pixels()));
    let first = Area { left: 0.1, top: 0.1, right: 0.4, bottom: 0.5 };
    let second = Area { left: 0.5, top: 0.2, right: 0.9, bottom: 0.8 };
    studio.set_saved_area(Some(second));
    let reply = crate::domain::studio::Snapshot {
        methods: vec![crate::domain::studio::Variant {
            key: skwd_palette::studio::AREA_KEY.into(),
            dark: skwd_palette::studio::from_theme(&skwd_palette::preset("nord").expect("nord")),
            light: skwd_palette::studio::from_theme(&skwd_palette::preset("nord").expect("nord")),
            shape: skwd_palette::studio::Shape::Plain,
        }],
        ..Default::default()
    };
    studio.adopt_area(first, &reply);
    studio.update(&StudioMsg::Pick(area().key()));
    let shown = studio.palette().expect("area palette");
    assert_ne!(shown.base, reply.methods[0].dark.base);
}

fn area_reply() -> crate::domain::studio::Snapshot {
    let nord = skwd_palette::studio::from_theme(&skwd_palette::preset("nord").expect("nord"));
    crate::domain::studio::Snapshot {
        methods: vec![crate::domain::studio::Variant {
            key: skwd_palette::studio::AREA_KEY.into(),
            dark: nord,
            light: nord,
            shape: skwd_palette::studio::Shape::Plain,
        }],
        ..Default::default()
    }
}

#[test]
fn late_pixels_fill_patches() {
    let mut studio = studio("theme:nord");
    let box_area = Area { left: 0.1, top: 0.1, right: 0.4, bottom: 0.5 };
    studio.set_saved_area(Some(box_area));
    studio.adopt_area(box_area, &area_reply());
    assert!(studio.model.area.as_ref().is_some_and(|picked| picked.patches.is_empty()));
    studio.set_decoded(Load::Ready(pixels()));
    assert!(studio.model.area.as_ref().is_some_and(|picked| !picked.patches.is_empty()));
}

#[test]
fn decode_fixes_scene_aspect() {
    let mut studio = studio("theme:nord");
    studio.set_decoded(Load::Ready(pixels()));
    assert_eq!(studio.scenes.shown().map(|scene| scene.aspect), Some(1.6));
}
