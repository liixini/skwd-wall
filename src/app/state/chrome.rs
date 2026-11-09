use std::collections::HashMap;

use iced::widget::canvas;

use crate::frontend::animation::{MotionProfile, MotionTier, Spring, Tween};

pub(crate) struct PaneScroll {
    pub(crate) spring: Spring,
    pub(crate) max: f32,
    pub(crate) dirty: bool,
}

impl PaneScroll {
    pub(crate) fn at(y: f32, max: f32, motion: MotionProfile) -> Self {
        let mut spring = motion.spring(0.0, MotionTier::Standard);
        spring.snap(y);
        Self { spring, max, dirty: false }
    }
}

pub(crate) struct BarState {
    pub(crate) cache: BarCache,
    pub(crate) hover: Option<usize>,
    pub(crate) menu_hover: Option<usize>,
    pub(crate) menu_scroll: Spring,
    pub(crate) menu: Option<crate::frontend::ui::MenuKind>,
    pub(crate) reorder: Option<crate::frontend::ui::ReorderPreview>,
}

impl BarState {
    fn new(motion: MotionProfile) -> Self {
        Self {
            cache: BarCache::new(),
            hover: None,
            menu_hover: None,
            menu_scroll: motion.spring(0.0, MotionTier::Standard),
            menu: None,
            reorder: None,
        }
    }
}

pub(crate) struct BarCache {
    canvas: canvas::Cache,
    zone_stale: bool,
}

impl BarCache {
    fn new() -> Self {
        Self { canvas: canvas::Cache::new(), zone_stale: true }
    }

    pub(crate) fn clear(&mut self) {
        self.canvas.clear();
        self.zone_stale = true;
    }

    pub(crate) fn repaint(&mut self) {
        self.canvas.clear();
    }

    pub(crate) fn canvas(&self) -> &canvas::Cache {
        &self.canvas
    }

    pub(crate) fn zone_stale(&self) -> bool {
        self.zone_stale
    }

    pub(crate) fn zone_placed(&mut self) {
        self.zone_stale = false;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BarHover {
    Out,
    Over,
    Muted,
}

pub(crate) struct ChromeState {
    pub(crate) filter_bar_visible: bool,
    pub(crate) filter_bar_forced: bool,
    pub(crate) filter_bar_mode: skwd_config::FilterBarVisibility,
    pub(crate) filter_bar_hover: BarHover,
    pub(crate) filter_bar_hover_zone: iced::Rectangle,
    pub(crate) filter_bar_reveal: Tween,
    pub(crate) filter_bar_orientation_reveal: Tween,
    pub(crate) filter_bar_vertical: bool,
    filter_bar_pending_vertical: Option<bool>,
    pub(crate) bar: BarState,
    pub(crate) cache: canvas::Cache,
    pub(crate) signature: u64,
    pub(crate) pane_scrolls: HashMap<&'static str, PaneScroll>,
    pub(crate) motion: MotionProfile,
}

impl ChromeState {
    pub(crate) fn new(
        filter_bar_mode: skwd_config::FilterBarVisibility,
        filter_bar_forced: bool,
        filter_bar_vertical: bool,
        motion: MotionProfile,
    ) -> Self {
        let filter_bar_visible =
            filter_bar_forced || filter_bar_mode == skwd_config::FilterBarVisibility::Always;
        let visible = if filter_bar_visible { 1.0 } else { 0.0 };
        Self {
            filter_bar_visible,
            filter_bar_forced,
            filter_bar_mode,
            filter_bar_hover: BarHover::Out,
            filter_bar_hover_zone: iced::Rectangle::default(),
            filter_bar_reveal: motion.tween(visible, MotionTier::Fast),
            filter_bar_orientation_reveal: motion.tween(1.0, MotionTier::Fast),
            filter_bar_vertical,
            filter_bar_pending_vertical: None,
            bar: BarState::new(motion),
            cache: canvas::Cache::new(),
            signature: 0,
            pane_scrolls: HashMap::new(),
            motion,
        }
    }

    pub(crate) fn set_motion_profile(&mut self, motion: MotionProfile) {
        self.motion = motion;
        motion.retime_tween(&mut self.filter_bar_reveal, MotionTier::Fast);
        motion.retime_tween(&mut self.filter_bar_orientation_reveal, MotionTier::Fast);
        motion.retime_spring(&mut self.bar.menu_scroll, MotionTier::Standard);
        if let Some(reorder) = self.bar.reorder.as_mut() {
            reorder.retime(motion);
        }
        for pane in self.pane_scrolls.values_mut() {
            motion.retime_spring(&mut pane.spring, MotionTier::Standard);
        }
    }

    pub(crate) fn sync_filter_bar(&mut self, vertical: bool) {
        self.filter_bar_reveal.retarget(if self.filter_bar_shown() { 1.0 } else { 0.0 });
        if vertical != self.filter_bar_vertical
            && self.filter_bar_pending_vertical != Some(vertical)
        {
            self.filter_bar_pending_vertical = Some(vertical);
            self.filter_bar_orientation_reveal.retarget(0.0);
        }
    }

    pub(crate) fn tick_filter_bar(&mut self, dt: f32) -> bool {
        let mut changed = self.filter_bar_reveal.tick(dt);
        changed |= self.filter_bar_orientation_reveal.tick(dt);
        if self.filter_bar_orientation_reveal.settled()
            && self.filter_bar_orientation_reveal.target == 0.0
            && let Some(vertical) = self.filter_bar_pending_vertical.take()
        {
            self.filter_bar_vertical = vertical;
            self.filter_bar_orientation_reveal.retarget(1.0);
            changed = true;
        }
        if let Some(reorder) = self.bar.reorder.as_mut() {
            let animating = reorder.animating();
            if animating {
                reorder.tick(dt);
                changed = true;
            }
            if reorder.finished() {
                self.bar.reorder = None;
                changed = true;
            }
        }
        changed
    }

    pub(crate) fn set_filter_bar(&mut self, shown: bool) {
        if self.filter_bar_hover != BarHover::Out {
            self.filter_bar_hover = if shown { BarHover::Over } else { BarHover::Muted };
        }
        self.filter_bar_visible = shown;
        self.bar.menu = None;
        self.bar.cache.repaint();
    }

    pub(crate) fn track_filter_hover(&mut self, inside: bool) -> bool {
        let next = match (inside, self.filter_bar_hover) {
            (false, _) => BarHover::Out,
            (true, BarHover::Out) => BarHover::Over,
            (true, held) => held,
        };
        let changed = next != self.filter_bar_hover;
        self.filter_bar_hover = next;
        if changed {
            self.bar.cache.repaint();
        }
        changed
    }

    pub(crate) fn clear_filter_hover(&mut self) -> bool {
        self.track_filter_hover(false)
    }

    pub(crate) fn snap_filter_bar(&mut self) {
        self.filter_bar_reveal.snap(if self.filter_bar_shown() { 1.0 } else { 0.0 });
        self.bar.cache.repaint();
    }

    pub(crate) fn filter_bar_shown(&self) -> bool {
        self.filter_bar_visible || self.filter_bar_hover == BarHover::Over
    }

    pub(crate) fn filter_bar_fade(&self) -> f32 {
        self.filter_bar_reveal.x * self.filter_bar_orientation_reveal.x
    }

    pub(crate) fn filter_bar_animating(&self) -> bool {
        !self.filter_bar_reveal.settled()
            || !self.filter_bar_orientation_reveal.settled()
            || self.filter_bar_pending_vertical.is_some()
            || self.bar.reorder.as_ref().is_some_and(crate::frontend::ui::ReorderPreview::animating)
    }
}

#[cfg(test)]
mod tests;
