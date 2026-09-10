use crate::frontend::animation::{smoothstep, window};
use crate::frontend::scene::BackPanel;
use crate::i18n::tr;

#[derive(Debug, Clone, Copy)]
enum ActionKind {
    Playlist,
    Effects,
    SceneProperties,
    ResetThumbnail,
    Overview,
    Delete,
}

pub struct BackLayout {
    pub card: (f32, f32, f32, f32),
    pub masthead: (f32, f32, f32, f32),
    pub sheet: (f32, f32, f32, f32),
    pub action_deck: (f32, f32, f32, f32),
    pub content_left: f32,
    pub content_right: f32,
    pub title_left: f32,
    pub title_right: f32,
    pub action_left: f32,
    pub action_right: f32,
    pub kicker_cy: f32,
    pub fav: (f32, f32, f32),
    pub title_cy: f32,
    pub title_size: f32,
    pub facts: Vec<(f32, f32, f32, f32)>,
    pub facts_rule_y: f32,
    pub tags_label_cy: f32,
    pub tags: Vec<(f32, f32, f32, f32)>,
    pub tag_overflow: Option<(f32, f32, f32, f32, usize)>,
    pub add: (f32, f32, f32, f32),
    pub actions_label_cy: f32,
    pub playlist: (f32, f32, f32, f32),
    pub effects: Option<(f32, f32, f32, f32)>,
    pub scene_properties: Option<(f32, f32, f32, f32)>,
    pub reset_thumbnail: Option<(f32, f32, f32, f32)>,
    pub overview: Option<(f32, f32, f32, f32)>,
    pub delete: (f32, f32, f32, f32),
}

pub fn back_rise(progress: f32, slot: f32) -> (f32, f32) {
    let start = (slot * 0.095).min(0.58);
    let amount = window(progress, start, start + 0.42);
    (amount, (1.0 - amount) * 14.0)
}

fn label_width(text: &str, extra: f32) -> f32 {
    super::super::super::misc::text_width(text, 10.0, false) + extra
}

fn tag_width(text: &str) -> f32 {
    label_width(text, 34.0).clamp(58.0, 176.0)
}

fn action_width(text: &str) -> f32 {
    label_width(text, 22.0).clamp(64.0, 112.0)
}

fn add_width(open: f32, available: f32) -> f32 {
    let closed = label_width(tr("card-back-add-tag"), 28.0);
    closed + (available - closed).max(0.0) * smoothstep(open)
}

struct TagFlow {
    widths: Vec<f32>,
    rows: Vec<Vec<usize>>,
    visible_tags: usize,
    overflow: usize,
}

fn tag_flow(panel: &BackPanel, available: f32, gap: f32, max_rows: usize) -> TagFlow {
    let tag_widths: Vec<f32> = panel.tags.iter().map(|tag| tag_width(tag)).collect();
    let add = add_width(panel.add_open, available.min(250.0));
    let mut widths = tag_widths.clone();
    widths.push(add);
    let rows = wrap_indices(&widths, gap, available);
    if rows.len() <= max_rows {
        return TagFlow { widths, rows, visible_tags: panel.tags.len(), overflow: 0 };
    }

    for visible_tags in (0..panel.tags.len()).rev() {
        let overflow = panel.tags.len() - visible_tags;
        let overflow_width =
            label_width(&crate::i18n::tr_args!("card-back-more", count => overflow), 22.0)
                .clamp(74.0, 124.0);
        let mut widths = tag_widths[..visible_tags].to_vec();
        widths.extend([overflow_width, add]);
        let rows = wrap_indices(&widths, gap, available);
        if rows.len() <= max_rows {
            return TagFlow { widths, rows, visible_tags, overflow };
        }
    }

    let item_width = ((available - gap) * 0.5).max(24.0);
    let widths = vec![item_width, item_width];
    let rows = wrap_indices(&widths, gap, available);
    TagFlow { widths, rows, visible_tags: 0, overflow: panel.tags.len() }
}

fn place_tag_flow(
    flow: &TagFlow,
    left: f32,
    gap: f32,
    y: f32,
    height: f32,
    step: f32,
) -> (Vec<(f32, f32, f32, f32)>, Option<(f32, f32, f32, f32, usize)>, (f32, f32, f32, f32), f32) {
    let mut rectangles = vec![(0.0, 0.0, 0.0, 0.0); flow.widths.len()];
    let bottom = place_rows(&flow.rows, &flow.widths, &mut rectangles, left, gap, y, height, step);
    let tags = rectangles.drain(..flow.visible_tags).collect();
    let overflow = (flow.overflow > 0).then(|| {
        let rectangle = rectangles.remove(0);
        (rectangle.0, rectangle.1, rectangle.2, rectangle.3, flow.overflow)
    });
    let add = rectangles.pop().expect("add-tag row");
    (tags, overflow, add, bottom)
}

fn action_specs(panel: &BackPanel, max_width: f32) -> Vec<(ActionKind, f32)> {
    let mut specs = Vec::with_capacity(5);
    if panel.overview_available {
        specs.push((ActionKind::Overview, action_width(tr("card-back-overview"))));
    }
    if panel.static_img {
        specs.push((ActionKind::Effects, action_width(tr("card-back-effects"))));
    }
    if panel.scene_properties {
        specs.push((ActionKind::SceneProperties, action_width(tr("card-back-scene-properties"))));
    }
    if panel.reset_thumbnail {
        specs.push((ActionKind::ResetThumbnail, action_width(tr("card-back-reset-thumbnail"))));
    }
    specs.extend([
        (ActionKind::Playlist, action_width(tr("card-back-playlist"))),
        (ActionKind::Delete, action_width(tr("card-back-delete"))),
    ]);
    for (_, width) in &mut specs {
        *width = width.min(max_width);
    }
    specs
}

pub fn back_layout(panel: &BackPanel) -> BackLayout {
    if panel.embedded { embedded_layout(panel) } else { external_layout(panel) }
}

fn embedded_layout(panel: &BackPanel) -> BackLayout {
    let card_width = panel.hw * 2.0;
    let card_height = panel.hh * 2.0;
    if card_width >= 620.0 && card_height >= 320.0 {
        embedded_wide_layout(panel)
    } else {
        embedded_stacked_layout(panel)
    }
}

fn embedded_wide_layout(panel: &BackPanel) -> BackLayout {
    let card_width = panel.hw * 2.0;
    let card_height = panel.hh * 2.0;
    let card_left = panel.cx - panel.hw;
    let card_top = panel.cy - panel.hh;
    let card_bottom = card_top + card_height;
    let inset = (card_width.min(card_height) * 0.025).clamp(5.0, 12.0);
    let sheet_left = card_left + inset;
    let sheet_width = (card_width - inset * 2.0).max(80.0);
    let horizontal_pad = (sheet_width * 0.024).clamp(12.0, 22.0);
    let skew_guard = panel.skew.abs().min(sheet_width * 0.16);
    let content_left = sheet_left + horizontal_pad + skew_guard;
    let content_right = sheet_left + sheet_width - horizontal_pad - skew_guard;
    let content_width = (content_right - content_left).max(180.0);
    let column_gap = (content_width * 0.035).clamp(16.0, 28.0);
    let split = content_left + content_width * 0.45;
    let tag_left = content_left;
    let tag_width_available = (split - column_gap * 0.5 - tag_left).max(80.0);
    let action_left = split + column_gap * 0.5;
    let action_width_available = (content_right - action_left).max(80.0);
    let gap = 7.0;
    let row_height = 25.0;
    let row_step = row_height + gap;

    let tag_flow = tag_flow(panel, tag_width_available, gap, 3);
    let specs = action_specs(panel, action_width_available);
    let action_widths: Vec<f32> = specs.iter().map(|(_, width)| *width).collect();
    let action_rows = wrap_indices(&action_widths, gap, action_width_available);

    let top_band = 50.0;
    let lower_label = 15.0;
    let lower_rows = tag_flow.rows.len().max(action_rows.len()).max(1) as f32;
    let desired_sheet_height = top_band + lower_label + lower_rows * row_step + 10.0;
    let sheet_height =
        desired_sheet_height.clamp(112.0, (card_height - inset * 2.0).min(card_height * 0.36));
    let sheet_top = card_bottom - inset - sheet_height;
    let masthead_height = 32.0;
    let masthead = (sheet_left, card_top + inset, sheet_width, masthead_height);
    let kicker_cy = masthead.1 + masthead_height * 0.5;
    let fav = (content_right - 1.0, kicker_cy, 18.0);

    let title_size = (card_width * 0.026).clamp(21.0, 25.0);
    let title_cy = sheet_top + 25.0;
    let title_left = content_left;
    let title_right = split - column_gap * 0.5;
    let facts_y = sheet_top + 7.0;
    let facts_left = split + column_gap * 0.5;
    let facts_width_available = (content_right - facts_left).max(80.0);
    let fact_count = fact_count(panel);
    let facts_height = 36.0;
    let fact_gap = 8.0;
    let fact_width = ((facts_width_available - fact_gap * fact_count.saturating_sub(1) as f32)
        / fact_count as f32)
        .max(24.0);
    let facts = (0..fact_count)
        .map(|index| {
            (facts_left + index as f32 * (fact_width + fact_gap), facts_y, fact_width, facts_height)
        })
        .collect();
    let facts_rule_y = sheet_top + top_band;
    let tags_label_cy = facts_rule_y + lower_label * 0.5;
    let tags_y = facts_rule_y + lower_label;
    let (tags, tag_overflow, add, _) =
        place_tag_flow(&tag_flow, tag_left, gap, tags_y, row_height, row_step);
    let actions_label_cy = tags_label_cy;
    let actions_y = tags_y;
    let action_rectangles = action_rectangles(
        &specs,
        &action_rows,
        &action_widths,
        action_left,
        gap,
        actions_y,
        row_height,
        row_step,
    );

    let ActionRects { playlist, effects, scene_properties, reset_thumbnail, overview, delete } =
        assign_actions(specs, action_rectangles);
    BackLayout {
        card: (card_left, card_top, card_width, card_height),
        masthead,
        sheet: (sheet_left, sheet_top, sheet_width, sheet_height),
        action_deck: (sheet_left, sheet_top, sheet_width, sheet_height),
        content_left,
        content_right,
        title_left,
        title_right,
        action_left,
        action_right: content_right,
        kicker_cy,
        fav,
        title_cy,
        title_size,
        facts,
        facts_rule_y,
        tags_label_cy,
        tags,
        tag_overflow,
        add,
        actions_label_cy,
        playlist,
        effects,
        scene_properties,
        reset_thumbnail,
        overview,
        delete,
    }
}

fn embedded_stacked_layout(panel: &BackPanel) -> BackLayout {
    let card_width = panel.hw * 2.0;
    let card_height = panel.hh * 2.0;
    let card_left = panel.cx - panel.hw;
    let card_top = panel.cy - panel.hh;
    let card_bottom = card_top + card_height;
    let inset = (card_width.min(card_height) * 0.025).clamp(5.0, 12.0);
    let sheet_left = card_left + inset;
    let sheet_width = (card_width - inset * 2.0).max(80.0);
    let horizontal_pad = (sheet_width * 0.032).clamp(11.0, 28.0);
    let content_left = sheet_left + horizontal_pad;
    let content_right = sheet_left + sheet_width - horizontal_pad;
    let content_width = (content_right - content_left).max(64.0);
    let compact = card_height < 360.0;
    let gap = if compact { 5.0 } else { 7.0 };
    let row_height = if compact { 24.0 } else { 27.0 };
    let row_step = row_height + gap;

    let max_tag_rows = if compact { 2 } else { 4 };
    let tag_flow = tag_flow(panel, content_width, gap, max_tag_rows);
    let specs = action_specs(panel, content_width);
    let action_widths: Vec<f32> = specs.iter().map(|(_, width)| *width).collect();
    let action_rows = wrap_indices(&action_widths, gap, content_width);

    let top_pad = if compact { 11.0 } else { 14.0 };
    let title_height = if compact { 27.0 } else { 32.0 };
    let facts_height = if compact { 28.0 } else { 32.0 };
    let label_gap = if compact { 12.0 } else { 14.0 };
    let block_gap = if compact { 6.0 } else { 8.0 };
    let bottom_pad = if compact { 10.0 } else { 13.0 };
    let desired_sheet_height = top_pad
        + title_height
        + block_gap
        + facts_height
        + block_gap
        + label_gap
        + tag_flow.rows.len() as f32 * row_step
        + block_gap
        + label_gap
        + action_rows.len() as f32 * row_step
        + bottom_pad;
    let sheet_height = desired_sheet_height
        .clamp(card_height * 0.40, (card_height - inset * 2.0).min(card_height * 0.72));
    let sheet_top = card_bottom - inset - sheet_height;
    let masthead_height = if compact { 29.0 } else { 34.0 };
    let masthead = (sheet_left, card_top + inset, sheet_width, masthead_height);
    let kicker_cy = masthead.1 + masthead_height * 0.5;
    let fav = (content_right - 1.0, kicker_cy, if compact { 17.0 } else { 19.0 });

    let title_size = (content_width * 0.042).clamp(if compact { 20.0 } else { 23.0 }, 34.0);
    let title_cy = sheet_top + top_pad + title_height * 0.5;
    let facts_y = sheet_top + top_pad + title_height + block_gap;
    let fact_count = fact_count(panel);
    let fact_gap = if content_width < 420.0 { 5.0 } else { 12.0 };
    let fact_width = ((content_width - fact_gap * fact_count.saturating_sub(1) as f32)
        / fact_count as f32)
        .max(24.0);
    let facts = (0..fact_count)
        .map(|index| {
            (
                content_left + index as f32 * (fact_width + fact_gap),
                facts_y,
                fact_width,
                facts_height,
            )
        })
        .collect();
    let facts_rule_y = facts_y + facts_height;
    let tags_label_cy = facts_rule_y + block_gap + label_gap * 0.5;
    let tags_y = tags_label_cy + label_gap * 0.5 + 3.0;
    let (tags, tag_overflow, add, tags_bottom) =
        place_tag_flow(&tag_flow, content_left, gap, tags_y, row_height, row_step);
    let actions_label_cy = tags_bottom + block_gap + label_gap * 0.5 - gap;
    let actions_y = actions_label_cy + label_gap * 0.5 + 3.0;
    let action_rectangles = action_rectangles(
        &specs,
        &action_rows,
        &action_widths,
        content_left,
        gap,
        actions_y,
        row_height,
        row_step,
    );

    let ActionRects { playlist, effects, scene_properties, reset_thumbnail, overview, delete } =
        assign_actions(specs, action_rectangles);
    BackLayout {
        card: (card_left, card_top, card_width, card_height),
        masthead,
        sheet: (sheet_left, sheet_top, sheet_width, sheet_height),
        action_deck: (sheet_left, sheet_top, sheet_width, sheet_height),
        content_left,
        content_right,
        title_left: content_left,
        title_right: content_right,
        action_left: content_left,
        action_right: content_right,
        kicker_cy,
        fav,
        title_cy,
        title_size,
        facts,
        facts_rule_y,
        tags_label_cy,
        tags,
        tag_overflow,
        add,
        actions_label_cy,
        playlist,
        effects,
        scene_properties,
        reset_thumbnail,
        overview,
        delete,
    }
}

fn external_layout(panel: &BackPanel) -> BackLayout {
    let card_width = panel.hw * 2.0;
    let card_height = panel.hh * 2.0;
    let card_left = panel.cx - panel.hw;
    let card_top = panel.cy - panel.hh;
    let masthead_height = (card_height * 0.052).clamp(42.0, 56.0);
    let rail_width = (card_width * 0.245).clamp(292.0, 410.0).min(card_width * 0.42);
    let sheet = (card_left, card_top + masthead_height, rail_width, card_height - masthead_height);
    let pad = (rail_width * 0.065).clamp(18.0, 28.0);
    let content_left = sheet.0 + pad;
    let content_right = sheet.0 + sheet.2 - pad;
    let content_width = (content_right - content_left).max(80.0);
    let masthead = (card_left, card_top, rail_width, masthead_height);
    let kicker_cy = card_top + masthead_height * 0.5;
    let fav = (content_right - 12.0, kicker_cy, 19.0);

    let compact = card_height < 460.0;
    let title_height = if compact { 38.0 } else { 44.0 };
    let title_left = content_left;
    let title_right = content_right;
    let title_cy = sheet.1 + if compact { 18.0 } else { 22.0 } + title_height * 0.5;
    let title_size = (content_width * 0.075).clamp(if compact { 19.0 } else { 21.0 }, 28.0);

    let facts_y = title_cy + title_height * 0.5 + if compact { 4.0 } else { 8.0 };
    let facts_height = if compact { 35.0 } else { 43.0 };
    let facts: Vec<_> = if compact {
        let fact_gap = 6.0;
        let fact_width = ((content_width - fact_gap) * 0.5).max(24.0);
        (0..fact_count(panel))
            .map(|index| {
                let column = index % 2;
                let row = index / 2;
                (
                    content_left + column as f32 * (fact_width + fact_gap),
                    facts_y + row as f32 * facts_height,
                    fact_width,
                    facts_height,
                )
            })
            .collect()
    } else {
        (0..fact_count(panel))
            .map(|index| {
                (content_left, facts_y + index as f32 * facts_height, content_width, facts_height)
            })
            .collect()
    };
    let facts_rule_y = facts.last().map_or(facts_y, |fact| fact.1 + fact.3);

    let gap = if compact { 5.0 } else { 7.0 };
    let row_height = if compact { 24.0 } else { 28.0 };
    let row_step = row_height + gap;
    let action_left = content_left;
    let action_right = content_right;
    let specs = action_specs(panel, content_width);
    let action_widths: Vec<f32> = specs.iter().map(|(_, width)| *width).collect();
    let action_rows = wrap_indices(&action_widths, gap, content_width);
    let bottom_pad = if compact { 12.0 } else { 18.0 };
    let action_label_height = if compact { 27.0 } else { 31.0 };
    let deck_height = bottom_pad + action_label_height + action_rows.len().max(1) as f32 * row_step;
    let action_deck = (sheet.0, card_top + card_height - deck_height, sheet.2, deck_height);
    let actions_label_cy = action_deck.1 + action_label_height * 0.5;
    let action_rectangles = action_rectangles(
        &specs,
        &action_rows,
        &action_widths,
        action_left,
        gap,
        action_deck.1 + action_label_height,
        row_height,
        row_step,
    );

    let tags_label_cy = facts_rule_y + if compact { 15.0 } else { 19.0 };
    let tags_y = tags_label_cy + if compact { 11.0 } else { 14.0 };
    let tag_bottom = action_deck.1 - gap;
    let max_tag_rows = (((tag_bottom - tags_y - row_height).max(0.0) / row_step).floor() as usize)
        .saturating_add(1)
        .max(1);
    let tag_flow = tag_flow(panel, content_width, gap, max_tag_rows);
    let (tags, tag_overflow, add, _) =
        place_tag_flow(&tag_flow, content_left, gap, tags_y, row_height, row_step);

    let ActionRects { playlist, effects, scene_properties, reset_thumbnail, overview, delete } =
        assign_actions(specs, action_rectangles);
    BackLayout {
        card: (card_left, card_top, card_width, card_height),
        masthead,
        sheet,
        action_deck,
        content_left,
        content_right,
        title_left,
        title_right,
        action_left,
        action_right,
        kicker_cy,
        fav,
        title_cy,
        title_size,
        facts,
        facts_rule_y,
        tags_label_cy,
        tags,
        tag_overflow,
        add,
        actions_label_cy,
        playlist,
        effects,
        scene_properties,
        reset_thumbnail,
        overview,
        delete,
    }
}

#[derive(Default)]
struct ActionRects {
    playlist: (f32, f32, f32, f32),
    scene_properties: Option<(f32, f32, f32, f32)>,
    reset_thumbnail: Option<(f32, f32, f32, f32)>,
    effects: Option<(f32, f32, f32, f32)>,
    overview: Option<(f32, f32, f32, f32)>,
    delete: (f32, f32, f32, f32),
}

fn assign_actions(
    specs: Vec<(ActionKind, f32)>,
    rectangles: Vec<(f32, f32, f32, f32)>,
) -> ActionRects {
    let mut rects = ActionRects::default();
    for ((kind, _), rectangle) in specs.into_iter().zip(rectangles) {
        match kind {
            ActionKind::Playlist => rects.playlist = rectangle,
            ActionKind::Effects => rects.effects = Some(rectangle),
            ActionKind::SceneProperties => rects.scene_properties = Some(rectangle),
            ActionKind::ResetThumbnail => rects.reset_thumbnail = Some(rectangle),
            ActionKind::Overview => rects.overview = Some(rectangle),
            ActionKind::Delete => rects.delete = rectangle,
        }
    }
    rects
}

fn fact_count(panel: &BackPanel) -> usize {
    panel.fields.len().max(1)
}

fn action_rectangles(
    specs: &[(ActionKind, f32)],
    rows: &[Vec<usize>],
    widths: &[f32],
    left: f32,
    gap: f32,
    y: f32,
    height: f32,
    step: f32,
) -> Vec<(f32, f32, f32, f32)> {
    let mut rectangles = vec![(0.0, 0.0, 0.0, 0.0); specs.len()];
    place_rows(rows, widths, &mut rectangles, left, gap, y, height, step);
    rectangles
}

fn wrap_indices(widths: &[f32], gap: f32, max_width: f32) -> Vec<Vec<usize>> {
    let mut rows: Vec<Vec<usize>> = vec![Vec::new()];
    let mut row_width = 0.0;
    for (index, &width) in widths.iter().enumerate() {
        let projected =
            if rows.last().is_none_or(Vec::is_empty) { width } else { row_width + gap + width };
        if rows.last().is_some_and(|row| !row.is_empty()) && projected > max_width {
            rows.push(vec![index]);
            row_width = width;
        } else {
            rows.last_mut().expect("initial row").push(index);
            row_width = projected;
        }
    }
    rows
}

fn place_rows(
    rows: &[Vec<usize>],
    widths: &[f32],
    output: &mut [(f32, f32, f32, f32)],
    left: f32,
    gap: f32,
    y0: f32,
    height: f32,
    step: f32,
) -> f32 {
    let mut y = y0;
    for indices in rows {
        let mut x = left;
        for &index in indices {
            output[index] = (x, y, widths[index], height);
            x += widths[index] + gap;
        }
        y += step;
    }
    y
}
