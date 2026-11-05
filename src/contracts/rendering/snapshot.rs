use std::sync::Arc;

pub const CHAMFER: u32 = 262_144;
pub const INSTANCE_FLAGS: usize = 3;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub(crate) rect: [f32; 4],
    pub(crate) radii: [f32; 4],
    pub(crate) fill: [f32; 4],
    pub(crate) tint: [f32; 4],
    pub(crate) border: [f32; 4],
    pub(crate) params: [f32; 4],
    pub(crate) uv: [f32; 4],
    pub(crate) crop: [f32; 4],
    pub(crate) misc: [u32; 4],
    pub(crate) flip: [f32; 4],
    pub(crate) shape: [f32; 4],
    pub(crate) quad_a: [f32; 4],
    pub(crate) quad_b: [f32; 4],
    pub(crate) quad_w: [f32; 4],
    pub(crate) quad_l: [f32; 4],
}

impl Default for InstanceRaw {
    fn default() -> Self {
        Self {
            rect: [0.0; 4],
            radii: [0.0; 4],
            fill: [0.0; 4],
            tint: [0.0; 4],
            border: [0.0; 4],
            params: [0.0; 4],
            uv: [0.0, 0.0, 1.0, 1.0],
            crop: [0.0, 0.0, 1.0, 1.0],
            misc: [0; 4],
            flip: [0.0; 4],
            shape: [0.0; 4],
            quad_a: [0.0; 4],
            quad_b: [0.0; 4],
            quad_w: [1.0; 4],
            quad_l: [0.0; 4],
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransSnap {
    pub old: Arc<Vec<InstanceRaw>>,
    pub progress: f32,
    pub kind: u32,
    pub origin: [f32; 2],
    pub accent: [f32; 4],
}

#[derive(Debug, Clone, Copy)]
pub struct SandySnap {
    pub layer_a: u32,
    pub layer_b: u32,
    pub progress: f32,
    pub center: [f32; 2],
    pub hero: [f32; 2],
    pub dir: f32,
    pub seed: f32,
    pub carry: f32,
    pub layer_b2: u32,
    pub layer_b3: u32,
    pub bcut: f32,
    pub bmix: f32,
    pub swirl: f32,
    pub wave: f32,
    pub ring: [f32; 4],
    pub grid: [f32; 2],
    pub res_scale: f32,
    pub video_in: f32,
    pub video_out: f32,
    pub knobs: [f32; 10],
}

/// Immutable frame payload consumed by the GPU scene pipeline.
///
/// Presentation-only hit regions and chrome deliberately live outside this contract.
#[derive(Debug, Default, Clone)]
pub struct RendererSnapshot {
    pub instances: Vec<InstanceRaw>,
    pub far_layers: u32,
    pub near_layers: u32,
    pub transition: Option<TransSnap>,
    pub clip: Option<[f32; 4]>,
    pub sandy: Option<SandySnap>,
    pub time: f32,
    pub vis: f32,
}
