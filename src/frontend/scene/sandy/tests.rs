#![cfg(test)]

use super::*;

#[test]
fn edge_pan_zones() {
    let y = 600.0 - BAR_ZONE - 10.0;
    assert_eq!(edge_pan(500.0, 200.0, 1000.0, 600.0, 232.0), 0.0);
    assert_eq!(edge_pan(500.0, y, 1000.0, 600.0, 232.0), 0.0);
    assert!(edge_pan(20.0, y, 1000.0, 600.0, 232.0) < -0.8);
    assert!(edge_pan(980.0, y, 1000.0, 600.0, 232.0) > 0.8);
    let shallow = edge_pan(130.0, y, 1000.0, 600.0, 232.0);
    assert!(shallow < 0.0 && shallow > -0.2);
    assert_eq!(edge_pan(20.0, 595.0, 1000.0, 600.0, 232.0), 0.0);
    assert_eq!(strip_bottom(600.0), 600.0 - BAR_ZONE);
}

fn p() -> SandyParams {
    SandyParams {
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
    }
}

#[test]
fn carousel_spacing() {
    let out = place(3000, 50.0, &p(), 1280.0, 2560.0);
    assert!(out.len() > 8);
    for pair in out.windows(2) {
        assert_eq!(pair[1].i, pair[0].i + 1);
        assert!((pair[1].x - pair[0].x - p().stride()).abs() < 0.01);
    }
}

#[test]
fn carousel_gap_midline() {
    let mut params = p();
    params.slice_w = 96.0;
    params.skew = 12.0;
    params.spacing = 0.0;
    assert_eq!(params.stride(), 84.0);

    let out = place(8, 0.0, &params, 640.0, 1280.0);
    for pair in out.windows(2) {
        assert!((pair[1].x - pair[0].x - 84.0).abs() < 0.01);
    }
}

#[test]
fn current_centred() {
    let out = place(3000, 50.0, &p(), 1280.0, 2560.0);
    let cur = out.iter().find(|pl| pl.i == 50).expect("current visible");
    assert!((cur.x - 1280.0).abs() < 0.01);
    assert!((cur.s - 1.0).abs() < 0.01);
}

#[test]
fn first_card_flush_left() {
    let out = place(3000, 0.0, &p(), 1280.0, 2560.0);
    let first = out.iter().find(|pl| pl.i == 0).expect("card 0 visible at start");
    assert!(first.x < 2560.0 * 0.15);
    assert!(first.x >= 0.0);
    assert!((first.s - 1.0).abs() < 0.01);
}

#[test]
fn last_card_flush_right() {
    let out = place(3000, 2999.0, &p(), 1280.0, 2560.0);
    let last = out.iter().find(|pl| pl.i == 2999).expect("last card visible at end");
    assert!(last.x > 2560.0 * 0.85);
    assert!((last.s - 1.0).abs() < 0.01);
}

#[test]
fn short_list_centres() {
    let out = place(4, 0.0, &p(), 1280.0, 2560.0);
    let first = out.iter().find(|pl| pl.i == 0).expect("card 0 visible");
    let last = out.iter().find(|pl| pl.i == 3).expect("card 3 visible");
    assert!((((first.x + last.x) * 0.5) - 1280.0).abs() < 0.01);
    assert!(first.x < last.x);
    for pair in out.windows(2) {
        assert_eq!(pair[1].i, pair[0].i + 1);
    }
}

#[test]
fn short_list_grows_outward() {
    let four = place(4, 0.0, &p(), 1280.0, 2560.0);
    let five = place(5, 0.0, &p(), 1280.0, 2560.0);
    assert!((four[0].x - five[0].x - p().stride() * 0.5).abs() < 0.01);
    assert!((five[4].x - four[3].x - p().stride() * 0.5).abs() < 0.01);
}

#[test]
fn edge_thumbs_taper() {
    let out = place(3000, 50.0, &p(), 1280.0, 2560.0);
    let first = out.first().unwrap();
    let last = out.last().unwrap();
    assert!(first.s < 1.0 || last.s < 1.0);
    assert!(out.iter().all(|pl| pl.s > 0.0));
}

#[test]
fn camera_slides() {
    let before = place(3000, 50.0, &p(), 1280.0, 2560.0);
    let after = place(3000, 50.4, &p(), 1280.0, 2560.0);
    let x_before = before.iter().find(|pl| pl.i == 52).unwrap().x;
    let x_after = after.iter().find(|pl| pl.i == 52).unwrap().x;
    assert!((x_before - x_after - 0.4 * p().stride()).abs() < 0.01);
}

#[test]
fn library_ends_safe() {
    let out = place(5, 0.0, &p(), 1280.0, 2560.0);
    assert!(out.iter().all(|pl| pl.i < 5));
    assert!(out.iter().any(|pl| pl.i == 0));
    assert_eq!(clamp_cam(-1.0, 5), 0.0);
    assert_eq!(clamp_cam(9.0, 5), 4.0);
    assert!(place(0, 0.0, &p(), 1280.0, 2560.0).is_empty());
}

#[test]
fn lod_grain_motion() {
    assert_eq!(sandy_lod_grain(3.0, 2.0, 0.0), 3.0);
    assert_eq!(sandy_lod_grain(3.0, 2.0, 1.0), 6.0);
    assert_eq!(sandy_lod_grain(3.0, 2.0, 0.5), 4.5);
    assert_eq!(sandy_lod_grain(3.0, 1.0, 1.0), 3.0);
    assert_eq!(sandy_lod_grain(20.0, 4.0, 1.0), 32.0);
}

#[test]
fn sandy_motion_band() {
    assert_eq!(sandy_motion_from_vel(0.0, 1.5, 6.0), 0.0);
    assert_eq!(sandy_motion_from_vel(1.0, 1.5, 6.0), 0.0);
    assert_eq!(sandy_motion_from_vel(10.0, 1.5, 6.0), 1.0);
    let mid = sandy_motion_from_vel(3.75, 1.5, 6.0);
    assert!(mid > 0.4 && mid < 0.6);
}

#[test]
fn grain_verts_scales() {
    assert_eq!(grain_verts(3.0, 391.0, 220.0), 261 * 147 * 12);
    let g3 = grain_verts(3.0, 391.0, 220.0);
    let g6 = grain_verts(6.0, 391.0, 220.0);
    assert!(g3 as f32 / g6 as f32 > 3.0);
}
