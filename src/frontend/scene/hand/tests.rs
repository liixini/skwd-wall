#![cfg(test)]

use super::*;
use glam::{Mat4, Vec3};

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

#[test]
fn bezier_endpoints_and_monotone_linear() {
    assert_eq!(EASE_DEAL_OUT.at(0.0), 0.0);
    assert_eq!(EASE_DEAL_OUT.at(1.0), 1.0);
    for step in 0..=10 {
        let x = step as f32 / 10.0;
        assert!(close(EASE_LINEAR.at(x), x), "{x}");
    }
    assert!(EASE_DEAL_IN.at(0.5) > 0.9);
    assert!(EASE_TWIST.at(0.5) > 1.0);
}

#[test]
fn css_rotation_handedness() {
    let p = rot(Y, 90.0).transform_point3(Vec3::X);
    assert!(close(p.x, 0.0) && close(p.z, -1.0));
    let p = rot(X, 90.0).transform_point3(Vec3::Y);
    assert!(close(p.y, 0.0) && close(p.z, 1.0));
    let p = rot(Z, 90.0).transform_point3(Vec3::X);
    assert!(close(p.x, 0.0) && close(p.y, 1.0));
}

#[test]
fn matrix_product_applies_right_first() {
    let m = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0)) * Mat4::from_scale(Vec3::splat(2.0));
    assert_eq!(m.transform_point3(Vec3::new(1.0, 1.0, 0.0)), Vec3::new(12.0, 2.0, 0.0));
}

#[test]
fn pose_mix_matching_orders_keeps_multi_turn() {
    let from = fan_pose(1.0, &DESIGN_FAN, 1.0, 1.0, 0.0, 0.0, 1.0);
    let to = move_pose(Move::Corkscrew, 1.0, 1.0, 1.0);
    let half = Pose::mix(&from, &to, 0.5);
    assert_eq!(half.n, 2);
    assert!(close(half.rots[0].deg, (-12.0 + 1030.0) * 0.5));
    assert!(close(half.s, 0.54));
    assert_eq!(Pose::mix(&from, &to, 0.0), from);
}

#[test]
fn pose_mix_mismatched_orders_slerps_shortest_path() {
    let from = fan_pose(0.0, &DESIGN_FAN, 1.0, 1.0, 0.0, 0.0, 1.0);
    let to = move_pose(Move::Shuffle, 0.0, 1.0, 1.0);
    let half = Pose::mix(&from, &to, 0.5);
    assert_eq!(half.n, 1);
    assert!(half.rots[0].deg.abs() < 180.0);
    let start = Pose::mix(&from, &to, 0.0);
    let p = start.matrix().transform_point3(Vec3::new(10.0, 0.0, 0.0));
    assert!(close(p.x, 10.0) && close(p.y, 0.0));
}

#[test]
fn pose_tween_respects_delay_and_end() {
    let tween = PoseTween::new(Pose::REST, sel_pose(1.0), 100.0, 400.0, EASE_LINEAR);
    assert!(tween.at(0.05).matrix().abs_diff_eq(Mat4::IDENTITY, 1e-5));
    assert!(close(tween.at(0.3).t[2], 85.0));
    assert!(close(tween.end(), 0.5));
    assert!(close(tween.at(1.0).t[1], -40.0));
}

#[test]
fn ribbon_cuts_partition_the_card() {
    let cuts = ribbon_cuts(6, 432.0, Cut::Straight, Variance::None, 0.31, 0, 2.0);
    assert_eq!(cuts.len(), 6);
    assert!(cuts[0].a0 < -218.0 && cuts[0].a1 == cuts[0].a0);
    assert!(cuts[5].b1 > 218.0 && cuts[5].b0 == cuts[5].b1);
    for pair in cuts.windows(2) {
        assert!(close(pair[0].b0 - pair[1].a0, 2.0 * RIBBON_OVERLAP));
        assert!(close(pair[0].b1 - pair[1].a1, 2.0 * RIBBON_OVERLAP));
    }
    let slanted = ribbon_cuts(6, 432.0, Cut::Steep, Variance::None, 0.31, 0, 2.0);
    assert!(close(slanted[1].a1 - slanted[1].a0, 2.4 * 72.0));
    assert!(close(slanted[1].mid, -108.0));
    let wild = ribbon_cuts(6, 432.0, Cut::Slant, Variance::Wild, 0.31, 2, 2.0);
    let widths: Vec<f32> = wild.iter().map(|c| c.mid).collect();
    assert!(widths.windows(2).all(|w| w[1] > w[0]));
    for seed in 0..40 {
        for nr in [2usize, 6, 14] {
            let cuts =
                ribbon_cuts(nr, 432.0, Cut::Steep, Variance::Wild, seed as f32 * 0.037, seed, 2.0);
            for cut in &cuts {
                assert!(
                    cut.b0 > cut.a0 && cut.b1 > cut.a1,
                    "bow-tie ribbon at seed {seed} nr {nr}: {cut:?}"
                );
            }
        }
    }
}

#[test]
fn ribbon_locals_follow_corner_order() {
    let cut = RibbonCut { a0: 1.0, a1: 2.0, b0: 3.0, b1: 4.0, mid: 2.5 };
    assert_eq!(ribbon_locals(&cut, Axis::Rows), [1.0, 2.0, 4.0, 3.0]);
    assert_eq!(ribbon_locals(&cut, Axis::Columns), [1.0, 3.0, 4.0, 2.0]);
    let rows = ribbon_corners(&cut, Axis::Rows, 84.0, 216.0, 0.0);
    assert_eq!(rows[1], [84.0, 2.0, 0.0]);
    let cols = ribbon_corners(&cut, Axis::Columns, 84.0, 216.0, 0.0);
    assert_eq!(cols[3], [2.0, 216.0, 0.0]);
}

#[test]
fn projection_identity_on_the_anchor_plane() {
    let cam = Camera { d: 1700.0, origin: [800.0, 400.0], shift: [0.0, 16.0] };
    let p = cam.project([50.0, -30.0, 0.0]);
    assert!(close(p[0], 850.0) && close(p[1], 370.0) && close(p[2], 1.0));
    let near = cam.project([50.0, 16.0, 850.0]);
    assert!(close(near[0], 900.0) && close(near[2], 0.5));
}

#[test]
fn quad_facing_flips_with_rotation() {
    let cam = Camera { d: 1700.0, origin: [0.0, 0.0], shift: [0.0, 0.0] };
    let front = project_quad(&cam, &Mat4::IDENTITY, &card_corners(84.0, 216.0, 0.0, 0.0));
    assert!(front.facing());
    let back = project_quad(&cam, &rot(Y, 180.0), &card_corners(84.0, 216.0, 0.0, 0.0));
    assert!(!back.facing());
    let edge_on = project_quad(&cam, &rot(X, 90.0), &card_corners(84.0, 216.0, 0.0, 0.0));
    let [_, _, _, hh] = edge_on.bounds();
    assert!(hh < 1.0);
}

#[test]
fn flip_ribbons_land_flat_and_return_home() {
    let nr = 6;
    for r in 0..nr {
        let landed = flip_ribbon_pose(flip_open_ms(nr) + 1.0, false, r, nr, 1.0, 1.0, Axis::Rows);
        assert_eq!(landed.rots[0].axis, X);
        assert!(close(landed.rots[0].deg, 180.0), "ribbon {r}: {}", landed.rots[0].deg);
        assert!(close(landed.t[2], 0.0));
        let home = flip_ribbon_pose(flip_close_ms(nr) + 1.0, true, r, nr, 1.0, 1.0, Axis::Rows);
        assert!(close(home.rots[0].deg, 360.0));
        let lifted = flip_ribbon_pose(300.0, false, r, nr, 1.0, 1.0, Axis::Rows);
        assert!(lifted.t[2] >= 0.0);
    }
    let start = flip_ribbon_pose(0.0, false, 0, nr, 1.0, 1.0, Axis::Rows);
    assert!(start.matrix().abs_diff_eq(Mat4::IDENTITY, 1e-5));
}

#[test]
fn twist_settles_after_landing() {
    let open = flip_twist_pose(FLIP_LIFT_MS + TWIST_MS * 1.2, false, 1.0);
    assert!(close(open.rots[0].deg, -13.0));
    let closed = flip_twist_pose(FLIP_CLOSE_LIFT_MS + TWIST_MS * 1.2, true, 1.0);
    assert!(close(closed.rots[0].deg, 0.0) && close(closed.s, 1.0));
}

#[test]
fn bob_is_periodic_and_bounded() {
    for step in 0..40 {
        let pose = bob(step as f32 * 0.3, 2, 1.0);
        assert!(pose.t[1] >= -6.001 && pose.t[1] <= 7.001);
    }
    let a = bob(1.0, 0, 1.0);
    let b = bob(1.0 + BOB_PERIOD_S, 0, 1.0);
    assert!(close(a.t[1], b.t[1]));
}

#[test]
fn hand_window_centres_and_clamps() {
    assert_eq!(hand_window(0, 20, 5), 0);
    assert_eq!(hand_window(4, 20, 5), 0);
    assert_eq!(hand_window(5, 20, 5), 5);
    assert_eq!(hand_window(7, 20, 5), 5);
    assert_eq!(hand_window(19, 20, 5), 15);
    assert_eq!(hand_window(11, 12, 5), 7);
    assert_eq!(hand_window(2, 3, 5), 0);
    assert_eq!(hand_len(15, 20, 5), 5);
    assert_eq!(hand_len(0, 3, 5), 3);
}

#[test]
fn move_cycle_wraps() {
    assert_eq!(Move::Spiral.index(), 4);
    assert_eq!(Move::ALL[Move::Cascade.index()], Move::Cascade);
    assert_eq!(Move::from_key("cascade"), Some(Move::Cascade));
    assert_eq!(Move::from_key("cycle"), None);
    assert_eq!(Cut::from_key("steep").factor(), 2.4);
    assert_eq!(Variance::from_key("wild").factor(), 1.05);
    assert_eq!(Axis::from_key("columns"), Axis::Columns);
}

#[test]
fn params_settle_only_when_equal() {
    let base = HandParams::default();
    let mut moving = HandParams { spread: 90.0, ..base };
    assert!(!moving.settled_to(&base));
    for _ in 0..200 {
        moving.morph_toward(&base, 0.2);
    }
    assert!(moving.settled_to(&base));
    let topo = HandParams { ribbons: 9, ..base };
    assert!(!topo.settled_to(&base));
}

#[test]
fn relax_window_is_a_smooth_step() {
    assert_eq!(relax(0.0), 0.0);
    assert_eq!(relax(RELAX_WINDOW.0), 0.0);
    assert_eq!(relax(RELAX_WINDOW.1), 1.0);
    assert_eq!(relax(1.0), 1.0);
    let mid = (RELAX_WINDOW.0 + RELAX_WINDOW.1) * 0.5;
    assert!(relax(mid) > 0.45 && relax(mid) < 0.55);
}

#[test]
fn stage_scale_prefers_the_tighter_axis() {
    assert!(close(stage_scale(1440.0, 900.0), 1.0));
    assert!(close(stage_scale(2560.0, 1440.0), 1.6));
    assert!(close(stage_scale(1024.0, 900.0), 1024.0 / 1440.0));
    let (min_w, min_h) = crate::frontend::ui::MIN_VIEWPORT;
    assert!(close(stage_scale(min_w, min_h), min_h / 900.0));
}

#[test]
fn selected_card_flies_to_the_middle() {
    let sel = sel_pose(1.0);
    assert_eq!(sel.t, [0.0, -40.0, 170.0]);
    assert_eq!(sel.rots[0].deg, 0.0);
    assert_eq!(sel.rots[1].deg, 0.0);
    let half = Pose::mix(&fan_pose(2.0, &DESIGN_FAN, 1.0, 1.24, 0.0, -150.0, 1.0), &sel, 0.5);
    assert!((half.t[0] - 2.0 * 126.0 * 1.24 * 0.5).abs() < 1e-3);
}

#[test]
fn fan_knobs_shape_the_hand() {
    let wide = Fan { spread: 200.0, angle: 30.0, roll: 0.0, arch: 0.0 };
    let pose = fan_pose(2.0, &wide, 1.0, 1.0, 0.0, 0.0, 1.0);
    assert_eq!(pose.t[0], 400.0);
    assert_eq!(pose.t[1], 0.0);
    assert_eq!(pose.rots[0].deg, -60.0);
    assert_eq!(pose.rots[1].deg, 0.0);
    let params = HandParams { skew: 30.0, ..HandParams::default() };
    assert_eq!(params.fan(2.0).spread, 252.0);
    assert_eq!(params.fan(2.0).arch, 40.0);
    assert_eq!(hand_window(15, 16, 16), 0);
    assert_eq!(hand_len(0, 3, 2), 2);
}

#[test]
fn fan_slots_close_the_gap_around_a_lifted_card() {
    assert_eq!(fan_slot(2, 5, None), 0.0);
    assert_eq!(fan_slot(0, 5, None), -2.0);
    assert_eq!(fan_slot(2, 5, Some(3)), 0.5);
    assert_eq!(fan_slot(4, 5, Some(3)), 1.5);
    assert_eq!(fan_slot(0, 5, Some(3)), -1.5);
    assert_eq!(fan_slot(3, 5, Some(3)), 1.0);
    assert_eq!(fan_slot(0, 1, Some(0)), 0.0);
}

#[test]
fn deal_moves_come_from_the_enabled_pool() {
    let only_two = [false, true, true, false, false];
    assert_eq!(pick_move(DealMode::Cycle, only_two, Move::Cascade, 0.0), Move::Shuffle);
    assert_eq!(pick_move(DealMode::Cycle, only_two, Move::Shuffle, 0.0), Move::Cascade);
    assert_eq!(pick_move(DealMode::Cycle, only_two, Move::Spiral, 0.0), Move::Cascade);
    assert_eq!(pick_move(DealMode::Cycle, [false; 5], Move::Spiral, 0.0), Move::Corkscrew);
    for step in 0..20 {
        let seed = step as f32 / 20.0;
        assert_eq!(pick_move(DealMode::Random, only_two, Move::Cascade, seed), Move::Shuffle);
    }
    let picks: Vec<Move> = (0..20)
        .map(|step| pick_move(DealMode::Random, [true; 5], Move::Spiral, step as f32 / 20.0))
        .collect();
    assert!(picks.iter().all(|mv| *mv != Move::Spiral));
    assert!(picks.contains(&Move::Corkscrew) && picks.contains(&Move::Ribbon));
    let solo = [false, false, false, true, false];
    assert_eq!(pick_move(DealMode::Random, solo, Move::Ribbon, 0.7), Move::Ribbon);
    assert_eq!(DealMode::from_key("random"), DealMode::Random);
    assert_eq!(DealMode::from_key("cascade"), DealMode::Cycle);
}

#[test]
fn mirrored_cut_lands_the_back_in_the_ribbon_slot() {
    let cut = RibbonCut { a0: -20.0, a1: 60.0, b0: 52.0, b1: 132.0, mid: 56.0 };
    let back = mirrored_cut(&cut);
    let flip = |y: f32| 2.0 * cut.mid - y;
    assert!(close(flip(back.a0), cut.b0));
    assert!(close(flip(back.a1), cut.b1));
    assert!(close(flip(back.b0), cut.a0));
    assert!(close(flip(back.b1), cut.a1));
    let outer = RibbonCut { a0: -650.0, a1: -650.0, b0: -230.0, b1: -57.0, mid: -182.0 };
    let back = mirrored_cut(&outer);
    let flip = |y: f32| 2.0 * outer.mid - y;
    assert!(close(flip(back.a0), outer.b0) && close(flip(back.a1), outer.b1));
    assert!(close(flip(back.b0), outer.a0) && close(flip(back.b1), outer.a1));
}

#[test]
fn slats_tile_a_fixed_aspect_frame_whatever_the_count() {
    let frame = reveal_frame(1440.0, 900.0, false);
    assert!(close(frame.0 / frame.1, THUMB_ASPECT));
    assert!(frame.0 <= 1440.0 * ROW_FIT + 0.5 && frame.1 <= 900.0 * COLUMN_FIT + 0.5);
    assert!(close(frame.1, 900.0 * COLUMN_FIT), "height is the limit on a 16:10 stage");
    let wide = reveal_frame(2560.0, 900.0, false);
    assert!(close(wide.1, 900.0 * COLUMN_FIT));
    let narrow = reveal_frame(1024.0, 900.0, false);
    assert!(close(narrow.0, 1024.0 * ROW_FIT), "width is the limit on a narrow stage");
    let tall = reveal_frame(1440.0, 900.0, true);
    assert!(close(tall.0 / tall.1, TALL_ASPECT) && close(tall.1, 900.0 * COLUMN_FIT));
    for len in [3usize, 5, 10, 16] {
        let (hw, hh) = slat_half_extent(len, frame, REVEAL_GAP, false);
        let total = len as f32 * hw * 2.0 + (len as f32 - 1.0) * REVEAL_GAP;
        assert!(close(total, frame.0), "{len} slats span the frame width: {total}");
        assert!(close(hh * 2.0, frame.1));
        let edge = row_pose((len as f32 - 1.0) * 0.5, hw * 2.0, REVEAL_GAP);
        assert!(close(edge.t[0] + hw, frame.0 * 0.5));
        assert!(edge.rots[..edge.n].iter().all(|rot| rot.deg == 0.0));
        let (chw, chh) = slat_half_extent(len, tall, REVEAL_GAP, true);
        let stack = len as f32 * chw * 2.0 + (len as f32 - 1.0) * REVEAL_GAP;
        assert!(close(stack, tall.1) && close(chh * 2.0, tall.0));
        let bottom = column_pose((len as f32 - 1.0) * 0.5, chw * 2.0, REVEAL_GAP);
        assert!(close(bottom.t[1] + chw, tall.1 * 0.5) && close(bottom.rots[1].deg, -90.0));
    }
    let five = slat_half_extent(5, frame, REVEAL_GAP, false);
    let ten = slat_half_extent(10, frame, REVEAL_GAP, false);
    assert!(ten.0 < five.0 && close(ten.1, five.1), "more cards means thinner slats, same height");
}

#[test]
fn reveal_turns_stagger_slot_by_slot() {
    let stagger = REVEAL_STAGGER_MS / 1000.0;
    assert_eq!(reveal_turn(0.0, 0, 1.0), 0.0);
    assert!(reveal_turn(stagger * 1.5, 0, 1.0) > 0.0);
    assert_eq!(reveal_turn(stagger * 1.5, 2, 1.0), 0.0);
    assert!(reveal_turn(stagger * 2.5, 2, 1.0) > 0.0);
    let end = reveal_turn_end(5, 1.0);
    assert!(close(end, (4.0 * REVEAL_STAGGER_MS + REVEAL_TURN_MS) / 1000.0));
    for slot in 0..5 {
        assert_eq!(reveal_turn(end, slot, 1.0), 1.0);
    }
    assert!(reveal_turn_end(5, 2.0) > end);
    assert!(reveal_turn(end * 0.5, 4, 1.0) < reveal_turn(end * 0.5, 0, 1.0));
}

#[test]
fn slats_lift_slap_past_flat_and_land_flat() {
    let (start, e0) = slat_turn_pose(0.0, 1.0, 1.0);
    assert_eq!(e0, 0.0);
    assert!(close(start.t[2], 0.0) && close(start.rots[0].deg, 0.0) && close(start.s, 1.0));
    let (mid, _) = slat_turn_pose(0.5, 1.0, 1.0);
    assert!(mid.t[2] > 100.0 && mid.t[1] < 0.0, "lifts toward the camera: {mid:?}");
    assert!(mid.rots[1].deg < 0.0 && mid.rots[2].deg > 0.0 && mid.s > 1.0);
    let overshoot =
        (1..100).map(|i| slat_turn_pose(i as f32 / 100.0, 1.0, 1.0).1).fold(0.0, f32::max);
    assert!(overshoot > 1.02, "slaps past flat: {overshoot}");
    let (end, e1) = slat_turn_pose(1.0, 1.0, 1.0);
    assert_eq!(e1, 1.0);
    assert!(close(end.rots[0].deg, 180.0) && close(end.t[2], 0.0) && close(end.s, 1.0));
    let (mirror, _) = slat_turn_pose(0.5, -1.0, 1.0);
    assert!(close(mirror.rots[0].deg, -mid.rots[0].deg));
    assert_eq!(EASE_REVEAL_TRAVEL.at(0.0), 0.0);
    assert_eq!(EASE_REVEAL_TRAVEL.at(1.0), 1.0);
    assert!(EASE_REVEAL_TRAVEL.at(0.5) > 0.3 && EASE_REVEAL_TRAVEL.at(0.5) < 0.9);
}

#[test]
fn reveal_sweeps_alternate_direction_and_spin() {
    assert_eq!(reveal_order(0, 5, 0), 0);
    assert_eq!(reveal_order(0, 5, 1), 4);
    assert_eq!(reveal_order(4, 5, 1), 0);
    assert_eq!(reveal_order(2, 5, 2), 2);
    assert_eq!(reveal_dir(0, 0), 1.0);
    assert_eq!(reveal_dir(1, 0), -1.0);
    assert_eq!(reveal_dir(0, 1), -1.0);
    assert_eq!(reveal_dir(1, 1), 1.0);
}

#[test]
fn reveal_face_swaps_at_the_quarter_turn() {
    assert_eq!(reveal_face(0, 0.0), 0);
    assert_eq!(reveal_face(0, 0.49), 0);
    assert_eq!(reveal_face(0, 0.51), 1);
    assert_eq!(reveal_face(1, 0.0), 1);
    assert_eq!(reveal_face(1, 0.51), 0);
    assert_eq!(reveal_face(2, 0.0), 0);
    assert_eq!(reveal_face(3, 0.6), 0);
}

#[test]
fn slice_crops_tile_the_row_edge_to_edge() {
    let len = 10;
    let (row_w, row_h) = (1248.0, 702.0);
    let gap = REVEAL_GAP;
    let crops: Vec<[f32; 4]> =
        (0..len).map(|slot| slice_crop(slot, len, row_w, row_h, gap)).collect();
    assert!(close(crops[0][0], 0.0));
    let last = crops[len - 1];
    assert!(close(last[0] + last[2], 1.0));
    for crop in &crops {
        assert!(close(crop[1], 0.0) && close(crop[3], 1.0), "a 16:9 row crops nothing away");
    }
    for pair in crops.windows(2) {
        let hole = pair[1][0] - (pair[0][0] + pair[0][2]);
        assert!(close(hole, gap / row_w), "the picture continues across the gap: {hole}");
    }
    let strip = slice_crop(0, 10, 1752.0, 432.0, gap);
    assert!(strip[1] > 0.2 && strip[3] < 0.6, "a card-sized row keeps a centred band: {strip:?}");
    assert!(close(strip[0], 0.0));
}

#[test]
fn mirrored_quad_faces_with_swapped_columns() {
    let cam = Camera { d: 1700.0, origin: [720.0, 432.0], shift: [0.0, 0.0] };
    let corners = card_corners(84.0, 216.0, 2.0, 2.0);
    let back = project_quad(&cam, &rot(Y, 180.0), &corners);
    assert!(!back.facing());
    let front = back.mirrored();
    assert!(front.facing());
    assert!(front.pts[0][0] < front.pts[1][0]);
    assert!(close(front.depth, back.depth));
}

#[test]
fn column_crops_walk_the_rotated_tile_from_right_to_left() {
    let len = 5;
    let (stack_w, stack_h) = (395.0, 702.0);
    let gap = REVEAL_GAP;
    let crops: Vec<[f32; 4]> =
        (0..len).map(|slot| column_crop(slot, len, stack_w, stack_h, gap)).collect();
    assert!(close(crops[0][0] + crops[0][2], 1.0), "the top slat reads the right edge");
    assert!(close(crops[len - 1][0], 0.0), "the bottom slat reads the left edge");
    for crop in &crops {
        assert!(close(crop[1], 0.0) && close(crop[3], 1.0));
    }
    for pair in crops.windows(2) {
        assert!(close(pair[0][0] - (pair[1][0] + pair[1][2]), gap / stack_h));
    }
    let narrow = column_crop(0, 5, 200.0, 702.0, gap);
    assert!(narrow[1] > 0.0 && narrow[3] < 1.0, "a narrow stack keeps a centred strip");
}

#[test]
fn keep_mode_scales_whole_cards_to_fit_the_frame() {
    let frame = reveal_frame(1440.0, 900.0, false);
    let card = (84.0, 216.0);
    let five = keep_extent(5, card, frame, REVEAL_GAP, false);
    assert!(close(five.0, 84.0) && close(five.1, 216.0), "five cards already fit: {five:?}");
    let ten = keep_extent(10, card, frame, REVEAL_GAP, false);
    assert!(ten.0 < 84.0 && close(ten.1 / ten.0, 216.0 / 84.0), "same shape, smaller: {ten:?}");
    assert!(close(10.0 * ten.0 * 2.0 + 9.0 * REVEAL_GAP, frame.0));
    let tall = reveal_frame(1440.0, 900.0, true);
    let stack = keep_extent(5, card, tall, REVEAL_GAP, true);
    assert!(5.0 * stack.0 * 2.0 + 4.0 * REVEAL_GAP <= tall.1 + 0.5);
    assert!(stack.1 * 2.0 <= tall.0 + 0.5);
}
