#![cfg(test)]

use super::*;

#[test]
fn grid_frame_counts_cards() {
    let grid = GridParams {
        thumb_w: 150.0,
        thumb_h: 90.0,
        gap_x: 8.0,
        gap_y: 8.0,
        ..GridParams::default()
    };
    assert_eq!(grid.columns_in(grid.span_w(4)), 4);
    assert_eq!(grid.rows_in(grid.span_h(3)), 3);
    assert_eq!(grid.columns_in(grid.span_w(4) - 1.0), 3);
    assert_eq!(grid.rows_in(grid.span_h(3) + grid.cell_h()), 4);
}

fn sp(slice_w: f32, spacing: f32, count: usize) -> SliceParams {
    SliceParams {
        offset_x: 0.0,
        offset_y: 0.0,
        slice_w,
        expanded_w: 600.0,
        slice_h: 400.0,
        spacing,
        skew: 10.0,
        edge_tilt: 0.0,
        visible_count: count,
        corners: CardCorners::round(8.0),
        wobble: false,
        wobble_strength: 1.0,
        parallax: false,
        shadows: true,
    }
}

#[test]
fn wobble_pad_strength() {
    assert!((wobble_pad(1.0) - 1.35).abs() < 0.001);
    assert!(wobble_pad(2.0) > wobble_pad(1.0));
    assert!((wobble_pad(0.2) - 1.35).abs() < 0.001);
}

#[test]
fn wobble_bend_saturates() {
    assert_eq!(wobble_bend(0.0, 900.0), 0.0);
    assert!(wobble_bend(2000.0, 900.0) > 0.5);
    assert_eq!(wobble_bend(1e6, 900.0), 1.35);
    assert_eq!(wobble_bend(-1e6, 900.0), -1.35);
}

#[test]
fn slice_width_midline() {
    assert_eq!(slice_midline_width(235.0, 0.0), 235.0);
    assert_eq!(slice_midline_width(235.0, 80.0), 155.0);
    assert_eq!(slice_midline_width(235.0, -80.0), 155.0);
    assert_eq!(slice_midline_width(50.0, 80.0), 1.0);

    let params = sp(235.0, 0.0, 8);
    assert_eq!(params.slice_stride(), 225.0);
}

#[test]
fn morph_eases_and_snaps() {
    let mut cur = sp(100.0, 10.0, 5);
    let tgt = SliceParams { edge_tilt: 40.0, ..sp(200.0, 30.0, 9) };
    assert!(!cur.settled_to(&tgt));
    cur.morph_toward(&tgt, 0.5);
    assert!((cur.slice_w - 150.0).abs() < 0.01);
    assert!((cur.spacing - 20.0).abs() < 0.01);
    assert!((cur.edge_tilt - 20.0).abs() < 0.01);
    assert_eq!(cur.visible_count, 9);
    assert!(!cur.settled_to(&tgt));
}

#[test]
fn morph_settles() {
    let mut cur = sp(100.0, 10.0, 5);
    let tgt = sp(200.0, 30.0, 9);
    for _ in 0..200 {
        cur.morph_toward(&tgt, 0.5);
    }
    assert!(cur.settled_to(&tgt));
}

#[test]
fn topology_vs_geometry() {
    let slice = sp(100.0, 10.0, 5);
    let mut resized = slice;
    resized.slice_w = 180.0;
    assert!(!slice.topology_differs(&resized));
    resized.visible_count = 7;
    assert!(slice.topology_differs(&resized));

    let grid = GridParams::default();
    let mut spaced = grid;
    spaced.gap_x = 40.0;
    assert!(!grid.topology_differs(&spaced));
    spaced.layout = GridLayout::Editorial;
    assert!(grid.topology_differs(&spaced));

    let hex = HexParams::default();
    let mut curved = hex;
    curved.curve_strength = 2.0;
    assert!(!hex.topology_differs(&curved));
    curved.shape = HexShape::Diamond;
    assert!(hex.topology_differs(&curved));
}

fn hit(skew: f32, hex: bool) -> Hit {
    Hit {
        index: 0,
        cx: 100.0,
        cy: 100.0,
        hw: 50.0,
        hh: 50.0,
        skew,
        edge_tilt: 0.0,
        hex,
        hex_shape: HexShape::Hexagon,
        triangle_direction: 0,
        quad: None,
    }
}

#[test]
fn contains_plain_rect() {
    let area = hit(0.0, false);
    assert!(area.contains(100.0, 100.0));
    assert!(area.contains(50.5, 50.5));
    assert!(area.contains(149.5, 149.5));
    assert!(!area.contains(49.0, 100.0));
    assert!(!area.contains(151.0, 100.0));
    assert!(!area.contains(100.0, 49.0));
    assert!(!area.contains(100.0, 151.0));
}

#[test]
fn contains_positive_skew() {
    let area = hit(30.0, false);
    assert!(area.contains(85.0, 50.5));
    assert!(!area.contains(75.0, 50.5));
    assert!(area.contains(149.0, 50.5));
    assert!(!area.contains(151.0, 50.5));
    assert!(area.contains(52.0, 149.5));
    assert!(!area.contains(49.0, 149.5));
    assert!(area.contains(115.0, 149.5));
    assert!(!area.contains(125.0, 149.5));
    assert!(area.contains(100.0, 100.0));
}

#[test]
fn contains_negative_skew() {
    let area = hit(-30.0, false);
    assert!(area.contains(75.0, 50.5));
    assert!(!area.contains(125.0, 50.5));
    assert!(area.contains(115.0, 50.5));
    assert!(area.contains(85.0, 149.5));
    assert!(!area.contains(75.0, 149.5));
    assert!(area.contains(149.0, 149.5));
    assert!(!area.contains(100.0, 49.0));
    assert!(!area.contains(100.0, 151.0));
}

#[test]
fn contains_positive_edge_tilt() {
    let area = Hit { edge_tilt: 30.0, ..hit(0.0, false) };
    assert!(!area.contains(50.5, 50.5));
    assert!(area.contains(50.5, 80.5));
    assert!(area.contains(149.5, 50.5));
    assert!(area.contains(50.5, 149.5));
    assert!(area.contains(149.5, 119.5));
    assert!(!area.contains(149.5, 130.0));
    assert!(area.contains(100.0, 100.0));
}

#[test]
fn contains_negative_edge_tilt() {
    let area = Hit { edge_tilt: -30.0, ..hit(0.0, false) };
    assert!(area.contains(50.5, 50.5));
    assert!(!area.contains(149.5, 50.5));
    assert!(area.contains(149.5, 80.5));
    assert!(!area.contains(50.5, 130.0));
    assert!(area.contains(149.5, 149.5));
}

#[test]
fn contains_degenerate() {
    let mut area = hit(0.0, false);
    area.hw = 0.0;
    assert!(!area.contains(100.0, 100.0));
    let mut area = hit(0.0, false);
    area.hh = 0.0;
    assert!(!area.contains(100.0, 100.0));
}

#[test]
fn contains_hex_edges() {
    let a = 50.0 * 0.866_025;
    let area = Hit { hh: a, ..hit(0.0, true) };
    assert!(area.contains(100.0, 100.0));
    assert!(area.contains(100.0 + 50.0, 100.0));
    assert!(!area.contains(100.0 + 50.5, 100.0));
    assert!(area.contains(100.0, 100.0 + a));
    assert!(!area.contains(100.0, 100.0 + 44.0));
    assert!(area.contains(100.0 - 50.0, 100.0));
    assert!(area.contains(100.0, 100.0 - a));
}

#[test]
fn stretched_hex_hit() {
    let area = Hit { hw: 100.0, hh: 50.0, ..hit(0.0, true) };
    assert!(area.contains(199.0, 100.0));
    assert!(area.contains(100.0, 149.0));
    assert!(!area.contains(190.0, 145.0));
}

#[test]
fn triangle_hit_alternates() {
    let up = Hit { hex_shape: HexShape::Triangle, ..hit(0.0, true) };
    assert!(up.contains(100.0, 51.0));
    assert!(!up.contains(55.0, 55.0));

    let down = Hit { triangle_direction: 1, ..up };
    assert!(down.contains(100.0, 149.0));
    assert!(!down.contains(55.0, 145.0));
}

#[test]
fn diamond_rhombus_hit() {
    let diamond = Hit { hw: 50.0, hh: 100.0, hex_shape: HexShape::Diamond, ..hit(0.0, true) };
    assert!(diamond.contains(100.0, 1.0));
    assert!(diamond.contains(100.0, 199.0));
    assert!(diamond.contains(149.0, 100.0));
    assert!(!diamond.contains(149.0, 149.0));

    let rhombus = Hit { hw: 100.0, hh: 50.0, hex_shape: HexShape::Rhombus, ..hit(0.0, true) };
    assert!(rhombus.contains(1.0, 100.0));
    assert!(rhombus.contains(199.0, 100.0));
    assert!(rhombus.contains(100.0, 149.0));
    assert!(!rhombus.contains(149.0, 149.0));
}

#[test]
fn diamond_lattice_stagger() {
    let diamond = HexParams {
        r: 50.0,
        rows: 3,
        cols: 4,
        gap_x: 0.0,
        gap_y: 0.0,
        shape: HexShape::Diamond,
        ..HexParams::default()
    };
    assert_eq!(diamond.column_x(1) - diamond.column_x(0), diamond.item_half_w());
    assert_eq!(diamond.stagger_offset(1), diamond.item_half_h());
    assert_eq!(diamond.stagger_offset(2), 0.0);

    let rhombus = HexParams { shape: HexShape::Rhombus, ..diamond };
    assert_eq!(rhombus.column_x(1) - rhombus.column_x(0), rhombus.item_half_w());
    assert_eq!(rhombus.stagger_offset(1), rhombus.item_half_h());
}

#[test]
fn geometric_cell_footprints() {
    let triangle = HexParams {
        r: 80.0,
        aspect: 1.0,
        gap_x: 0.0,
        gap_y: 0.0,
        shape: HexShape::Triangle,
        ..HexParams::default()
    };
    let hexagon = HexParams { shape: HexShape::Hexagon, ..triangle };
    let diamond = HexParams { shape: HexShape::Diamond, ..triangle };
    let rhombus = HexParams { shape: HexShape::Rhombus, ..triangle };
    assert_eq!(triangle.step_x(), 80.0);
    assert_eq!(hexagon.step_x(), 120.0);
    assert_eq!(diamond.step_x(), 80.0);
    assert_eq!(rhombus.step_x(), 160.0);
    assert_eq!(triangle.content_h(), triangle.hex_h() * 3.0);
    assert_eq!(hexagon.content_h(), hexagon.hex_h() * 3.5);
    assert_eq!(diamond.content_h(), diamond.hex_h() * 7.0);
    assert_eq!(rhombus.content_h(), rhombus.hex_h() * 3.5);
}

#[test]
fn stage_transform_depth() {
    let stage = StageParams {
        offset_x: 0.1,
        offset_y: -0.2,
        scale: 1.5,
        rotation: 90.0,
        perspective: 0.4,
        ..StageParams::default()
    };
    let (x, y, scale) = stage.transform(600.0, 450.0, (600.0, 400.0), (1200.0, 800.0));
    assert!((x - 585.0).abs() < 0.01);
    assert!((y - 320.0).abs() < 0.01);
    assert!((scale - 1.5).abs() < 0.01);
}

#[test]
fn stage_shear_independent() {
    let stage =
        StageParams { shear_x: 0.5, perspective: 0.4, depth_angle: 90.0, ..StageParams::default() };
    let (x, y, scale) = stage.transform(500.0, 450.0, (400.0, 400.0), (800.0, 800.0));
    assert!((x - 540.625).abs() < 0.01);
    assert!((y - 450.0).abs() < 0.01);
    assert!((scale - 1.125).abs() < 0.01);
}

#[test]
fn hex_deform_identity_stable() {
    let plain = HexParams::default();
    assert_eq!(plain.deform(7, 520.0, 330.0, (400.0, 300.0)), (520.0, 330.0));

    let shaped = HexParams { orbit: 0.75, twist: 35.0, scatter: 24.0, ..HexParams::default() };
    let first = shaped.deform(7, 520.0, 330.0, (400.0, 300.0));
    assert_ne!(first, (520.0, 330.0));
    assert_eq!(first, shaped.deform(7, 520.0, 330.0, (400.0, 300.0)));
    assert_ne!(first, shaped.deform(8, 520.0, 330.0, (400.0, 300.0)));
}

#[test]
fn cylinder_bend_sign() {
    let flat = GridParams::default();
    assert_eq!(flat.cylinder_transform(140.0, 200.0, (640.0, 360.0)), (140.0, 200.0, 1.0));

    let zeroed =
        GridParams { layout: GridLayout::Cylinder, cylinder_bend: 0.0, ..GridParams::default() };
    assert_eq!(zeroed.cylinder_transform(140.0, 200.0, (640.0, 360.0)), (140.0, 200.0, 1.0));

    let recede = GridParams {
        layout: GridLayout::Cylinder,
        cylinder_bend: 0.8,
        cylinder_radius: 600.0,
        ..GridParams::default()
    };
    assert_eq!(recede.cylinder_transform(640.0, 200.0, (640.0, 360.0)), (640.0, 200.0, 1.0));
    let (x, y, scale) = recede.cylinder_transform(140.0, 200.0, (640.0, 360.0));
    assert!(scale < 1.0);
    assert_eq!(y, 200.0);
    assert!((x - 640.0).abs() < 500.0);

    let bow = GridParams { cylinder_bend: -0.8, ..recede };
    let (_, _, scale) = bow.cylinder_transform(140.0, 200.0, (640.0, 360.0));
    assert!(scale > 1.0);
}

#[test]
fn layout_keys_round_trip() {
    for layout in [
        GridLayout::Uniform,
        GridLayout::Brick,
        GridLayout::Masonry,
        GridLayout::Justified,
        GridLayout::Editorial,
        GridLayout::Cylinder,
    ] {
        assert_eq!(GridLayout::from_key(layout.as_key()), layout);
    }
    assert_eq!(GridLayout::from_key("cascade"), GridLayout::Uniform);
    assert_eq!(GridLayout::from_key("constellation"), GridLayout::Uniform);
}

#[test]
fn shape_keys_round_trip() {
    use super::HexShape;

    for shape in [HexShape::Hexagon, HexShape::Triangle, HexShape::Diamond, HexShape::Rhombus] {
        assert_eq!(HexShape::from_key(shape.as_key()), shape);
    }
    assert_eq!(HexShape::from_key("made-up"), HexShape::Hexagon);
    assert_eq!(HexShape::from_key("octagram"), HexShape::Hexagon);
}

#[test]
fn contains_hex_corners() {
    let hexa = hit(0.0, true);
    let rect = hit(0.0, false);
    for (px, py) in [(135.0, 135.0), (65.0, 135.0), (135.0, 65.0), (65.0, 65.0)] {
        assert!(rect.contains(px, py));
        assert!(!hexa.contains(px, py), "corner ({px},{py})");
    }
}

#[test]
fn projected_hit_follows_card_edges_in_both_windings() {
    let quad = [[-50.0, -50.0], [50.0, -50.0], [30.0, 50.0], [-30.0, 50.0]];
    for quad in [quad, [quad[3], quad[2], quad[1], quad[0]]] {
        let area = Hit { quad: Some(quad), ..hit(0.0, false) };
        assert!(area.contains(0.0, 0.0));
        assert!(area.contains(45.0, -40.0));
        assert!(!area.contains(45.0, 40.0));
        assert!(!area.contains(0.0, 51.0));
    }
    assert!(!Hit { quad: Some([[0.0; 2]; 4]), ..hit(0.0, false) }.contains(0.0, 0.0));
}

#[test]
fn cuts_clamp_like_radii() {
    let cut = CardCorners { sizes: [64.0, 4.0, 64.0, 64.0], cut: true };
    let round = CardCorners { cut: false, ..cut };
    let sheared = cut.sheared(40.0, 300.0, 30.0, 6.0);
    assert_eq!(sheared.sizes, round.sheared(40.0, 300.0, 30.0, 6.0).sizes);
    assert!(sheared.cut);
    let edge = ((40.0_f32 - 30.0).powi(2) + 6.0_f32.powi(2)).sqrt();
    assert!(sheared.sizes.iter().all(|size| *size <= edge / 2.0 - 1.0));
    assert_eq!(sheared.sizes[1], 4.0);
    assert_eq!(cut.clamped(20.0).sizes, [20.0, 4.0, 20.0, 20.0]);
    assert_eq!(cut.clamped(-3.0).sizes, [0.0; 4]);
}

#[test]
fn stamp_sets_chamfer() {
    let mut inst = InstanceRaw { misc: [0, 0, 0, CHAMFER | 2], ..InstanceRaw::default() };
    CardCorners::round(9.0).stamp(&mut inst);
    assert_eq!((inst.radii, inst.misc[3]), ([9.0; 4], 2));
    CardCorners { sizes: [3.0; 4], cut: true }.stamp(&mut inst);
    assert_eq!(inst.misc[3], CHAMFER | 2);
}

#[test]
fn cut_holds_until_square() {
    let mut corners = CardCorners { sizes: [20.0; 4], cut: true };
    let square = CardCorners::default();
    corners.morph_toward(&square, 0.5);
    assert!(corners.cut && !corners.settled_to(&square));
    for _ in 0..20 {
        corners.morph_toward(&square, 0.5);
    }
    assert!(!corners.cut && corners.settled_to(&square));
    corners.morph_toward(&CardCorners { sizes: [8.0; 4], cut: true }, 0.1);
    assert!(corners.cut);
}

#[test]
fn reads_config_corners() {
    use skwd_config::corners::{CornerStyle, Corners};
    let each = Corners {
        style: CornerStyle::Chamfered,
        same: false,
        all: 1.0,
        values: [1.0, 2.0, 3.0, 4.0],
    };
    assert_eq!(CardCorners::read(&each), CardCorners { sizes: [1.0, 2.0, 3.0, 4.0], cut: true });
    let same = Corners { style: CornerStyle::Round, same: true, ..each };
    assert_eq!(CardCorners::read(&same), CardCorners::round(1.0));
    let straight = Corners { style: CornerStyle::Straight, ..each };
    assert_eq!(CardCorners::read(&straight), CardCorners::default());
}
