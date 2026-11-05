use super::labels;
use super::model::{Anchor, Prompt, Target, Tool};
use super::state::{Arrange, Choice, Extra};
use crate::domain::strand::{CORNERS, Item, Rect, anchor, thread_y, zone_top};
use crate::frontend::strand::draw::selection::corner;
use crate::frontend::strand::labels::plate_width;
use crate::frontend::strand::selection::{Bar, BarPlate, shake};
use crate::frontend::strand::tokens::{FRAME, HIT_BEAD, LOOK, PLATE};
use crate::i18n::tr;

const PICKER_TOOLS: [Tool; 6] =
    [Tool::Prev, Tool::Layout, Tool::Next, Tool::Spacing, Tool::Corners, Tool::Reset];
const FLOATING_TOOLS: [Tool; 2] = [Tool::Hide, Tool::Reset];
const REVIEW_HEIGHT: f32 = 96.0;
const REVIEW_GAP: f32 = 24.0;
const ROW_GAP: f32 = 10.0;

pub(crate) struct Slot {
    pub(crate) choice: Option<Choice>,
    pub(crate) label: String,
    pub(crate) rect: Rect,
    pub(crate) on: bool,
}

pub(crate) struct Review {
    pub(crate) before: Rect,
    pub(crate) after: Rect,
}

impl Arrange {
    pub(crate) fn tools(&self, target: Target) -> Vec<Tool> {
        match target {
            Target::Picker => PICKER_TOOLS
                .into_iter()
                .filter(|tool| match tool {
                    Tool::Spacing => self.board.style.spacing.is_some(),
                    Tool::Corners => self.board.style.corners.is_some(),
                    _ => true,
                })
                .collect(),
            Target::Bar | Target::Search => vec![Tool::Reset],
            Target::Arrange | Target::Swatch => FLOATING_TOOLS.to_vec(),
        }
    }

    pub(crate) fn bar(&self, target: Target, frame: Rect) -> Bar<Tool> {
        let style = &self.board.style;
        let mut plates = Vec::new();
        let mut x: f32 = 0.0;
        for tool in self.tools(target) {
            if tool.leads() && !plates.is_empty() {
                x += PLATE.group - PLATE.gap;
            }
            let label = labels::tool(tool, style);
            let width = plate_width(&label);
            plates.push(BarPlate {
                tool,
                label,
                rect: Rect::new(x, 0.0, width, PLATE.height),
                on: false,
                dim: false,
                rank: 0,
            });
            x += width + PLATE.gap;
        }
        let placed = Bar::place(plates, frame, self.output);
        let floating = target.edit_id().is_some();
        let held = self.spool.held();
        let others: Vec<Rect> = self
            .targets()
            .into_iter()
            .filter(|other| *other != target)
            .filter(
                |other| {
                    if floating { *other != Target::Picker } else { other.edit_id().is_none() }
                },
            )
            .map(|other| self.rect(other))
            .chain(held.iter().copied())
            .collect();
        if !placed.overlaps(&others) {
            return placed;
        }
        let below = frame.bottom() + PLATE.lift;
        let room = zone_top(self.output[1]) - FRAME.edge - PLATE.height;
        let mut spots = Vec::new();
        if !floating {
            spots.push((frame.y + PLATE.lift, false));
        }
        if below <= room {
            spots.insert(0, (below, true));
        }
        let clear = held.iter().map(|rect| rect.y).fold(f32::INFINITY, f32::min);
        if clear - PLATE.gap - PLATE.height >= FRAME.edge {
            spots.push((clear - PLATE.gap - PLATE.height, false));
        }
        for (y, under) in spots {
            let moved = placed.clone().at(y, under);
            if !moved.overlaps(&others) {
                return moved;
            }
        }
        placed
    }

    pub(crate) fn shaking(&self, tool: Tool) -> f32 {
        let Some(at) = self.shake else { return 0.0 };
        if matches!(tool, Tool::Prev | Tool::Layout | Tool::Next) {
            shake(self.now, at)
        } else {
            0.0
        }
    }

    pub(crate) fn framed(&self) -> Option<Rect> {
        let [screen, high] = self.output;
        let frame = self.selection.frame_rect()?;
        let x = frame.x.max(FRAME.edge);
        let y = frame.y.max(FRAME.edge);
        let right = frame.right().min(screen - FRAME.edge);
        let bottom = frame.bottom().min(high - FRAME.edge);
        Some(Rect::new(x, y, right - x, bottom - y))
    }

    pub(crate) fn screen(&self) -> Rect {
        Rect::new(0.0, 0.0, self.output[0], self.output[1])
    }

    pub(crate) fn anchor_of(&self, target: Target, rect: Rect) -> Anchor {
        let picker = self.objects[Target::Picker.index()].target();
        let attached =
            target != Target::Picker && self.guides.iter().any(|guide| guide.with == Some(picker));
        if attached {
            Anchor::Picker(anchor(rect, picker))
        } else {
            Anchor::Screen(anchor(rect, self.screen()))
        }
    }

    pub(crate) fn kept_anchor(&self, target: Target, rect: Rect) -> Anchor {
        let picker = self.objects[Target::Picker.index()].target();
        if self.board.attached[target.index()] {
            Anchor::Picker(anchor(rect, picker))
        } else {
            Anchor::Screen(anchor(rect, self.screen()))
        }
    }

    pub(crate) fn anchor_text(&self, target: Target, rect: Rect) -> String {
        let (picker, cell, frame) = match self.anchor_of(target, rect) {
            Anchor::Picker(cell) => (true, cell, self.objects[Target::Picker.index()].target()),
            Anchor::Screen(cell) => (false, cell, self.screen()),
        };
        let [cx, cy] = rect.centre();
        let [fx, fy] = frame.centre();
        let dx = [rect.x - frame.x, cx - fx, frame.right() - rect.right()][cell[0]];
        let dy = [rect.y - frame.y, cy - fy, frame.bottom() - rect.bottom()][cell[1]];
        labels::anchor(picker, cell, [dx, dy])
    }

    fn row(&self, entries: Vec<(Option<Choice>, String, bool)>, y: f32) -> Vec<Slot> {
        let widths: Vec<f32> = entries.iter().map(|(_, label, _)| plate_width(label)).collect();
        let total = widths.iter().sum::<f32>() + PLATE.gap * widths.len().saturating_sub(1) as f32;
        let mut x = (self.output[0] / 2.0 - total / 2.0).max(FRAME.edge);
        entries
            .into_iter()
            .zip(widths)
            .map(|((choice, label, on), width)| {
                let rect = Rect::new(x, y, width, PLATE.height);
                x += width + PLATE.gap;
                Slot { choice, label, rect, on }
            })
            .collect()
    }

    fn base(&self) -> f32 {
        thread_y(self.output[1]) - LOOK.busy_rise - PLATE.height
    }

    pub(crate) fn panel(&self) -> Vec<Slot> {
        let base = self.base();
        if let Some(prompt) = &self.board.prompt {
            let entries = match prompt {
                Prompt::Conflict => vec![
                    (None, tr("arrange-conflict-label").to_owned(), false),
                    (Some(Choice::Keep), tr("arrange-conflict-keep").to_owned(), true),
                    (Some(Choice::Saved), tr("arrange-conflict-saved").to_owned(), true),
                ],
                Prompt::Disconnected(outputs) => vec![
                    (None, labels::disconnected(outputs), false),
                    (Some(Choice::Consent), tr("arrange-disconnected-confirm").to_owned(), true),
                ],
            };
            return self.row(entries, base);
        }
        let Some(copy) = &self.board.copy else { return Vec::new() };
        let reviewed = copy.review.as_ref().map(|review| review.output.as_str());
        let mut slots = self.row(
            copy.recipients
                .iter()
                .enumerate()
                .map(|(index, recipient)| {
                    let on = reviewed == Some(recipient.name.as_str());
                    (Some(Choice::Recipient(index)), labels::recipient(recipient), on)
                })
                .collect(),
            base,
        );
        if let Some(review) = &copy.review {
            let mut actions = Vec::new();
            if !review.compatible {
                actions.push((Some(Choice::Fit), tr("arrange-display-fit").to_owned(), false));
            }
            if !review.connected {
                let label = tr("arrange-display-save-disconnected").to_owned();
                actions.push((Some(Choice::Reconnect), label, review.save_disconnected));
            }
            actions.push((Some(Choice::Apply), tr("arrange-display-apply").to_owned(), true));
            actions.push((Some(Choice::Skip), tr("arrange-display-skip").to_owned(), false));
            if !review.connected {
                actions.push((
                    Some(Choice::Forget),
                    tr("arrange-display-forget").to_owned(),
                    false,
                ));
            }
            slots.extend(self.row(actions, base - PLATE.height - ROW_GAP));
        }
        slots
    }

    pub(crate) fn review(&self) -> Option<Review> {
        let review = self.board.copy.as_ref()?.review.as_ref()?;
        let shape = |viewport: (f32, f32)| {
            let width = REVIEW_HEIGHT * viewport.0 / viewport.1.max(1.0);
            [width, REVIEW_HEIGHT]
        };
        let [before_w, height] = shape(review.before.viewport);
        let [after_w, _] = shape(review.after.viewport);
        let total = before_w + REVIEW_GAP + after_w;
        let left = self.output[0] / 2.0 - total / 2.0;
        let y = self.base() - 2.0 * (PLATE.height + ROW_GAP) - height;
        Some(Review {
            before: Rect::new(left, y, before_w, height),
            after: Rect::new(left + before_w + REVIEW_GAP, y, after_w, height),
        })
    }

    pub(crate) fn back_rect(&self) -> Rect {
        let width = plate_width(tr(labels::BACK));
        let done = self
            .spool
            .laid
            .as_ref()
            .and_then(|laid| laid.plan.items.iter().find(|placed| placed.item == Item::Done));
        let right = done.map_or(self.output[0] / 2.0 + width / 2.0, |placed| {
            self.spool.item_x(self.now, placed.item) + placed.reach[1]
        });
        let x = (right - width).clamp(FRAME.edge, self.output[0] - FRAME.edge - width);
        let y = self.spool.plate_y(self.now, Item::Done, right - width, width);
        Rect::new(x, y, width, PLATE.height)
    }

    pub(crate) fn plates(&self) -> Vec<(String, Rect)> {
        let mut plates = Vec::new();
        if self.preview {
            plates.push(("back".to_owned(), self.back_rect()));
            return plates;
        }
        if let Some(laid) = &self.spool.laid {
            for placed in laid.plan.items.iter().filter(|placed| !placed.off) {
                let x = self.spool.item_x(self.now, placed.item);
                let y = self.spool.thread_at(x);
                let (id, rect) = match placed.item {
                    Item::Bead(kind, style) => {
                        let id = self.bead_id((kind, style)).unwrap_or_default();
                        (format!("bead:{id}"), Rect::centred([x, y], [2.0 * HIT_BEAD; 2]))
                    }
                    Item::Group(kind) => {
                        (format!("group:{kind}"), Rect::centred([x, y], [2.0 * HIT_BEAD; 2]))
                    }
                    Item::Kind(kind) => {
                        (format!("family:{kind}"), self.item_rect(placed.item, x, placed.reach[1]))
                    }
                    Item::Extra(index) => {
                        let extra = self.extras().get(index).map_or("extra", |extra| match extra {
                            Extra::Preview => "preview",
                            Extra::Displays => "displays",
                        });
                        (extra.to_owned(), self.item_rect(placed.item, x, placed.reach[1]))
                    }
                    Item::Cancel => {
                        ("cancel".to_owned(), self.item_rect(placed.item, x, placed.reach[1]))
                    }
                    Item::Done => {
                        ("done".to_owned(), self.item_rect(placed.item, x, placed.reach[1]))
                    }
                };
                plates.push((id, rect));
            }
        }
        if let (Some(target), Some(frame)) = (self.selection.id, self.framed()) {
            for plate in self.bar(target, frame).plates {
                plates.push((format!("tool:{:?}", plate.tool), plate.rect));
            }
            if target == Target::Picker {
                for index in 0..CORNERS.len() {
                    let gem = Rect::centred(corner(frame, index), [2.0 * FRAME.gem_hit; 2]);
                    plates.push((format!("gem:{index}"), gem));
                }
            }
        }
        if let Some(toast) = self.spool.toast_plates(self.now) {
            plates.push(("undo".to_owned(), toast.undo_rect()));
        }
        for slot in self.panel() {
            if let Some(choice) = slot.choice {
                plates.push((format!("choice:{choice:?}"), slot.rect));
            }
        }
        plates
    }

    fn item_rect(&self, item: Item, x: f32, width: f32) -> Rect {
        Rect::new(x, self.spool.plate_y(self.now, item, x, width), width, PLATE.height)
    }
}
