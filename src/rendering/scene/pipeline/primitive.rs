use std::sync::Arc;

use iced::Rectangle;
use iced::widget::shader::{Primitive, Viewport};

use super::super::atlas;
use super::geometry::{
    draw_sandy, place_instances, sandy_is_scaled, sandy_target_dimensions, scale_instances,
};
use super::model::{
    BrowserScenePipeline, BrowserScenePrimitive, Globals, SandyUniformRaw, ScenePipeline,
    ScenePrimitive, TransUniformRaw,
};
use super::textures::{upload_instances, upload_near_container, upload_near_rgba};
use crate::contracts::preview::Upload;
use crate::contracts::rendering::TransSnap;

pub(super) fn sandy_target_required(
    visible: bool,
    transitioning: bool,
    res_scale: Option<f32>,
) -> bool {
    visible && !transitioning && res_scale.is_some_and(sandy_is_scaled)
}

impl Primitive for ScenePrimitive {
    type Pipeline = ScenePipeline;

    fn prepare(
        &self,
        pipeline: &mut ScenePipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        let sf = viewport.scale_factor();
        crate::contracts::display::set_surface_scale(sf);
        let transitioning = self.render.transition.is_some();
        if !transitioning && pipeline.targets.is_some() {
            pipeline.release_transition_targets();
        }
        let sandy_target = sandy_target_required(
            self.render.vis > 0.0,
            transitioning,
            self.render.sandy.as_ref().map(|snap| snap.res_scale),
        );
        if !sandy_target && pipeline.sandy_target.is_some() {
            pipeline.release_sandy_target();
        }
        if pipeline.far_layers < self.render.far_layers {
            pipeline.recreate_far(device, self.render.far_layers);
        }
        if pipeline.near_layers < self.render.near_layers {
            let target = self
                .render
                .near_layers
                .next_multiple_of(pipeline.near_grow_chunk)
                .min(pipeline.near_cap);
            pipeline.recreate_near(device, queue, target);
        }
        self.drain_uploads(pipeline, queue);
        if self.vis() <= 0.0 {
            return;
        }
        if let Some(nv) = &self.render.sandy {
            pipeline.ensure_sandy(device);
            let scaled = sandy_is_scaled(nv.res_scale);
            let ssf = if scaled { sf * nv.res_scale } else { sf };
            if sandy_target {
                let (ow, oh) =
                    sandy_target_dimensions(bounds.width, bounds.height, sf, nv.res_scale);
                pipeline.ensure_sandy_blit(device);
                pipeline.ensure_sandy_target(device, ow, oh);
            }
            let uniform = SandyUniformRaw {
                center: [nv.center[0] * ssf, nv.center[1] * ssf],
                resolution: [bounds.width * ssf, bounds.height * ssf],
                hero: [nv.hero[0] * ssf, nv.hero[1] * ssf],
                dir: nv.dir,
                layer_a: nv.layer_a as f32,
                layer_b: nv.layer_b as f32,
                progress: nv.progress,
                time: self.render.time,
                vis: self.vis(),
                seed: nv.seed,
                strands: nv.knobs[0],
                twist: nv.knobs[1],
                orbit: nv.knobs[2],
                turbulence: nv.knobs[3],
                waist: nv.knobs[4],
                front: nv.knobs[5],
                fan: nv.knobs[6],
                carry: nv.carry,
                layer_b2: nv.layer_b2 as f32,
                layer_b3: nv.layer_b3 as f32,
                bcut: nv.bcut,
                bmix: nv.bmix,
                arc: nv.knobs[7],
                swap_loop: nv.knobs[8],
                swirl: nv.swirl,
                wave_flag: nv.wave,
                ring_spin: nv.ring[0],
                ring_wave: nv.ring[1],
                ring_soft: nv.ring[2],
                grid: nv.grid,
                swap_style: nv.knobs[9],
                video_in: nv.video_in,
                video_out: nv.video_out,
                ring_size: nv.ring[3],
                _pad0: 0.0,
                _pad1: 0.0,
            };
            queue.write_buffer(&pipeline.sandy_uniform, 0, bytemuck::bytes_of(&uniform));
        }
        let clip = self.render.clip.map_or([-1e9, -1e9, 1e9, 1e9], |rect| rect.map(|val| val * sf));
        let globals = Globals {
            resolution: [bounds.width * sf, bounds.height * sf],
            time: self.render.time,
            vis: self.vis(),
            clip,
        };
        queue.write_buffer(&pipeline.globals, 0, bytemuck::bytes_of(&globals));

        let main_same = pipeline
            .last_main
            .as_ref()
            .is_some_and(|(prev, scale)| Arc::ptr_eq(prev, &self.render) && *scale == sf);
        if !main_same {
            let scaled = scale_instances(&self.render.instances, sf);
            pipeline.instance_count = upload_instances(
                device,
                queue,
                &mut pipeline.instances,
                &mut pipeline.instance_capacity,
                &scaled,
            );
            pipeline.last_main = Some((self.render.clone(), sf));
        }

        if let Some(trans) = &self.render.transition {
            prepare_transition(pipeline, device, queue, bounds, sf, trans);
        }
    }

    fn draw(&self, _pipeline: &ScenePipeline, _render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        false
    }

    fn render(
        &self,
        pipeline: &ScenePipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        if self.vis() <= 0.0 {
            return;
        }
        if pipeline.out_copy {
            encoder.copy_texture_to_texture(
                pipeline.preview_tex.as_image_copy(),
                pipeline.preview_out_tex.as_image_copy(),
                wgpu::Extent3d {
                    width: atlas::NEAR_W,
                    height: atlas::NEAR_H,
                    depth_or_array_layers: 1,
                },
            );
        }
        if self.render.transition.is_none() {
            let sandy_offscreen =
                self.render.sandy.as_ref().is_some_and(|nv| sandy_is_scaled(nv.res_scale))
                    && pipeline.sandy_target.is_some()
                    && pipeline.sandy_blit_pipeline.is_some()
                    && pipeline.sandy_bind.is_some();
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("skwd scene direct"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: target,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_scissor_rect(
                    clip_bounds.x,
                    clip_bounds.y,
                    clip_bounds.width,
                    clip_bounds.height,
                );
                if pipeline.instance_count > 0 {
                    pass.set_pipeline(&pipeline.pipeline);
                    pass.set_bind_group(0, &pipeline.bind_group, &[]);
                    pass.set_vertex_buffer(0, pipeline.instances.slice(..));
                    pass.draw(0..6, 0..pipeline.instance_count);
                }
                if !sandy_offscreen
                    && let Some(nv) = &self.render.sandy
                    && let (Some(np), Some(nb)) = (&pipeline.sandy_pipeline, &pipeline.sandy_bind)
                {
                    draw_sandy(&mut pass, np, nb, nv);
                }
            }
            if sandy_offscreen
                && let Some(nv) = &self.render.sandy
                && let (Some(rt), Some(np), Some(nb), Some(bp), Some(bb)) = (
                    &pipeline.sandy_target,
                    &pipeline.sandy_pipeline,
                    &pipeline.sandy_bind,
                    &pipeline.sandy_blit_pipeline,
                    &pipeline.sandy_blit_bind,
                )
            {
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("skwd sandy offscreen"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &rt.view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    draw_sandy(&mut pass, np, nb, nv);
                }
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("skwd sandy composite"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: target,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    pass.set_scissor_rect(
                        clip_bounds.x,
                        clip_bounds.y,
                        clip_bounds.width,
                        clip_bounds.height,
                    );
                    pass.set_pipeline(bp);
                    pass.set_bind_group(0, bb, &[]);
                    pass.draw(0..3, 0..1);
                }
            }
            return;
        }
        let Some(targets) = &pipeline.targets else { return };
        let Some(trans_bind) = &pipeline.trans_bind else { return };
        let Some(trans_pipeline) = &pipeline.trans_pipeline else { return };
        for (view, buffer, count) in [
            (&targets.a_view, &pipeline.old_instances, pipeline.old_count),
            (&targets.b_view, &pipeline.instances, pipeline.instance_count),
        ] {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("skwd transition scene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if count > 0 {
                pass.set_pipeline(&pipeline.pipeline);
                pass.set_bind_group(0, &pipeline.bind_group, &[]);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..6, 0..count);
            }
        }
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("skwd transition composite"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_scissor_rect(
            clip_bounds.x.min(targets.width.saturating_sub(1)),
            clip_bounds.y.min(targets.height.saturating_sub(1)),
            clip_bounds.width.min(targets.width),
            clip_bounds.height.min(targets.height),
        );
        pass.set_pipeline(trans_pipeline);
        pass.set_bind_group(0, trans_bind, &[]);
        pass.draw(0..3, 0..1);
    }
}

impl Primitive for BrowserScenePrimitive {
    type Pipeline = BrowserScenePipeline;

    fn prepare(
        &self,
        pipeline: &mut BrowserScenePipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        let scene = ScenePrimitive {
            render: self.render.clone(),
            uploads: self.uploads.clone(),
            pool: self.pool.clone(),
            fade: 1.0,
        };
        let origin = self.origin;
        if pipeline.0.embedded_origin != origin {
            pipeline.0.embedded_origin = origin;
            pipeline.0.last_main = None;
            pipeline.0.last_old = None;
        }
        <ScenePrimitive as Primitive>::prepare(
            &scene,
            &mut pipeline.0,
            device,
            queue,
            bounds,
            viewport,
        );
        let sf = viewport.scale_factor();
        let clip = self.render.clip.map_or([-1e9, -1e9, 1e9, 1e9], |rect| {
            [
                (rect[0] + origin[0]) * sf,
                (rect[1] + origin[1]) * sf,
                (rect[2] + origin[0]) * sf,
                (rect[3] + origin[1]) * sf,
            ]
        });
        let globals = Globals {
            resolution: [viewport.physical_width() as f32, viewport.physical_height() as f32],
            time: self.render.time,
            vis: self.render.vis,
            clip,
        };
        queue.write_buffer(&pipeline.0.globals, 0, bytemuck::bytes_of(&globals));
        let placed = place_instances(&self.render.instances, sf, origin);
        pipeline.0.instance_count = upload_instances(
            device,
            queue,
            &mut pipeline.0.instances,
            &mut pipeline.0.instance_capacity,
            &placed,
        );
    }

    fn draw(
        &self,
        pipeline: &BrowserScenePipeline,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) -> bool {
        let scene = ScenePrimitive {
            render: self.render.clone(),
            uploads: self.uploads.clone(),
            pool: self.pool.clone(),
            fade: 1.0,
        };
        <ScenePrimitive as Primitive>::draw(&scene, &pipeline.0, render_pass)
    }

    fn render(
        &self,
        pipeline: &BrowserScenePipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        let scene = ScenePrimitive {
            render: self.render.clone(),
            uploads: self.uploads.clone(),
            pool: self.pool.clone(),
            fade: 1.0,
        };
        <ScenePrimitive as Primitive>::render(&scene, &pipeline.0, encoder, target, clip_bounds);
    }
}

impl ScenePrimitive {
    fn vis(&self) -> f32 {
        self.render.vis * self.fade
    }

    fn drain_uploads(&self, pipeline: &mut ScenePipeline, queue: &wgpu::Queue) {
        pipeline.out_copy = false;
        let mut uploads = self.uploads.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        for up in uploads.drain(..) {
            self.apply_upload(pipeline, queue, up);
        }
    }

    fn apply_upload(&self, pipeline: &mut ScenePipeline, queue: &wgpu::Queue, mut up: Upload) {
        if up.tier == 5 {
            pipeline.out_copy = true;
            return;
        }
        if up.tier == 2 && up.layer >= pipeline.far_layers {
            return;
        }
        if up.tier == 1 {
            if up.layer >= pipeline.near_layers {
                return;
            }
            if up.compressed {
                upload_near_container(queue, &pipeline.near_tex, up.layer, &up.data);
            } else {
                upload_near_rgba(queue, &pipeline.near_tex, up.layer, up.w, up.h, &up.data);
            }
            return;
        }
        write_atlas_tile(queue, pipeline, &up);
        if up.tier >= 3 || !up.compressed {
            self.pool.put(std::mem::take(&mut up.data));
        }
    }
}

fn write_atlas_tile(queue: &wgpu::Queue, pipeline: &ScenePipeline, up: &Upload) {
    let tex = match up.tier {
        3 => &pipeline.preview_tex,
        4 => &pipeline.preview_out_tex,
        _ => &pipeline.far_tex,
    };
    let (bytes_per_row, rows_per_image) =
        if up.tier >= 3 || !up.compressed { (up.w * 4, up.h) } else { ((up.w / 4) * 8, up.h / 4) };
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d { x: up.x, y: up.y, z: up.layer },
            aspect: wgpu::TextureAspect::All,
        },
        &up.data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_row),
            rows_per_image: Some(rows_per_image),
        },
        wgpu::Extent3d { width: up.w, height: up.h, depth_or_array_layers: 1 },
    );
}

fn prepare_transition(
    pipeline: &mut ScenePipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bounds: &Rectangle,
    sf: f32,
    trans: &TransSnap,
) {
    let old_same = pipeline
        .last_old
        .as_ref()
        .is_some_and(|(prev, scale)| Arc::ptr_eq(prev, &trans.old) && *scale == sf);
    if !old_same {
        let scaled_old = scale_instances(&trans.old, sf);
        pipeline.old_count = upload_instances(
            device,
            queue,
            &mut pipeline.old_instances,
            &mut pipeline.old_capacity,
            &scaled_old,
        );
        pipeline.last_old = Some((trans.old.clone(), sf));
    }
    let w = (bounds.width * sf).round().max(1.0) as u32;
    let h = (bounds.height * sf).round().max(1.0) as u32;
    pipeline.ensure_transition_pipeline(device);
    pipeline.ensure_transition_targets(device, w, h);
    let uniform = TransUniformRaw {
        resolution: [w as f32, h as f32],
        origin: trans.origin,
        progress: trans.progress,
        kind: trans.kind,
        _pad0: 0.0,
        _pad1: 0.0,
        accent: trans.accent,
    };
    queue.write_buffer(&pipeline.trans_uniform, 0, bytemuck::bytes_of(&uniform));
}
