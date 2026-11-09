use crate::frontend::animation::MotionTier;
use crate::frontend::scene::hand::{
    self, CARD_STAGGER_MS, EASE_DEAL_IN, EASE_DEAL_OUT, EASE_RIBBON_IN, EASE_RIBBON_OUT,
    GHOST_LEVELS, IN_MS, Move, OUT_MS, Pose, PoseTween, RIBBON_CARD_STAGGER_MS, RIBBON_STAGGER_MS,
    SWAP_GAP_MS,
};
use crate::frontend::scene::layout::Mode;

use super::model::{
    CardAnim, DealPhase, Face, HandDeal, Layout, RebuildCtx, Reveal, RevealPhase, SceneCore,
};

const FLIP_KICK: f32 = 260.0;
const REVEAL_KICK: f32 = 300.0;
const SLAT_KICK: f32 = 90.0;
const DEAL_KICK_Y: f32 = 620.0;
const DEAL_KICK_X: f32 = 280.0;
const DRAG_GAIN_X: f32 = 0.34;
const DRAG_GAIN_Y: f32 = 0.42;

impl SceneCore {
    pub(super) fn hand_stage_scale(&self) -> f32 {
        hand::stage_scale(self.viewport.0, self.viewport.1)
    }

    pub(super) fn hand_push(&self) -> f32 {
        let len = self.hand.shown.len();
        (0..len).map(|slot| self.hand_sel_t(self.hand.offset + slot)).sum::<f32>().clamp(0.0, 1.0)
    }

    pub(super) fn hand_sel_t(&self, idx: usize) -> f32 {
        self.card
            .selection
            .get(&idx)
            .map_or(if idx == self.current { 1.0 } else { 0.0 }, |spr| spr.x.clamp(0.0, 1.0))
    }

    pub(super) fn hand_lift_t(&self, idx: usize) -> f32 {
        self.hand.lift.get(&idx).map_or(0.0, |spr| spr.x.clamp(0.0, 1.0))
    }

    pub(super) fn hand_rest_pose(&self, slot: usize, len: usize, push: f32) -> Pose {
        let hp = self.xp.hand;
        let k = self.hand_stage_scale();
        let idx = self.hand.offset + slot;
        let base = hand::fan_slot(slot, len, None);
        let mut n = base;
        for other in (0..len).filter(|other| *other != slot) {
            let t = self.hand_sel_t(self.hand.offset + other);
            if t > 0.0 {
                n += t * (hand::fan_slot(slot, len, Some(other)) - base);
            }
        }
        let sel_t = self.hand_sel_t(idx);
        let lift_t = self.hand_lift_t(idx);
        let fan = hand::fan_pose(
            n,
            &hp.fan(k),
            k,
            1.0 + 0.24 * push,
            -24.0 * k * lift_t,
            -150.0 * k * push,
            1.0 + 0.06 * lift_t,
        );
        Pose::mix(&fan, &hand::sel_pose(k), sel_t)
    }

    pub(super) fn hand_slot_n(slot: usize, len: usize) -> f32 {
        slot as f32 - (len.max(1) as f32 - 1.0) * 0.5
    }

    pub fn hand_pointer(&mut self, x: f32, y: f32, hit: Option<usize>) {
        let (x, y) = self.unplace_pointer(x, y);
        let (vw, vh) = self.viewport;
        let hp = self.xp.hand;
        if let Some((last_x, last_y)) = self.hand.drag {
            let dx = x - last_x;
            let dy = y - last_y;
            self.hand.drag = Some((x, y));
            self.hand.rig_y.x += dx * DRAG_GAIN_Y;
            self.hand.rig_x.x -= dy * DRAG_GAIN_X;
            self.hand.rig_y.v = dx * DRAG_GAIN_Y * 30.0;
            self.hand.rig_x.v = -dy * DRAG_GAIN_X * 30.0;
            self.motion.needs_frame = true;
        } else {
            let (tx, ty) = hand::tilt_target(x.clamp(0.0, vw), y.clamp(0.0, vh), vw, vh, hp.tilt);
            self.hand.rig_x.retarget(tx);
            self.hand.rig_y.retarget(ty);
        }
        let motion_ms = self.motion_ms(MotionTier::Fast);
        let hovered =
            if self.hand.deal.is_some() || self.hand.reveal.is_some() { None } else { hit };
        for (idx, spring) in &mut self.hand.lift {
            if Some(*idx) != hovered {
                spring.retarget(0.0);
            }
        }
        if let Some(idx) = hovered {
            self.hand
                .lift
                .entry(idx)
                .or_insert_with(|| self.motion.profile.override_spring(0.0, motion_ms))
                .retarget(1.0);
        }
        self.motion.needs_frame = true;
    }

    pub fn hand_drag_start(&mut self, x: f32, y: f32) {
        self.hand.drag = Some(self.unplace_pointer(x, y));
    }

    pub fn hand_drag_end(&mut self) {
        if self.hand.drag.take().is_some() {
            self.motion.needs_frame = true;
        }
    }

    pub(super) fn hand_select(&mut self, idx: usize, count: usize) {
        let size = self.xp.hand.count;
        let len = hand::hand_len(self.hand.offset, count, size);
        let inside = idx >= self.hand.offset && idx < self.hand.offset + len;
        if inside && self.hand.deal.is_none() {
            if self.hand.reveal.is_some() {
                let old = self.current;
                self.card.selection.insert(old, self.motion.profile.override_spring(0.0, 1.0));
                self.card.selection.insert(idx, self.motion.profile.override_spring(1.0, 1.0));
                return;
            }
            let motion_ms = self.motion_ms(MotionTier::Standard);
            let old = self.current;
            let mut off = self
                .motion
                .profile
                .override_spring(self.card.selection.get(&old).map_or(1.0, |spr| spr.x), motion_ms);
            off.retarget(0.0);
            self.card.selection.insert(old, off);
            let mut on = self
                .motion
                .profile
                .override_spring(self.card.selection.get(&idx).map_or(0.0, |spr| spr.x), motion_ms);
            on.retarget(1.0);
            self.card.selection.insert(idx, on);
            return;
        }
        if inside {
            return;
        }
        let new_offset = hand::hand_window(idx, count, size);
        if self.hand.reveal.is_some() {
            let old = self.current;
            self.card.selection.insert(old, self.motion.profile.override_spring(0.0, 1.0));
            self.card.selection.insert(idx, self.motion.profile.override_spring(1.0, 1.0));
            self.hand.offset = new_offset;
            self.motion.needs_frame = true;
            return;
        }
        self.current = idx;
        self.hand_deal(new_offset);
    }

    pub(super) fn hand_deal(&mut self, new_offset: usize) {
        let hp = self.xp.hand;
        let k = self.hand_stage_scale();
        let ss = hp.speed_scale();
        let pick_seed = hand::rnd(self.hand.deals as f32 + 0.5, 9.0, 8.0, 7.0);
        let mv = hand::pick_move(hp.deal_mode, hp.moves, self.hand.cycle, pick_seed);
        self.hand.cycle = mv;
        self.hand.deals += 1;
        let seed = hand::rnd(self.hand.deals as f32, 1.0, 2.0, 3.0);
        self.hand.lseed = hand::rnd(self.hand.deals as f32, 5.0, 6.0, 7.0);
        let nr = hp.ribbons;
        let axis = hp.axis;
        let push = self.hand_push();
        let len = self.hand.shown.len();
        let cards = match self.hand.deal.take() {
            Some(deal) => {
                let now = deal.t;
                let mut cards = deal.cards;
                for card in &mut cards {
                    let to = hand::move_pose(mv, card.n, 1.0, k);
                    let delay = card.slot as f32 * CARD_STAGGER_MS * ss;
                    card.pose.retarget(now, to, delay, OUT_MS * ss, EASE_DEAL_OUT);
                    for (level, ghost) in card.ghosts.iter_mut().enumerate() {
                        ghost.retarget(
                            now,
                            to,
                            delay,
                            OUT_MS * GHOST_LEVELS[level].2 * ss,
                            EASE_DEAL_OUT,
                        );
                    }
                    if mv == Move::Ribbon {
                        card.ribbons.resize(
                            nr,
                            PoseTween::new(Pose::REST, Pose::REST, 0.0, 1.0, EASE_RIBBON_OUT),
                        );
                        for (r, ribbon) in card.ribbons.iter_mut().enumerate() {
                            ribbon.retarget(
                                now,
                                hand::ribbon_deal_pose(r, 1.0, k, axis),
                                (card.slot as f32 * RIBBON_CARD_STAGGER_MS
                                    + r as f32 * RIBBON_STAGGER_MS)
                                    * ss,
                                OUT_MS * ss,
                                EASE_RIBBON_OUT,
                            );
                        }
                    } else {
                        for ribbon in &mut card.ribbons {
                            ribbon.retarget(now, Pose::REST, 0.0, OUT_MS * ss, EASE_RIBBON_OUT);
                        }
                    }
                }
                cards
            }
            None => (0..len)
                .map(|slot| {
                    let n = Self::hand_slot_n(slot, len);
                    let from = self.hand_rest_pose(slot, len, push);
                    let to = hand::move_pose(mv, n, 1.0, k);
                    let delay = slot as f32 * CARD_STAGGER_MS * ss;
                    let ribbons = if mv == Move::Ribbon {
                        (0..nr)
                            .map(|r| {
                                PoseTween::new(
                                    Pose::REST,
                                    hand::ribbon_deal_pose(r, 1.0, k, axis),
                                    (slot as f32 * RIBBON_CARD_STAGGER_MS
                                        + r as f32 * RIBBON_STAGGER_MS)
                                        * ss,
                                    OUT_MS * ss,
                                    EASE_RIBBON_OUT,
                                )
                            })
                            .collect()
                    } else {
                        Vec::new()
                    };
                    CardAnim {
                        store: self.hand.shown[slot],
                        slot,
                        n,
                        pose: PoseTween::new(from, to, delay, OUT_MS * ss, EASE_DEAL_OUT),
                        ribbons,
                        ghosts: GHOST_LEVELS.map(|(_, _, mult)| {
                            PoseTween::new(from, to, delay, OUT_MS * mult * ss, EASE_DEAL_OUT)
                        }),
                    }
                })
                .collect(),
        };
        let end = Self::deal_end(&cards) + SWAP_GAP_MS * ss / 1000.0;
        self.hand.deal = Some(HandDeal { phase: DealPhase::Out, t: 0.0, mv, cards, end });
        self.hand.reveal = None;
        self.hand.offset = new_offset;
        self.card.selection.clear();
        self.hand.lift.clear();
        self.hover = None;
        self.hand.rig_y.v += (hand::rnd(seed, 1.0, 2.0, 3.0) - 0.5) * DEAL_KICK_Y;
        self.hand.rig_x.v += (hand::rnd(seed, 3.0, 2.0, 1.0) - 0.5) * DEAL_KICK_X;
        self.motion.needs_frame = true;
    }

    fn deal_end(cards: &[CardAnim]) -> f32 {
        cards
            .iter()
            .map(|card| card.ribbons.iter().map(PoseTween::end).fold(card.pose.end(), f32::max))
            .fold(0.0, f32::max)
    }

    fn hand_swap(&mut self, ctx: &RebuildCtx<'_>) {
        let hp = self.xp.hand;
        let k = self.hand_stage_scale();
        let ss = hp.speed_scale();
        let count = ctx.filtered.len();
        let Some(deal) = self.hand.deal.as_mut() else { return };
        let len = hand::hand_len(self.hand.offset, count, hp.count);
        let mv = deal.mv;
        deal.cards = (0..len)
            .map(|slot| {
                let n = Self::hand_slot_n(slot, len);
                let from = hand::move_pose(mv, n, -1.0, k);
                let to = hand::fan_pose(n, &hp.fan(k), k, 1.0, 0.0, 0.0, 1.0);
                let delay = slot as f32 * CARD_STAGGER_MS * ss;
                let ribbons = if mv == Move::Ribbon {
                    (0..hp.ribbons)
                        .map(|r| {
                            PoseTween::new(
                                hand::ribbon_deal_pose(r, -1.0, k, hp.axis),
                                Pose::REST,
                                (slot as f32 * RIBBON_CARD_STAGGER_MS
                                    + (hp.ribbons - 1 - r) as f32 * RIBBON_STAGGER_MS)
                                    * ss,
                                IN_MS * ss,
                                EASE_RIBBON_IN,
                            )
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                CardAnim {
                    store: ctx.filtered[self.hand.offset + slot] as usize,
                    slot,
                    n,
                    pose: PoseTween::new(from, to, delay, IN_MS * ss, EASE_DEAL_IN),
                    ribbons,
                    ghosts: GHOST_LEVELS.map(|(_, _, mult)| {
                        PoseTween::new(from, to, delay, IN_MS * mult * ss, EASE_DEAL_IN)
                    }),
                }
            })
            .collect();
        deal.end = Self::deal_end(&deal.cards);
        deal.phase = DealPhase::In;
        deal.t = 0.0;
    }

    fn hand_land(&mut self) {
        self.hand.deal = None;
        let motion_ms = self.motion_ms(MotionTier::Standard);
        let mut on = self.motion.profile.override_spring(0.0, motion_ms);
        on.retarget(1.0);
        self.card.selection.insert(self.current, on);
        self.motion.needs_frame = true;
    }

    pub(super) fn tick_hand(&mut self, ctx: &RebuildCtx<'_>, dt: f32) {
        if self.mode != Mode::Hand {
            return;
        }
        self.hand.rig_x.tick(dt);
        self.hand.rig_y.tick(dt);
        if self.xp.hand.bob {
            self.hand.bob_t += dt;
        }
        for spring in self.hand.lift.values_mut() {
            spring.tick(dt);
        }
        self.hand.lift.retain(|_, spring| !(spring.settled() && spring.target == 0.0));
        self.hand.backdrop_fade.tick(dt);
        if self.hand.backdrop_fade.settled() {
            self.hand.backdrop_prev = None;
        }
        let step = dt.min(0.05);
        if let Some(deal) = self.hand.deal.as_mut() {
            deal.t += step;
            if deal.t >= deal.end {
                match deal.phase {
                    DealPhase::Out => {
                        if ctx.filtered.is_empty() {
                            self.hand.deal = None;
                        } else {
                            self.hand_swap(ctx);
                        }
                    }
                    DealPhase::In => self.hand_land(),
                }
            }
        }
        self.hand_flip_clock(step);
        self.hand_reveal_clock(ctx, step);
    }

    pub fn hand_reveal_toggle(&mut self) {
        let opening = self.hand.reveal.as_ref().is_none_or(|reveal| !reveal.open);
        if !opening && self.card.flipped.is_some() {
            self.close_flip();
        }
        self.hand.rig_y.v += if opening { REVEAL_KICK } else { -REVEAL_KICK };
        self.hand.rig_x.v -= REVEAL_KICK * 0.4;
        for spring in self.card.selection.values_mut() {
            spring.snap(spring.target);
        }
        match self.hand.reveal.as_mut() {
            None => {
                if self.card.flipped.is_some() {
                    self.close_flip();
                }
                self.hand.lift.clear();
                self.hand.reveal = Some(Reveal {
                    open: true,
                    phase: RevealPhase::Held,
                    from: Layout::Fan,
                    to: Layout::Fan,
                    turn: 0.0,
                    turns_done: 0,
                    faces: [Face::Card; 2],
                    len: self.hand.shown.len(),
                });
            }
            Some(reveal) => reveal.open = !reveal.open,
        }
        self.motion.needs_frame = true;
    }

    pub fn hand_reveal_close(&mut self) {
        if let Some(reveal) = self.hand.reveal.as_mut() {
            reveal.open = false;
            self.motion.needs_frame = true;
        }
    }

    pub fn hand_reveal_open(&self) -> bool {
        self.hand.reveal.as_ref().is_some_and(|reveal| reveal.open)
    }

    fn hand_reveal_clock(&mut self, ctx: &RebuildCtx<'_>, step: f32) {
        let ss = self.xp.hand.speed_scale();
        let cur = ctx.filtered.get(self.current).map(|&store| store as usize);
        let near_ready = cur.filter(|&store| {
            ctx.atlas.as_ref().is_some_and(|atlas| atlas.near.ready(store).is_some())
        });
        let tall = cur.is_some_and(|store| ctx.catalog.items[store].is_tall());
        let column = tall && self.hand.tall_ready == cur;
        let cur_ready = if tall && !column {
            near_ready.filter(|&store| self.hand.tall_failed == Some(store))
        } else {
            near_ready
        };
        let details = self.card.flipped == Some(self.current) && self.card.flip.target > 0.5;
        let Some(reveal) = self.hand.reveal.as_mut() else { return };
        if reveal.phase == RevealPhase::Turning {
            reveal.turn += step;
            if reveal.turn < hand::reveal_turn_end(reveal.len, ss) {
                return;
            }
            reveal.turns_done += 1;
            reveal.turn = 0.0;
            reveal.phase = RevealPhase::Held;
            reveal.from = reveal.to;
            if reveal.to == Layout::Fan && !reveal.open {
                self.hand.reveal = None;
                return;
            }
        }
        let visible = reveal.faces[hand::reveal_face(reveal.turns_done, 0.0)];
        let (incoming, to) = if reveal.open && details {
            let store = cur_ready.or(cur).unwrap_or(0);
            let back = Face::Back { store, column: column && cur_ready.is_some() };
            if visible == back { (None, reveal.to) } else { (Some(back), reveal.to) }
        } else if reveal.open {
            match cur_ready {
                Some(store) if visible != (Face::Slice { store, column }) => (
                    Some(Face::Slice { store, column }),
                    if column { Layout::Column } else { Layout::Row },
                ),
                _ => (None, reveal.to),
            }
        } else if reveal.turns_done == 0 {
            self.hand.reveal = None;
            return;
        } else {
            (Some(Face::Card), Layout::Fan)
        };
        if let Some(face) = incoming {
            reveal.faces[((reveal.turns_done + 1) % 2) as usize] = face;
            reveal.len = self.hand.shown.len().max(1);
            reveal.turn = 0.0;
            reveal.phase = RevealPhase::Turning;
            reveal.to = to;
            let dir = hand::reveal_dir(0, reveal.turns_done);
            self.hand.rig_y.v += dir * SLAT_KICK;
            self.hand.rig_x.v += SLAT_KICK * 0.5;
        }
    }

    pub fn tall_ready(&mut self, store: usize) {
        self.hand.tall_ready = Some(store);
        self.motion.needs_frame = true;
    }

    pub fn tall_failed(&mut self, store: usize) {
        self.hand.tall_failed = Some(store);
        self.motion.needs_frame = true;
    }

    pub fn tall_needs(&mut self, store: usize) -> bool {
        let wanted = self.mode == Mode::Hand
            && self.hand.reveal.as_ref().is_some_and(|reveal| reveal.open)
            && self.hand.tall_ready != Some(store)
            && self.hand.tall_failed != Some(store)
            && self.hand.tall_requested != Some(store);
        if wanted {
            self.hand.tall_requested = Some(store);
        }
        wanted
    }

    pub(super) fn hand_tall_in_use(&self) -> bool {
        self.mode == Mode::Hand
            && self.hand.reveal.as_ref().is_some_and(|reveal| {
                reveal.faces.iter().any(|face| matches!(face, Face::Slice { column: true, .. }))
            })
    }

    fn hand_reveal_running(&self) -> bool {
        self.hand.reveal.as_ref().is_some_and(|reveal| reveal.phase != RevealPhase::Held)
    }

    pub(super) fn hand_flip_begin(&mut self, closing: bool) {
        if closing {
            self.hand.flip_closing = self.card.flip.x >= 0.999;
            self.hand.rig_y.v -= FLIP_KICK;
        } else {
            self.hand.flip_p = 0.0;
            self.hand.flip_closing = false;
            self.hand.flip_seed =
                hand::rnd(self.hand.deals as f32 + 11.0, self.current as f32, 3.0, 9.0);
            self.hand.rig_y.v += FLIP_KICK;
        }
        if self.hand.deal.is_some() {
            self.hand.deal = None;
            self.card.selection.clear();
        }
        self.motion.needs_frame = true;
    }

    fn hand_flip_clock(&mut self, dt: f32) {
        if self.card.flipped.is_none() {
            self.hand.flip_p = 0.0;
            return;
        }
        let total = self.card.flip_duration_ms / 1000.0;
        let opening = self.card.flip.target > 0.5;
        if opening {
            self.hand.flip_p = (self.hand.flip_p + dt / total).min(1.0);
        } else {
            self.hand.flip_p = (self.hand.flip_p - dt / total).max(0.0);
        }
        if (opening && self.hand.flip_p >= 1.0) || (!opening && self.hand.flip_p <= 0.0) {
            self.card.flip.snap(self.card.flip.target);
        } else {
            self.card.flip.x = self.hand.flip_p;
        }
    }

    fn hand_flip_running(&self) -> bool {
        self.card.flipped.is_some()
            && if self.card.flip.target > 0.5 {
                self.hand.flip_p < 1.0
            } else {
                self.hand.flip_p > 0.0
            }
    }

    pub(super) fn hand_animating(&self) -> bool {
        self.mode == Mode::Hand
            && (self.hand.deal.is_some()
                || self.hand_flip_running()
                || self.hand_reveal_running()
                || !self.hand.backdrop_fade.settled()
                || !self.hand.rig_x.settled()
                || !self.hand.rig_y.settled()
                || self.hand.lift.values().any(|spring| !spring.settled())
                || (self.xp.hand.bob && !self.hidden()))
    }

    pub(super) fn hand_reason(&self) -> Option<&'static str> {
        if self.mode != Mode::Hand {
            return None;
        }
        if self.hand.deal.is_some() {
            Some("hand_deal")
        } else if self.hand_flip_running() {
            Some("hand_flip")
        } else if self.hand_reveal_running() {
            Some("hand_reveal")
        } else if !self.hand.backdrop_fade.settled() {
            Some("hand_backdrop")
        } else if !self.hand.rig_x.settled() || !self.hand.rig_y.settled() {
            Some("hand_rig")
        } else if self.hand.lift.values().any(|spring| !spring.settled()) {
            Some("hand_lift")
        } else if self.xp.hand.bob && !self.hidden() {
            Some("hand_bob")
        } else {
            None
        }
    }

    pub(super) fn hand_backdrop_target(
        &mut self,
        store: usize,
        ready: bool,
    ) -> (Option<usize>, Option<usize>, f32) {
        if self.hand.backdrop_store != Some(store) && ready {
            if self.hand.backdrop_store.is_some() {
                self.hand.backdrop_prev = self.hand.backdrop_store;
                self.hand.backdrop_fade.run(0.0, 1.0);
            }
            self.hand.backdrop_store = Some(store);
        }
        (
            self.hand.backdrop_store,
            self.hand.backdrop_prev,
            self.hand.backdrop_fade.x.clamp(0.0, 1.0),
        )
    }

    pub(super) fn hand_start_middle(&mut self, count: usize) {
        if !self.user_engaged && self.current == 0 && count > 0 {
            self.current = (self.xp.hand.count / 2).min(count - 1);
        }
        if self.hand.deal.is_none() {
            self.hand.offset = hand::hand_window(self.current, count, self.xp.hand.count);
        }
    }

    pub(super) fn hand_reset(&mut self, count: usize) {
        self.hand_start_middle(count);
        self.hand.deal = None;
        self.hand.reveal = None;
        self.hand.lift.clear();
        self.hand.drag = None;
        self.motion.needs_frame = true;
    }

    pub(super) fn hand_shift_after_insert(&mut self, pos: usize) {
        if pos <= self.hand.offset && self.hand.offset > 0 {
            self.hand.offset += 1;
        }
    }

    #[cfg(test)]
    pub(super) fn hand_reveal_phase(
        &self,
    ) -> Option<(RevealPhase, u32, [Face; 2], (Layout, Layout))> {
        self.hand
            .reveal
            .as_ref()
            .map(|reveal| (reveal.phase, reveal.turns_done, reveal.faces, (reveal.from, reveal.to)))
    }

    #[cfg(test)]
    pub(crate) fn hand_dealing(&self) -> bool {
        self.hand.deal.is_some()
    }

    #[cfg(test)]
    pub(crate) fn hand_offset(&self) -> usize {
        self.hand.offset
    }

    #[cfg(test)]
    pub(crate) fn hand_rig(&self) -> (f32, f32) {
        (self.hand.rig_x.x, self.hand.rig_y.x)
    }

    #[cfg(test)]
    pub(crate) fn hand_dragging(&self) -> bool {
        self.hand.drag.is_some()
    }
}
