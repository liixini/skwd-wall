#![cfg(test)]

use std::time::{Duration, Instant};

use crate::contracts::preview::UploadQueue;
use crate::domain::library::catalog::{Catalog, Wallpaper, WallpaperKind};
use crate::frontend::animation::{MotionProfile, MotionTier, Spring, Tween};
use crate::frontend::scene::layout::{
    ExtraParams, GridLayout, GridParams, HexParams, Mode, SliceParams, StageParams,
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
    assert_eq!(center_layout(1000.0, 0.0), (500.0, 1000.0));
    assert_eq!(center_layout(1000.0, 400.0), (700.0, 600.0));
    assert_eq!(center_layout(1000.0, 900.0), (800.0, 400.0));
    assert_eq!(center_layout(1000.0, -50.0), (500.0, 1000.0));
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
        corners: [8.0; 4],
        wobble: false,
        wobble_strength: 1.0,
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
    SceneCore::new(mode, sp, gp, hp, test_extra())
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
    scene.set_params(scene.sp_target, resized, scene.hp_target, scene.xp_target, true);
    assert!(!scene.layout_transition_active());

    let mut topology = resized;
    topology.layout = GridLayout::Editorial;
    scene.set_params(scene.sp_target, topology, scene.hp_target, scene.xp_target, true);
    assert!(scene.layout_transition_active());
}

fn test_extra() -> ExtraParams {
    ExtraParams {
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
            corners: [0.0; 4],
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
    use std::sync::{Arc, Mutex};
    let hp = HexParams {
        r: 40.0,
        rows: 3,
        cols: 5,
        curve: crate::frontend::scene::layout::HexCurve::Flat,
        curve_strength: 0.0,
        ..HexParams::default()
    };
    let mut scene = SceneCore::new(mode, sp, gp, hp, test_extra());
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
        corners: [0.0; 4],
        wobble: false,
        wobble_strength: 1.0,
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
        corner_radius: 6.0,
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

    let sharp =
        rendered_scene(Mode::Grid, base_sp(), GridParams { corner_radius: 0.0, ..base_gp() });
    let round = rendered_scene(
        Mode::Grid,
        base_sp(),
        GridParams { corner_radius: 18.0, border_width: 4.0, ..base_gp() },
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

    let sharp =
        rendered_scene(Mode::Slices, SliceParams { corners: [0.0; 4], ..base_sp() }, base_gp());
    let round =
        rendered_scene(Mode::Slices, SliceParams { corners: [16.0; 4], ..base_sp() }, base_gp());
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
                scene.gp.corner_radius = 0.0;
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
