pub use crate::contracts::picker::Mode;

use crate::contracts::rendering::{CHAMFER, INSTANCE_FLAGS, InstanceRaw};
use crate::contracts::settings::DEFAULT_CORNER;
use crate::contracts::settings::corners::{CornerStyle, Corners};

#[derive(Debug, Clone, Copy)]
pub struct ExtraParams {
    pub depth: DepthParams,
    pub collection: CollectionParams,
    pub sandy: super::sandy::SandyParams,
    pub hand: super::hand::HandParams,
}

impl ExtraParams {
    pub fn morph_toward(&mut self, target: &ExtraParams, amt: f32) {
        self.depth.morph_toward(&target.depth, amt);
        self.collection.morph_toward(&target.collection, amt);
        self.sandy.morph_toward(&target.sandy, amt);
        self.hand.morph_toward(&target.hand, amt);
    }

    pub fn settled_to(&self, target: &ExtraParams) -> bool {
        self.depth.settled_to(&target.depth)
            && self.collection.settled_to(&target.collection)
            && self.sandy.settled_to(&target.sandy)
            && self.hand.settled_to(&target.hand)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DepthParams {
    pub selection_frame: bool,
    pub width: f32,
    pub spacing: f32,
    pub falloff: f32,
    pub navigation_ms: f32,
}
impl Default for DepthParams {
    fn default() -> Self {
        Self {
            selection_frame: false,
            width: 280.0,
            spacing: 280.0,
            falloff: 0.05,
            navigation_ms: 1000.0,
        }
    }
}
impl DepthParams {
    fn morph_toward(&mut self, target: &Self, amt: f32) {
        self.selection_frame = target.selection_frame;
        self.width = lerp(self.width, target.width, amt);
        self.spacing = lerp(self.spacing, target.spacing, amt);
        self.falloff = lerp(self.falloff, target.falloff, amt);
        self.navigation_ms = lerp(self.navigation_ms, target.navigation_ms, amt);
    }
    fn settled_to(&self, target: &Self) -> bool {
        self.selection_frame == target.selection_frame
            && feq(self.width, target.width)
            && feq(self.spacing, target.spacing)
            && (self.falloff - target.falloff).abs() < 0.00001
            && feq(self.navigation_ms, target.navigation_ms)
    }
}
#[derive(Debug, Clone, Copy)]
pub struct CollectionParams {
    pub size: f32,
    pub spacing: f32,
    pub count: f32,
    pub tilt: f32,
    pub corners: CardCorners,
    pub speed: f32,
}
impl Default for CollectionParams {
    fn default() -> Self {
        Self {
            size: 42.0,
            spacing: 17.0,
            count: 7.0,
            tilt: 52.0,
            corners: CardCorners::round(2.0),
            speed: 100.0,
        }
    }
}
impl CollectionParams {
    fn morph_toward(&mut self, target: &Self, amt: f32) {
        self.size = lerp(self.size, target.size, amt);
        self.spacing = lerp(self.spacing, target.spacing, amt);
        self.count = lerp(self.count, target.count, amt);
        self.tilt = lerp(self.tilt, target.tilt, amt);
        self.corners.morph_toward(&target.corners, amt);
        self.speed = lerp(self.speed, target.speed, amt);
    }
    fn settled_to(&self, target: &Self) -> bool {
        feq(self.size, target.size)
            && feq(self.spacing, target.spacing)
            && feq(self.count, target.count)
            && feq(self.tilt, target.tilt)
            && self.corners.settled_to(&target.corners)
            && feq(self.speed, target.speed)
    }
}

pub const MIN_HEX_R: f32 = 20.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HexCurve {
    Flat,
    Arc,
    Wave,
    S,
    Cylinder,
}

impl HexCurve {
    pub fn from_key(value: &str) -> Self {
        match value {
            "flat" => Self::Flat,
            "wave" => Self::Wave,
            "s" => Self::S,
            "cylinder" => Self::Cylinder,
            _ => Self::Arc,
        }
    }

    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Flat => "flat",
            Self::Arc => "arc",
            Self::Wave => "wave",
            Self::S => "s",
            Self::Cylinder => "cylinder",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HexShape {
    #[default]
    Hexagon,
    Triangle,
    Diamond,
    Rhombus,
}

impl HexShape {
    pub fn from_key(value: &str) -> Self {
        match value {
            "triangle" => Self::Triangle,
            "diamond" => Self::Diamond,
            "rhombus" => Self::Rhombus,
            _ => Self::Hexagon,
        }
    }

    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Hexagon => "hexagon",
            Self::Triangle => "triangle",
            Self::Diamond => "diamond",
            Self::Rhombus => "rhombus",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridLayout {
    Uniform,
    Brick,
    Masonry,
    Justified,
    Editorial,
    Cylinder,
}

impl GridLayout {
    pub fn from_key(value: &str) -> Self {
        match value {
            "brick" => Self::Brick,
            "masonry" => Self::Masonry,
            "justified" => Self::Justified,
            "editorial" => Self::Editorial,
            "cylinder" => Self::Cylinder,
            _ => Self::Uniform,
        }
    }

    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Uniform => "uniform",
            Self::Brick => "brick",
            Self::Masonry => "masonry",
            Self::Justified => "justified",
            Self::Editorial => "editorial",
            Self::Cylinder => "cylinder",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StageParams {
    pub offset_x: f32,
    pub offset_y: f32,
    pub scale: f32,
    pub rotation: f32,
    pub perspective: f32,
    pub shear_x: f32,
    pub shear_y: f32,
    pub depth_angle: f32,
}

impl Default for StageParams {
    fn default() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            scale: 1.0,
            rotation: 0.0,
            perspective: 0.0,
            shear_x: 0.0,
            shear_y: 0.0,
            depth_angle: 0.0,
        }
    }
}

impl StageParams {
    pub fn morph_toward(&mut self, target: &Self, amt: f32) {
        self.offset_x = lerp(self.offset_x, target.offset_x, amt);
        self.offset_y = lerp(self.offset_y, target.offset_y, amt);
        self.scale = lerp(self.scale, target.scale, amt);
        self.rotation = lerp(self.rotation, target.rotation, amt);
        self.perspective = lerp(self.perspective, target.perspective, amt);
        self.shear_x = lerp(self.shear_x, target.shear_x, amt);
        self.shear_y = lerp(self.shear_y, target.shear_y, amt);
        self.depth_angle = lerp(self.depth_angle, target.depth_angle, amt);
    }

    pub fn settled_to(&self, target: &Self) -> bool {
        feq(self.offset_x, target.offset_x)
            && feq(self.offset_y, target.offset_y)
            && feq(self.scale, target.scale)
            && feq(self.rotation, target.rotation)
            && feq(self.perspective, target.perspective)
            && feq(self.shear_x, target.shear_x)
            && feq(self.shear_y, target.shear_y)
            && feq(self.depth_angle, target.depth_angle)
    }

    pub fn transform(
        &self,
        x: f32,
        y: f32,
        origin: (f32, f32),
        viewport: (f32, f32),
    ) -> (f32, f32, f32) {
        let (ox, oy) = origin;
        let radians = self.rotation.to_radians();
        let (sin, cos) = radians.sin_cos();
        let lx = (x - ox) * self.scale;
        let ly = (y - oy) * self.scale;
        let sheared_x = lx + self.shear_x * ly;
        let sheared_y = ly + self.shear_y * lx;
        let rx = sheared_x * cos - sheared_y * sin;
        let ry = sheared_x * sin + sheared_y * cos;
        let depth_radians = self.depth_angle.to_radians();
        let depth_axis = rx * depth_radians.sin() + ry * depth_radians.cos();
        let depth =
            (1.0 + self.perspective * depth_axis / (viewport.1 * 0.5).max(1.0)).clamp(0.35, 2.0);
        (
            ox + self.offset_x * viewport.0 * 0.5 + rx * depth,
            oy + self.offset_y * viewport.1 * 0.5 + ry,
            self.scale * depth,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HexParams {
    pub parallax: bool,
    pub r: f32,
    pub rows: usize,
    pub cols: usize,
    pub scroll_step: usize,
    pub curve: HexCurve,
    pub shape: HexShape,
    pub curve_strength: f32,
    pub curve_frequency: f32,
    pub gap_x: f32,
    pub gap_y: f32,
    pub aspect: f32,
    pub stagger: f32,
    pub lens: f32,
    pub lens_radius: f32,
    pub orbit: f32,
    pub orbit_radius: f32,
    pub twist: f32,
    pub scatter: f32,
    pub stage: StageParams,
}

impl Default for HexParams {
    fn default() -> Self {
        Self {
            parallax: false,
            r: 140.0,
            rows: 3,
            cols: 7,
            scroll_step: 1,
            curve: HexCurve::Arc,
            shape: HexShape::Hexagon,
            curve_strength: 1.2,
            curve_frequency: 1.0,
            gap_x: 6.0,
            gap_y: 6.0,
            aspect: 1.0,
            stagger: 0.5,
            lens: 0.0,
            lens_radius: 620.0,
            orbit: 0.0,
            orbit_radius: 620.0,
            twist: 0.0,
            scatter: 0.0,
            stage: StageParams::default(),
        }
    }
}

impl HexParams {
    pub fn topology_differs(&self, target: &HexParams) -> bool {
        self.rows != target.rows
            || self.cols != target.cols
            || self.scroll_step != target.scroll_step
            || self.curve != target.curve
            || self.shape != target.shape
    }

    pub fn morph_toward(&mut self, target: &HexParams, amt: f32) {
        self.parallax = target.parallax;
        self.r = lerp(self.r, target.r, amt);
        self.rows = target.rows;
        self.cols = target.cols;
        self.scroll_step = target.scroll_step;
        self.curve = target.curve;
        self.shape = target.shape;
        self.curve_strength = lerp(self.curve_strength, target.curve_strength, amt);
        self.curve_frequency = lerp(self.curve_frequency, target.curve_frequency, amt);
        self.gap_x = lerp(self.gap_x, target.gap_x, amt);
        self.gap_y = lerp(self.gap_y, target.gap_y, amt);
        self.aspect = lerp(self.aspect, target.aspect, amt);
        self.stagger = lerp(self.stagger, target.stagger, amt);
        self.lens = lerp(self.lens, target.lens, amt);
        self.lens_radius = lerp(self.lens_radius, target.lens_radius, amt);
        self.orbit = lerp(self.orbit, target.orbit, amt);
        self.orbit_radius = lerp(self.orbit_radius, target.orbit_radius, amt);
        self.twist = lerp(self.twist, target.twist, amt);
        self.scatter = lerp(self.scatter, target.scatter, amt);
        self.stage.morph_toward(&target.stage, amt);
    }

    pub fn settled_to(&self, target: &HexParams) -> bool {
        self.parallax == target.parallax
            && feq(self.r, target.r)
            && self.rows == target.rows
            && self.cols == target.cols
            && self.scroll_step == target.scroll_step
            && self.curve == target.curve
            && self.shape == target.shape
            && feq(self.curve_strength, target.curve_strength)
            && feq(self.curve_frequency, target.curve_frequency)
            && feq(self.gap_x, target.gap_x)
            && feq(self.gap_y, target.gap_y)
            && feq(self.aspect, target.aspect)
            && feq(self.stagger, target.stagger)
            && feq(self.lens, target.lens)
            && feq(self.lens_radius, target.lens_radius)
            && feq(self.orbit, target.orbit)
            && feq(self.orbit_radius, target.orbit_radius)
            && feq(self.twist, target.twist)
            && feq(self.scatter, target.scatter)
            && self.stage.settled_to(&target.stage)
    }

    pub fn hex_h(&self) -> f32 {
        (self.r * 1.73205).ceil()
    }

    pub fn item_half_w(&self) -> f32 {
        match self.shape {
            HexShape::Rhombus => 2.0 * self.r * self.aspect,
            HexShape::Hexagon | HexShape::Triangle | HexShape::Diamond => self.r * self.aspect,
        }
    }

    pub fn item_half_h(&self) -> f32 {
        match self.shape {
            HexShape::Diamond => self.hex_h(),
            HexShape::Hexagon | HexShape::Triangle | HexShape::Rhombus => self.hex_h() / 2.0,
        }
    }

    pub fn step_x(&self) -> f32 {
        match self.shape {
            HexShape::Triangle => self.r * self.aspect + self.gap_x,
            HexShape::Diamond | HexShape::Rhombus => self.item_half_w() + self.gap_x,
            HexShape::Hexagon => 1.5 * self.r * self.aspect + self.gap_x,
        }
    }

    pub fn step_y(&self) -> f32 {
        self.item_half_h() * 2.0 + self.gap_y
    }

    pub fn content_h(&self) -> f32 {
        let stagger_tail =
            if matches!(self.shape, HexShape::Hexagon | HexShape::Diamond | HexShape::Rhombus) {
                self.step_y() * self.stagger
            } else {
                0.0
            };
        (self.rows.saturating_sub(1)) as f32 * self.step_y()
            + self.item_half_h() * 2.0
            + stagger_tail
    }

    pub fn visible_band(&self) -> f32 {
        self.column_x(self.cols.saturating_sub(1)) + self.item_half_w()
    }

    pub fn column_x(&self, col: usize) -> f32 {
        col as f32 * self.step_x() + self.item_half_w()
    }

    pub fn row_y(&self, row: usize) -> f32 {
        row as f32 * self.step_y() + self.item_half_h()
    }

    pub fn stagger_offset(&self, col: usize) -> f32 {
        if col.is_multiple_of(2)
            || !matches!(self.shape, HexShape::Hexagon | HexShape::Diamond | HexShape::Rhombus)
        {
            0.0
        } else {
            self.step_y() * self.stagger
        }
    }

    pub fn deform(&self, index: usize, x: f32, y: f32, origin: (f32, f32)) -> (f32, f32) {
        let mut lx = x - origin.0;
        let mut ly = y - origin.1;
        let radius = self.orbit_radius.max(1.0);
        let orbit = if self.curve == HexCurve::Cylinder {
            self.curve_strength.clamp(-1.0, 1.0)
        } else {
            self.orbit
        };
        let orbit_mix = orbit.abs().clamp(0.0, 1.0);
        if orbit_mix > 0.0 {
            let angle = lx / radius * orbit.signum();
            let ring = (radius + ly).max(radius * 0.08);
            lx = lerp(lx, angle.sin() * ring, orbit_mix);
            ly = lerp(ly, radius - angle.cos() * ring, orbit_mix);
        }
        if self.twist.abs() > 0.001 {
            let angle = self.twist.to_radians() * lx.hypot(ly) / radius;
            let (sin, cos) = angle.sin_cos();
            (lx, ly) = (lx * cos - ly * sin, lx * sin + ly * cos);
        }
        if self.scatter > 0.001 {
            lx += signed_hash(index, 0x4f1b_bcdd) * self.scatter;
            ly += signed_hash(index, 0x9e37_79b9) * self.scatter;
        }
        (origin.0 + lx, origin.1 + ly)
    }
}

pub(crate) use crate::frontend::animation::lerp;

pub(crate) fn feq(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.05
}

pub(crate) fn signed_hash(index: usize, salt: u32) -> f32 {
    let mut value = (index as u32).wrapping_add(salt);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^= value >> 16;
    value as f32 / u32::MAX as f32 * 2.0 - 1.0
}

#[derive(Debug, Clone, Copy)]
pub struct SliceParams {
    pub parallax: bool,
    pub shadows: bool,
    pub offset_x: f32,
    pub offset_y: f32,
    pub slice_w: f32,
    pub expanded_w: f32,
    pub slice_h: f32,
    pub spacing: f32,
    pub skew: f32,
    pub edge_tilt: f32,
    pub visible_count: usize,
    pub corners: CardCorners,
    pub wobble: bool,
    pub wobble_strength: f32,
}

pub fn wobble_pad(strength: f32) -> f32 {
    1.05 + 0.30 * strength.max(1.0)
}

pub fn wobble_bend(vel: f32, scale: f32) -> f32 {
    (vel / (scale * 2.2).max(1.0)).clamp(-1.35, 1.35)
}

pub fn slice_midline_width(width: f32, skew: f32) -> f32 {
    (width - skew.abs()).max(1.0)
}

impl SliceParams {
    pub fn topology_differs(&self, target: &SliceParams) -> bool {
        self.visible_count != target.visible_count || self.wobble != target.wobble
    }

    pub fn layout_width(&self, width: f32) -> f32 {
        slice_midline_width(width, self.skew)
    }

    pub fn slice_stride(&self) -> f32 {
        self.layout_width(self.slice_w) + self.spacing
    }

    pub fn morph_toward(&mut self, target: &SliceParams, amt: f32) {
        self.parallax = target.parallax;
        self.shadows = target.shadows;
        self.offset_x = lerp(self.offset_x, target.offset_x, amt);
        self.offset_y = lerp(self.offset_y, target.offset_y, amt);
        self.slice_w = lerp(self.slice_w, target.slice_w, amt);
        self.expanded_w = lerp(self.expanded_w, target.expanded_w, amt);
        self.slice_h = lerp(self.slice_h, target.slice_h, amt);
        self.spacing = lerp(self.spacing, target.spacing, amt);
        self.skew = lerp(self.skew, target.skew, amt);
        self.edge_tilt = lerp(self.edge_tilt, target.edge_tilt, amt);
        self.visible_count = target.visible_count;
        self.wobble = target.wobble;
        self.wobble_strength = lerp(self.wobble_strength, target.wobble_strength, amt);
        self.corners.morph_toward(&target.corners, amt);
    }

    pub fn settled_to(&self, target: &SliceParams) -> bool {
        self.parallax == target.parallax
            && self.shadows == target.shadows
            && feq(self.offset_x, target.offset_x)
            && feq(self.offset_y, target.offset_y)
            && feq(self.slice_w, target.slice_w)
            && feq(self.expanded_w, target.expanded_w)
            && feq(self.slice_h, target.slice_h)
            && feq(self.spacing, target.spacing)
            && feq(self.skew, target.skew)
            && feq(self.edge_tilt, target.edge_tilt)
            && self.visible_count == target.visible_count
            && self.wobble == target.wobble
            && feq(self.wobble_strength, target.wobble_strength)
            && self.corners.settled_to(&target.corners)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GridParams {
    pub cols: usize,
    pub rows: usize,
    pub thumb_w: f32,
    pub thumb_h: f32,
    pub gap_x: f32,
    pub gap_y: f32,
    pub corners: CardCorners,
    pub border_width: f32,
    pub layout: GridLayout,
    pub stagger: f32,
    pub selected_scale: f32,
    pub flow_wave: f32,
    pub flow_frequency: f32,
    pub scatter: f32,
    pub scale_variance: f32,
    pub cylinder_bend: f32,
    pub cylinder_radius: f32,
    pub stage: StageParams,
}

const GRID_FIT_EPSILON: f32 = 0.01;

impl Default for GridParams {
    fn default() -> Self {
        Self {
            cols: 6,
            rows: 3,
            thumb_w: 300.0,
            thumb_h: 169.0,
            gap_x: 16.0,
            gap_y: 16.0,
            corners: CardCorners::round(DEFAULT_CORNER as f32),
            border_width: 1.0,
            layout: GridLayout::Uniform,
            stagger: 0.5,
            selected_scale: 1.0,
            flow_wave: 0.0,
            flow_frequency: 1.0,
            scatter: 0.0,
            scale_variance: 0.0,
            cylinder_bend: 0.65,
            cylinder_radius: 720.0,
            stage: StageParams::default(),
        }
    }
}

impl GridParams {
    pub fn topology_differs(&self, target: &GridParams) -> bool {
        self.cols != target.cols || self.rows != target.rows || self.layout != target.layout
    }

    pub fn morph_toward(&mut self, target: &GridParams, amt: f32) {
        self.cols = target.cols;
        self.rows = target.rows;
        self.thumb_w = lerp(self.thumb_w, target.thumb_w, amt);
        self.thumb_h = lerp(self.thumb_h, target.thumb_h, amt);
        self.gap_x = lerp(self.gap_x, target.gap_x, amt);
        self.gap_y = lerp(self.gap_y, target.gap_y, amt);
        self.corners.morph_toward(&target.corners, amt);
        self.border_width = lerp(self.border_width, target.border_width, amt);
        self.layout = target.layout;
        self.stagger = lerp(self.stagger, target.stagger, amt);
        self.selected_scale = lerp(self.selected_scale, target.selected_scale, amt);
        self.flow_wave = lerp(self.flow_wave, target.flow_wave, amt);
        self.flow_frequency = lerp(self.flow_frequency, target.flow_frequency, amt);
        self.scatter = lerp(self.scatter, target.scatter, amt);
        self.scale_variance = lerp(self.scale_variance, target.scale_variance, amt);
        self.cylinder_bend = lerp(self.cylinder_bend, target.cylinder_bend, amt);
        self.cylinder_radius = lerp(self.cylinder_radius, target.cylinder_radius, amt);
        self.stage.morph_toward(&target.stage, amt);
    }

    pub fn settled_to(&self, target: &GridParams) -> bool {
        self.cols == target.cols
            && self.rows == target.rows
            && feq(self.thumb_w, target.thumb_w)
            && feq(self.thumb_h, target.thumb_h)
            && feq(self.gap_x, target.gap_x)
            && feq(self.gap_y, target.gap_y)
            && self.corners.settled_to(&target.corners)
            && feq(self.border_width, target.border_width)
            && self.layout == target.layout
            && feq(self.stagger, target.stagger)
            && feq(self.selected_scale, target.selected_scale)
            && feq(self.flow_wave, target.flow_wave)
            && feq(self.flow_frequency, target.flow_frequency)
            && feq(self.scatter, target.scatter)
            && feq(self.scale_variance, target.scale_variance)
            && feq(self.cylinder_bend, target.cylinder_bend)
            && feq(self.cylinder_radius, target.cylinder_radius)
            && self.stage.settled_to(&target.stage)
    }
}

impl GridParams {
    pub fn cylinder_transform(&self, x: f32, y: f32, origin: (f32, f32)) -> (f32, f32, f32) {
        let bend = self.cylinder_bend.clamp(-1.0, 1.0);
        if self.layout != GridLayout::Cylinder || bend.abs() <= 0.001 {
            return (x, y, 1.0);
        }
        let radius = self.cylinder_radius.max(1.0);
        let local_x = x - origin.0;
        let angle = (local_x / radius).clamp(-1.35, 1.35);
        let projected_x = angle.sin() * radius;
        let mix = bend.abs();
        let curved_x = lerp(local_x, projected_x, mix);
        let edge_depth = 1.0 - angle.cos();
        let scale = (1.0 - bend * edge_depth).clamp(0.35, 2.0);
        (origin.0 + curved_x, y, scale)
    }

    pub fn cell_w(&self) -> f32 {
        self.thumb_w + self.gap_x
    }

    pub fn cell_h(&self) -> f32 {
        self.thumb_h + self.gap_y
    }

    pub fn total_w(&self) -> f32 {
        self.span_w(self.cols)
    }

    pub fn total_h(&self) -> f32 {
        self.span_h(self.rows)
    }

    pub fn span_w(&self, columns: usize) -> f32 {
        self.thumb_w * columns as f32 + self.gap_x * columns.saturating_sub(1) as f32
    }

    pub fn span_h(&self, rows: usize) -> f32 {
        self.thumb_h * rows as f32 + self.gap_y * rows.saturating_sub(1) as f32
    }

    pub fn columns_in(&self, width: f32) -> usize {
        ((width + self.gap_x + GRID_FIT_EPSILON) / self.cell_w().max(1.0)).floor().max(1.0) as usize
    }

    pub fn rows_in(&self, height: f32) -> usize {
        ((height + self.gap_y + GRID_FIT_EPSILON) / self.cell_h().max(1.0)).floor().max(1.0)
            as usize
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Hit {
    pub index: usize,
    pub cx: f32,
    pub cy: f32,
    pub hw: f32,
    pub hh: f32,
    pub skew: f32,
    pub edge_tilt: f32,
    pub hex: bool,
    pub hex_shape: HexShape,
    pub triangle_direction: u8,
    pub quad: Option<[[f32; 2]; 4]>,
}

impl Hit {
    pub fn contains(&self, px: f32, py: f32) -> bool {
        if let Some(quad) = self.quad {
            let mut positive = false;
            let mut negative = false;
            for i in 0..4 {
                let a = quad[i];
                let b = quad[(i + 1) % 4];
                let cross = (b[0] - a[0]) * (py - a[1]) - (b[1] - a[1]) * (px - a[0]);
                positive |= cross > 0.0;
                negative |= cross < 0.0;
            }
            return positive != negative;
        }
        if self.hex {
            let qx = (px - self.cx).abs();
            let qy = (py - self.cy).abs();
            if self.hw <= 0.0 || self.hh <= 0.0 {
                return false;
            }
            if matches!(self.hex_shape, HexShape::Diamond | HexShape::Rhombus) {
                let x = (px - self.cx).abs() / self.hw;
                let y = (py - self.cy).abs() / self.hh;
                return x + y <= 1.0;
            }
            if self.hex_shape == HexShape::Triangle {
                let mut x = (px - self.cx) / self.hw;
                let mut y = (py - self.cy) / self.hh;
                match self.triangle_direction {
                    1 => y = -y,
                    2 => (x, y) = (y, x),
                    3 => (x, y) = (y, -x),
                    _ => {}
                }
                let side = |a: (f32, f32), b: (f32, f32)| {
                    (x - b.0) * (a.1 - b.1) - (a.0 - b.0) * (y - b.1)
                };
                let d1 = side((0.0, -1.0), (1.0, 1.0));
                let d2 = side((1.0, 1.0), (-1.0, 1.0));
                let d3 = side((-1.0, 1.0), (0.0, -1.0));
                let has_negative = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
                let has_positive = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
                return !(has_negative && has_positive);
            }
            return qy <= self.hh && qx / self.hw + 0.5 * qy / self.hh <= 1.0;
        }
        sheared_contains(self.cx, self.cy, self.hw, self.hh, self.skew, self.edge_tilt, px, py)
    }
}

pub fn sheared_contains(
    cx: f32,
    cy: f32,
    hw: f32,
    hh: f32,
    skew: f32,
    edge_tilt: f32,
    px: f32,
    py: f32,
) -> bool {
    if hw <= 0.0 || hh <= 0.0 {
        return false;
    }
    let sx = skew * 0.5;
    let ty = edge_tilt * 0.5;
    let bx = (hw - sx.abs()).max(1.0);
    let by = (hh - ty.abs()).max(1.0);
    let det = bx * by - sx * ty;
    if det.abs() < 1.0 {
        return false;
    }
    let x = px - cx;
    let y = py - cy;
    let u = (by * x + sx * y) / det;
    let v = (ty * x + bx * y) / det;
    u.abs() <= 1.0 && v.abs() <= 1.0
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CardCorners {
    pub sizes: [f32; 4],
    pub cut: bool,
}

impl CardCorners {
    pub fn read(corners: &Corners) -> Self {
        Self {
            sizes: corners.sizes().map(|size| size.max(0.0) as f32),
            cut: corners.style == CornerStyle::Chamfered,
        }
    }

    pub fn round(radius: f32) -> Self {
        Self { sizes: [radius; 4], cut: false }
    }

    pub fn scaled(self, factor: f32) -> Self {
        Self { sizes: self.sizes.map(|size| size * factor), ..self }
    }

    pub fn clamped(self, limit: f32) -> Self {
        Self { sizes: self.sizes.map(|size| size.clamp(0.0, limit.max(0.0))), ..self }
    }

    pub fn sheared(self, w: f32, h: f32, skew: f32, edge_tilt: f32) -> Self {
        Self { sizes: slice_clamped_corners(self.sizes, w, h, skew, edge_tilt), ..self }
    }

    pub fn square(&self) -> bool {
        self.sizes.iter().all(|size| feq(*size, 0.0))
    }

    pub fn stamp(self, inst: &mut InstanceRaw) {
        inst.radii = self.sizes;
        if self.cut {
            inst.misc[INSTANCE_FLAGS] |= CHAMFER;
        } else {
            inst.misc[INSTANCE_FLAGS] &= !CHAMFER;
        }
    }

    pub fn morph_toward(&mut self, target: &Self, amt: f32) {
        for (size, goal) in self.sizes.iter_mut().zip(target.sizes) {
            *size = lerp(*size, goal, amt);
        }
        if !target.square() || self.square() {
            self.cut = target.cut;
        }
    }

    pub fn settled_to(&self, target: &Self) -> bool {
        self.cut == target.cut && self.sizes.iter().zip(target.sizes).all(|(a, b)| feq(*a, b))
    }
}

pub fn corner_clamp(corners: [f32; 4], flat_w: f32, slant_len: f32) -> [f32; 4] {
    let rc_max = (flat_w / 2.0 - 1.0).min(slant_len / 2.0 - 1.0).max(0.0);
    corners.map(|r| r.clamp(0.0, rc_max))
}

pub fn slice_clamped_corners(
    corners: [f32; 4],
    w: f32,
    h: f32,
    skew: f32,
    edge_tilt: f32,
) -> [f32; 4] {
    let sk_abs = skew.abs();
    let tilt_abs = edge_tilt.abs();
    let horizontal_edge = ((w - sk_abs).max(0.001).powi(2) + edge_tilt.powi(2)).sqrt();
    let vertical_edge = (skew.powi(2) + (h - tilt_abs).max(0.001).powi(2)).sqrt();
    corner_clamp(corners, horizontal_edge, vertical_edge)
}

pub fn cover_crop(rect_w: f32, rect_h: f32, tex_w: f32, tex_h: f32) -> ([f32; 2], [f32; 2]) {
    let scale = (rect_w / tex_w).max(rect_h / tex_h);
    let vis_w = (rect_w / scale / tex_w).min(1.0);
    let vis_h = (rect_h / scale / tex_h).min(1.0);
    ([(1.0 - vis_w) * 0.5, (1.0 - vis_h) * 0.5], [vis_w, vis_h])
}

pub fn slice_opacity(
    item_center_x: f32,
    view_center_x: f32,
    half_view: f32,
    expanded_layout_w: f32,
    slice_stride: f32,
) -> f32 {
    if half_view <= 0.0 {
        return 1.0;
    }
    let full_zone = (0.6_f32).min((expanded_layout_w / 2.0 + 2.0 * slice_stride) / half_view);
    let norm_dist = (item_center_x - view_center_x).abs() / half_view;
    if norm_dist <= full_zone {
        1.0
    } else {
        (1.0 - (norm_dist - full_zone) / (1.2 - full_zone)).max(0.0)
    }
}

mod tests;
