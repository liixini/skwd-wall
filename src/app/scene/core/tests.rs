#![cfg(test)]

use std::time::{Duration, Instant};

use crate::contracts::preview::UploadQueue;
use crate::contracts::rendering::CHAMFER;
use crate::domain::library::catalog::{Catalog, Wallpaper, WallpaperKind};
use crate::frontend::animation::{MotionProfile, MotionTier, Spring, Tween};
use crate::frontend::scene::layout::{
    CardCorners, ExtraParams, GridLayout, GridParams, HexParams, Mode, SliceParams, StageParams,
};
use crate::frontend::scene::{InstanceRaw, RenderSnapshot, cell_key, sandy};
use crate::frontend::theme::Palette;
use crate::infrastructure::preview::DecodePool;
use crate::rendering::scene::atlas::AtlasMap;

use super::super::widget::{pointer_motion_changed, shimmer_eligible, sync_pointer_gate};
use super::layout_helpers::{
    NEAR_MIN_DISPLAY_W, center_layout, near_qualifies, prefetch_reach, slice_scroll_steps,
};
use super::model::{RebuildCtx, SceneCore};

#[test]
fn sandy_out_capture() {
    let mut scene = test_scene(Mode::Sandy);
    scene.set_current(1, 10);
    assert!(scene.preview_state.sandy_out_pending);
    scene.preview_state.sandy_out_pending = false;
    scene.sandy.prog = Tween::for_duration_ms(0.5, 1250.0);
    scene.sandy.prog.retarget(1.0);
    scene.set_current(2, 10);
    assert!(!scene.preview_state.sandy_out_pending);
}

#[test]
fn center_layout_clamps() {
    assert_eq!(center_layout(1000.0, 0.0, false), (500.0, 1000.0));
    assert_eq!(center_layout(1000.0, 400.0, false), (700.0, 600.0));
    assert_eq!(center_layout(1000.0, 900.0, false), (800.0, 400.0));
    assert_eq!(center_layout(1000.0, -50.0, false), (500.0, 1000.0));
    assert_eq!(center_layout(1000.0, 400.0, true), (300.0, 600.0));
    assert_eq!(center_layout(1000.0, 900.0, true), (200.0, 400.0));
    assert_eq!(center_layout(1000.0, 0.0, true), (500.0, 1000.0));
}

#[test]
fn shimmer_eligibility() {
    assert!(shimmer_eligible(0, Mode::Grid, true, false, false));
    assert!(!shimmer_eligible(0, Mode::Grid, true, true, false));
    assert!(!shimmer_eligible(0, Mode::Grid, true, false, true));
    assert!(!shimmer_eligible(0, Mode::Grid, false, false, false));
    assert!(!shimmer_eligible(1, Mode::Grid, true, false, false));
    assert!(!shimmer_eligible(0, Mode::Slices, true, false, false));
}

#[test]
fn near_vs_far() {
    assert!(near_qualifies(false, false, 235.0, 132.0));
    assert!(near_qualifies(false, false, 300.0, 169.0));
    assert!(near_qualifies(false, false, 900.0, 506.0));
    assert!(!near_qualifies(false, false, 120.0, 68.0));
    assert!(!near_qualifies(false, false, NEAR_MIN_DISPLAY_W - 1.0, 45.0));
    assert!(near_qualifies(false, false, 96.0, 180.0));
    assert!(near_qualifies(true, false, 80.0, 45.0));
    assert!(near_qualifies(true, true, 80.0, 45.0));
    assert!(!near_qualifies(false, true, 900.0, 506.0));
}

#[test]
fn prefetch_reach_budget() {
    let cap = 64;
    let (far, near) = prefetch_reach(36, cap);
    assert_eq!(far, 36);
    assert_eq!(near, 7, "(cap - span) / 4");
    assert_eq!(prefetch_reach(12, cap).1, 13);
    assert_eq!(prefetch_reach(64, cap).1, 0);
    assert_eq!(prefetch_reach(200, cap).1, 0);
    assert_eq!(prefetch_reach(0, cap).0, 1);
}

fn test_scene(mode: Mode) -> SceneCore {
    let sp = SliceParams {
        offset_x: 0.0,
        offset_y: 0.0,
        slice_w: 80.0,
        expanded_w: 200.0,
        slice_h: 120.0,
        spacing: 8.0,
        skew: 0.0,
        edge_tilt: 0.0,
        visible_count: 5,
        corners: CardCorners::round(8.0),
        wobble: false,
        wobble_strength: 1.0,
        parallax: false,
        shadows: true,
    };
    let gp = base_gp();
    let hp = HexParams {
        r: 40.0,
        rows: 3,
        cols: 5,
        curve: crate::frontend::scene::layout::HexCurve::Flat,
        curve_strength: 0.0,
        ..HexParams::default()
    };
    SceneCore::new(mode, sp, gp, hp, &test_extra())
}

#[test]
fn layout_crossfade_vs_morph() {
    let mut scene = test_scene(Mode::Grid);
    scene.render = std::sync::Arc::new(RenderSnapshot {
        renderer: std::sync::Arc::new(crate::contracts::rendering::RendererSnapshot {
            instances: vec![InstanceRaw::default()],
            ..crate::contracts::rendering::RendererSnapshot::default()
        }),
        ..RenderSnapshot::default()
    });

    let mut resized = scene.gp_target;
    resized.thumb_w += 40.0;
    let xp = scene.xp_target;
    scene.set_params(scene.sp_target, resized, scene.hp_target, &xp, true);
    assert!(!scene.layout_transition_active());

    let mut topology = resized;
    topology.layout = GridLayout::Editorial;
    scene.set_params(scene.sp_target, topology, scene.hp_target, &xp, true);
    assert!(scene.layout_transition_active());
}

fn test_extra() -> ExtraParams {
    ExtraParams {
        depth: crate::frontend::scene::layout::DepthParams::default(),
        collection: crate::frontend::scene::layout::CollectionParams::default(),
        sandy: sandy::SandyParams {
            offset_x: 0.0,
            offset_y: 0.0,
            center_h: 440.0,
            slice_w: 96.0,
            slice_h: 180.0,
            spacing: 26.0,
            duration_ms: 1250.0,
            blend_ms: 900.0,
            strands: 22.0,
            twist: 1.0,
            orbit: 1.0,
            turbulence: 1.0,
            waist: 1.0,
            front: 0.65,
            fan: 0.6,
            arc: 1.0,
            swap_loop: false,
            swap_style: 0.0,
            skew: 12.0,
            corners: CardCorners::default(),
            edge_speed: 14.0,
            ring_size: 1.0,
            ring_spin: 1.0,
            ring_wave: 1.0,
            ring_soft: 1.0,
            ring_blend: 1.0,
            ring_hold: 0.2,
            grain: 3.0,
            video_out_live: true,
        },
        hand: crate::frontend::scene::hand::HandParams::default(),
    }
}

#[test]
fn slice_selection_springs() {
    let mut scene = test_scene(Mode::Slices);
    assert!(scene.card.selection.is_empty());
    scene.set_current(3, 10);
    assert_eq!(scene.current, 3);
    let old = scene.card.selection.get(&0).expect("old slice spring");
    let new = scene.card.selection.get(&3).expect("new slice spring");
    assert_eq!(old.target, 0.0);
    assert_eq!(new.target, 1.0);
    assert!(new.x < 0.5);
    assert!(old.x > 0.5);
}

#[test]
fn grid_no_springs() {
    let mut scene = test_scene(Mode::Grid);
    scene.set_current(2, 10);
    assert!(scene.card.selection.is_empty());
}

fn flip_seed(count: usize, base: u32, roll: f32) -> Vec<(InstanceRaw, u32, f32)> {
    (0..count)
        .map(|idx| {
            let inst = InstanceRaw {
                rect: [idx as f32 * 90.0, 120.0, 40.0, 40.0],
                params: [0.0, 0.0, 1.0, 0.0],
                ..Default::default()
            };
            (inst, base + idx as u32, roll)
        })
        .collect()
}

#[test]
fn reel_ease_monotone() {
    assert_eq!(crate::frontend::animation::smoothstep(-0.4), 0.0);
    assert_eq!(crate::frontend::animation::smoothstep(0.0), 0.0);
    assert_eq!(crate::frontend::animation::smoothstep(1.0), 1.0);
    assert_eq!(crate::frontend::animation::smoothstep(2.0), 1.0);
    let mut last = 0.0;
    for i in 0..=20 {
        let val = crate::frontend::animation::smoothstep(i as f32 / 20.0);
        assert!(val >= last);
        last = val;
    }
}

#[test]
fn one_ghost_per_cell() {
    let mut scene = test_scene(Mode::Grid);
    scene.card.filter_cache = flip_seed(8, 0, 1.0);
    scene.filter_storm(None);
    assert_eq!(scene.filter_flip_count(), 8);
    assert!(scene.filter_flip_running());
    scene.card.filter_cache = flip_seed(8, 100, 0.4);
    scene.filter_storm(None);
    assert_eq!(scene.filter_flip_count(), 8);
    let mut moved = flip_seed(3, 200, 1.0);
    for (inst, _, _) in &mut moved {
        inst.rect[0] += 900.0;
    }
    scene.card.filter_cache = moved;
    scene.filter_storm(None);
    assert_eq!(scene.filter_flip_count(), 11);
    for (_, _, t) in &scene.card.filter_old {
        assert!(*t <= 0.0);
    }
}

#[test]
fn same_thumb_no_roll() {
    let mut scene = test_scene(Mode::Grid);
    scene.card.filter_cache = flip_seed(2, 0, 1.0);
    scene.filter_storm(None);
    assert_eq!(scene.flip_roll_for(0, 0.0, 120.0), 1.0);
    assert!(scene.card.filter_old.iter().any(|(_, seed, t)| *seed == 0 && *t >= 1.0));
    assert!(scene.flip_roll_for(99, 90.0, 120.0) < 1.0);
}

#[test]
fn mid_roll_return() {
    let mut scene = test_scene(Mode::Grid);
    scene.card.filter_cache = flip_seed(2, 0, 1.0);
    scene.filter_storm(None);
    scene.card.filter_old[0].2 = 0.3;
    let roll = scene.flip_roll_for(0, 0.0, 120.0);
    assert!((roll - crate::frontend::animation::smoothstep(0.3)).abs() < 1e-6);
    assert!(scene.card.filter_old[0].2 < 1.0);
}

#[test]
fn empty_cell_wave() {
    let mut scene = test_scene(Mode::Grid);
    scene.card.filter_cache = flip_seed(3, 0, 1.0);
    scene.filter_storm(None);
    let roll = scene.flip_roll_for(50, 700.0, 120.0);
    assert!(roll < 1.0);
    assert!(scene.card.filter_in.contains_key(&cell_key(700.0, 120.0)));
    assert!(scene.filter_flip_running());
    scene.card.filter_wave = 10.0;
    scene.card.filter_in.clear();
    scene.card.filter_old.clear();
    scene.card.filter_cell.clear();
    assert_eq!(scene.flip_roll_for(51, 900.0, 500.0), 1.0);
}

#[test]
fn open_fade_cycle() {
    let mut scene = test_scene(Mode::Grid);
    assert!(!scene.motion.entrance.settled());
    assert!(scene.open_fade() < 1.0);
    let mid = scene.motion.entrance.x;
    scene.set_open_fade_ms(1000.0);
    assert!((scene.motion.entrance.x - mid).abs() < 1e-6);
    assert_eq!(scene.motion.entrance.target, 1.0);
    scene.motion.entrance.snap(1.0);
    assert!((scene.open_fade() - 1.0).abs() < 1e-6);
    scene.set_visible(false);
    scene.motion.visibility.snap(0.0);
    scene.set_visible(true);
    assert!(!scene.open_fade_settled());
    assert!(scene.open_fade() < 1.0);
}

#[test]
fn warm_reveal_fade() {
    let mut scene = test_scene(Mode::Grid);
    scene.motion.entrance.snap(1.0);
    assert!(scene.open_fade_settled());
    scene.begin_open_fade();
    assert!(scene.open_fade() < 0.05);
    scene.start_pending_entrance();
    assert!(!scene.open_fade_settled());
}

#[test]
fn open_fade_floor() {
    let mut scene = test_scene(Mode::Grid);
    scene.set_open_fade_from(0.3);
    assert!(scene.motion.entrance.x >= 0.3);
    scene.motion.entrance.snap(1.0);
    scene.begin_open_fade();
    assert!((scene.open_fade() - 0.3).abs() < 1e-6);
    scene.start_pending_entrance();
    assert_eq!(scene.motion.entrance.target, 1.0);
}

#[test]
fn zero_duration_fade() {
    let mut scene = test_scene(Mode::Grid);
    scene.set_open_fade_ms(0.0);
    scene.begin_open_fade();
    assert!(scene.open_fade_settled());
    assert_eq!(scene.open_fade(), 1.0);

    scene.set_visible(false);
    assert!(scene.hidden());
    scene.set_visible(true);
    assert!(scene.open_fade_settled());
    assert_eq!(scene.open_fade(), 1.0);
}

#[test]
fn hex_curve_falloff() {
    let mut scene = test_scene(Mode::Hex);
    let flat_edge = scene.hex_scale(0, 2540.0, 0.0, 0.0, 2560.0);
    let flat_past = scene.hex_scale(1, 2560.0 + scene.hp.step_x() * 0.5, 0.0, 0.0, 2560.0);
    assert!((flat_edge - 1.0).abs() < 0.01);
    assert!((flat_past - 1.0).abs() < 0.01);

    scene.hp.curve = crate::frontend::scene::layout::HexCurve::Arc;
    let center = scene.hex_scale(0, 1280.0, 0.0, 0.0, 2560.0);
    assert!((center - 1.0).abs() < 0.01);
    let mid = scene.hex_scale(2, 2000.0, 0.0, 0.0, 2560.0);
    assert!(mid > 0.97);
    let edge = scene.hex_scale(3, 2540.0, 0.0, 0.0, 2560.0);
    assert!((0.7..0.9).contains(&edge));
    let step = scene.hp.step_x();
    let past = scene.hex_scale(4, 2560.0 + step * 0.5, 0.0, 0.0, 2560.0);
    assert!(past > 0.05 && past < edge);
    let gone = scene.hex_scale(5, 2560.0 + step * 2.0, 0.0, 0.0, 2560.0);
    assert!(gone < 0.01);
    scene.set_current(6, 40);
    assert!(scene.card.selection.get(&6).is_some_and(|spring| spring.target == 1.0));
    assert!(scene.card.selection.get(&0).is_some_and(|spring| spring.target == 0.0));
}

#[test]
fn filter_reset() {
    for mode in [Mode::Slices, Mode::Hex, Mode::Grid] {
        let mut scene = test_scene(mode);
        scene.set_current(40, 60);
        scene.camera.snap(9999.0);
        assert_eq!(scene.current, 40, "{mode:?}");
        scene.reset_to_start(50);
        assert_eq!(scene.current, 0, "{mode:?}");
        assert_eq!(scene.camera.x, scene.start_camera(), "{mode:?}");
        assert!(scene.hover.is_none(), "{mode:?}");
    }
}

#[test]
fn relayout_clamps() {
    let mut scene = test_scene(Mode::Slices);
    scene.set_current(40, 60);
    scene.relayout(60);
    assert_eq!(scene.current, 40);
    scene.relayout(10);
    assert_eq!(scene.current, 9);
}

#[test]
fn sandy_interrupt_ring() {
    let mut scene = test_scene(Mode::Sandy);
    scene.set_current(1, 10);
    assert_eq!(scene.sandy.storm_to, Some(1));
    scene.sandy.prog = Tween::for_duration_ms(0.5, 1250.0);
    scene.sandy.prog.retarget(1.0);
    scene.set_current(2, 10);
    assert!(scene.sandy.swirl.target > 0.5);
    assert_eq!(scene.sandy.bfrom, Some(1));
    assert_eq!(scene.sandy.ring_to, Some(2));
    assert!((scene.sandy.prog.x - 0.5).abs() < 0.01);

    scene.sandy_prog_tick(0.016);
    assert!((scene.sandy.prog.x - 0.5).abs() < 0.01);

    scene.sandy.swirl_hold = 0.0;
    scene.sandy_swirl_tick(0.016);
    assert!(scene.sandy.swirl.target < 0.5);
    assert!(scene.sandy.prog.settled() && scene.sandy.prog.x == 1.0);
    assert!(scene.sandy.storm_to.is_none());

    let mut tail = test_scene(Mode::Sandy);
    tail.set_current(1, 10);
    tail.sandy.prog = Tween::for_duration_ms(0.95, 1250.0);
    tail.sandy.prog.retarget(1.0);
    tail.set_current(2, 10);
    assert!(tail.sandy.swirl.target > 0.5);
    assert!((tail.sandy.prog.x - 0.95).abs() < 0.01);

    let mut idle = test_scene(Mode::Sandy);
    idle.set_current(1, 10);
    idle.sandy.prog.snap(1.0);
    idle.sandy.swirl = Spring::for_duration_ms(0.0, 550.0);
    idle.set_current(2, 10);
    assert!(idle.sandy.swirl.target < 0.5);
    assert!(idle.sandy.prog.x < 0.5);
}

#[test]
fn ring_sweep_capture() {
    let mut scene = test_scene(Mode::Sandy);
    scene.set_current(1, 20);
    scene.sandy.prog.snap(1.0);
    scene.sandy.swirl = Spring::for_duration_ms(1.0, 550.0);
    scene.sandy.swirl_hold = 0.2;
    scene.set_current(3, 20);
    assert_eq!(scene.sandy.ring_to, Some(3));
    assert_eq!(scene.sandy.bfrom, Some(1));

    scene.set_current(6, 20);
    scene.set_current(9, 20);
    assert_eq!(scene.sandy.ring_to, Some(3));
    assert_eq!(scene.sandy.bfrom, Some(1));

    let mut done = Spring::for_duration_ms(0.95, 400.0);
    done.retarget(1.0);
    scene.sandy.bmix = done;
    scene.sandy_ring_chain_tick();
    assert_eq!(scene.sandy.bfrom, Some(3));
    assert_eq!(scene.sandy.ring_to, Some(9));
    assert_eq!(scene.sandy.bfrom2, Some(1));
    assert!(scene.sandy.bmix.x < 0.5);
}

#[test]
fn held_ring_pick() {
    let mut scene = test_scene(Mode::Sandy);
    scene.set_current(1, 10);
    scene.sandy.prog.snap(1.0);
    scene.sandy.swirl = Spring::for_duration_ms(1.0, 550.0);
    scene.sandy.swirl_hold = 0.2;
    scene.set_current(3, 10);
    assert!(scene.sandy.swirl.target > 0.5);
    assert!(!scene.sandy.prog.settled() || scene.sandy.prog.x >= 0.999);
    assert!((scene.sandy.swirl_hold - scene.xp.sandy.ring_hold).abs() < 1e-6);
}

#[test]
fn sandy_swap_loop() {
    let mut scene = test_scene(Mode::Sandy);
    scene.xp.sandy.swap_loop = true;
    scene.set_current(1, 10);
    scene.sandy.prog = Tween::for_duration_ms(0.5, 1250.0);
    scene.sandy.prog.retarget(1.0);
    scene.set_current(2, 10);
    assert_eq!(scene.sandy.bto, Some(2));
    assert!(scene.sandy.swirl.target < 0.5);
    scene.set_current(3, 10);
    assert!(scene.sandy.swirl.target > 0.5);
}

#[test]
fn ring_fade_chain() {
    let mut scene = test_scene(Mode::Sandy);
    scene.set_current(1, 20);
    scene.sandy.prog.snap(1.0);
    scene.sandy.bfrom = Some(1);
    let mut fade = Spring::for_duration_ms(0.1, 400.0);
    fade.retarget(1.0);
    scene.sandy.bmix = fade;
    scene.set_current(2, 20);
    assert!(scene.sandy.swirl.target > 0.5);
    assert_eq!(scene.sandy.bfrom, Some(1));
    assert!(!scene.sandy.bmix.settled());

    let mut mid = Spring::for_duration_ms(0.4, 400.0);
    mid.retarget(1.0);
    scene.sandy.bmix = mid;
    scene.set_current(6, 20);
    scene.set_current(4, 20);
    assert_eq!(scene.sandy.bfrom, Some(1));
    assert_eq!(scene.sandy.bcut, 1.0);
    assert!((scene.sandy.bmix.x - 0.4).abs() < 0.05);

    let mut done = Spring::for_duration_ms(0.92, 400.0);
    done.retarget(1.0);
    scene.sandy.bmix = done;
    scene.set_current(9, 20);
    assert_eq!(scene.sandy.bfrom2, Some(1));
    assert!((scene.sandy.bcut - 0.92).abs() < 0.05);
    assert_eq!(scene.sandy.bfrom, Some(4));
    assert!(scene.sandy.bmix.x < 0.5);

    scene.sandy.swirl_hold = 0.0;
    scene.sandy_swirl_tick(0.016);
    assert!(scene.sandy.swirl.target < 0.5);
    assert!(scene.sandy.bto.is_none());
}

#[test]
fn ring_keeps_frames() {
    let mut scene = test_scene(Mode::Sandy);
    scene.motion.needs_frame = false;
    scene.motion.entrance.snap(1.0);
    scene.motion.visibility.snap(1.0);
    scene.camera.snap(scene.camera.target);
    assert!(!scene.is_animating());
    scene.sandy.swirl = Spring::for_duration_ms(1.0, 550.0);
    assert!(scene.is_animating());
    scene.sandy.swirl = Spring::for_duration_ms(0.0, 550.0);
    assert!(!scene.is_animating());
}

#[test]
fn hero_conceal_cycle() {
    let mut scene = test_scene(Mode::Sandy);
    assert!(!scene.hero_concealed());
    let now = Instant::now();
    scene.conceal_hero(now + Duration::from_millis(50));
    assert!(scene.hero_concealed());
    assert!(scene.is_animating());
    for _ in 0..60 {
        scene.hero_fade_tick(now, 1.0 / 60.0);
    }
    assert!(scene.sandy.hero_fade.x < 0.05);
    scene.hero_fade_tick(now + Duration::from_millis(60), 1.0 / 60.0);
    assert!(!scene.hero_concealed());
    for _ in 0..120 {
        scene.hero_fade_tick(now + Duration::from_millis(80), 1.0 / 60.0);
    }
    assert!(scene.sandy.hero_fade.settled());
    assert!((scene.sandy.hero_fade.x - 1.0).abs() < 0.01);
    scene.conceal_hero(now + Duration::from_secs(60));
    scene.reveal_hero();
    assert!(!scene.hero_concealed());
}

#[test]
fn conceal_fast_forward() {
    let mut scene = test_scene(Mode::Sandy);
    scene.set_current(3, 10);
    assert!(!scene.sandy.prog.settled());
    let now = Instant::now();
    scene.conceal_hero(now + Duration::from_secs(2));
    assert_eq!(scene.sandy.hero_fade.x, 0.0);
    assert!(scene.sandy.prog.settled());
    assert!(scene.sandy.storm_to.is_none());
    scene.set_current(7, 10);
    assert_eq!(scene.current, 7);
    assert!(scene.sandy.prog.settled());
    assert!(!scene.sandy.pop.settled());
    scene.reveal_hero();
    assert!(!scene.hero_concealed());
    assert!(scene.sandy.prog.settled());
}

#[test]
fn sandy_pop_settles() {
    let mut scene = test_scene(Mode::Sandy);
    assert!(scene.sandy.pop.settled());
    scene.set_current(3, 10);
    assert!(!scene.sandy.pop.settled());
    assert!(scene.is_animating());
    for _ in 0..300 {
        scene.sandy_swirl_tick(1.0 / 60.0);
    }
    assert!(scene.sandy.pop.settled());
    assert!(((std::f32::consts::PI * scene.sandy.pop.x.clamp(0.0, 1.0)).sin()).abs() < 0.05);
    let before = scene.sandy.pop.x;
    scene.set_current(3, 10);
    assert_eq!(scene.sandy.pop.x, before);
}

#[test]
fn sandy_pointer_hover() {
    let mut scene = test_scene(Mode::Sandy);
    scene.viewport = (1000.0, 600.0);
    let y = 600.0 - crate::frontend::scene::sandy::BAR_ZONE - 10.0;
    scene.sandy_pointer(500.0, y, Some(3), 10);
    assert_eq!(scene.current, 3);
    assert!(scene.sandy.cam_free);
    scene.sandy_pointer(503.0, y, Some(4), 10);
    assert_eq!(scene.current, 3);
    scene.sandy_pointer(560.0, y, Some(4), 10);
    assert_eq!(scene.current, 4);
    assert!(scene.sandy.cam_free);
    scene.sandy_pointer(20.0, y, Some(7), 10);
    assert!(scene.sandy.edge_pan < -0.8);
    assert!(scene.sandy.cam_free);
    assert_eq!(scene.current, 4);
    scene.sandy_pointer(500.0, 200.0, None, 10);
    assert_eq!(scene.sandy.edge_pan, 0.0);
    assert!(!scene.sandy.cam_free);
    scene.sandy_pointer(20.0, 595.0, Some(7), 10);
    assert_eq!(scene.sandy.edge_pan, 0.0);
    assert_eq!(scene.current, 4);
    scene.set_current(8, 10);
    assert!(!scene.sandy.cam_free);
}

#[test]
fn pointer_motion_filtered() {
    let mut last = None;
    assert!(pointer_motion_changed(&mut last, 100.0, 100.0));
    assert!(!pointer_motion_changed(&mut last, 101.0, 102.0));
    assert!(!pointer_motion_changed(&mut last, 102.0, 98.0));
    assert!(pointer_motion_changed(&mut last, 102.1, 100.0));
    assert_eq!(last, Some((102.1, 100.0)));
}

#[test]
fn pointer_gate_resets_baseline() {
    let mut last = None;
    let mut enabled = false;
    sync_pointer_gate(&mut last, &mut enabled, true);
    assert!(pointer_motion_changed(&mut last, 100.0, 100.0));
    sync_pointer_gate(&mut last, &mut enabled, false);
    assert_eq!(last, None);
    sync_pointer_gate(&mut last, &mut enabled, true);
    assert!(pointer_motion_changed(&mut last, 100.5, 100.5));
}

#[test]
fn pointer_filter_keeps_sandy_steps() {
    let mut scene = test_scene(Mode::Sandy);
    scene.viewport = (1000.0, 600.0);
    let strip_y = 600.0 - crate::frontend::scene::sandy::BAR_ZONE - 10.0;
    let mut last = None;
    if pointer_motion_changed(&mut last, 500.0, strip_y) {
        scene.sandy_pointer(500.0, strip_y, Some(3), 10);
    }
    assert_eq!(scene.current, 3);
    if pointer_motion_changed(&mut last, 501.0, strip_y) {
        scene.sandy_pointer(501.0, strip_y, Some(4), 10);
    }
    assert_eq!(scene.current, 3);
    if pointer_motion_changed(&mut last, 511.0, strip_y) {
        scene.sandy_pointer(511.0, strip_y, Some(4), 10);
    }
    assert_eq!(scene.current, 4);
    if pointer_motion_changed(&mut last, 20.0, strip_y) {
        scene.sandy_pointer(20.0, strip_y, Some(4), 10);
    }
    assert!(scene.sandy.edge_pan < -0.8);
    if pointer_motion_changed(&mut last, 500.0, 200.0) {
        scene.sandy_pointer(500.0, 200.0, None, 10);
    }
    assert_eq!(scene.sandy.edge_pan, 0.0);
    assert!(!scene.sandy.cam_free);
}

#[test]
fn sandy_strip_requests_near() {
    use std::sync::{Arc, Mutex};

    let mut scene = test_scene(Mode::Sandy);
    scene.viewport = (1600.0, 600.0);
    scene.motion.entrance.snap(1.0);
    scene.motion.visibility.snap(1.0);
    scene.sandy_settle_now();

    let uploads: UploadQueue = Arc::new(Mutex::new(Vec::new()));
    let (tx, _rx) = futures_channel::mpsc::unbounded();
    let pool = DecodePool::start(uploads.clone(), tx, 0);
    let mut store = Catalog::default();
    for index in 0..20 {
        store.items.push(Wallpaper {
            key: format!("wall-{index}"),
            name: format!("wall-{index}"),
            thumb: format!("/tmp/wall-{index}.webp"),
            path: format!("/tmp/wall-{index}.webp"),
            kind: WallpaperKind::Static,
            ..Default::default()
        });
    }
    let filtered: Vec<u32> = (0..20).collect();
    let palette = Palette::default();
    let mut atlas = Some(AtlasMap::new(store.items.len()));

    scene.rebuild(RebuildCtx {
        catalog: &store,
        filtered: &filtered,
        atlas: &mut atlas,
        pool: &pool,
        palette: &palette,
        uploads: &uploads,
        hover_fades: None,
    });

    assert!(scene.render.hits.iter().any(|hit| hit.index == 7));
    assert!(atlas.as_ref().unwrap().near.is_known(7));
}

#[test]
fn sandy_flip_back_panel() {
    use std::sync::{Arc, Mutex};

    let mut scene = test_scene(Mode::Sandy);
    scene.viewport = (1000.0, 600.0);
    let uploads: UploadQueue = Arc::new(Mutex::new(Vec::new()));
    let (tx, _rx) = futures_channel::mpsc::unbounded();
    let pool = DecodePool::start(uploads.clone(), tx, 0);
    let mut store = Catalog::default();
    for name in ["a", "b", "c"] {
        store.items.push(Wallpaper {
            key: name.into(),
            name: name.into(),
            thumb: format!("/tmp/{name}.png"),
            path: format!("/tmp/{name}.png"),
            kind: WallpaperKind::Static,
            ..Default::default()
        });
    }
    let filtered: Vec<u32> = vec![0, 1, 2];
    let pal = Palette::default();
    let mut atlas: Option<AtlasMap> = None;

    scene.set_current(1, 3);
    scene.sandy_settle_now();
    scene.toggle_flip(1);
    scene.motion.entrance.snap(1.0);
    scene.motion.visibility.snap(1.0);
    scene.camera.snap(scene.camera.target);
    scene.tick(
        Instant::now(),
        RebuildCtx {
            catalog: &store,
            filtered: &filtered,
            atlas: &mut atlas,
            pool: &pool,
            palette: &pal,
            uploads: &uploads,
            hover_fades: None,
        },
    );
    assert!(scene.render.back.as_ref().is_some_and(|back| back.coordinated_flip));
    let hero = scene
        .render
        .instances
        .iter()
        .find(|instance| {
            (instance.rect[2] - scene.xp.sandy.center_hw()).abs() < 0.01
                && (instance.rect[3] - scene.xp.sandy.center_hh()).abs() < 0.01
        })
        .expect("Sandy hero instance");
    assert_eq!(hero.radii, [0.0; 4]);
}

#[cfg(feature = "obs-heap")]
#[test]
fn settled_tick_alloc_free() {
    use std::sync::{Arc, Mutex};

    let mut scene = test_scene(Mode::Grid);
    scene.viewport = (1000.0, 600.0);
    let uploads: UploadQueue = Arc::new(Mutex::new(Vec::new()));
    let (tx, _rx) = futures_channel::mpsc::unbounded();
    let pool = DecodePool::start(uploads.clone(), tx, 0);
    let mut store = Catalog::default();
    for name in ["a", "b", "c"] {
        store.items.push(Wallpaper {
            key: name.into(),
            name: name.into(),
            thumb: format!("/tmp/{name}.png"),
            path: format!("/tmp/{name}.png"),
            kind: WallpaperKind::Static,
            ..Default::default()
        });
    }
    let filtered: Vec<u32> = vec![0, 1, 2];
    let pal = Palette::default();
    let mut map = AtlasMap::new(3);
    for idx in 0..3 {
        map.near.acquire(idx);
        map.near.mark_ready(idx);
    }
    let mut atlas = Some(map);
    scene.set_current(1, 3);
    scene.motion.entrance.snap(1.0);
    scene.motion.visibility.snap(1.0);
    scene.camera.snap(scene.camera.target);
    let tick_once = |scene: &mut SceneCore, atlas: &mut Option<AtlasMap>| {
        scene.tick(
            Instant::now(),
            RebuildCtx {
                catalog: &store,
                filtered: &filtered,
                atlas,
                pool: &pool,
                palette: &pal,
                uploads: &uploads,
                hover_fades: None,
            },
        );
    };
    for _ in 0..3 {
        tick_once(&mut scene, &mut atlas);
        scene.camera.snap(scene.camera.target);
    }
    let allocs = crate::infrastructure::observability::allocation::thread_alloc_count();
    let frees = crate::infrastructure::observability::allocation::thread_free_count();
    for _ in 0..256 {
        tick_once(&mut scene, &mut atlas);
    }
    assert_eq!(crate::infrastructure::observability::allocation::thread_alloc_count() - allocs, 0);
    assert_eq!(crate::infrastructure::observability::allocation::thread_free_count() - frees, 0);
}

#[cfg(all(feature = "obs-heap", target_os = "linux"))]
#[test]
#[ignore = "retained manual animated-rebuild allocation/RSS measurement"]
fn animated_rebuild_allocation_measurement() {
    use std::sync::{Arc, Mutex};

    const ITEMS: usize = 120;
    const FRAMES: usize = 240;
    const ACTIONS: [&str; 4] = ["scroll", "transition", "flip", "filter"];
    const MODES: [(Mode, &str); 4] = [
        (Mode::Slices, "slices"),
        (Mode::Grid, "wall"),
        (Mode::Hex, "hex"),
        (Mode::Sandy, "sandy"),
    ];
    let selected = std::env::var("SKWD_SCENE_ALLOC_CASE").ok();
    let mut cases_run = 0usize;

    for (mode, mode_name) in MODES {
        for action in ACTIONS {
            if selected
                .as_deref()
                .is_some_and(|selected| selected.split_once('/') != Some((mode_name, action)))
            {
                continue;
            }
            cases_run += 1;
            let mut scene = test_scene(mode);
            scene.viewport = (1200.0, 700.0);
            let uploads: UploadQueue = Arc::new(Mutex::new(Vec::new()));
            let (tx, _rx) = futures_channel::mpsc::unbounded();
            let pool = DecodePool::start(uploads.clone(), tx, 0);
            let mut store = Catalog::default();
            for index in 0..ITEMS {
                store.items.push(Wallpaper {
                    key: format!("wall-{index}"),
                    name: format!("Wall {index}"),
                    thumb: format!("/tmp/wall-{index}.bc7"),
                    path: format!("/tmp/wall-{index}.png"),
                    kind: WallpaperKind::Static,
                    ..Default::default()
                });
            }
            let filtered = (0..ITEMS as u32).collect::<Vec<_>>();
            let filtered_half = (0..ITEMS as u32).step_by(2).collect::<Vec<_>>();
            let palette = Palette::default();
            let mut map = AtlasMap::new(ITEMS);
            for index in 0..64 {
                map.near.acquire(index);
                map.near.mark_ready(index);
            }
            let mut atlas = Some(map);
            scene.set_current(0, ITEMS);
            scene.motion.entrance.snap(1.0);
            scene.motion.visibility.snap(1.0);
            scene.camera.snap(0.0);
            scene.rebuild(RebuildCtx {
                catalog: &store,
                filtered: &filtered,
                atlas: &mut atlas,
                pool: &pool,
                palette: &palette,
                uploads: &uploads,
                hover_fades: None,
            });

            if action == "transition" {
                scene.begin_transition(0, [0.5, 0.5]);
            } else if action == "flip" {
                scene.toggle_flip(0);
                scene.card.flip.snap(0.5);
                scene.card.det_p = 0.75;
                scene.card.det_target = 1.0;
                scene.card.det_src = [600.0, 350.0, 160.0, 90.0];
            } else if action == "filter" {
                scene.seed_filter_cache(24);
                scene.filter_storm(None);
            }

            scene.rebuild(RebuildCtx {
                catalog: &store,
                filtered: &filtered,
                atlas: &mut atlas,
                pool: &pool,
                palette: &palette,
                uploads: &uploads,
                hover_fades: None,
            });

            // SAFETY: `malloc_trim` accepts any non-negative padding value and does not
            // receive or expose Rust-owned pointers. This Linux-only measurement uses
            // zero solely to discard allocator residue before sampling process memory.
            unsafe { libc::malloc_trim(0) };
            let (rss_before_kib, pss_before_kib) = scene_memory_kib();
            let allocations_before =
                crate::infrastructure::observability::allocation::thread_alloc_count();
            let frees_before =
                crate::infrastructure::observability::allocation::thread_free_count();
            let live_before = crate::infrastructure::observability::allocation::live_bytes();
            let mut rebuild_elapsed = Duration::ZERO;
            let mut order_digest = 0xcbf2_9ce4_8422_2325u64;
            let mut max_instances = 0usize;
            let mut max_hits = 0usize;
            let mut max_chrome = 0usize;

            for frame in 0..FRAMES {
                match action {
                    "scroll" => {
                        scene.camera.snap((frame as f32 * 9.0) % 640.0);
                        if mode == Mode::Sandy {
                            scene.current = (frame / 4) % ITEMS;
                        }
                    }
                    "transition" => {
                        if frame > 0 && frame % 60 == 0 {
                            scene.motion.transition = None;
                            scene.begin_transition((frame / 60) as u32, [0.35, 0.65]);
                        }
                        if let Some(transition) = scene.motion.transition.as_mut() {
                            transition.progress.snap((frame % 60) as f32 / 59.0);
                        }
                    }
                    "flip" => {
                        scene.card.flip.snap((frame % 60) as f32 / 59.0);
                        scene.card.det_p = scene.card.flip.x;
                    }
                    "filter" if frame > 0 && frame % 60 == 0 => {
                        scene.seed_filter_cache(24);
                        scene.filter_storm(None);
                    }
                    "filter" => {
                        scene.card.filter_wave = (frame % 60) as f32 / 35.0;
                    }
                    _ => {}
                }
                let active = if action == "filter" && (frame / 60).is_multiple_of(2) {
                    &filtered_half
                } else {
                    &filtered
                };
                let rebuild_started = Instant::now();
                scene.rebuild(RebuildCtx {
                    catalog: &store,
                    filtered: active,
                    atlas: &mut atlas,
                    pool: &pool,
                    palette: &palette,
                    uploads: &uploads,
                    hover_fades: None,
                });
                rebuild_elapsed += rebuild_started.elapsed();
                max_instances = max_instances.max(scene.render.instances.len());
                max_hits = max_hits.max(scene.render.hits.len());
                max_chrome = max_chrome.max(scene.render.chrome.len());
                scene_allocation_mix_output(&mut order_digest, &scene.render, &palette);
            }

            let allocations =
                crate::infrastructure::observability::allocation::thread_alloc_count()
                    - allocations_before;
            let frees = crate::infrastructure::observability::allocation::thread_free_count()
                - frees_before;
            let live_after = crate::infrastructure::observability::allocation::live_bytes();
            // SAFETY: as above, no Rust pointer or lifetime crosses the C boundary;
            // trimming allocator residue makes the post-workload RSS/PSS sample useful.
            unsafe { libc::malloc_trim(0) };
            let (rss_after_kib, pss_after_kib) = scene_memory_kib();
            let live_delta = live_after as i64 - live_before as i64;
            println!(
                "{}",
                serde_json::json!({
                    "mode": mode_name,
                    "action": action,
                    "frames": FRAMES,
                    "allocations": allocations,
                    "allocationsPerFrame": allocations as f64 / FRAMES as f64,
                    "frees": frees,
                    "liveDeltaBytes": live_delta,
                    "elapsedMs": rebuild_elapsed.as_secs_f64() * 1000.0,
                    "maxInstances": max_instances,
                    "maxHits": max_hits,
                    "maxChrome": max_chrome,
                    "rssBeforeKiB": rss_before_kib,
                    "rssAfterKiB": rss_after_kib,
                    "pssBeforeKiB": pss_before_kib,
                    "pssAfterKiB": pss_after_kib,
                    "orderDigest": format!("{order_digest:016x}"),
                })
            );
            assert!(max_instances > 0 && max_hits > 0);
            assert_eq!(
                order_digest,
                scene_allocation_order_digest(mode_name, action),
                "{mode_name}/{action} ordered instance/hit/chrome output changed"
            );
            assert!(
                allocations <= FRAMES * scene_allocation_budget(mode_name),
                "{mode_name}/{action} exceeded its per-frame allocation budget"
            );
            assert_eq!(
                frees, allocations,
                "{mode_name}/{action} retained whole allocations after warm-up"
            );
            assert!(
                live_delta <= 8 * 1024,
                "{mode_name}/{action} retained more than 8 KiB through reallocations"
            );
            assert!(
                rss_after_kib.saturating_sub(rss_before_kib) <= 1024,
                "{mode_name}/{action} grew RSS by more than 1 MiB"
            );
            assert!(
                pss_after_kib.saturating_sub(pss_before_kib) <= 1024,
                "{mode_name}/{action} grew PSS by more than 1 MiB"
            );
        }
    }
    assert!(cases_run > 0, "SKWD_SCENE_ALLOC_CASE did not name a measured case");
}

#[cfg(all(feature = "obs-heap", target_os = "linux"))]
fn scene_allocation_mix_output(digest: &mut u64, snapshot: &RenderSnapshot, palette: &Palette) {
    fn mix(digest: &mut u64, bytes: &[u8]) {
        for byte in bytes {
            *digest ^= u64::from(*byte);
            *digest = digest.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    mix(digest, &snapshot.instances.len().to_le_bytes());
    mix(digest, bytemuck::cast_slice(&snapshot.instances));
    mix(digest, &snapshot.hits.len().to_le_bytes());
    for hit in &snapshot.hits {
        mix(digest, &hit.index.to_le_bytes());
        for value in [hit.cx, hit.cy, hit.hw, hit.hh, hit.skew] {
            mix(digest, &value.to_bits().to_le_bytes());
        }
        mix(digest, &[hit.hex as u8, hit.triangle_direction]);
        mix(digest, hit.hex_shape.as_key().as_bytes());
    }
    mix(digest, &snapshot.chrome.len().to_le_bytes());
    mix(
        digest,
        &crate::frontend::ui::chrome_signature(&snapshot.chrome, snapshot.back.as_ref(), palette)
            .to_le_bytes(),
    );
}

#[cfg(all(feature = "obs-heap", target_os = "linux"))]
fn scene_allocation_budget(mode: &str) -> usize {
    match mode {
        "slices" => 20,
        "wall" => 22,
        "hex" => 26,
        "sandy" => 21,
        _ => unreachable!("all measured scene modes have an allocation budget"),
    }
}

#[cfg(all(feature = "obs-heap", target_os = "linux"))]
fn scene_allocation_order_digest(mode: &str, action: &str) -> u64 {
    match (mode, action) {
        ("slices", "scroll") => 0xdaa0_206a_3ccb_432b,
        ("slices", "transition") => 0xc740_3d1f_6b13_e9e5,
        ("slices", "flip") => 0xc61a_d6a2_30ea_7f05,
        ("slices", "filter") => 0xeb87_030a_9235_8945,
        ("wall", "scroll") => 0xd0e6_7049_7889_388e,
        ("wall", "transition") => 0xda5e_8dbe_ff90_f6e5,
        ("wall", "flip") => 0xe283_1f7b_4783_1c4d,
        ("wall", "filter") => 0xa81c_f102_73a1_9905,
        ("hex", "scroll") => 0xf079_3774_46fb_0447,
        ("hex", "transition") => 0x5bca_72e3_2d14_a165,
        ("hex", "flip") => 0x295c_b0f4_923e_00a5,
        ("hex", "filter") => 0x9a76_b20d_b489_a235,
        ("sandy", "scroll") => 0xef87_2dd6_79ea_5467,
        ("sandy", "transition") => 0xd86a_87e4_eef6_57c5,
        ("sandy", "flip") => 0x3de9_0be5_f2e1_7c55,
        ("sandy", "filter") => 0xe571_690b_2ce5_5db5,
        _ => unreachable!("all measured scene actions have an ordering digest"),
    }
}

#[cfg(all(feature = "obs-heap", target_os = "linux"))]
fn scene_memory_kib() -> (u64, u64) {
    let rollup = std::fs::read_to_string("/proc/self/smaps_rollup").unwrap();
    let value = |name: &str| {
        rollup
            .lines()
            .find_map(|line| {
                line.strip_prefix(name)
                    .and_then(|tail| tail.split_whitespace().next())
                    .and_then(|value| value.parse().ok())
            })
            .unwrap_or(0)
    };
    (value("Rss:"), value("Pss:"))
}

#[test]
fn sandy_grain_grid() {
    use std::sync::{Arc, Mutex};

    let mut scene = test_scene(Mode::Sandy);
    scene.viewport = (1000.0, 600.0);
    let uploads: UploadQueue = Arc::new(Mutex::new(Vec::new()));
    let (tx, _rx) = futures_channel::mpsc::unbounded();
    let pool = DecodePool::start(uploads.clone(), tx, 0);
    let mut store = Catalog::default();
    for name in ["a", "b", "c"] {
        store.items.push(Wallpaper {
            key: name.into(),
            name: name.into(),
            thumb: format!("/tmp/{name}.png"),
            path: format!("/tmp/{name}.png"),
            kind: WallpaperKind::Static,
            ..Default::default()
        });
    }
    let filtered: Vec<u32> = vec![0, 1, 2];
    let pal = Palette::default();
    let mut map = AtlasMap::new(3);
    for idx in 0..2 {
        map.near.acquire(idx);
        map.near.mark_ready(idx);
    }
    let mut atlas = Some(map);

    scene.motion.entrance.snap(1.0);
    scene.motion.visibility.snap(1.0);
    scene.set_current(1, 3);
    scene.tick(
        Instant::now(),
        RebuildCtx {
            catalog: &store,
            filtered: &filtered,
            atlas: &mut atlas,
            pool: &pool,
            palette: &pal,
            uploads: &uploads,
            hover_fades: None,
        },
    );
    let snap = scene.render.sandy.expect("sandy snap");
    assert_eq!(snap.grid, [261.0, 147.0], "782x440 hero, grain 3");

    scene.xp.sandy.grain = 100.0;
    scene.tick(
        Instant::now(),
        RebuildCtx {
            catalog: &store,
            filtered: &filtered,
            atlas: &mut atlas,
            pool: &pool,
            palette: &pal,
            uploads: &uploads,
            hover_fades: None,
        },
    );
    let snap = scene.render.sandy.expect("second sandy snap");
    assert_eq!(snap.grid, [32.0, 18.0], "shader minimum grid");
}

#[test]
fn sandy_flip_toggle() {
    let mut scene = test_scene(Mode::Sandy);
    scene.set_current(2, 10);
    scene.toggle_flip(2);
    assert_eq!(scene.flipped(), Some(2));
    assert!(scene.flip_open());
    scene.toggle_flip(2);
    assert!(!scene.flip_open());
    scene.toggle_flip(2);
    scene.set_current(3, 10);
    assert_eq!(scene.flipped(), None);
}

#[test]
fn flip_phases_skipped_instant() {
    let mut scene = test_scene(Mode::Slices);
    scene.set_card_flip_options(640.0, false, false);
    assert_eq!(scene.card.flip_duration_ms, 640.0);
    assert!(!scene.card.flip_shader_enabled);
    assert!(!scene.card.flip_back_enabled);

    scene.toggle_flip(2);
    assert_eq!(scene.flipped(), Some(2));
    assert_eq!(scene.card.flip.x, 1.0);
    scene.toggle_flip(2);
    assert_eq!(scene.flipped(), None);
    assert_eq!(scene.card.flip.x, 0.0);
}

#[test]
fn strip_hover_no_preview() {
    let mut scene = test_scene(Mode::Sandy);
    scene.hover = Some(3);
    assert!(scene.preview_hover().is_none());
    scene.mode = Mode::Slices;
    assert_eq!(scene.preview_hover(), Some(3));
}

#[test]
fn wheel_scroll_engages() {
    let mut scene = test_scene(Mode::Slices);
    assert!(!scene.user_engaged);
    scene.slice_scroll(-3.0, 10);
    assert!(scene.user_engaged);
}

#[test]
fn settled_tick_reuse() {
    use std::sync::{Arc, Mutex};

    let mut scene = test_scene(Mode::Slices);
    scene.viewport = (800.0, 600.0);

    let uploads: UploadQueue = Arc::new(Mutex::new(Vec::new()));
    let (tx, _rx) = futures_channel::mpsc::unbounded();
    let pool = DecodePool::start(uploads.clone(), tx, 0);
    let store = Catalog::default();
    let filtered: Vec<u32> = Vec::new();
    let pal = Palette::default();
    let mut atlas: Option<AtlasMap> = None;

    macro_rules! do_tick {
        () => {
            scene.tick(
                Instant::now(),
                RebuildCtx {
                    catalog: &store,
                    filtered: &filtered,
                    atlas: &mut atlas,
                    pool: &pool,
                    palette: &pal,
                    uploads: &uploads,
                    hover_fades: None,
                },
            )
        };
    }

    do_tick!();
    scene.motion.needs_frame = false;
    scene.motion.entrance.snap(1.0);
    scene.motion.visibility.snap(1.0);
    scene.camera.snap(scene.camera.target);
    assert!(!scene.is_animating());

    let before = scene.render.clone();
    do_tick!();
    assert!(Arc::ptr_eq(&before, &scene.render));

    scene.set_current(1, 5);
    do_tick!();
    assert!(!Arc::ptr_eq(&before, &scene.render));
}

#[test]
fn pending_preview_deadline() {
    let mut scene = test_scene(Mode::Slices);
    scene.motion.needs_frame = false;
    scene.motion.entrance.snap(1.0);
    scene.motion.visibility.snap(1.0);
    scene.camera.snap(scene.camera.target);
    assert!(!scene.is_animating());
    let deadline = Instant::now() + Duration::from_millis(250);
    scene.preview_state.pending_preview = Some((0, deadline));
    assert!(!scene.is_animating());
    assert_eq!(scene.preview_deadline(), Some(deadline));
    assert!(!scene.preview_deadline_due(deadline.checked_sub(Duration::from_millis(1)).unwrap()));
    assert!(scene.preview_deadline_due(deadline));
}

#[test]
fn sandy_preview_no_delay() {
    let mut scene = test_scene(Mode::Sandy);
    scene.preview_state.preview_delay_ms = 250;
    assert_eq!(scene.preview_delay(), 0);
    scene.mode = Mode::Slices;
    assert_eq!(scene.preview_delay(), 250);
}

#[test]
fn preview_disable_releases_state() {
    let mut scene = test_scene(Mode::Slices);
    scene.preview_state.pending_preview = Some((0, Instant::now()));
    scene.preview_state.live_target = Some(0);
    scene.preview_state.live_injected = Some(0);

    scene.set_preview_config(false, 400, 24);

    assert!(!scene.preview_state.preview_enabled);
    assert!(scene.preview_state.pending_preview.is_none());
    assert!(scene.preview_state.live_target.is_none());
    assert!(scene.preview_state.live_injected.is_none());
    assert_eq!(scene.preview_state.preview_delay_ms, 400);
    assert_eq!(scene.preview_state.live_fps, 24);
}

#[test]
fn fractional_notch() {
    let mut accum = 0.0f32;
    for _ in 0..40 {
        assert_eq!(slice_scroll_steps(&mut accum, 1.25), 1);
    }
    for _ in 0..40 {
        assert_eq!(slice_scroll_steps(&mut accum, -1.25), -1);
    }
}

#[test]
fn clean_notch() {
    let mut accum = 0.0f32;
    assert_eq!(slice_scroll_steps(&mut accum, 1.0), 1);
    assert_eq!(slice_scroll_steps(&mut accum, -1.0), -1);
    assert_eq!(accum, 0.0);
}

#[test]
fn fast_notch() {
    let mut accum = 0.0f32;
    assert_eq!(slice_scroll_steps(&mut accum, 3.0), 3);
    assert_eq!(slice_scroll_steps(&mut accum, -2.0), -2);
}

#[test]
fn grid_scroll_row() {
    let mut scene = test_scene(Mode::Grid);
    let start = scene.camera.target;
    scene.grid_scroll(1.0);
    assert_eq!(scene.camera.target - start, scene.gp.cell_h());
    assert_eq!(scene.camera.target - start, 68.0);
    scene.grid_scroll(-1.0);
    assert_eq!(scene.camera.target, start);

    scene.gp.thumb_h = 200.0;
    scene.grid_scroll(1.0);
    assert_eq!(scene.camera.target - start, 208.0);
    scene.grid_scroll(-1.0);
    assert_eq!(scene.camera.target, start);
}

#[test]
fn wall_camera_slow_tier() {
    let wall = test_scene(Mode::Grid);
    let slices = test_scene(Mode::Slices);

    assert!(
        (wall.camera.duration_ms() - MotionProfile::default().duration_ms(MotionTier::Slow)).abs()
            < 0.01
    );
    assert!(
        (slices.camera.duration_ms() - MotionProfile::default().duration_ms(MotionTier::Standard))
            .abs()
            < 0.01
    );
}

#[test]
fn slice_camera_anchors_selection() {
    let mut scene = test_scene(Mode::Sandy);
    scene.set_mode(Mode::Slices);
    scene.position_slice_camera(120_000.0);
    assert_eq!(scene.camera_pos(), 120_000.0);
    assert_eq!(scene.camera_target(), 120_000.0);

    scene.sp_target.spacing = 52.0;
    scene.position_slice_camera(180_000.0);
    assert_eq!(scene.camera_pos(), 180_000.0);
    assert_eq!(scene.camera_target(), 180_000.0);

    scene.sp = scene.sp_target;
    scene.position_slice_camera(180_200.0);
    assert_eq!(scene.camera_pos(), 180_000.0);
    assert_eq!(scene.camera_target(), 180_200.0);
}

#[test]
fn hex_camera_anchors_selection() {
    let mut scene = test_scene(Mode::Slices);
    scene.set_mode(Mode::Hex);
    scene.position_hex_camera(90_000.0);
    assert_eq!(scene.camera_pos(), 90_000.0);
    assert_eq!(scene.camera_target(), 90_000.0);

    scene.hp_target.r = 118.0;
    scene.position_hex_camera(140_000.0);
    assert_eq!(scene.camera_pos(), 140_000.0);
    assert_eq!(scene.camera_target(), 140_000.0);

    scene.hp = scene.hp_target;
    scene.position_hex_camera(140_200.0);
    assert_eq!(scene.camera_pos(), 140_000.0);
    assert_eq!(scene.camera_target(), 140_200.0);
}

#[test]
fn fine_pixels_accumulate() {
    let mut accum = 0.0f32;
    assert_eq!(slice_scroll_steps(&mut accum, 0.25), 0);
    assert_eq!(slice_scroll_steps(&mut accum, 0.25), 0);
    assert_eq!(slice_scroll_steps(&mut accum, 0.25), 0);
    assert_eq!(slice_scroll_steps(&mut accum, 0.25), 1);
    assert_eq!(slice_scroll_steps(&mut accum, 0.25), 0);
}

#[test]
fn tag_morph_settles() {
    let mut scene = test_scene(Mode::Slices);
    assert!(scene.card.tag_open.settled());
    assert!(scene.card.chip_pop.settled());
    scene.set_tag_editing(true);
    assert!(!scene.card.tag_open.settled());
    assert!(scene.is_animating());
    for _ in 0..300 {
        scene.card.tag_open.tick(1.0 / 60.0);
    }
    assert!(scene.card.tag_open.settled());
    assert!((scene.card.tag_open.x - 1.0).abs() < 0.01);
    scene.pop_tag_chip(4);
    assert_eq!(scene.card.pop_idx, 4);
    assert!(!scene.card.chip_pop.settled());
    assert!(scene.is_animating());
    for _ in 0..300 {
        scene.card.chip_pop.tick(1.0 / 60.0);
    }
    assert!(scene.card.chip_pop.settled());
    assert!((scene.card.chip_pop.x - 1.0).abs() < 0.01);
    scene.toggle_flip(0);
    scene.close_flip();
    assert!(scene.card.tag_open.settled() && scene.card.tag_open.x == 0.0);
}

fn rendered_scene(mode: Mode, sp: SliceParams, gp: GridParams) -> std::sync::Arc<RenderSnapshot> {
    rendered_scene_with_bar(mode, sp, gp, None)
}

fn rendered_scene_with_bar(
    mode: Mode,
    sp: SliceParams,
    gp: GridParams,
    bar: Option<(bool, f32, f32)>,
) -> std::sync::Arc<RenderSnapshot> {
    rendered_scene_with_count_and_bar(mode, sp, gp, 5, bar)
}

fn rendered_scene_with_count_and_bar(
    mode: Mode,
    sp: SliceParams,
    gp: GridParams,
    count: usize,
    bar: Option<(bool, f32, f32)>,
) -> std::sync::Arc<RenderSnapshot> {
    rendered_scene_with_count_bar_camera(mode, sp, gp, count, bar, 0.0)
}

fn rendered_scene_with_count_bar_camera(
    mode: Mode,
    sp: SliceParams,
    gp: GridParams,
    count: usize,
    bar: Option<(bool, f32, f32)>,
    camera: f32,
) -> std::sync::Arc<RenderSnapshot> {
    rendered_scene_with_setup(mode, sp, gp, count, bar, camera, |_| {})
}

fn rendered_scene_with_setup(
    mode: Mode,
    sp: SliceParams,
    gp: GridParams,
    count: usize,
    bar: Option<(bool, f32, f32)>,
    camera: f32,
    setup: impl FnOnce(&mut SceneCore),
) -> std::sync::Arc<RenderSnapshot> {
    use std::sync::{Arc, Mutex};
    let hp = HexParams {
        r: 40.0,
        rows: 3,
        cols: 5,
        curve: crate::frontend::scene::layout::HexCurve::Flat,
        curve_strength: 0.0,
        ..HexParams::default()
    };
    let mut scene = SceneCore::new(mode, sp, gp, hp, &test_extra());
    scene.viewport = (1200.0, 700.0);
    scene.set_filter_bar_footprint(bar);
    let uploads: UploadQueue = Arc::new(Mutex::new(Vec::new()));
    let (tx, _rx) = futures_channel::mpsc::unbounded();
    let pool = DecodePool::start(uploads.clone(), tx, 0);
    let mut store = Catalog::default();
    for idx in 0..count {
        let name = format!("wall-{idx}");
        store.items.push(Wallpaper {
            key: name.clone(),
            name: name.clone(),
            thumb: format!("/tmp/{name}.png"),
            path: format!("/tmp/{name}.png"),
            kind: WallpaperKind::Static,
            ..Default::default()
        });
    }
    let filtered: Vec<u32> = (0..count as u32).collect();
    let pal = Palette::default();
    let mut map = AtlasMap::new(count);
    for idx in 0..count {
        map.near.acquire(idx);
        map.near.mark_ready(idx);
    }
    let mut atlas = Some(map);
    scene.set_current(0, count);
    scene.camera.snap(camera);
    if camera != 0.0 {
        scene.camera.retarget(camera + 1.0);
    }
    scene.motion.entrance.snap(1.0);
    scene.motion.visibility.snap(1.0);
    setup(&mut scene);
    scene.tick(
        Instant::now(),
        RebuildCtx {
            catalog: &store,
            filtered: &filtered,
            atlas: &mut atlas,
            pool: &pool,
            palette: &pal,
            uploads: &uploads,
            hover_fades: None,
        },
    );
    scene.render.clone()
}

#[test]
fn framed_hits_match_cards() {
    for mode in [Mode::Slices, Mode::Hex, Mode::Grid, Mode::Sandy, Mode::Hand] {
        let snap = rendered_scene_with_setup(mode, base_sp(), base_gp(), 20, None, 0.0, |scene| {
            scene.set_picker_frame(Some([0.1, 0.2, 0.6, 0.5]));
        });
        assert_eq!(snap.clip, Some([120.0, 140.0, 840.0, 490.0]));
        assert!(!snap.hits.is_empty(), "{mode:?}");
        for hit in &snap.hits {
            assert!(hit.contains(hit.cx, hit.cy), "{mode:?}");
        }
        let top = &snap.hits[0];
        assert_eq!(
            snap.hits.iter().find(|hit| hit.contains(top.cx, top.cy)).unwrap().index,
            top.index
        );
    }
}

#[test]
fn grid_frame_adds_cards() {
    let grid = GridParams { thumb_w: 150.0, thumb_h: 90.0, ..base_gp() };
    let render = |width| {
        rendered_scene_with_setup(Mode::Grid, base_sp(), grid, 40, None, 0.0, |scene| {
            scene.set_picker_frame(Some([0.1, 0.2, width, 0.5]));
        })
    };
    let small = render(0.3);
    let wide = render(0.6);
    assert!(wide.hits.len() > small.hits.len());
    let small_card = small.hits.iter().find(|hit| hit.index == 0).unwrap();
    let wide_card = wide.hits.iter().find(|hit| hit.index == 0).unwrap();
    assert_eq!((small_card.hw, small_card.hh), (wide_card.hw, wide_card.hh));
}

#[test]
fn grid_content_changes_cards() {
    let frame = [0.1, 0.2, 0.6, 0.5];
    let render = |thumb_w| {
        rendered_scene_with_setup(
            Mode::Grid,
            base_sp(),
            GridParams { thumb_w, ..base_gp() },
            40,
            None,
            0.0,
            |scene| scene.set_picker_frame(Some(frame)),
        )
    };
    let small = render(100.0);
    let large = render(150.0);
    assert_eq!(small.clip, large.clip);
    assert!(small.hits.len() > large.hits.len());
    let small_card = small.hits.iter().find(|hit| hit.index == 0).unwrap();
    let large_card = large.hits.iter().find(|hit| hit.index == 0).unwrap();
    assert!(small_card.hw < large_card.hw);
}

#[test]
fn shown_frame_overrides_target() {
    let mut scene = test_scene(Mode::Grid);
    scene.viewport = (1200.0, 700.0);
    scene.set_picker_frame(Some([0.2, 0.2, 0.5, 0.5]));
    let target = scene.picker_frame_bounds().unwrap();
    let shown = iced::Rectangle { x: 120.0, ..target };
    scene.show_frame(Some(shown));
    assert_eq!(scene.picker_frame_bounds(), Some(shown));
    assert_eq!(scene.configured_frame_bounds(), Some(target));
    assert_eq!(scene.frame_demand(), crate::app::scene::FrameDemand::Motion);
    scene.show_frame(None);
    assert_eq!(scene.picker_frame_bounds(), Some(target));
}

#[test]
fn shown_frame_moves_cards() {
    let render = |shift: f32| {
        rendered_scene_with_setup(Mode::Grid, base_sp(), base_gp(), 20, None, 0.0, |scene| {
            scene.set_picker_frame(Some([0.2, 0.2, 0.5, 0.5]));
            let target = scene.picker_frame_bounds().unwrap();
            scene.show_frame(Some(iced::Rectangle { x: target.x - shift, ..target }));
        })
    };
    let x = |snap: &RenderSnapshot| snap.hits.iter().find(|hit| hit.index == 0).unwrap().cx;
    assert!(x(&render(120.0)) < x(&render(60.0)));
    assert!(x(&render(60.0)) < x(&render(0.0)));
}

#[test]
fn editorial_columns_retile() {
    let render = |width| {
        rendered_scene_with_setup(
            Mode::Grid,
            base_sp(),
            GridParams { layout: GridLayout::Editorial, ..base_gp() },
            20,
            None,
            0.0,
            |scene| scene.set_picker_frame(Some([0.1, 0.2, width, 0.5])),
        )
    };
    let four = render(0.4);
    let five = render(0.5);
    let card = |snap: &RenderSnapshot| {
        let hit = snap.hits.iter().find(|hit| hit.index == 5).unwrap();
        (hit.cx, hit.cy, hit.hw, hit.hh)
    };
    assert_ne!(card(&four), card(&five));
    assert!(five.hits.iter().any(|hit| hit.index == 6));
}

#[test]
fn editorial_legacy_blocks() {
    let mut scene = test_scene(Mode::Grid);
    scene.gp.layout = GridLayout::Editorial;
    scene.gp.cols = 5;
    assert_eq!(scene.editorial_block_items(5), super::layout_helpers::EDITORIAL_CARDS);
    let snap = rendered_scene_with_count_and_bar(Mode::Grid, base_sp(), scene.gp, 20, None);
    let hero = snap.hits.iter().find(|hit| hit.index == 5).unwrap();
    let small = snap.hits.iter().find(|hit| hit.index == 6).unwrap();
    assert!(hero.hw > small.hw);
}

#[test]
fn editorial_narrow_frames() {
    let frame = [0.1, 0.2, 0.6, 0.5];
    for cols in [1, 2] {
        let snap = rendered_scene_with_setup(
            Mode::Grid,
            base_sp(),
            GridParams { cols, layout: GridLayout::Editorial, ..base_gp() },
            20,
            None,
            0.0,
            |scene| scene.set_picker_frame(Some(frame)),
        );
        let clip = snap.clip.unwrap();
        assert!(!snap.hits.is_empty());
        for hit in &snap.hits {
            assert!([hit.cx, hit.cy, hit.hw, hit.hh].into_iter().all(f32::is_finite));
            assert!(hit.cx - hit.hw >= clip[0] - 0.01);
            assert!(hit.cx + hit.hw <= clip[2] + 0.01);
        }
    }
}

#[test]
fn editorial_follow_blocks() {
    let mut scene = test_scene(Mode::Grid);
    scene.viewport = (1200.0, 700.0);
    scene.gp.layout = GridLayout::Editorial;
    scene.gp.cols = 5;
    scene.gp.rows = 2;
    scene.kb_nav = true;
    scene.set_picker_frame(Some([0.1, 0.2, 0.6, 0.3]));
    let block_items = scene.editorial_block_items(scene.effective_grid_params().cols);
    scene.set_current(block_items - 1, 20);
    assert_eq!(scene.camera.target, 0.0);
    scene.set_current(block_items, 20);
    assert!(scene.camera.target > 0.0);
}

#[test]
fn framed_grid_navigation() {
    let mut scene = test_scene(Mode::Grid);
    scene.viewport = (1200.0, 700.0);
    scene.gp.rows = 3;
    scene.set_picker_frame(Some([0.1, 0.2, 0.3, 0.3]));
    let gp = scene.effective_grid_params();
    scene.grid_scroll(1.0);
    assert_eq!(scene.camera.target, gp.cell_h());
    assert_eq!(gp.cell_h(), scene.gp.cell_h());
    scene.camera.snap(0.0);
    scene.kb_nav = true;
    scene.set_current(12, 20);
    assert!(scene.camera.target > 0.0);

    let snap = rendered_scene_with_setup(
        Mode::Grid,
        base_sp(),
        GridParams { rows: 3, ..base_gp() },
        20,
        None,
        0.0,
        |scene| {
            scene.set_picker_frame(Some([0.1, 0.2, 0.3, 0.3]));
            scene.kb_nav = true;
            scene.set_current(12, 20);
            scene.camera.snap(scene.camera.target);
        },
    );
    let selected = snap.hits.iter().find(|hit| hit.index == 12).unwrap();
    let clip = snap.clip.unwrap();
    assert!(selected.cy - selected.hh >= clip[1]);
    assert!(selected.cy + selected.hh <= clip[3]);
}

#[test]
fn framed_pointer_modes() {
    let bounds = iced::Rectangle { x: 0.0, y: 0.0, width: 1000.0, height: 600.0 };
    let mut hand = test_scene(Mode::Hand);
    hand.viewport = (1000.0, 600.0);
    hand.set_picker_frame(Some([0.2, 0.2, 0.6, 0.6]));
    hand.natural_frame = Some(bounds);
    let place = hand.picker_placement().unwrap();
    let (x, y) = place.point(750.0, 150.0);
    hand.hand_pointer(x, y, None);
    let (tx, ty) = crate::frontend::scene::hand::tilt_target(
        750.0,
        150.0,
        hand.viewport.0,
        hand.viewport.1,
        hand.xp.hand.tilt,
    );
    assert!((hand.hand.rig_x.target - tx).abs() < 0.01);
    assert!((hand.hand.rig_y.target - ty).abs() < 0.01);
    let (start_x, start_y) = place.point(500.0, 300.0);
    let (end_x, end_y) = place.point(600.0, 300.0);
    hand.hand_drag_start(start_x, start_y);
    hand.hand_pointer(end_x, end_y, None);
    let (drag_x, drag_y) = hand.hand.drag.unwrap();
    assert!((drag_x - 600.0).abs() < 0.01 && (drag_y - 300.0).abs() < 0.01);

    let mut sandy = test_scene(Mode::Sandy);
    sandy.viewport = (1000.0, 600.0);
    sandy.set_picker_frame(Some([0.2, 0.2, 0.6, 0.6]));
    sandy.natural_frame = Some(bounds);
    let y = crate::frontend::scene::sandy::strip_bottom(600.0) - 10.0;
    let (edge_x, edge_y) = sandy.picker_placement().unwrap().point(10.0, y);
    sandy.sandy_pointer(edge_x, edge_y, None, 20);
    assert!(sandy.sandy.edge_pan < 0.0);
    sandy.sandy_pointer(100.0, edge_y, None, 20);
    assert_eq!(sandy.sandy.edge_pan, 0.0);
    assert!(!sandy.sandy.cam_free);
}

#[test]
fn framed_detail_source() {
    let mut scene = test_scene(Mode::Grid);
    scene.viewport = (1000.0, 600.0);
    scene.set_picker_frame(Some([0.2, 0.2, 0.6, 0.6]));
    scene.natural_frame = Some(iced::Rectangle { x: 0.0, y: 0.0, width: 1000.0, height: 600.0 });
    let (x, y) = scene.picker_placement().unwrap().point(500.0, 300.0);
    scene.open_detail(0, [x, y, 60.0, 30.0]);
    for (actual, expected) in scene.card.det_src.into_iter().zip([500.0, 300.0, 60.0, 30.0]) {
        assert!((actual - expected).abs() < 0.01);
    }
}

#[test]
fn framed_collection_projection() {
    use crate::rendering::scene::PROJECTED;

    for open in [0.0, 0.5, 1.0] {
        let snap = rendered_scene_with_setup(
            Mode::Collection,
            base_sp(),
            base_gp(),
            20,
            None,
            0.0,
            |scene| {
                scene.set_picker_frame(Some([0.1, 0.2, 0.6, 0.5]));
                scene.select_collection(0, 20);
                scene.collection_open.snap(open);
            },
        );
        let projected: Vec<_> = snap
            .instances
            .iter()
            .filter(|inst| inst.misc[0] > 0 && inst.misc[3] & PROJECTED != 0)
            .collect();
        assert!(!snap.hits.is_empty());
        assert_eq!(projected.len(), snap.hits.len());
        for (body, hit) in projected.into_iter().zip(snap.hits.iter().rev()) {
            let quad = hit.quad.unwrap();
            for (actual, expected) in [
                (body.quad_a[0], quad[0][0]),
                (body.quad_a[1], quad[0][1]),
                (body.quad_a[2], quad[1][0]),
                (body.quad_a[3], quad[1][1]),
                (body.quad_b[0], quad[2][0]),
                (body.quad_b[1], quad[2][1]),
                (body.quad_b[2], quad[3][0]),
                (body.quad_b[3], quad[3][1]),
            ] {
                assert!((actual - expected).abs() < 0.01);
            }
        }
    }
}

fn base_sp() -> SliceParams {
    SliceParams {
        offset_x: 0.0,
        offset_y: 0.0,
        slice_w: 80.0,
        expanded_w: 200.0,
        slice_h: 120.0,
        spacing: 8.0,
        skew: 0.0,
        edge_tilt: 0.0,
        visible_count: 5,
        corners: CardCorners::default(),
        wobble: false,
        wobble_strength: 1.0,
        parallax: false,
        shadows: true,
    }
}

fn base_gp() -> GridParams {
    GridParams {
        cols: 4,
        rows: 4,
        thumb_w: 100.0,
        thumb_h: 60.0,
        gap_x: 8.0,
        gap_y: 8.0,
        corners: CardCorners::round(6.0),
        border_width: 2.0,
        ..GridParams::default()
    }
}

fn max_radius(snap: &RenderSnapshot) -> f32 {
    snap.instances.iter().map(|inst| inst.radii[0]).fold(0.0, f32::max)
}

#[test]
fn wall_spacing_edge_to_edge() {
    let grid = base_gp();
    assert_eq!(grid.cell_w(), 108.0);
    assert_eq!(grid.cell_h(), 68.0);
    assert_eq!(grid.total_w(), 424.0);
    assert_eq!(grid.total_h(), 264.0);
}

#[test]
fn wall_field_centred() {
    let snap = rendered_scene(Mode::Grid, base_sp(), base_gp());
    let first_row: Vec<_> = snap.hits.iter().filter(|hit| hit.cy < 300.0).collect();
    let centre = first_row.iter().map(|hit| hit.cx).sum::<f32>() / first_row.len() as f32;

    assert!((centre - 600.0).abs() < 0.05);
    assert!((first_row[0].cy - 248.0).abs() < 0.05);
}

#[test]
fn wall_centred_with_horizontal_bar() {
    let snap =
        rendered_scene_with_bar(Mode::Grid, base_sp(), base_gp(), Some((false, 700.0, 24.0)));
    let first_row: Vec<_> = snap.hits.iter().filter(|hit| hit.cy < 320.0).collect();
    let centre = first_row.iter().map(|hit| hit.cx).sum::<f32>() / first_row.len() as f32;
    assert!((centre - 600.0).abs() < 0.05);
    assert!((first_row[0].cy - 264.0).abs() < 0.05);
}

#[test]
fn wall_centred_with_vertical_bar() {
    let snap =
        rendered_scene_with_bar(Mode::Grid, base_sp(), base_gp(), Some((true, 100.0, 600.0)));
    let first_row: Vec<_> = snap.hits.iter().filter(|hit| hit.cy < 300.0).collect();
    let centre = first_row.iter().map(|hit| hit.cx).sum::<f32>() / first_row.len() as f32;
    assert!((centre - 654.0).abs() < 0.05);
    assert!((first_row[0].cy - 248.0).abs() < 0.05);
}

#[test]
fn wall_clips_at_aperture() {
    let snap = rendered_scene(
        Mode::Grid,
        base_sp(),
        GridParams { cols: 4, rows: 2, thumb_w: 400.0, thumb_h: 220.0, ..base_gp() },
    );

    assert_eq!(snap.hits.len(), 5);
    let frame = snap.clip.expect("wall clip");
    assert!(snap.hits.iter().any(|hit| hit.cx - hit.hw < frame[0]));
    assert!(snap.hits.iter().any(|hit| hit.cx + hit.hw > frame[2]));
}

#[test]
fn editorial_keeps_partial_cards() {
    let gp = GridParams { rows: 3, layout: GridLayout::Editorial, ..base_gp() };
    let snap = rendered_scene_with_count_and_bar(Mode::Grid, base_sp(), gp, 9, None);

    assert_eq!(snap.hits.len(), 8);
    let frame = snap.clip.expect("wall clip");
    assert!(
        snap.hits.iter().any(|hit| hit.cy + hit.hh > frame[3]),
        "frame={frame:?}, hits={:?}",
        snap.hits
    );
}

#[test]
fn wall_initial_frame_capacity() {
    let gp = GridParams { cols: 4, rows: 2, ..base_gp() };
    let snap = rendered_scene_with_count_and_bar(Mode::Grid, base_sp(), gp, 24, None);

    assert_eq!(snap.hits.len(), 8);
    assert!(snap.hits.iter().all(|hit| hit.index < 8));
}

#[test]
fn wall_boundary_cards_full_size() {
    let gp = GridParams { cols: 4, rows: 2, ..base_gp() };
    let snap = rendered_scene_with_count_bar_camera(
        Mode::Grid,
        base_sp(),
        gp,
        24,
        None,
        gp.cell_h() * 0.5,
    );
    let outgoing = snap.hits.iter().find(|hit| hit.index == 0).expect("outgoing first row");

    assert!((outgoing.hw - gp.thumb_w * 0.5).abs() < 0.01);
    assert!((outgoing.hh - gp.thumb_h * 0.5).abs() < 0.01);
}

#[test]
fn flat_hex_keeps_honeycomb() {
    let snap = rendered_scene(Mode::Hex, base_sp(), base_gp());
    let selected = snap.hits.iter().find(|hit| hit.index == 0).expect("selected hex");
    let same_column = snap.hits.iter().find(|hit| hit.index == 1).expect("same-column hex");
    let staggered = snap.hits.iter().find(|hit| hit.index == 3).expect("staggered hex");

    assert!((selected.hw - same_column.hw).abs() < 0.01);
    assert!((selected.hh - same_column.hh).abs() < 0.01);

    let expected_stagger = HexParams {
        r: 40.0,
        rows: 3,
        cols: 5,
        curve: crate::frontend::scene::layout::HexCurve::Flat,
        curve_strength: 0.0,
        ..HexParams::default()
    }
    .step_y()
        * 0.5;
    assert!(((staggered.cy - selected.cy) - expected_stagger).abs() < 0.01);
}

#[test]
fn appearance_settings_reach_render() {
    let small = rendered_scene(Mode::Grid, base_sp(), GridParams { thumb_w: 100.0, ..base_gp() });
    let large = rendered_scene(Mode::Grid, base_sp(), GridParams { thumb_w: 220.0, ..base_gp() });
    assert!(!small.hits.is_empty() && !large.hits.is_empty());
    assert!(large.hits[0].hw > small.hits[0].hw, "{} vs {}", large.hits[0].hw, small.hits[0].hw);

    let sharp = rendered_scene(
        Mode::Grid,
        base_sp(),
        GridParams { corners: CardCorners::default(), ..base_gp() },
    );
    let round = rendered_scene(
        Mode::Grid,
        base_sp(),
        GridParams { corners: CardCorners::round(18.0), border_width: 4.0, ..base_gp() },
    );
    assert!(
        max_radius(&round) >= 18.0 && max_radius(&sharp) == 0.0,
        "sharp={}, round={}",
        max_radius(&sharp),
        max_radius(&round)
    );
    let wall_widths: Vec<f32> = round
        .instances
        .iter()
        .filter(|instance| instance.misc[2] == 1)
        .map(|instance| instance.rect[2])
        .collect();
    let widest = wall_widths.iter().copied().fold(0.0, f32::max);
    let narrowest = wall_widths.iter().copied().fold(f32::MAX, f32::min);
    assert!((widest - narrowest).abs() < 0.05, "widest={widest}, narrowest={narrowest}");

    let straight = rendered_scene(Mode::Slices, SliceParams { skew: 0.0, ..base_sp() }, base_gp());
    let sheared = rendered_scene(Mode::Slices, SliceParams { skew: 30.0, ..base_sp() }, base_gp());
    assert!(!sheared.hits.is_empty());
    assert_eq!(straight.hits[0].skew, 0.0);
    assert!(sheared.hits[0].skew.abs() > 0.0, "skew={}", sheared.hits[0].skew);

    let tilted =
        rendered_scene(Mode::Slices, SliceParams { edge_tilt: 48.0, ..base_sp() }, base_gp());
    assert_eq!(tilted.hits[0].edge_tilt, 48.0);
    assert_eq!(tilted.instances.last().expect("slice body").shape[0], 48.0);

    let sharp = rendered_scene(
        Mode::Slices,
        SliceParams { corners: CardCorners::default(), ..base_sp() },
        base_gp(),
    );
    let round = rendered_scene(
        Mode::Slices,
        SliceParams { corners: CardCorners::round(16.0), ..base_sp() },
        base_gp(),
    );
    assert!(
        max_radius(&round) > max_radius(&sharp) && max_radius(&round) >= 15.0,
        "sharp={}, round={}",
        max_radius(&sharp),
        max_radius(&round)
    );
}

#[test]
fn wall_deformers_reach_render() {
    let uniform = rendered_scene(Mode::Grid, base_sp(), base_gp());
    let flowed = rendered_scene(
        Mode::Grid,
        base_sp(),
        GridParams {
            flow_wave: 45.0,
            flow_frequency: 1.5,
            scatter: 18.0,
            scale_variance: 0.2,
            ..base_gp()
        },
    );
    assert_ne!(
        uniform.hits.iter().map(|hit| (hit.cx, hit.cy, hit.hw)).collect::<Vec<_>>(),
        flowed.hits.iter().map(|hit| (hit.cx, hit.cy, hit.hw)).collect::<Vec<_>>()
    );
}

#[test]
fn wall_cylinder_projection() {
    let flat = base_gp();
    let point = (1_000.0, 420.0);
    assert_eq!(flat.cylinder_transform(point.0, point.1, (600.0, 350.0)), (1_000.0, 420.0, 1.0));

    let receding = GridParams {
        layout: GridLayout::Cylinder,
        cylinder_bend: 0.8,
        cylinder_radius: 600.0,
        ..base_gp()
    };
    let bulging = GridParams {
        layout: GridLayout::Cylinder,
        cylinder_bend: -0.8,
        cylinder_radius: 600.0,
        ..base_gp()
    };
    let receded = receding.cylinder_transform(point.0, point.1, (600.0, 350.0));
    let bulged = bulging.cylinder_transform(point.0, point.1, (600.0, 350.0));
    assert!(receded.0 < point.0);
    assert!(receded.2 < 1.0);
    assert!(bulged.2 > 1.0);

    let flat_render = rendered_scene(Mode::Grid, base_sp(), base_gp());
    let curved_render = rendered_scene(Mode::Grid, base_sp(), receding);
    assert_ne!(
        flat_render.hits.iter().map(|hit| (hit.cx, hit.hw)).collect::<Vec<_>>(),
        curved_render.hits.iter().map(|hit| (hit.cx, hit.hw)).collect::<Vec<_>>()
    );
}

#[test]
fn launch_none_snaps_entrance() {
    let mut scene = test_scene(Mode::Slices);
    scene.set_launch_animation("none");
    scene.begin_open_fade();
    assert!(scene.motion.entrance.settled());
    assert!((scene.motion.entrance.x - 1.0).abs() < f32::EPSILON);
    scene.set_launch_animation("fade");
    scene.begin_open_fade();
    assert!(scene.motion.entrance_pending);
    assert!(scene.motion.entrance.x < 0.01);
}

#[test]
fn insert_before_hero() {
    let mut scene = test_scene(Mode::Grid);
    scene.set_current(5, 10);
    assert_eq!(scene.current, 5);
    scene.shift_after_insert(3);
    assert_eq!(scene.current, 6);
    scene.shift_after_insert(9);
    assert_eq!(scene.current, 6);
    scene.shift_after_insert(6);
    assert_eq!(scene.current, 7);
}

#[test]
fn insert_remaps_index_keyed_state() {
    let mut scene = test_scene(Mode::Sandy);
    scene.set_current(4, 10);
    scene.sandy.ring_to = Some(4);
    scene.hover = Some(2);
    scene.shift_after_insert(3);
    assert_eq!(scene.current, 5);
    assert_eq!(scene.sandy.ring_to, Some(5));
    assert_eq!(scene.hover, Some(2));
}

#[test]
fn motion_profile_retimes_cards() {
    let mut scene = test_scene(Mode::Slices);
    scene.card.widths.insert(0, MotionProfile::default().spring(0.0, MotionTier::Standard));
    scene.card.selection.insert(0, MotionProfile::default().spring(0.0, MotionTier::Standard));
    scene.card.fades.insert(0, MotionProfile::default().spring(0.0, MotionTier::Fast));

    scene.set_motion_profile(MotionProfile::new(90.0, 360.0, 900.0));

    assert!((scene.card.widths[&0].duration_ms() - 360.0).abs() < 0.01);
    assert!((scene.card.selection[&0].duration_ms() - 360.0).abs() < 0.01);
    assert!((scene.card.fades[&0].duration_ms() - 90.0).abs() < 0.01);
}

#[test]
fn wall_stage_translates_uniformly() {
    let flowed = GridParams { flow_wave: 45.0, flow_frequency: 1.5, ..base_gp() };
    let base = rendered_scene(Mode::Grid, base_sp(), flowed);
    let staged = GridParams {
        stage: StageParams { offset_x: 0.02, offset_y: 0.03, ..StageParams::default() },
        ..flowed
    };
    let moved = rendered_scene(Mode::Grid, base_sp(), staged);

    assert_eq!(base.hits.len(), moved.hits.len());
    for (before, after) in base.hits.iter().zip(moved.hits.iter()) {
        assert_eq!(before.index, after.index);
        assert!((after.cx - before.cx - 12.0).abs() < 0.01, "{} -> {}", before.cx, after.cx);
        assert!((after.cy - before.cy - 10.5).abs() < 0.01, "{} -> {}", before.cy, after.cy);
        assert!((after.hw - before.hw).abs() < 0.01 && (after.hh - before.hh).abs() < 0.01);
    }
}

#[test]
fn slices_position_translates_uniformly() {
    let base = rendered_scene(Mode::Slices, base_sp(), base_gp());
    let moved = rendered_scene(
        Mode::Slices,
        SliceParams { offset_x: 0.1, offset_y: -0.2, ..base_sp() },
        base_gp(),
    );

    assert_eq!(base.hits.len(), moved.hits.len());
    for (before, after) in base.hits.iter().zip(moved.hits.iter()) {
        assert_eq!(before.index, after.index);
        assert!((after.cx - before.cx - 60.0).abs() < 0.01);
        assert!((after.cy - before.cy + 70.0).abs() < 0.01);
    }
}

#[test]
fn sandy_pointer_tracks_vertical_position() {
    let mut scene = test_scene(Mode::Sandy);
    scene.viewport = (1000.0, 600.0);
    scene.xp.sandy.offset_y = -0.2;
    let shifted_strip_y = 600.0 - sandy::BAR_ZONE - 10.0 - 60.0;

    scene.sandy_pointer(20.0, shifted_strip_y, None, 5);

    assert!(scene.sandy.edge_pan < -0.8);
}

fn hex_stage_render(offset_x: f32) -> std::sync::Arc<RenderSnapshot> {
    use std::sync::{Arc, Mutex};
    let mut scene = test_scene(Mode::Hex);
    scene.viewport = (1200.0, 700.0);
    scene.hp.stage.offset_x = offset_x;
    scene.hp_target.stage.offset_x = offset_x;
    let uploads: UploadQueue = Arc::new(Mutex::new(Vec::new()));
    let (tx, _rx) = futures_channel::mpsc::unbounded();
    let pool = DecodePool::start(uploads.clone(), tx, 0);
    let mut store = Catalog::default();
    for idx in 0..120 {
        let name = format!("wall-{idx}");
        store.items.push(Wallpaper {
            key: name.clone(),
            name: name.clone(),
            thumb: format!("/tmp/{name}.png"),
            path: format!("/tmp/{name}.png"),
            kind: WallpaperKind::Static,
            ..Default::default()
        });
    }
    let filtered: Vec<u32> = (0..120).collect();
    let pal = Palette::default();
    let mut map = AtlasMap::new(120);
    for idx in 0..120 {
        map.near.acquire(idx);
        map.near.mark_ready(idx);
    }
    let mut atlas = Some(map);
    scene.set_current(90, 120);
    scene.kb_nav = true;
    scene.camera.snap(scene.hp.column_x(30));
    scene.motion.entrance.snap(1.0);
    scene.motion.visibility.snap(1.0);
    scene.tick(
        Instant::now(),
        RebuildCtx {
            catalog: &store,
            filtered: &filtered,
            atlas: &mut atlas,
            pool: &pool,
            palette: &pal,
            uploads: &uploads,
            hover_fades: None,
        },
    );
    scene.render.clone()
}

#[test]
fn hex_coarse_pass_with_stage() {
    let centered = hex_stage_render(0.0);
    let shifted = hex_stage_render(1.5);

    let centered_min =
        centered.hits.iter().map(|hit| hit.index).min().expect("centered hex places cards");
    assert!(!shifted.hits.is_empty());
    let shifted_min = shifted.hits.iter().map(|hit| hit.index).min().unwrap();
    assert!(shifted_min < centered_min, "{shifted_min} vs {centered_min}");
}

#[test]
fn expanded_proof_flip_terminal() {
    use std::sync::{Arc, Mutex};
    let mut scene = test_scene(Mode::Grid);
    scene.viewport = (1000.0, 600.0);
    let uploads: UploadQueue = Arc::new(Mutex::new(Vec::new()));
    let (tx, _rx) = futures_channel::mpsc::unbounded();
    let pool = DecodePool::start(uploads.clone(), tx, 0);
    let mut store = Catalog::default();
    for name in ["a", "b", "c"] {
        store.items.push(Wallpaper {
            key: name.into(),
            name: name.into(),
            thumb: format!("/tmp/{name}.png"),
            path: format!("/tmp/{name}.png"),
            kind: WallpaperKind::Static,
            ..Default::default()
        });
    }
    let filtered: Vec<u32> = vec![0, 1, 2];
    let pal = Palette::default();
    let mut map = AtlasMap::new(3);
    for idx in 0..3 {
        map.near.acquire(idx);
        map.near.mark_ready(idx);
    }
    let mut atlas = Some(map);

    scene.set_current(1, 3);
    scene.toggle_flip(1);
    scene.card.flip.snap(1.0);
    scene.card.det_target = 1.0;
    scene.card.det_p = 1.0;
    scene.motion.entrance.snap(1.0);
    scene.motion.visibility.snap(1.0);
    scene.camera.snap(scene.camera.target);
    scene.tick(
        Instant::now(),
        RebuildCtx {
            catalog: &store,
            filtered: &filtered,
            atlas: &mut atlas,
            pool: &pool,
            palette: &pal,
            uploads: &uploads,
            hover_fades: None,
        },
    );

    let proof = scene
        .render
        .instances
        .iter()
        .find(|instance| (instance.rect[0] - 500.0).abs() < 0.01 && instance.flip[0] > 0.0)
        .expect("flip proof instance");
    assert!(proof.flip[0] > 0.99, "{}", proof.flip[0]);
}

#[test]
fn grid_hover_border_fade() {
    use std::sync::{Arc, Mutex};

    let mut scene = test_scene(Mode::Grid);
    scene.viewport = (1280.0, 720.0);
    scene.motion.entrance.snap(1.0);
    scene.motion.visibility.snap(1.0);

    let uploads: UploadQueue = Arc::new(Mutex::new(Vec::new()));
    let (tx, _rx) = futures_channel::mpsc::unbounded();
    let pool = DecodePool::start(uploads.clone(), tx, 0);
    let mut store = Catalog::default();
    for index in 0..4 {
        store.items.push(Wallpaper {
            key: format!("wall-{index}"),
            name: format!("wall-{index}"),
            kind: WallpaperKind::Static,
            ..Default::default()
        });
    }
    let filtered: Vec<u32> = (0..4).collect();
    let palette = Palette::default();
    let mut atlas = Some(AtlasMap::new(store.items.len()));

    let mut spring = Spring::for_duration_ms(0.0, 400.0);
    spring.retarget(1.0);
    spring.tick(0.02);
    let lift = spring.value().clamp(0.0, 1.0);
    assert!(lift > 0.01 && lift < 0.9, "{lift}");
    let mut fades = std::collections::HashMap::new();
    fades.insert(1usize, spring);

    scene.hover = Some(1);
    scene.rebuild(RebuildCtx {
        catalog: &store,
        filtered: &filtered,
        atlas: &mut atlas,
        pool: &pool,
        palette: &palette,
        uploads: &uploads,
        hover_fades: Some(&fades),
    });
    let carriers = scene
        .render
        .instances
        .iter()
        .filter(|inst| inst.params[1] > 0.0 && (inst.border[3] - lift).abs() < 1e-4)
        .count();
    assert_eq!(carriers, 1);

    scene.hover = None;
    scene.rebuild(RebuildCtx {
        catalog: &store,
        filtered: &filtered,
        atlas: &mut atlas,
        pool: &pool,
        palette: &palette,
        uploads: &uploads,
        hover_fades: Some(&fades),
    });
    let lingering = scene.render.instances.iter().filter(|inst| inst.params[1] > 0.0).count();
    assert_eq!(lingering, 1);

    scene.hover = Some(1);
    scene.rebuild(RebuildCtx {
        catalog: &store,
        filtered: &filtered,
        atlas: &mut atlas,
        pool: &pool,
        palette: &palette,
        uploads: &uploads,
        hover_fades: None,
    });
    assert!(
        scene
            .render
            .instances
            .iter()
            .any(|inst| inst.params[1] > 0.0 && (inst.border[3] - 1.0).abs() < 1e-4)
    );
}

#[test]
fn grid_thumbnail_spacing_matches_configured_gaps() {
    use std::sync::{Arc, Mutex};

    let uploads: UploadQueue = Arc::new(Mutex::new(Vec::new()));
    let (tx, _rx) = futures_channel::mpsc::unbounded();
    let pool = DecodePool::start(uploads.clone(), tx, 0);
    let mut store = Catalog::default();
    for index in 0..8 {
        store.items.push(Wallpaper {
            key: format!("wall-{index}"),
            kind: WallpaperKind::Static,
            width: 100,
            height: 60,
            ..Default::default()
        });
    }
    let filtered: Vec<u32> = (0..8).collect();
    let palette = Palette::default();
    let mut map = AtlasMap::new(8);
    for index in 0..8 {
        map.near.acquire(index);
        map.near.mark_ready(index);
    }
    let mut atlas = Some(map);
    for gap in [0.0, 8.0] {
        for border in [0.0, 2.0, 8.0] {
            for hover in [None, Some(1)] {
                let mut scene = test_scene(Mode::Grid);
                scene.viewport = (1280.0, 720.0);
                scene.gp.gap_x = gap;
                scene.gp.gap_y = gap;
                scene.gp.corners = CardCorners::default();
                scene.gp.border_width = border;
                scene.hover = hover;
                scene.motion.entrance.snap(1.0);
                scene.motion.visibility.snap(1.0);
                scene.rebuild(RebuildCtx {
                    catalog: &store,
                    filtered: &filtered,
                    atlas: &mut atlas,
                    pool: &pool,
                    palette: &palette,
                    uploads: &uploads,
                    hover_fades: None,
                });
                let bodies: Vec<_> = scene
                    .render
                    .instances
                    .iter()
                    .filter(|instance| instance.misc[0] == 1)
                    .collect();
                assert_eq!(bodies.len(), 8);
                let a = bodies[0].rect;
                let right = bodies[1].rect;
                let below = bodies[4].rect;
                assert_eq!(right[0] - right[2] - (a[0] + a[2]), gap);
                assert_eq!(below[1] - below[3] - (a[1] + a[3]), gap);
                for (index, body) in bodies.iter().enumerate() {
                    let hit = scene.render.hits.iter().find(|hit| hit.index == index).unwrap();
                    assert_eq!(body.rect, [hit.cx, hit.cy, hit.hw, hit.hh]);
                }
                if hover.is_some() {
                    assert_eq!(bodies[1].params[1], border);
                    assert_eq!(bodies[1].border[3], 1.0);
                }
            }
        }
    }
}

fn hand_fixture(
    count: usize,
) -> (Catalog, Vec<u32>, UploadQueue, DecodePool, Palette, Option<AtlasMap>) {
    use std::sync::{Arc, Mutex};
    let uploads: UploadQueue = Arc::new(Mutex::new(Vec::new()));
    let (tx, _rx) = futures_channel::mpsc::unbounded();
    let pool = DecodePool::start(uploads.clone(), tx, 0);
    let mut store = Catalog::default();
    for index in 0..count {
        store.items.push(Wallpaper {
            key: format!("wall-{index}"),
            name: format!("wall-{index}"),
            thumb: format!("/tmp/wall-{index}.webp"),
            path: format!("/tmp/wall-{index}.webp"),
            kind: WallpaperKind::Static,
            ..Default::default()
        });
    }
    let filtered: Vec<u32> = (0..count as u32).collect();
    let atlas = Some(AtlasMap::new(store.items.len()));
    (store, filtered, uploads, pool, Palette::default(), atlas)
}

fn hand_scene() -> SceneCore {
    let mut scene = test_scene(Mode::Hand);
    scene.viewport = (1440.0, 900.0);
    scene.motion.entrance.snap(1.0);
    scene.motion.visibility.snap(1.0);
    scene
}

fn hand_frames(
    scene: &mut SceneCore,
    fixture: &mut (Catalog, Vec<u32>, UploadQueue, DecodePool, Palette, Option<AtlasMap>),
    now: &mut Instant,
    frames: usize,
) {
    for _ in 0..frames {
        *now += Duration::from_millis(16);
        scene.tick(
            *now,
            RebuildCtx {
                catalog: &fixture.0,
                filtered: &fixture.1,
                atlas: &mut fixture.5,
                pool: &fixture.3,
                palette: &fixture.4,
                uploads: &fixture.2,
                hover_fades: None,
            },
        );
    }
}

#[test]
fn hand_select_inside_window_pops_without_dealing() {
    let mut scene = hand_scene();
    scene.set_current(2, 20);
    assert!(!scene.hand_dealing());
    assert_eq!(scene.current, 2);
    assert_eq!(scene.hand_offset(), 0);
    assert!(scene.card.selection.contains_key(&0));
    assert!(scene.card.selection.contains_key(&2));
    assert!(scene.is_animating());
}

#[test]
fn hand_select_outside_window_deals_and_settles() {
    let mut scene = hand_scene();
    let mut fixture = hand_fixture(20);
    let mut now = Instant::now();
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    let projected = |scene: &SceneCore| {
        scene
            .render
            .instances
            .iter()
            .filter(|inst| inst.misc[3] & crate::rendering::scene::PROJECTED != 0)
            .count()
    };
    assert_eq!(projected(&scene), 5);
    assert!(scene.render.hits.iter().any(|hit| hit.index == 4));
    assert!(scene.render.chrome.len() == 1);

    scene.set_current(9, 20);
    assert!(scene.hand_dealing());
    assert_eq!(scene.hand_offset(), 5);
    assert_eq!(scene.current, 9);
    hand_frames(&mut scene, &mut fixture, &mut now, 3);
    assert!(scene.hand_dealing());
    assert!(scene.render.hits.is_empty());
    assert!(
        scene
            .render
            .instances
            .iter()
            .any(|inst| inst.misc[3] & crate::rendering::scene::PROJECTED != 0)
    );
    hand_frames(&mut scene, &mut fixture, &mut now, 400);
    assert!(!scene.hand_dealing());
    assert!(scene.render.hits.iter().any(|hit| hit.index == 9));
    assert_eq!(scene.visible_range(), (5, 9));
    hand_frames(&mut scene, &mut fixture, &mut now, 300);
    assert!(!scene.is_animating(), "held by {}", scene.anim_reason());
}

#[test]
fn hand_deal_interrupt_retargets_without_snapping_home() {
    let mut scene = hand_scene();
    let mut fixture = hand_fixture(40);
    let mut now = Instant::now();
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    scene.set_current(12, 40);
    hand_frames(&mut scene, &mut fixture, &mut now, 10);
    scene.set_current(30, 40);
    assert!(scene.hand_dealing());
    assert_eq!(scene.hand_offset(), 30);
    hand_frames(&mut scene, &mut fixture, &mut now, 500);
    assert!(!scene.hand_dealing());
    assert!(scene.render.hits.iter().any(|hit| hit.index == 30));
}

#[test]
fn hand_flip_lands_face_on_with_back_panel() {
    let mut scene = hand_scene();
    let mut fixture = hand_fixture(12);
    let mut now = Instant::now();
    scene.set_card_flip_options(1200.0, true, true);
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    scene.toggle_flip(scene.current);
    assert!(scene.flip_open());
    hand_frames(&mut scene, &mut fixture, &mut now, 20);
    assert!(scene.render.back.is_none());
    hand_frames(&mut scene, &mut fixture, &mut now, 34);
    let plates = scene
        .render
        .instances
        .iter()
        .filter(|inst| {
            inst.misc[3] & crate::rendering::scene::PROJECTED != 0
                && inst.misc[0] == 0
                && (inst.fill[3] - 1.0).abs() < 1e-3
        })
        .count();
    assert_eq!(plates, 6, "all ribbons lie flat on their backs before the panel surface fades in");
    assert!(scene.render.back.as_ref().is_some_and(|back| back.progress < 0.8));
    assert!(
        scene
            .render
            .instances
            .iter()
            .filter(|inst| inst.misc[3] & crate::rendering::scene::PROJECTED != 0)
            .count()
            > 5
    );
    hand_frames(&mut scene, &mut fixture, &mut now, 120);
    let back = scene.render.back.clone().expect("back panel after landing");
    assert!(back.progress >= 0.999);
    assert!(back.embedded && back.coordinated_flip && back.picture_behind);
    let hit = scene.render.hits.iter().find(|hit| hit.index == scene.current).expect("flipped hit");
    assert!((hit.cx - back.cx).abs() < 1.0 && (hit.cy - back.cy).abs() < 1.0);
    let expected_hw = 168.0 * 0.5 * 1.02 * 1700.0 / (1700.0 - 170.0);
    assert!((back.hw - expected_hw).abs() < expected_hw * 0.08, "{} vs {expected_hw}", back.hw);
    scene.close_flip();
    hand_frames(&mut scene, &mut fixture, &mut now, 160);
    assert!(scene.flipped().is_none());
    assert!(scene.render.back.is_none());
    hand_frames(&mut scene, &mut fixture, &mut now, 200);
    assert!(!scene.is_animating(), "held by {}", scene.anim_reason());
}

#[test]
fn hand_rig_follows_pointer_and_settles() {
    let mut scene = hand_scene();
    let mut fixture = hand_fixture(8);
    let mut now = Instant::now();
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    scene.hand_pointer(1440.0, 0.0, None);
    assert!(scene.is_animating());
    hand_frames(&mut scene, &mut fixture, &mut now, 200);
    let (rx, ry) = scene.hand_rig();
    assert!((rx - 2.25).abs() < 0.1 && (ry - 3.0).abs() < 0.1);
    assert!(!scene.is_animating(), "held by {}", scene.anim_reason());
    scene.hand_drag_start(700.0, 450.0);
    scene.hand_pointer(760.0, 450.0, None);
    assert!(scene.hand_dragging());
    assert!(scene.hand_rig().1 > ry + 10.0);
    scene.hand_drag_end();
    hand_frames(&mut scene, &mut fixture, &mut now, 300);
    assert!(!scene.is_animating(), "held by {}", scene.anim_reason());
    assert!((scene.hand_rig().1 - 3.0).abs() < 0.1);
}

#[test]
fn hand_hover_lift_settles_and_bob_holds_frames() {
    let mut scene = hand_scene();
    let mut fixture = hand_fixture(8);
    let mut now = Instant::now();
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    scene.hand_pointer(700.0, 450.0, Some(3));
    hand_frames(&mut scene, &mut fixture, &mut now, 100);
    scene.hand_pointer(700.0, 450.0, None);
    hand_frames(&mut scene, &mut fixture, &mut now, 200);
    assert!(!scene.is_animating(), "held by {}", scene.anim_reason());
    let mut bob = scene.xp_target;
    bob.hand.bob = true;
    scene.set_params(scene.sp_target, scene.gp_target, scene.hp_target, &bob, false);
    hand_frames(&mut scene, &mut fixture, &mut now, 1);
    assert!(scene.is_animating());
    assert_eq!(scene.anim_reason(), "hand_bob");
}

#[test]
fn hand_filter_storm_deals_from_shown_cards() {
    let mut scene = hand_scene();
    let mut fixture = hand_fixture(12);
    let mut now = Instant::now();
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    scene.reset_to_index(0, 12);
    scene.filter_storm(None);
    assert!(scene.hand_dealing());
    assert_eq!(scene.filter_flip_count(), 0);
    hand_frames(&mut scene, &mut fixture, &mut now, 400);
    assert!(!scene.hand_dealing());
}

#[test]
fn hand_backdrop_crossfades_between_selections() {
    let mut scene = hand_scene();
    let mut fixture = hand_fixture(8);
    let mut now = Instant::now();
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    let (store, prev, fade) = scene.hand_backdrop_target(0, false);
    assert_eq!((store, prev, fade), (None, None, 1.0));
    let (store, prev, fade) = scene.hand_backdrop_target(0, true);
    assert_eq!((store, prev, fade), (Some(0), None, 1.0));
    assert!(!scene.is_animating(), "held by {}", scene.anim_reason());
    let (store, prev, fade) = scene.hand_backdrop_target(3, true);
    assert_eq!((store, prev), (Some(3), Some(0)));
    assert_eq!(fade, 0.0);
    assert!(scene.is_animating());
    assert_eq!(scene.anim_reason(), "hand_backdrop");
    hand_frames(&mut scene, &mut fixture, &mut now, 10);
    let (_, prev, fade) = scene.hand_backdrop_target(3, true);
    assert_eq!(prev, Some(0));
    assert!(fade > 0.2 && fade < 0.6, "{fade}");
    hand_frames(&mut scene, &mut fixture, &mut now, 60);
    let (_, prev, fade) = scene.hand_backdrop_target(3, true);
    assert_eq!((prev, fade), (None, 1.0));
    assert!(!scene.is_animating(), "held by {}", scene.anim_reason());
}

#[test]
fn hand_reveal_flips_every_slat_into_the_row_and_back_home() {
    use super::model::{Face, Layout, RevealPhase};
    use crate::rendering::scene::{GHOST, PROJECTED};
    let mut scene = hand_scene();
    let mut fixture = hand_fixture(12);
    let mut now = Instant::now();
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    let cur = fixture.1[scene.current] as usize;
    let atlas = fixture.5.as_mut().expect("atlas");
    atlas.near.acquire(cur);
    atlas.near.mark_ready(cur);
    scene.touch();
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    assert!(!scene.is_animating(), "held by {}", scene.anim_reason());
    let fan: Vec<(usize, i32, i32)> =
        scene.render.hits.iter().map(|hit| (hit.index, hit.cx as i32, hit.cy as i32)).collect();
    let fan_left = scene.render.hits.iter().map(|hit| hit.cx).fold(f32::MAX, f32::min);
    scene.hand_reveal_toggle();
    assert!(scene.hand_reveal_open() && scene.is_animating());
    hand_frames(&mut scene, &mut fixture, &mut now, 1);
    assert_eq!(scene.anim_reason(), "hand_reveal");
    let (phase, turns, faces, layouts) = scene.hand_reveal_phase().expect("reveal");
    assert_eq!((phase, turns, layouts), (RevealPhase::Turning, 0, (Layout::Fan, Layout::Row)));
    assert_eq!(faces, [Face::Card, Face::Slice { store: cur, column: false }]);
    let mut slices = 0;
    let mut ghosts = 0;
    let mut moved_before_flat = false;
    while matches!(scene.hand_reveal_phase(), Some((RevealPhase::Turning, ..))) {
        hand_frames(&mut scene, &mut fixture, &mut now, 1);
        let last = scene.render.hits.iter().find(|hit| hit.index == fan[0].0);
        if let Some(hit) = last
            && (hit.cx as i32 - fan[0].1).abs() > 4
            && scene.render.instances.iter().any(|inst| inst.misc[3] & GHOST != 0)
        {
            moved_before_flat = true;
        }
        ghosts = ghosts
            .max(scene.render.instances.iter().filter(|inst| inst.misc[3] & GHOST != 0).count());
        slices = slices.max(
            scene
                .render
                .instances
                .iter()
                .filter(|inst| {
                    inst.misc[3] & PROJECTED != 0
                        && inst.misc[3] & GHOST == 0
                        && inst.misc[0] == 1
                        && inst.crop[2] < 0.3
                })
                .count(),
        );
    }
    assert_eq!(slices, 5, "every slat ends up showing its column of the preview");
    assert!(ghosts >= 2, "turning slats trail ghosts: {ghosts}");
    assert!(moved_before_flat, "slats travel while they flip, not before");
    assert!(scene.render.instances.iter().all(|inst| inst.misc[3] & GHOST == 0));
    assert!(matches!(
        scene.hand_reveal_phase(),
        Some((RevealPhase::Held, 1, _, (Layout::Row, Layout::Row)))
    ));
    assert!(!scene.is_animating(), "held by {}", scene.anim_reason());
    let mut xs: Vec<f32> = scene.render.hits.iter().map(|hit| hit.cx).collect();
    xs.sort_by(f32::total_cmp);
    let gaps: Vec<f32> = xs.windows(2).map(|pair| pair[1] - pair[0]).collect();
    assert!(gaps.iter().all(|gap| (gap - gaps[0]).abs() < 2.0), "orderly row: {gaps:?}");
    let right = scene.render.hits.iter().map(|hit| hit.cx + hit.hw).fold(f32::MIN, f32::max);
    assert!(xs[0] < fan_left && right < 1440.0, "the row spreads wider than the fan yet fits");
    let crops: Vec<[f32; 4]> = scene
        .render
        .instances
        .iter()
        .filter(|inst| inst.misc[3] & PROJECTED != 0)
        .map(|inst| inst.crop)
        .collect();
    assert_eq!(crops.len(), 5);
    assert!(crops.iter().all(|crop| (crop[3] - 1.0).abs() < 1e-4 && crop[2] < 0.25));
    let frame = crate::frontend::scene::hand::reveal_frame(1440.0, 900.0, false);
    let left = scene.render.hits.iter().map(|hit| hit.cx - hit.hw).fold(f32::MAX, f32::min);
    assert!(
        (right - left - frame.0).abs() < 12.0,
        "slats span the 16:9 frame: {} vs {}",
        right - left,
        frame.0
    );
    assert!(scene.render.hits.iter().all(|hit| hit.index == scene.current));
    scene.hand_pointer(700.0, 450.0, Some(scene.current + 1));
    hand_frames(&mut scene, &mut fixture, &mut now, 30);
    assert!(!scene.is_animating(), "hover does not lift a revealed slat: {}", scene.anim_reason());
    let next = scene.current + 1;
    let next_store = fixture.1[next] as usize;
    let atlas = fixture.5.as_mut().expect("atlas");
    atlas.near.acquire(next_store);
    atlas.near.mark_ready(next_store);
    scene.set_current(next, 12);
    assert!(scene.card.selection.values().all(Spring::settled), "selection snaps while revealed");
    hand_frames(&mut scene, &mut fixture, &mut now, 8);
    let (phase, turns, faces, layouts) = scene.hand_reveal_phase().expect("reveal");
    assert_eq!((phase, turns, layouts), (RevealPhase::Turning, 1, (Layout::Row, Layout::Row)));
    assert_eq!(
        faces,
        [
            Face::Slice { store: next_store, column: false },
            Face::Slice { store: cur, column: false }
        ]
    );
    hand_frames(&mut scene, &mut fixture, &mut now, 90);
    assert!(matches!(scene.hand_reveal_phase(), Some((RevealPhase::Held, 2, _, _))));
    assert!(!scene.is_animating(), "held by {}", scene.anim_reason());
    scene.hand_reveal_toggle();
    assert!(!scene.hand_reveal_open());
    hand_frames(&mut scene, &mut fixture, &mut now, 8);
    let (phase, turns, faces, layouts) = scene.hand_reveal_phase().expect("reveal");
    assert_eq!((phase, turns, layouts), (RevealPhase::Turning, 2, (Layout::Row, Layout::Fan)));
    assert_eq!(faces[1], Face::Card);
    hand_frames(&mut scene, &mut fixture, &mut now, 20);
    assert!(scene.hand_reveal_phase().is_some(), "still travelling home");
    hand_frames(&mut scene, &mut fixture, &mut now, 40);
    assert!(scene.hand_reveal_phase().is_none(), "no separate release stage");
    assert!(!scene.is_animating(), "held by {}", scene.anim_reason());
    let back: Vec<(usize, i32, i32)> =
        scene.render.hits.iter().map(|hit| (hit.index, hit.cx as i32, hit.cy as i32)).collect();
    let moved: Vec<_> = fan
        .iter()
        .zip(&back)
        .filter(|(a, b)| a.0 != b.0 || (a.1 - b.1).abs() > 2 || (a.2 - b.2).abs() > 2)
        .collect();
    assert!(
        moved.len() <= 2,
        "the fan returns to the press state except the moved selection: {moved:?}"
    );
}

#[test]
fn hand_reveal_stacks_tall_wallpapers_in_a_column_once_the_tall_tile_lands() {
    use super::model::{Face, Layout, RevealPhase};
    use crate::rendering::scene::PROJECTED;
    let mut scene = hand_scene();
    let mut fixture = hand_fixture(12);
    let mut now = Instant::now();
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    let cur = fixture.1[scene.current] as usize;
    fixture.0.items[cur].width = 1080;
    fixture.0.items[cur].height = 1920;
    assert!(fixture.0.items[cur].is_tall());
    let atlas = fixture.5.as_mut().expect("atlas");
    atlas.near.acquire(cur);
    atlas.near.mark_ready(cur);
    scene.touch();
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    assert!(!scene.tall_needs(cur), "nothing is requested while the fan is closed");
    scene.hand_reveal_toggle();
    assert!(scene.tall_needs(cur));
    assert!(!scene.tall_needs(cur), "one request per store");
    hand_frames(&mut scene, &mut fixture, &mut now, 30);
    assert!(
        matches!(scene.hand_reveal_phase(), Some((RevealPhase::Held, 0, ..))),
        "waits for the tall tile: {:?}",
        scene.hand_reveal_phase()
    );
    assert!(!scene.is_animating() || scene.anim_reason() != "hand_reveal");
    scene.tall_ready(cur);
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    let (phase, _, faces, layouts) = scene.hand_reveal_phase().expect("reveal");
    assert_eq!((phase, layouts), (RevealPhase::Turning, (Layout::Fan, Layout::Column)));
    assert_eq!(faces[1], Face::Slice { store: cur, column: true });
    assert!(scene.hand_tall_in_use());
    hand_frames(&mut scene, &mut fixture, &mut now, 80);
    assert!(matches!(
        scene.hand_reveal_phase(),
        Some((RevealPhase::Held, 1, _, (Layout::Column, Layout::Column)))
    ));
    assert!(!scene.is_animating(), "held by {}", scene.anim_reason());
    let mut ys: Vec<f32> = scene.render.hits.iter().map(|hit| hit.cy).collect();
    ys.sort_by(f32::total_cmp);
    let gaps: Vec<f32> = ys.windows(2).map(|pair| pair[1] - pair[0]).collect();
    assert!(gaps.iter().all(|&gap| gap > 20.0 && (gap - gaps[0]).abs() < 2.0), "stack: {gaps:?}");
    let xs: Vec<f32> = scene.render.hits.iter().map(|hit| hit.cx).collect();
    assert!(xs.iter().all(|x| (x - xs[0]).abs() < 2.0), "one column: {xs:?}");
    let top = ys[0] - scene.render.hits[0].hh;
    assert!(top > 40.0 && ys[4] + scene.render.hits[0].hh < 900.0, "fits under the bar");
    let slices: Vec<&InstanceRaw> =
        scene.render.instances.iter().filter(|inst| inst.misc[3] & PROJECTED != 0).collect();
    assert_eq!(slices.len(), 5);
    assert!(
        slices.iter().all(|inst| inst.misc[0] == 3 && inst.crop[2] < 0.25 && inst.crop[3] > 0.8)
    );
    let fixture_failed = cur;
    scene.hand_reveal_toggle();
    hand_frames(&mut scene, &mut fixture, &mut now, 120);
    assert!(scene.hand_reveal_phase().is_none());
    scene.tall_failed(fixture_failed);
    scene.hand.tall_ready = None;
    scene.hand_reveal_toggle();
    hand_frames(&mut scene, &mut fixture, &mut now, 80);
    let (_, _, faces, layouts) = scene.hand_reveal_phase().expect("reveal");
    assert_eq!(layouts.1, Layout::Row, "a failed tall tile falls back to the row");
    assert!(faces.contains(&Face::Slice { store: cur, column: false }));
}

#[test]
fn hand_reveal_right_click_flips_the_preview_to_the_details_panel() {
    use super::model::{Face, RevealPhase};
    let mut scene = hand_scene();
    scene.set_card_flip_options(400.0, true, true);
    let mut fixture = hand_fixture(12);
    let mut now = Instant::now();
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    let cur = fixture.1[scene.current] as usize;
    let atlas = fixture.5.as_mut().expect("atlas");
    atlas.near.acquire(cur);
    atlas.near.mark_ready(cur);
    scene.touch();
    scene.hand_reveal_toggle();
    hand_frames(&mut scene, &mut fixture, &mut now, 90);
    assert!(matches!(scene.hand_reveal_phase(), Some((RevealPhase::Held, 1, ..))));
    assert!(scene.render.back.is_none());
    scene.toggle_flip(scene.current);
    assert!(scene.flip_open());
    hand_frames(&mut scene, &mut fixture, &mut now, 6);
    let (phase, turns, faces, _) = scene.hand_reveal_phase().expect("reveal");
    assert_eq!((phase, turns), (RevealPhase::Turning, 1));
    assert_eq!(faces[0], Face::Back { store: cur, column: false }, "the muted artwork turns in");
    hand_frames(&mut scene, &mut fixture, &mut now, 90);
    assert!(matches!(scene.hand_reveal_phase(), Some((RevealPhase::Held, 2, ..))));
    assert!(!scene.is_animating(), "held by {}", scene.anim_reason());
    let muted = scene
        .render
        .instances
        .iter()
        .filter(|inst| inst.misc[3] & crate::rendering::scene::MUTED != 0)
        .count();
    assert_eq!(muted, 5, "every slat shows the muted artwork behind the panel");
    assert!(
        scene
            .render
            .instances
            .iter()
            .filter(|inst| inst.misc[3] & crate::rendering::scene::MUTED != 0)
            .all(|inst| inst.misc[0] == 1)
    );
    let back = scene.render.back.clone().expect("details panel over the preview");
    assert!(back.progress >= 0.999 && back.embedded && back.picture_behind);
    let frame = crate::frontend::scene::hand::reveal_frame(1440.0, 900.0, false);
    assert!(
        (back.hw * 2.0 - frame.0).abs() < 16.0,
        "panel spans the frame: {} vs {}",
        back.hw * 2.0,
        frame.0
    );
    assert!(scene.render.hits.iter().all(|hit| hit.index == scene.current));
    scene.close_flip();
    hand_frames(&mut scene, &mut fixture, &mut now, 6);
    let (phase, _, faces, _) = scene.hand_reveal_phase().expect("reveal");
    assert_eq!(phase, RevealPhase::Turning);
    assert!(faces.contains(&Face::Slice { store: cur, column: false }));
    hand_frames(&mut scene, &mut fixture, &mut now, 120);
    assert!(scene.render.back.is_none() && !scene.flip_open());
    assert!(matches!(scene.hand_reveal_phase(), Some((RevealPhase::Held, 3, ..))));
    assert!(!scene.is_animating(), "held by {}", scene.anim_reason());
    scene.toggle_flip(scene.current);
    hand_frames(&mut scene, &mut fixture, &mut now, 100);
    assert!(scene.render.back.is_some());
    scene.hand_reveal_toggle();
    hand_frames(&mut scene, &mut fixture, &mut now, 140);
    assert!(scene.hand_reveal_phase().is_none() && !scene.flip_open());
    assert!(scene.render.back.is_none(), "closing the reveal drops the details too");
    assert!(!scene.is_animating(), "held by {}", scene.anim_reason());
}

#[test]
fn hand_reveal_keep_mode_preserves_card_shape() {
    use super::model::RevealPhase;
    let mut scene = hand_scene();
    scene.xp.hand.reveal_fill = false;
    scene.xp_target.hand.reveal_fill = false;
    let mut fixture = hand_fixture(12);
    let mut now = Instant::now();
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    let cur = fixture.1[scene.current] as usize;
    let atlas = fixture.5.as_mut().expect("atlas");
    atlas.near.acquire(cur);
    atlas.near.mark_ready(cur);
    scene.touch();
    let card = scene.render.hits.iter().find(|hit| hit.index == 0).map(|hit| hit.hh).expect("card");
    scene.hand_reveal_toggle();
    hand_frames(&mut scene, &mut fixture, &mut now, 90);
    assert!(matches!(scene.hand_reveal_phase(), Some((RevealPhase::Held, 1, ..))));
    let slat = scene.render.hits[0];
    assert!(
        (slat.hh / slat.hw - 432.0 / 168.0).abs() < 0.15,
        "card aspect kept: {} x {}",
        slat.hw,
        slat.hh
    );
    assert!(slat.hh <= card + 2.0, "never larger than the card itself");
    let crops: Vec<[f32; 4]> = scene
        .render
        .instances
        .iter()
        .filter(|inst| inst.misc[3] & crate::rendering::scene::PROJECTED != 0)
        .map(|inst| inst.crop)
        .collect();
    assert!(crops.iter().all(|crop| crop[3] < 0.95), "a card-shaped row shows a centred band");
}

#[test]
fn hand_page_change_stays_revealed_and_lands_on_the_new_page() {
    let mut scene = hand_scene();
    let mut fixture = hand_fixture(12);
    let mut now = Instant::now();
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    scene.hand_reveal_toggle();
    hand_frames(&mut scene, &mut fixture, &mut now, 40);
    let store = fixture.1[9] as usize;
    let atlas = fixture.5.as_mut().expect("atlas");
    atlas.near.acquire(store);
    atlas.near.mark_ready(store);
    scene.set_current(9, 12);
    assert!(!scene.hand_dealing(), "no hand swap while revealed");
    assert!(scene.hand_reveal_phase().is_some());
    assert_eq!((scene.current, scene.hand_offset()), (9, 5));
    hand_frames(&mut scene, &mut fixture, &mut now, 120);
    assert!(!scene.is_animating(), "held by {}", scene.anim_reason());
    assert!(scene.render.hits.iter().all(|hit| hit.index == 9));
    scene.hand_reveal_toggle();
    hand_frames(&mut scene, &mut fixture, &mut now, 120);
    assert!(scene.hand_reveal_phase().is_none() && !scene.hand_dealing());
    let shown: Vec<usize> = scene.render.hits.iter().map(|hit| hit.index).collect();
    assert!(shown.contains(&9) && shown.contains(&5) && !shown.contains(&4), "{shown:?}");
    assert!(!scene.is_animating(), "held by {}", scene.anim_reason());
}

#[test]
fn hand_starts_on_the_middle_card() {
    let mut scene = hand_scene();
    scene.reset_to_index(0, 20);
    assert_eq!(scene.current, 2);
    assert_eq!(scene.hand_offset(), 0);
    scene.reset_to_index(7, 20);
    assert_eq!(scene.current, 7);
    assert_eq!(scene.hand_offset(), 5);
    scene.reset_to_index(0, 2);
    assert_eq!(scene.current, 1);
    let mut fresh = hand_scene();
    fresh.relayout(20);
    assert_eq!(fresh.current, 2);
    fresh.set_current(9, 20);
    fresh.relayout(20);
    assert_eq!(fresh.current, 9);
    assert!(fresh.hand_dealing());
}

#[test]
fn hand_push_holds_steady_while_selection_moves() {
    let mut scene = hand_scene();
    let mut fixture = hand_fixture(8);
    let mut now = Instant::now();
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    scene.set_current(3, 8);
    for _ in 0..40 {
        hand_frames(&mut scene, &mut fixture, &mut now, 1);
        let push = scene.hand_push();
        assert!(push > 0.97 && push <= 1.0, "push dipped to {push}");
    }
}

#[test]
fn hand_refans_around_the_selected_card() {
    let mut scene = hand_scene();
    let mut fixture = hand_fixture(8);
    let mut now = Instant::now();
    scene.reset_to_index(0, 8);
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    assert_eq!(scene.current, 2);
    let k = scene.hand_stage_scale();
    let spread = scene.xp.hand.spread * k * 1.24;
    let centre_slot = scene.hand_rest_pose(2, 5, 1.0);
    assert!(centre_slot.t[0].abs() < 1e-3);
    scene.set_current(3, 8);
    hand_frames(&mut scene, &mut fixture, &mut now, 200);
    let lifted = scene.hand_rest_pose(3, 5, 1.0);
    assert!(lifted.t[0].abs() < 1e-3 && lifted.t[2] > 0.0);
    let vacated = scene.hand_rest_pose(2, 5, 1.0);
    assert!((vacated.t[0] - 0.5 * spread).abs() < 1.0, "{} vs {}", vacated.t[0], 0.5 * spread);
    let edge = scene.hand_rest_pose(0, 5, 1.0);
    assert!((edge.t[0] + 1.5 * spread).abs() < 1.0);
}

fn hand_dump(scene: &SceneCore, label: &str) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    for inst in &scene.render.instances {
        if inst.misc[3] & crate::rendering::scene::PROJECTED == 0 {
            continue;
        }
        let q: Vec<String> = inst
            .quad_a
            .iter()
            .chain(&inst.quad_b)
            .chain(&inst.quad_w)
            .chain(&inst.quad_l)
            .map(|v| format!("{v:.3}"))
            .collect();
        let _ = writeln!(out, "{label} {} {} {}", inst.misc[3], inst.misc[0], q.join(" "));
    }
    out
}

#[test]
#[ignore = "manual geometry snapshot for the math swap"]
fn hand_geometry_snapshot() {
    let mut scene = hand_scene();
    let mut fixture = hand_fixture(20);
    let mut now = Instant::now();
    scene.reset_to_index(0, 20);
    hand_frames(&mut scene, &mut fixture, &mut now, 30);
    let mut dump = hand_dump(&scene, "rest");
    scene.hand_pointer(300.0, 200.0, Some(1));
    hand_frames(&mut scene, &mut fixture, &mut now, 12);
    dump.push_str(&hand_dump(&scene, "tilt"));
    scene.set_current(9, 20);
    hand_frames(&mut scene, &mut fixture, &mut now, 25);
    dump.push_str(&hand_dump(&scene, "deal_out"));
    hand_frames(&mut scene, &mut fixture, &mut now, 40);
    dump.push_str(&hand_dump(&scene, "deal_in"));
    hand_frames(&mut scene, &mut fixture, &mut now, 300);
    scene.toggle_flip(scene.current);
    hand_frames(&mut scene, &mut fixture, &mut now, 28);
    dump.push_str(&hand_dump(&scene, "flip_lift"));
    hand_frames(&mut scene, &mut fixture, &mut now, 30);
    dump.push_str(&hand_dump(&scene, "flip_land"));
    let path =
        std::env::var("SKWD_HAND_SNAPSHOT").unwrap_or_else(|_| "/tmp/hand-snapshot.txt".into());
    std::fs::write(&path, dump).unwrap();
}

#[test]
#[ignore = "manual rebuild benchmark for the math swap"]
fn hand_rebuild_bench() {
    let mut scene = hand_scene();
    let mut fixture = hand_fixture(40);
    let mut now = Instant::now();
    let mut ribbon = scene.xp_target;
    ribbon.hand.moves = [false, false, false, true, false];
    scene.set_params(scene.sp_target, scene.gp_target, scene.hp_target, &ribbon, false);
    scene.reset_to_index(0, 40);
    hand_frames(&mut scene, &mut fixture, &mut now, 10);
    let started = Instant::now();
    let mut frames = 0usize;
    for round in 0..20 {
        scene.set_current(if round % 2 == 0 { 30 } else { 5 }, 40);
        hand_frames(&mut scene, &mut fixture, &mut now, 100);
        frames += 100;
    }
    let deal_us = started.elapsed().as_secs_f64() * 1e6 / frames as f64;
    let started = Instant::now();
    scene.toggle_flip(scene.current);
    for _ in 0..40 {
        hand_frames(&mut scene, &mut fixture, &mut now, 100);
        scene.toggle_flip(scene.current);
    }
    let flip_us = started.elapsed().as_secs_f64() * 1e6 / 4000.0;
    eprintln!(
        "hand_bench deal_frame_us={deal_us:.2} flip_frame_us={flip_us:.2} instances={}",
        scene.render.instances.len()
    );
}

#[test]
fn hand_steep_cuts_still_land_every_ribbon_on_its_back() {
    use crate::frontend::scene::hand::{Cut, Variance};
    for (cut, variance) in [
        (Cut::Straight, Variance::None),
        (Cut::Steep, Variance::None),
        (Cut::Steep, Variance::Wild),
    ] {
        let mut scene = hand_scene();
        let mut fixture = hand_fixture(12);
        let mut now = Instant::now();
        let mut steep = scene.xp_target;
        steep.hand.cut = cut;
        steep.hand.variance = variance;
        scene.set_params(scene.sp_target, scene.gp_target, scene.hp_target, &steep, false);
        scene.reset_to_index(0, 12);
        hand_frames(&mut scene, &mut fixture, &mut now, 2);
        scene.toggle_flip(scene.current);
        hand_frames(&mut scene, &mut fixture, &mut now, 200);
        let plates = scene
            .render
            .instances
            .iter()
            .filter(|inst| {
                inst.misc[3] & crate::rendering::scene::PROJECTED != 0
                    && inst.misc[0] == 0
                    && (inst.fill[3] - 1.0).abs() < 1e-3
            })
            .count();
        assert_eq!(plates, 6, "{cut:?} {variance:?}");
        let k = scene.hand_stage_scale();
        let back = scene.render.back.clone().expect("panel");
        for inst in scene.render.instances.iter().filter(|inst| {
            inst.misc[3] & crate::rendering::scene::PROJECTED != 0
                && inst.misc[0] == 0
                && (inst.fill[3] - 1.0).abs() < 1e-3
                && inst.quad_l.iter().all(|v| v.abs() < 260.0 * k)
        }) {
            let ys = [inst.quad_a[1], inst.quad_a[3], inst.quad_b[1], inst.quad_b[3]];
            for y in ys {
                assert!(
                    (y - back.cy).abs() <= back.hh * 1.45 + 6.0,
                    "{cut:?}: interior plate corner {y} leaves the card around {} +-{}",
                    back.cy,
                    back.hh
                );
            }
        }
        let k = scene.hand_stage_scale();
        assert!(
            (back.hh - 216.0 * k * 1.02 * 1700.0 / 1530.0).abs() < 216.0 * k * 0.08,
            "{}",
            back.hh
        );
    }
}

#[test]
fn depth_cards_shrink_toward_edges_and_hits_follow_visible_bounds() {
    let render = rendered_scene_with_count_and_bar(
        Mode::Depth,
        SliceParams { visible_count: 11, ..base_sp() },
        base_gp(),
        1000,
        None,
    );
    assert!(render.hits.len() <= 24);
    let center = render.hits.iter().find(|hit| hit.index == 0).unwrap();
    let mut last = *center;
    for idx in 1..5 {
        let hit = render.hits.iter().find(|hit| hit.index == idx).unwrap();
        assert!(hit.hw < last.hw && hit.hh < last.hh);
        assert!(hit.cx > last.cx);
        assert!((hit.cy - center.cy).abs() < 0.01);
        assert!(hit.contains(hit.cx, hit.cy));
        let body =
            render.instances.iter().find(|body| (body.rect[0] - hit.cx).abs() < 0.01).unwrap();
        assert_eq!(body.rect, [hit.cx, hit.cy, hit.hw, hit.hh]);
        assert!(body.crop[0] >= 0.0 && body.crop[0] + body.crop[2] <= 1.001);
        assert!(body.crop[1] >= 0.0 && body.crop[1] + body.crop[3] <= 1.001);
        last = *hit;
    }
    assert_eq!(render.hits[0].index, 0);
}

#[test]
fn depth_restored_selection_centers_and_flips_inline() {
    let mut scene = test_scene(Mode::Depth);
    scene.restore_selection(42, 100, None);
    assert!(scene.layout_camera_anchor);
    scene.position_slice_camera(scene.current as f32);
    assert_eq!(scene.camera_pos(), 42.0);
    scene.toggle_flip(42);
    assert!(scene.flip_open());
    assert_eq!(scene.card.det_target, 0.0);
    scene.set_current(43, 100);
    assert!(!scene.flip_open());
}

#[test]
fn depth_position_drives_parallax_and_pointer_position_has_no_effect() {
    let render = |camera: f32, pointer: (f32, f32)| {
        rendered_scene_with_setup(
            Mode::Depth,
            SliceParams { visible_count: 11, slice_h: 520.0, ..base_sp() },
            base_gp(),
            11,
            None,
            0.0,
            |scene| {
                scene.current = 5;
                scene.camera.snap(camera);
                scene.last_mouse = Some(pointer);
            },
        )
    };
    let rest = render(5.0, (0.0, 0.0));
    let pointer_moved = render(5.0, (1200.0, 700.0));
    let scrolled = render(6.0, (0.0, 0.0));
    let get = |snap: &RenderSnapshot, idx: usize| {
        let hit = *snap.hits.iter().find(|hit| hit.index == idx).unwrap();
        let body = *snap
            .instances
            .iter()
            .find(|body| body.misc[0] > 0 && (body.rect[0] - hit.cx).abs() < 0.01)
            .unwrap();
        (hit, body)
    };
    let center = get(&rest, 5).0;
    for distance in 1..=4 {
        let left = get(&rest, 5 - distance).0;
        let right = get(&rest, 5 + distance).0;
        assert!((left.hw - right.hw).abs() < 0.01);
        assert!((left.hh - right.hh).abs() < 0.01);
        assert!((left.cx + right.cx - 2.0 * center.cx).abs() < 0.01);
    }
    for idx in 2..=8 {
        let (_, before) = get(&rest, idx);
        let (_, after) = get(&pointer_moved, idx);
        assert_eq!(before.rect, after.rect);
        assert_eq!(before.crop, after.crop);
    }
    let (_, middle) = get(&rest, 5);
    let (after, art_after) = get(&scrolled, 5);
    assert!(after.cx < center.cx);
    let mid_crop = middle.crop[0] + middle.crop[2] * 0.5;
    let shifted_crop = art_after.crop[0] + art_after.crop[2] * 0.5;
    assert!(shifted_crop < mid_crop - middle.crop[2] * 0.05);
    assert!((middle.crop[1] + middle.crop[3] * 0.5 - 0.5).abs() < 0.01);
}

#[test]
fn depth_scroll_parallax_reaches_visual_completion_and_idles() {
    let mut scene = test_scene(Mode::Depth);
    scene.viewport = (1200.0, 700.0);
    scene.motion.entrance.snap(1.0);
    scene.motion.needs_frame = false;
    assert!(!scene.is_animating());
    scene.set_current(5, 20);
    scene.position_slice_camera(5.0);
    scene.motion.needs_frame = false;
    assert!(scene.is_animating());
    rebuild_collection_test(&mut scene, 20, 180);
    assert!((scene.camera.x - 5.0).abs() * 300.0 < 0.5);
    assert!(!scene.is_animating());
}

#[test]
fn depth_parallax_keeps_moving_through_both_fading_edges_at_every_card_count() {
    for visible_count in [3, 5, 8, 12, 21] {
        let radius = (visible_count as f32 - 1.0) * 0.5;
        for direction in [-1.0, 1.0] {
            let mut previous = None;
            for fade_distance in [0.1, 0.4, 0.7, 0.95] {
                let distance = direction * (radius + fade_distance);
                let render = rendered_scene_with_setup(
                    Mode::Depth,
                    SliceParams { visible_count, slice_h: 520.0, ..base_sp() },
                    base_gp(),
                    31,
                    None,
                    0.0,
                    |scene| {
                        scene.current = 15;
                        scene.camera.snap(15.0 - distance);
                    },
                );
                let hit = render.hits.iter().find(|hit| hit.index == 15).unwrap();
                let body = render
                    .instances
                    .iter()
                    .find(|body| body.misc[0] > 0 && (body.rect[0] - hit.cx).abs() < 0.01)
                    .unwrap();
                let pan = body.crop[0] / (1.0 - body.crop[2]);
                assert!(
                    pan > 0.0 && pan < 1.0,
                    "count={visible_count}, distance={distance}, pan={pan}"
                );
                if let Some(before) = previous {
                    assert!(
                        (pan - before) * direction > 0.001,
                        "image stopped at count={visible_count}, distance={distance}"
                    );
                }
                previous = Some(pan);
            }
        }
    }
}

#[test]
fn optional_parallax_changes_art_without_moving_cards_or_hiding_badges() {
    for mode in [Mode::Slices, Mode::Hex] {
        let render = |enabled| {
            rendered_scene_with_setup(mode, base_sp(), base_gp(), 31, None, 0.0, |scene| {
                scene.sp.parallax = enabled;
                scene.sp_target = scene.sp;
                scene.hp.parallax = enabled;
                scene.hp_target = scene.hp;
                scene.camera.snap(250.0);
            })
        };
        let off = render(false);
        let on = render(true);
        assert_eq!(off.hits.len(), on.hits.len());
        for (a, b) in off.hits.iter().zip(&on.hits) {
            assert_eq!((a.index, a.cx, a.cy, a.hw, a.hh), (b.index, b.cx, b.cy, b.hw, b.hh));
        }
        for (a, b) in off.chrome.iter().zip(&on.chrome) {
            assert_eq!(a.opacity, b.opacity);
        }
        let off_art: Vec<_> = off.instances.iter().filter(|body| body.misc[0] > 0).collect();
        let on_art: Vec<_> = on.instances.iter().filter(|body| body.misc[0] > 0).collect();
        assert!(!off_art.is_empty());
        assert_eq!(off_art.len(), on_art.len());
        assert!(off_art.iter().zip(on_art).all(|(a, b)| a.crop != b.crop));
    }
}

#[test]
fn optional_parallax_keeps_moving_at_visible_layout_edges() {
    for mode in [Mode::Slices, Mode::Hex] {
        for side in [-1.0, 1.0] {
            let mut last_pan = None;
            for step in [0.0, 10.0, 20.0] {
                let render =
                    rendered_scene_with_setup(mode, base_sp(), base_gp(), 31, None, 0.0, |scene| {
                        scene.sp.parallax = true;
                        scene.sp_target = scene.sp;
                        scene.hp.parallax = true;
                        scene.hp_target = scene.hp;
                        let distance = match mode {
                            Mode::Slices => 320.0 - step,
                            _ => 620.0 - step,
                        } * side;
                        let item_center = match mode {
                            Mode::Slices => 208.0 + 4.0 * 88.0 + 40.0,
                            _ => scene.hp.column_x(5 / scene.hp.rows),
                        };
                        scene.camera.snap(item_center - distance);
                    });
                let hit = render.hits.iter().find(|hit| hit.index == 5).unwrap();
                let body = render
                    .instances
                    .iter()
                    .find(|body| {
                        body.misc[0] > 0
                            && (body.rect[0] - hit.cx).abs() < 0.01
                            && (body.rect[1] - hit.cy).abs() < 0.01
                    })
                    .unwrap();
                let pan = body.crop[0] / (1.0 - body.crop[2]);
                assert!(pan > 0.0 && pan < 1.0, "{mode:?} edge crop saturated: {pan}");
                if let Some(before) = last_pan {
                    assert!((before - pan) * side > 0.001, "{mode:?} edge crop stopped");
                }
                last_pan = Some(pan);
            }
        }
    }
}

fn rebuild_collection_test(scene: &mut SceneCore, count: usize, frames: usize) {
    use std::sync::{Arc, Mutex};
    let uploads: UploadQueue = Arc::new(Mutex::new(Vec::new()));
    let (tx, _rx) = futures_channel::mpsc::unbounded();
    let pool = DecodePool::start(uploads.clone(), tx, 0);
    let catalog = Catalog {
        items: (0..count)
            .map(|idx| Wallpaper {
                key: format!("collection-{idx}"),
                width: 1920,
                height: 1080,
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };
    let filtered: Vec<u32> = (0..count as u32).collect();
    let mut atlas = None;
    let palette = Palette::default();
    let start = scene.motion.last_tick.unwrap_or_else(Instant::now);
    for frame in 1..=frames {
        scene.tick(
            start + Duration::from_secs_f32(frame as f32 / 60.0),
            RebuildCtx {
                catalog: &catalog,
                filtered: &filtered,
                atlas: &mut atlas,
                pool: &pool,
                palette: &palette,
                uploads: &uploads,
                hover_fades: None,
            },
        );
    }
}

#[test]
fn collection_stack_is_bounded_and_visible_at_library_edges() {
    for viewport in [(1440.0, 900.0), (420.0, 760.0), (900.0, 420.0)] {
        for count in [0usize, 1, 3, 100_000] {
            for idx in [0, count / 2, count.saturating_sub(1)] {
                let mut scene = test_scene(Mode::Collection);
                scene.viewport = viewport;
                scene.reset_to_index(idx, count);
                rebuild_collection_test(&mut scene, count, 120);
                assert!(!scene.is_animating());
                assert!(scene.render.hits.len() <= 9);
                if count == 0 {
                    assert!(scene.render.hits.is_empty());
                    continue;
                }
                let hit = scene.render.hits.iter().find(|hit| hit.index == idx).unwrap();
                let quad = hit.quad.unwrap();
                let top = ((quad[0][0] + quad[1][0]) * 0.5, quad[0][1] + 5.0);
                assert!(top.0 > 0.0 && top.0 < viewport.0);
                assert!(top.1 > 0.0 && top.1 < viewport.1);
                assert_eq!(
                    scene.render.hits.iter().find(|h| h.contains(top.0, top.1)).unwrap().index,
                    idx
                );
                assert!(scene.render.chrome.iter().all(|chrome| chrome.opacity == 0.0));
            }
        }
    }
}

#[test]
fn collection_expands_to_landscape_and_closes_without_idle_frames() {
    let mut scene = test_scene(Mode::Collection);
    scene.viewport = (1440.0, 900.0);
    scene.reset_to_index(10, 30);
    rebuild_collection_test(&mut scene, 30, 120);
    scene.select_collection(10, 30);
    assert!(scene.is_animating());
    rebuild_collection_test(&mut scene, 30, 90);
    assert!(!scene.is_animating());
    assert!(scene.collection_open.x >= 0.999);
    let hit = &scene.render.hits[0];
    assert_eq!(hit.index, 10);
    assert!((hit.hw / hit.hh - 16.0 / 9.0).abs() < 0.02);
    assert_eq!(scene.render.chrome.iter().filter(|c| c.opacity > 0.0).count(), 1);
    assert!(scene.close_collection());
    rebuild_collection_test(&mut scene, 30, 90);
    assert!(!scene.is_animating());
    assert_eq!(scene.collection_card, None);
    assert!(scene.collection_open.x <= 0.001);
}

#[test]
fn collection_navigation_and_filter_reset_cancel_expansion() {
    let mut scene = test_scene(Mode::Collection);
    scene.viewport = (1440.0, 900.0);
    scene.select_collection(20, 100);
    rebuild_collection_test(&mut scene, 100, 120);
    scene.slice_scroll(-1.0, 100);
    assert!(scene.current > 20);
    rebuild_collection_test(&mut scene, 100, 120);
    assert!(!scene.is_animating());
    assert_eq!(scene.collection_card, None);
    scene.select_collection(30, 100);
    scene.reset_to_index(0, 2);
    rebuild_collection_test(&mut scene, 2, 120);
    assert_eq!(scene.collection_open.x, 0.0);
    assert_eq!(scene.collection_card, None);
    assert!(scene.render.hits.iter().all(|hit| hit.index < 2));
    scene.select_collection(0, 2);
    scene.set_mode(Mode::Slices);
    assert_eq!(scene.collection_open.x, 0.0);
    assert_eq!(scene.collection_card, None);
}

#[test]
fn depth_falloff_and_spacing_change_rendered_cards() {
    let render = |falloff, spacing| {
        rendered_scene_with_setup(
            Mode::Depth,
            SliceParams { visible_count: 7, ..base_sp() },
            base_gp(),
            30,
            None,
            0.0,
            |scene| {
                scene.xp.depth.falloff = falloff;
                scene.xp.depth.spacing = spacing;
                scene.xp_target = scene.xp;
            },
        )
    };
    let flat = render(0.0, 140.0);
    let deep = render(0.8, 140.0);
    let wide = render(0.0, 220.0);
    let hit = |r: &RenderSnapshot, idx| {
        let h = r.hits.iter().find(|h| h.index == idx).unwrap();
        (h.cx, h.hw)
    };
    assert!((hit(&flat, 0).1 - hit(&flat, 2).1).abs() < 0.1);
    assert!(hit(&deep, 2).1 < hit(&deep, 0).1 * 0.5);
    assert!(hit(&wide, 1).0 - hit(&wide, 0).0 > hit(&flat, 1).0 - hit(&flat, 0).0);
}

#[test]
fn collection_settings_change_geometry_and_settle() {
    let mut scene = test_scene(Mode::Collection);
    scene.viewport = (1440.0, 900.0);
    scene.reset_to_index(10, 30);
    rebuild_collection_test(&mut scene, 30, 120);
    let before = *scene.render.hits.iter().find(|h| h.index == 10).unwrap();
    let mut params = scene.xp;
    params.collection.size = 25.0;
    params.collection.tilt = 0.0;
    params.collection.count = 3.0;
    params.collection.corners = CardCorners::round(30.0);
    scene.set_params(scene.sp, scene.gp, scene.hp, &params, true);
    rebuild_collection_test(&mut scene, 30, 180);
    assert!(!scene.is_animating());
    let after = scene.render.hits.iter().find(|h| h.index == 10).unwrap();
    assert!(after.hw < before.hw);
    assert!((after.hw - after.hh).abs() < 0.1);
    assert_eq!(scene.render.hits.len(), 3);
    assert!(scene.render.instances.iter().any(|i| i.radii == [30.0; 4]));
}

#[test]
fn layout_motion_speed_changes_navigation_and_collection_opening() {
    for mode in [Mode::Depth, Mode::Collection] {
        let mut slow = test_scene(mode);
        let mut fast = test_scene(mode);
        for (scene, speed) in [(&mut slow, 25.0), (&mut fast, 300.0)] {
            let mut params = scene.xp;
            params.depth.navigation_ms = 25000.0 / speed;
            params.collection.speed = speed;
            scene.set_params(scene.sp, scene.gp, scene.hp, &params, false);
            scene.camera.retarget(10.0);
            scene.collection_open.retarget(1.0);
            scene.camera.tick(0.05);
            scene.collection_open.tick(0.05);
        }
        assert!(fast.camera.x > slow.camera.x);
        assert!(fast.collection_open.x > slow.collection_open.x);
        for _ in 0..300 {
            fast.camera.tick(1.0 / 60.0);
            slow.camera.tick(1.0 / 60.0);
        }
        assert!((slow.camera.x - 10.0).abs() < 0.001);
        assert!((fast.camera.x - 10.0).abs() < 0.001);
    }
}

#[test]
fn depth_camera_finishes_below_a_pixel_without_a_final_snap() {
    for duration in [83.0, 250.0, 1000.0, 4000.0] {
        let mut scene = test_scene(Mode::Depth);
        let mut params = scene.xp;
        params.depth.navigation_ms = duration;
        scene.set_params(scene.sp, scene.gp, scene.hp, &params, false);
        scene.camera.snap(10.0);
        scene.camera.retarget(11.0);
        let mut previous = scene.camera.x;
        let mut finished = false;
        for _ in 0..2400 {
            scene.camera.tick_with_precision(1.0 / 60.0, scene.camera_precision());
            assert!(scene.camera.x >= previous);
            if scene.camera_settled() {
                assert!(
                    (scene.camera.x - previous).abs() * scene.xp.depth.spacing < 0.1,
                    "duration={duration}"
                );
                finished = true;
                break;
            }
            previous = scene.camera.x;
        }
        assert!(finished, "duration={duration}");
    }
}

#[test]
fn depth_selection_frame_does_not_change_geometry_or_selected_brightness() {
    let render = |enabled| {
        rendered_scene_with_setup(Mode::Depth, base_sp(), base_gp(), 30, None, 0.0, |scene| {
            scene.xp.depth.selection_frame = enabled;
            scene.xp_target = scene.xp;
        })
    };
    let on = render(true);
    let off = render(false);
    let art = |r: &RenderSnapshot| {
        *r.instances.iter().find(|i| i.misc[0] > 0 && i.tint[3] == 0.0).unwrap()
    };
    let on = art(&on);
    let off = art(&off);
    assert_eq!(on.rect, off.rect);
    assert_eq!(on.tint, off.tint);
    assert_eq!(on.params[1], 3.0);
    assert_eq!(off.params[1], 0.0);
    assert_eq!(off.border, [0.0; 4]);
    assert_ne!(on.border, off.border);
}

#[test]
fn depth_width_and_spacing_are_independent_of_height() {
    let render = |height| {
        rendered_scene_with_setup(
            Mode::Depth,
            SliceParams { slice_h: height, visible_count: 3, ..base_sp() },
            base_gp(),
            30,
            None,
            0.0,
            |scene| {
                scene.xp.depth.width = 100.0;
                scene.xp.depth.spacing = 150.0;
                scene.xp.depth.falloff = 0.0;
                scene.xp_target = scene.xp;
            },
        )
    };
    let low = render(200.0);
    let high = render(300.0);
    for idx in [0, 1] {
        let a = low.hits.iter().find(|h| h.index == idx).unwrap();
        let b = high.hits.iter().find(|h| h.index == idx).unwrap();
        assert_eq!(a.hw, 50.0);
        assert_eq!(b.hw, 50.0);
        assert_eq!(a.cx, b.cx);
        assert_eq!(a.hh, 100.0);
        assert_eq!(b.hh, 150.0);
    }
    let a = low.hits.iter().find(|h| h.index == 0).unwrap();
    let b = low.hits.iter().find(|h| h.index == 1).unwrap();
    assert!((b.cx - a.cx - 150.0).abs() < 0.01);
}

#[test]
fn depth_edge_cards_reach_transparency_before_they_are_removed() {
    let opacity = |fade: f32| {
        let render = rendered_scene_with_setup(
            Mode::Depth,
            SliceParams { visible_count: 5, ..base_sp() },
            base_gp(),
            31,
            None,
            0.0,
            |scene| {
                scene.current = 15;
                scene.camera.snap(15.0 - 2.0 - fade);
            },
        );
        render.hits.iter().find(|hit| hit.index == 15).map_or(0.0, |hit| {
            render
                .instances
                .iter()
                .find(|body| body.misc[0] > 0 && (body.rect[0] - hit.cx).abs() < 0.01)
                .unwrap()
                .params[2]
        })
    };
    assert!((opacity(0.5) - 0.5).abs() < 0.001);
    assert!(opacity(0.9) < 0.03);
    assert!(opacity(0.99) > 0.0 && opacity(0.99) < 0.001);
    assert_eq!(opacity(0.999), 0.0);
}

#[test]
fn depth_frame_off_removes_borders_during_hover_and_selection_animation() {
    for hover in [None, Some(0), Some(1)] {
        let snap =
            rendered_scene_with_setup(Mode::Depth, base_sp(), base_gp(), 30, None, 0.0, |scene| {
                scene.xp.depth.selection_frame = false;
                scene.xp_target = scene.xp;
                scene.hover = hover;
                scene
                    .card
                    .selection
                    .insert(0, crate::frontend::animation::Spring::for_duration_ms(0.5, 250.0));
            });
        let bodies: Vec<_> = snap.instances.iter().filter(|i| i.misc[0] > 0).collect();
        assert!(!bodies.is_empty());
        for body in bodies {
            assert_eq!(body.params[1], 0.0);
            assert_eq!(body.border, [0.0; 4]);
        }
    }
}

#[test]
fn depth_and_slices_shadows_toggle_without_changing_cards() {
    for mode in [Mode::Depth, Mode::Slices] {
        let on = rendered_scene(mode, base_sp(), base_gp());
        let off = rendered_scene(mode, SliceParams { shadows: false, ..base_sp() }, base_gp());
        let bodies = |snap: &RenderSnapshot| {
            snap.instances
                .iter()
                .filter(|i| i.misc[0] > 0)
                .map(|i| (i.rect, i.border, i.tint))
                .collect::<Vec<_>>()
        };
        assert_eq!(bodies(&on), bodies(&off));
        let shadow = |i: &&crate::frontend::scene::InstanceRaw| {
            i.fill[0..3] == [0.0; 3] && (i.fill[3] == 0.3 || i.fill[3] == 0.5) && i.misc[0] == 0
        };
        assert_eq!(on.instances.iter().filter(shadow).count(), on.hits.len());
        assert_eq!(off.instances.iter().filter(shadow).count(), 0);
        assert_eq!(on.instances.len() - off.instances.len(), on.hits.len());
    }
}

#[test]
fn chamfer_reaches_cards() {
    let cut = CardCorners { sizes: [10.0; 4], cut: true };
    for (mode, sp, gp) in [
        (Mode::Grid, base_sp(), GridParams { corners: cut, ..base_gp() }),
        (Mode::Slices, SliceParams { corners: cut, ..base_sp() }, base_gp()),
        (Mode::Depth, SliceParams { corners: cut, ..base_sp() }, base_gp()),
    ] {
        let snap = rendered_scene(mode, sp, gp);
        let cards: Vec<_> = snap.instances.iter().filter(|inst| inst.radii[0] > 0.0).collect();
        assert!(!cards.is_empty(), "{mode:?}");
        let flagged = |inst: &&InstanceRaw| inst.misc[3] & CHAMFER != 0;
        assert!(cards.iter().all(flagged), "{mode:?}");
    }
}

#[test]
fn chamfer_reaches_extras() {
    let cut = CardCorners { sizes: [10.0; 4], cut: true };
    for mode in [Mode::Hand, Mode::Collection, Mode::Sandy] {
        let snap = rendered_scene_with_setup(mode, base_sp(), base_gp(), 8, None, 0.0, |scene| {
            scene.xp.hand.corners = cut;
            scene.xp.collection.corners = cut;
            scene.xp.sandy.corners = cut;
            scene.xp_target = scene.xp;
        });
        let cards: Vec<_> =
            snap.instances.iter().filter(|inst| inst.misc[0] > 0 && inst.radii[0] > 0.0).collect();
        assert!(!cards.is_empty(), "{mode:?}");
        assert!(cards.iter().all(|inst| inst.misc[3] & CHAMFER != 0), "{mode:?}");
    }
}

#[test]
fn chamfer_reaches_hand_slats() {
    use crate::rendering::scene::{GHOST, PROJECTED};
    let cut = CardCorners { sizes: [10.0; 4], cut: true };
    let mut scene = hand_scene();
    scene.set_card_flip_options(400.0, true, true);
    scene.xp.hand.corners = cut;
    scene.xp_target = scene.xp;
    let mut fixture = hand_fixture(12);
    let mut now = Instant::now();
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    let cur = fixture.1[scene.current] as usize;
    let atlas = fixture.5.as_mut().expect("atlas");
    atlas.near.acquire(cur);
    atlas.near.mark_ready(cur);
    scene.touch();
    hand_frames(&mut scene, &mut fixture, &mut now, 2);
    scene.hand_reveal_toggle();
    let (mut slats, mut ghosts) = (0, 0);
    for _ in 0..240 {
        hand_frames(&mut scene, &mut fixture, &mut now, 1);
        for inst in scene.render.instances.iter().filter(|inst| inst.radii[0] > 0.0) {
            assert!(inst.misc[3] & CHAMFER != 0);
            slats += usize::from(inst.misc[3] & PROJECTED != 0);
            ghosts += usize::from(inst.misc[3] & GHOST != 0);
        }
    }
    assert!(slats > 0 && ghosts > 0, "slats {slats} ghosts {ghosts}");
    scene.toggle_flip(scene.current);
    hand_frames(&mut scene, &mut fixture, &mut now, 100);
    let panel = scene.render.back.as_ref().expect("ribbon panel");
    assert_eq!(panel.corners, cut);
}

#[test]
fn unframed_depth_rectangles_use_clean_edges_without_affecting_rounded_or_skewed_cards() {
    for (mode, enabled, corners, skew, clean) in [
        (Mode::Depth, false, 0.0, 0.0, true),
        (Mode::Depth, true, 0.0, 0.0, false),
        (Mode::Depth, false, 12.0, 0.0, false),
        (Mode::Depth, false, 0.0, 20.0, false),
        (Mode::Slices, false, 0.0, 0.0, false),
    ] {
        let snap = rendered_scene_with_setup(
            mode,
            SliceParams { skew, corners: CardCorners::round(corners), ..base_sp() },
            base_gp(),
            30,
            None,
            0.0,
            |scene| {
                scene.xp.depth.selection_frame = enabled;
                scene.xp_target = scene.xp;
            },
        );
        let bodies: Vec<_> = snap.instances.iter().filter(|i| i.misc[0] > 0).collect();
        assert!(!bodies.is_empty());
        for body in bodies {
            assert_eq!(body.misc[3] & crate::rendering::scene::UNFRAMED_RECT != 0, clean);
        }
    }
}

const FLIP_FRAME: Duration = Duration::from_micros(16_667);
const FLIP_SCALE: f32 = 0.5;

fn flipping(mode: Mode, ms: f32, run: impl FnOnce(&mut SceneCore, &mut dyn FnMut(&mut SceneCore))) {
    use std::sync::{Arc, Mutex};
    let mut scene = test_scene(mode);
    scene.viewport = (1000.0, 600.0);
    scene.set_motion_scale(FLIP_SCALE);
    scene.xp.hand.speed = 2.0;
    scene.set_card_flip_options(ms, true, true);
    let uploads: UploadQueue = Arc::new(Mutex::new(Vec::new()));
    let (tx, _rx) = futures_channel::mpsc::unbounded();
    let pool = DecodePool::start(uploads.clone(), tx, 0);
    let mut store = Catalog::default();
    for name in ["a", "b", "c"] {
        store.items.push(Wallpaper {
            key: name.into(),
            name: name.into(),
            thumb: format!("/tmp/{name}.png"),
            path: format!("/tmp/{name}.png"),
            kind: WallpaperKind::Static,
            ..Default::default()
        });
    }
    let filtered: Vec<u32> = vec![0, 1, 2];
    let pal = Palette::default();
    let mut atlas = Some(AtlasMap::new(3));
    scene.motion.entrance.snap(1.0);
    scene.motion.visibility.snap(1.0);
    scene.set_current(1, 3);
    let mut now = Instant::now();
    let mut tick = |scene: &mut SceneCore| {
        now += FLIP_FRAME;
        let ctx = RebuildCtx {
            catalog: &store,
            filtered: &filtered,
            atlas: &mut atlas,
            pool: &pool,
            palette: &pal,
            uploads: &uploads,
            hover_fades: None,
        };
        scene.tick(now, ctx);
    };
    tick(&mut scene);
    run(&mut scene, &mut tick);
}

fn frame_ms(frames: usize) -> f32 {
    frames as f32 * FLIP_FRAME.as_secs_f32() * 1000.0
}

fn flip_times(mode: Mode, ms: f32) -> [f32; 2] {
    let mut times = [0.0; 2];
    flipping(mode, ms, |scene, tick| {
        for (leg, target) in [1.0, 0.0].into_iter().enumerate() {
            match (mode, target > 0.5) {
                (Mode::Grid, true) => scene.open_detail(1, [500.0, 300.0, 80.0, 60.0]),
                (_, true) => scene.toggle_flip(1),
                (_, false) => scene.close_flip(),
            }
            let done = |scene: &SceneCore| match mode {
                Mode::Grid => scene.card.det_p == target,
                Mode::Hand => scene.hand.flip_p == target && scene.card.flip.settled(),
                _ => scene.card.flip.x == target && scene.card.flip.settled(),
            };
            let mut frames = 0;
            while !done(scene) {
                tick(scene);
                frames += 1;
                assert!(frames < 1000, "flip never settled");
            }
            times[leg] = frame_ms(frames);
        }
    });
    times
}

#[test]
fn flip_takes_override() {
    let frame = frame_ms(1);
    let standard = MotionProfile::default().duration_ms(MotionTier::Standard) * FLIP_SCALE;
    for mode in [Mode::Slices, Mode::Hand, Mode::Grid] {
        for ms in [300.0, 2000.0] {
            let expected = if mode == Mode::Grid { standard } else { ms };
            for took in flip_times(mode, ms) {
                assert!((took - expected).abs() <= frame, "{mode:?} {ms}: {took}");
            }
        }
    }
}

#[test]
fn flip_reverses_smoothly() {
    const MS: f32 = 2000.0;
    flipping(Mode::Slices, MS, |scene, tick| {
        scene.toggle_flip(1);
        for _ in 0..30 {
            tick(scene);
        }
        let turned = scene.card.flip.x;
        scene.close_flip();
        assert_eq!(scene.card.flip.x, turned);
        let (mut last, mut frames) = (turned, 0);
        while scene.card.flip.x > 0.0 {
            tick(scene);
            frames += 1;
            let step = last - scene.card.flip.x;
            assert!((0.0..0.05).contains(&step), "jump {step} at {frames}");
            last = scene.card.flip.x;
        }
        assert!((frame_ms(frames) - MS * turned).abs() <= frame_ms(1), "{frames} frames");
    });
}
