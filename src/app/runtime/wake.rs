use log::{info, warn};
use serde_json::json;

use crate::infrastructure::preview::DecodeFailed;
use crate::infrastructure::runtime::Wake;
use crate::rendering::scene::atlas::AtlasMap;

#[allow(clippy::wildcard_imports)]
use super::super::*;

impl App {
    pub(in crate::app) fn handle_wake(&mut self, wake: Wake) -> bool {
        match wake {
            Wake::SelfTest(step) => {
                let color = if step == 3 {
                    -1
                } else {
                    crate::frontend::ui::cycle_color_right(self.library_session.filters.color)
                };
                info!("selftest step {step}: color filter -> {color}");
                let kind = step.min(3);
                self.scene.begin_transition(kind, [0.3 + 0.2 * step as f32, 0.5]);
                self.change_filters(|filters| filters.color = color);
                true
            }
            Wake::HudTick => {
                self.call_tracked("diag", json!({}), Pending::Diag);
                false
            }
            Wake::Ipc(message) => self.handle_ipc(message),
            Wake::Desktop(desktop) => {
                crate::app::update::strand::delivered(self, desktop);
                true
            }
            Wake::Decoded => {
                self.drain_decodes();
                true
            }
            Wake::BrowserPreview { key, generation, size, result } => {
                self.on_browser_preview_decoded(&key, generation, size, result);
                true
            }
            Wake::EffectSrc { card, source, rgba, w, h } => {
                if let Some(effects) = self.panels.effects.as_mut()
                    && effects.matches_source(card, &source)
                {
                    effects.set_source_texture_arc(rgba, w, h);
                }
                true
            }
            Wake::Semantic(result) => {
                self.apply_semantic_result(result);
                true
            }
            Wake::Frame(_) | Wake::Toggle | Wake::Hide | Wake::Command(_) | Wake::Query(..) => {
                log::error!("warm: {wake:?} must be handled by update_inner, not handle_wake");
                false
            }
        }
    }

    fn drain_decodes(&mut self) {
        self.drain_browser_wall_decodes();
        if self.preview_resources.decoder.is_idle() {
            if self.preview_resources.decoder_was_busy {
                crate::shell::trim_heap();
            }
            self.preview_resources.decoder_was_busy = false;
        } else {
            self.preview_resources.decoder_was_busy = true;
        }
        let (count, average_ms, max_ms) = self.preview_resources.decoder.drain_stats();
        if count > 0 {
            self.runtime_state.metrics.note_decodes(count, average_ms, max_ms);
        }
        let cancelled = self.preview_resources.decoder.drain_cancelled();
        let drained = self.preview_resources.decoder.drain_done();
        let Some(atlas) = self.preview_resources.atlas.as_mut() else {
            return;
        };
        for index in cancelled {
            atlas.near.release(index);
        }
        for decoded in drained.done {
            mark_decoded(atlas, &mut self.scene, decoded.store_idx, decoded.tier);
        }
        if !drained.failed.is_empty() {
            release_failed(atlas, &mut self.scene, &drained.failed);
        }
    }

    fn drain_browser_wall_decodes(&mut self) {
        let wall = &mut self.source_browser.wall;
        wall.decoder_was_busy = !wall.decoder.is_idle();
        let (count, average_ms, max_ms) = wall.decoder.drain_stats();
        if count > 0 {
            self.runtime_state.metrics.note_decodes(count, average_ms, max_ms);
        }
        let cancelled = wall.decoder.drain_cancelled();
        let drained = wall.decoder.drain_done();
        let Some(atlas) = wall.atlas.as_mut() else {
            return;
        };
        for index in cancelled {
            atlas.near.release(index);
        }
        for decoded in drained.done {
            mark_decoded(atlas, &mut wall.scene, decoded.store_idx, decoded.tier);
        }
        if !drained.failed.is_empty() {
            release_failed(atlas, &mut wall.scene, &drained.failed);
        }
        if !drained.failed.is_empty() || count > 0 {
            wall.chrome_cache.clear();
        }
    }
}

fn mark_decoded(
    atlas: &mut AtlasMap,
    scene: &mut crate::app::scene::SceneCore,
    index: usize,
    tier: u32,
) {
    if tier == 3 {
        scene.tall_ready(index);
        scene.touch();
        return;
    }
    if tier != 1 {
        atlas.far.mark_ready(index);
        scene.touch();
        return;
    }
    atlas.near.mark_ready(index);
    if atlas.far.ready(index).is_none() {
        scene.on_decoded(index);
    } else {
        scene.touch();
    }
}

fn release_failed(
    atlas: &mut AtlasMap,
    scene: &mut crate::app::scene::SceneCore,
    failed: &[DecodeFailed],
) {
    for entry in failed {
        if entry.tier == 3 {
            scene.tall_failed(entry.store_idx);
        } else if entry.tier == 1 {
            atlas.near.release(entry.store_idx);
            atlas.near_failed.insert(entry.store_idx);
        } else {
            atlas.far.release(entry.store_idx);
            atlas.failed.insert(entry.store_idx);
        }
    }
    warn!(
        "{} thumb decodes failed (total {}), sample: {}",
        failed.len(),
        atlas.failed.len(),
        failed[0].error
    );
}
