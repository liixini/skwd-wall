use std::sync::Arc;

use crate::contracts::preview::{BufPool, UploadQueue};
use crate::contracts::rendering::{InstanceRaw, RendererSnapshot};

#[derive(Debug)]
pub struct ScenePrimitive {
    pub(crate) render: Arc<RendererSnapshot>,
    pub(crate) uploads: UploadQueue,
    pub(crate) pool: Arc<BufPool>,
    pub(crate) fade: f32,
}

#[derive(Debug)]
pub struct BrowserScenePrimitive {
    pub(crate) render: Arc<RendererSnapshot>,
    pub(crate) uploads: UploadQueue,
    pub(crate) pool: Arc<BufPool>,
    pub(crate) origin: [f32; 2],
}

#[derive(Clone, Copy)]
pub(super) enum ThumbFormat {
    Compressed,
    Rgba,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Globals {
    pub(super) resolution: [f32; 2],
    pub(super) time: f32,
    pub(super) vis: f32,
    pub(super) clip: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct TransUniformRaw {
    pub(super) resolution: [f32; 2],
    pub(super) origin: [f32; 2],
    pub(super) progress: f32,
    pub(super) kind: u32,
    pub(super) _pad0: f32,
    pub(super) _pad1: f32,
    pub(super) accent: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct SandyUniformRaw {
    pub(super) center: [f32; 2],
    pub(super) resolution: [f32; 2],
    pub(super) hero: [f32; 2],
    pub(super) dir: f32,
    pub(super) layer_a: f32,
    pub(super) layer_b: f32,
    pub(super) progress: f32,
    pub(super) time: f32,
    pub(super) vis: f32,
    pub(super) seed: f32,
    pub(super) strands: f32,
    pub(super) twist: f32,
    pub(super) orbit: f32,
    pub(super) turbulence: f32,
    pub(super) waist: f32,
    pub(super) front: f32,
    pub(super) fan: f32,
    pub(super) carry: f32,
    pub(super) layer_b2: f32,
    pub(super) layer_b3: f32,
    pub(super) bcut: f32,
    pub(super) bmix: f32,
    pub(super) arc: f32,
    pub(super) swap_loop: f32,
    pub(super) swirl: f32,
    pub(super) wave_flag: f32,
    pub(super) ring_spin: f32,
    pub(super) ring_wave: f32,
    pub(super) ring_soft: f32,
    pub(super) grid: [f32; 2],
    pub(super) swap_style: f32,
    pub(super) video_in: f32,
    pub(super) video_out: f32,
    pub(super) ring_size: f32,
    pub(super) _pad0: f32,
    pub(super) _pad1: f32,
}

pub(super) struct SandyTarget {
    pub(super) view: wgpu::TextureView,
    pub(super) width: u32,
    pub(super) height: u32,
}

pub(super) struct TransitionTargets {
    pub(super) a_view: wgpu::TextureView,
    pub(super) b_view: wgpu::TextureView,
    pub(super) width: u32,
    pub(super) height: u32,
}

pub struct ScenePipeline {
    pub(super) pipeline: wgpu::RenderPipeline,
    pub(super) globals: wgpu::Buffer,
    pub(super) bind_layout: wgpu::BindGroupLayout,
    pub(super) bind_group: wgpu::BindGroup,
    pub(super) sampler: wgpu::Sampler,
    pub(super) near_tex: wgpu::Texture,
    pub(super) far_tex: wgpu::Texture,
    pub(super) preview_tex: wgpu::Texture,
    pub(super) preview_out_tex: wgpu::Texture,
    pub(super) out_copy: bool,
    pub(super) near_format: wgpu::TextureFormat,
    pub(super) far_format: wgpu::TextureFormat,
    pub(super) atlas_size: u32,
    pub(super) near_cap: u32,
    pub(super) near_grow_chunk: u32,
    pub(super) near_layers: u32,
    pub(super) far_layers: u32,
    pub(super) instances: wgpu::Buffer,
    pub(super) instance_capacity: u64,
    pub(super) instance_count: u32,
    pub(super) old_instances: wgpu::Buffer,
    pub(super) old_capacity: u64,
    pub(super) old_count: u32,
    pub(super) format: wgpu::TextureFormat,
    pub(super) targets: Option<TransitionTargets>,
    pub(super) trans_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) trans_layout: wgpu::BindGroupLayout,
    pub(super) trans_uniform: wgpu::Buffer,
    pub(super) sandy_layout: wgpu::BindGroupLayout,
    pub(super) sandy_uniform: wgpu::Buffer,
    pub(super) sandy_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) sandy_bind: Option<wgpu::BindGroup>,
    pub(super) sandy_target: Option<SandyTarget>,
    pub(super) sandy_blit_layout: wgpu::BindGroupLayout,
    pub(super) sandy_blit_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) sandy_blit_bind: Option<wgpu::BindGroup>,
    pub(super) trans_bind: Option<wgpu::BindGroup>,
    pub(super) last_main: Option<(Arc<RendererSnapshot>, f32)>,
    pub(super) last_old: Option<(Arc<Vec<InstanceRaw>>, f32)>,
    pub(super) embedded_origin: [f32; 2],
}

pub struct BrowserScenePipeline(pub(super) ScenePipeline);

impl std::fmt::Debug for ScenePipeline {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("ScenePipeline").finish()
    }
}
