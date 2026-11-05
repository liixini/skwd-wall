#[cfg(test)]
mod tests;

use std::borrow::Cow;

use iced::widget::text::LineHeight;
use iced::widget::{Space, column, container, mouse_area, scrollable, text_input};
use iced::{Alignment, Border, Color, Element, Length, Padding};

use crate::app::Message;
use crate::contracts::picker::KEY_BINDINGS;
use crate::domain::input::{
    InputAction, InputMap, MouseButton, SLOTS, Trigger, binding_label, parse_binding, slot_triggers,
};
use crate::frontend::theme::Palette;
use crate::frontend::ui::calm::{
    self, KeyLook, Piece, Popover, SELECTED_TINT, SentenceSpec, Side, Token, tint,
};
use crate::frontend::ui::{
    UI_FONT, bg_style, box_style, cached_line_width, end, folio_horizontal_rule, label,
    legible_type_scale, logical_padding, outline_scroll, row, spaced_vbar,
};
use crate::i18n::{tr, tr_args};

use super::super::keybinds::{
    GROUPS_PER_COLUMN, KeysCtx, SHEET_GROUPS, keys_find_id, links_control, matches_query,
    second_keys, second_keys_row, second_keys_token, second_summary, shared_key,
};
use super::super::places::{LIST_KEY, listed, marked};
use super::super::{ActionId, Control, Row, Section, SettingsFocus, SettingsMsg};
use super::sections::{
    BODY, CUSTOM_KEY, Item, Paint, SURFACE, TokenSide, action_button, body_inset, body_scroll_id,
    fitted_menu, row_widget_id, section_identity, section_widget_id, token_key,
};

pub(super) const SHEET_GAP: f32 = 10.0;
const COLUMNS: usize = SHEET_GROUPS.len().div_ceil(GROUPS_PER_COLUMN);
const ROW_CELLS: usize = 4;
const TOOL_ITEMS: usize = 4;
const CELL_GAPS: f32 = (ROW_CELLS - 1) as f32;
const TOOL_GAPS: f32 = (TOOL_ITEMS - 1) as f32;
pub(super) const MARK_GUTTER: f32 = 6.0;

struct SheetMetrics {
    column_gap: f32,
    tool: f32,
    tool_gap: f32,
    status_lines: f32,
    status_gap: f32,
    wrap_slack: f32,
    find: [f32; 2],
    find_pad: f32,
    find_gap: f32,
    find_mark: &'static str,
    find_glyph: f32,
    find_size: f32,
    find_alpha: f32,
    find_fill: f32,
    find_rule: f32,
    find_ring: f32,
    label_gap: f32,
    line: f32,
    line_leading: f32,
    hint_alpha: f32,
    hint_leading: f32,
    cell_gap: f32,
    heading: f32,
    heading_pad: [f32; 4],
    heading_rule: f32,
    head: f32,
    head_alpha: f32,
    head_drop: f32,
    row: f32,
    row_pad: [f32; 2],
    row_rule: f32,
    strut: f32,
    title: f32,
    title_leading: f32,
    title_pad: f32,
    title_floor: f32,
    chip: f32,
    chip_leading: f32,
    or: f32,
    or_alpha: f32,
    group_gap: f32,
    mark: f32,
    mark_inset: f32,
    tag: f32,
    tag_height: f32,
    tag_pad: f32,
    tag_rule: f32,
    tag_gap: f32,
    dim: f32,
}

const SHEET: SheetMetrics = SheetMetrics {
    column_gap: 20.0,
    tool: 34.0,
    tool_gap: 16.0,
    status_lines: 3.0,
    status_gap: 4.0,
    wrap_slack: 12.0,
    find: [210.0, 30.0],
    find_pad: 8.0,
    find_gap: 8.0,
    find_mark: "⌕",
    find_glyph: 13.0,
    find_size: 12.0,
    find_alpha: 0.56,
    find_fill: 0.58,
    find_rule: 0.28,
    find_ring: 2.0,
    label_gap: 6.0,
    line: 11.0,
    line_leading: 1.3,
    hint_alpha: 0.5,
    hint_leading: 1.4,
    cell_gap: 7.0,
    heading: 13.0,
    heading_pad: [2.0, 4.0, 5.0, 4.0],
    heading_rule: 0.58,
    head: 10.0,
    head_alpha: 0.62,
    head_drop: 1.0,
    row: 25.0,
    row_pad: [1.0, 4.0],
    row_rule: 0.16,
    strut: 1.0,
    title: 11.5,
    title_leading: 1.15,
    title_pad: 1.0,
    title_floor: 120.0,
    chip: 11.5,
    chip_leading: 1.2,
    or: 10.0,
    or_alpha: 0.42,
    group_gap: 12.0,
    mark: 2.0,
    mark_inset: 6.0,
    tag: 9.0,
    tag_height: 14.0,
    tag_pad: 5.0,
    tag_rule: 0.5,
    tag_gap: 4.0,
    dim: 0.28,
};

const CHIP_TIERS: usize = 3;
const CHORD_JOIN: char = '+';
const WORD_JOIN: char = ' ';
const CHORD_BREAK: &str = "\n";
const FIND_LABEL: &str = "settings-keybinds-find";
const KEY_HEAD: &str = "settings-keybinds-key";
const SECOND_HEAD: &str = "settings-keybinds-second-key";
const OR: &str = "settings-keybinds-or";
const ADD: &str = "settings-keybinds-add";
const UNBOUND: &str = "settings-keybind-unbound";
const LISTENING: &str = "settings-keybinds-press";
const LISTENING_HINT: &str = "settings-keybinds-press-hint";
const SECOND_TITLE: &str = "settings-keybind-capture-second-title";
const DEFAULT_BOTH: &str = "settings-keybinds-default-both";
const UNBIND: &str = "settings-keybinds-unbind";
const CANCEL: &str = "settings-keybind-capture-cancel";
const CALM_LINE: &str = "settings-keybinds-calm";
const MOVED_LINE: &str = "settings-keybinds-moved";
const SAME_LINE: &str = "settings-keybinds-same";
const CONFLICT_LINE: &str = "settings-keybinds-conflict-line";
const CONFLICT_TAG: &str = "settings-keybinds-conflict-tag";
const FIXED_LINE: &str = "settings-keybinds-fixed-line";
const SCROLL_LABEL: &str = "settings-keybinds-mouse-scroll-label";
const HOVER_LABEL: &str = "settings-keybinds-mouse-hover-label";
const FIXED_ESCAPE: &str = "settings-keybinds-fixed-escape-label";
const NAV_LABEL: &str = "settings-keybinds-second-keys-label";
const NAV_HEADING: &str = "settings-keybinds-second-keys-heading";
const NAV_CUSTOM: &str = "settings-keybinds-second-keys-custom";
const KEY_ARG: &str = "key";
const FROM_ARG: &str = "from";
const UNDO_ARG: &str = "undo";
const FIRST_ARG: &str = "first";
const SECOND_ARG: &str = "second";
const WINNER_ARG: &str = "winner";
const ESCAPE_ARG: &str = "escape";
const SCROLL_ARG: &str = "scroll";
const HOVER_ARG: &str = "hover";
const LINK_TOKEN: &str = "settings-keys-link";
const UNDO_TOKEN: &str = "settings-keys-undo";
const UNDO: &str = "settings-paths-undo-action";

struct Line {
    flat: usize,
    path: String,
    title: String,
    default_text: String,
    chips: [Chip; SLOTS],
    changed: bool,
    clash: bool,
    shade: Shade,
    side: Side,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Shade {
    Plain,
    Dim,
    Lit,
}

struct Chip {
    label: String,
    look: KeyLook,
}

struct Group {
    title: String,
    lines: Vec<Line>,
}

fn lit(paint: Paint<'_>, flat: usize) -> bool {
    paint.focus.keyboard_focus == SettingsFocus::Controls
        && paint.calm.section == 0
        && paint.focus.focused_control == flat
}

struct Clashes {
    actions: Vec<InputAction>,
    shared: Option<Trigger>,
}

impl Clashes {
    fn of(map: &InputMap) -> Self {
        let actions = map.conflicts().into_iter().flat_map(|(first, second)| [first, second]);
        Self { actions: actions.collect(), shared: shared_key(map) }
    }
}

fn line(
    (flat, row): (usize, &Row),
    map: &InputMap,
    clashes: &Clashes,
    keys: KeysCtx<'_>,
    side: Side,
) -> Option<Line> {
    let Control::KeyBinding { path, default, .. } = &row.control else {
        return None;
    };
    let action = KEY_BINDINGS.into_iter().find(|entry| entry.path == path)?.action;
    let triggers = map.triggers(action);
    let clash = clashes.actions.contains(&action);
    let defaults = parse_binding(default).unwrap_or_default();
    let labels: Vec<_> = (0..SLOTS)
        .map(|slot| binding_label(slot_triggers(triggers, slot)))
        .filter(|label| !label.is_empty())
        .collect();
    let chips = std::array::from_fn(|slot| {
        let shown = slot_triggers(triggers, slot);
        let listening = keys.capture == Some((path.as_str(), slot));
        let (caption, look) = if listening {
            (tr(LISTENING).to_string(), KeyLook::Listening)
        } else if shown.is_empty() {
            let (key, look) =
                if slot == 0 { (UNBOUND, KeyLook::Unbound) } else { (ADD, KeyLook::Add) };
            (tr(key).to_string(), look)
        } else {
            let shared = clash && clashes.shared.as_ref().is_some_and(|key| shown.contains(key));
            (binding_label(shown), if shared { KeyLook::Clash } else { KeyLook::Bound })
        };
        Chip { label: caption, look }
    });
    Some(Line {
        flat,
        path: path.clone(),
        title: row.title.clone(),
        default_text: default_text(&defaults),
        chips,
        changed: triggers != defaults.as_slice(),
        clash,
        shade: if keys.lit == Some(path.as_str()) {
            Shade::Lit
        } else if matches_query(keys.query, &row.title, &labels) {
            Shade::Plain
        } else {
            Shade::Dim
        },
        side,
    })
}

pub(crate) fn default_text(defaults: &[Trigger]) -> String {
    if defaults.is_empty() {
        return tr(UNBOUND).to_string();
    }
    let labels: Vec<_> = defaults.iter().map(Trigger::label).collect();
    let labels: Vec<_> = labels.iter().map(String::as_str).collect();
    listed(LIST_KEY, &labels)
}

fn groups(section: &Section, map: &InputMap, keys: KeysCtx<'_>) -> Vec<Group> {
    let clashes = Clashes::of(map);
    let mut groups: Vec<Group> = Vec::new();
    for (flat, row) in section.rows().enumerate() {
        if matches!(row.control, Control::Segment) {
            groups.push(Group { title: row.title.clone(), lines: Vec::new() });
            continue;
        }
        let rises = groups.len().is_multiple_of(GROUPS_PER_COLUMN);
        let side = if rises { Side::AboveEnd } else { Side::BelowEnd };
        if let Some(group) = groups.last_mut() {
            group.lines.extend(line((flat, row), map, &clashes, keys, side));
        }
    }
    groups
}

fn dimmed(palette: &Palette, dim: bool, alpha: f32) -> Palette {
    if !dim {
        return *palette;
    }
    let plane = palette.surface;
    Palette {
        primary: tint(plane, palette.primary, alpha),
        surface_text: tint(plane, palette.surface_text, alpha),
        ..*palette
    }
}

pub(super) fn sheet<'a>(
    section: &Section,
    map: &InputMap,
    keys: KeysCtx<'a>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let paint = Paint { owner: section_identity(section.id), ..paint };
    let inset = logical_padding(0.0, 0.0, 0.0, paint.size(MARK_GUTTER));
    let width = paint.body + body_inset(paint.scale) + paint.size(MARK_GUTTER);
    let gaps = paint.size(SHEET.column_gap) * (COLUMNS - 1) as f32;
    let cell = (width - gaps) / COLUMNS as f32;
    let mut grid =
        row![].spacing(paint.size(SHEET.column_gap)).align_y(Alignment::Start).width(Length::Fill);
    let all = groups(section, map, keys);
    let mut split = all.into_iter().peekable();
    while split.peek().is_some() {
        let groups: Vec<_> = split.by_ref().take(GROUPS_PER_COLUMN).collect();
        grid = grid.push(sheet_column(groups, cell, paint));
    }
    let hint = container(fixed_hint(paint)).padding(inset);
    let tool = container(tool(section, map, keys, paint)).padding(inset);
    let body = scrollable(column![grid, hint].spacing(paint.size(SHEET_GAP)))
        .id(body_scroll_id(section.id))
        .width(Length::Fill)
        .height(Length::Fill)
        .direction(spaced_vbar(paint.size(BODY.bar_gap)))
        .style(outline_scroll(paint.palette.outline));
    let room =
        paint.reading + paint.size(SURFACE.gap - SHEET_GAP) - paint.size(BODY.reading_margin);
    container(column![tool, body].spacing(paint.size(SHEET_GAP)).height(Length::Fill))
        .id(section_widget_id(section.id))
        .width(Length::Fill)
        .height(Length::Fixed(room))
        .into()
}

fn sheet_column(groups: Vec<Group>, cell: f32, paint: Paint<'_>) -> Element<'_, Message> {
    let measure = |text: &str, look| {
        calm::key_width(text, look, paint.size(SHEET.chip) * legible_type_scale(1.0))
    };
    let widest = |slot: usize, part: &dyn Fn(&Chip) -> f32| {
        groups
            .iter()
            .flat_map(|group| &group.lines)
            .map(|line| part(&line.chips[slot]))
            .fold(0.0, f32::max)
    };
    let whole = |chip: &Chip| measure(&chip.label, chip.look);
    let parts = |chip: &Chip| {
        chip_split(&chip.label, chip.look).map_or_else(
            || whole(chip),
            |(lead, rest)| measure(lead, chip.look).max(measure(rest, chip.look)),
        )
    };
    let steady =
        |chip: &Chip| if chip.look == KeyLook::Listening { parts(chip) } else { whole(chip) };
    let head = |text: &str| {
        cached_line_width(
            &text.to_uppercase(),
            paint.size(SHEET.head) * legible_type_scale(1.0),
            UI_FONT,
        )
    };
    let or = cached_line_width(tr(OR), paint.size(SHEET.or) * legible_type_scale(1.0), UI_FONT);
    let [_, across] = SHEET.row_pad.map(|px| paint.size(px));
    let frame =
        or.ceil() + paint.size(SHEET.cell_gap) * CELL_GAPS + across * 2.0 + paint.size(MARK_GUTTER);
    let room = cell - frame - paint.size(SHEET.title_floor);
    let heads = [KEY_HEAD, SECOND_HEAD];
    let word = |slot: usize| tr(heads[slot]).split_whitespace().map(&head).fold(0.0, f32::max);
    let [key, second] = fitted_chips(
        [
            [0, 1].map(|slot| {
                let lent = if slot == 1 { or.ceil() + paint.size(SHEET.cell_gap) } else { 0.0 };
                widest(slot, &steady).max(head(tr(heads[slot])) - lent).ceil()
            }),
            [0, 1].map(|slot| widest(slot, &steady).max(word(slot)).ceil()),
            [0, 1].map(|slot| widest(slot, &parts).max(word(slot)).ceil()),
        ],
        room,
    );
    let columns =
        Columns { title: (cell - frame - key - second).max(0.0), key, or: or.ceil(), second };
    let mut stack = column![].spacing(paint.size(SHEET.group_gap)).width(Length::FillPortion(1));
    for group in groups {
        stack = stack.push(group_view(group, columns, paint));
    }
    stack.into()
}

fn fitted_chips(tiers: [[f32; SLOTS]; CHIP_TIERS], room: f32) -> [f32; SLOTS] {
    let [roomy, wrapped, split] = tiers;
    [roomy, wrapped].into_iter().find(|tier| tier.iter().sum::<f32>() <= room).unwrap_or(split)
}

#[derive(Clone, Copy)]
struct Columns {
    title: f32,
    key: f32,
    or: f32,
    second: f32,
}

fn group_view(group: Group, columns: Columns, paint: Paint<'_>) -> Element<'_, Message> {
    let palette = paint.palette;
    let ground = palette.surface;
    let [top, right, bottom, left] = SHEET.heading_pad.map(|px| paint.size(px));
    let head_ink = tint(ground, palette.surface_text, SHEET.head_alpha);
    let head = |key: &'static str, width: f32| {
        container(label(tr(key).to_uppercase(), SHEET.head, paint.scale, head_ink))
            .padding(Padding { bottom: paint.size(SHEET.head_drop), ..Padding::ZERO })
            .width(Length::Fixed(width))
    };
    let mut heading = row![
        container(label(group.title, SHEET.heading, paint.scale, palette.surface_text))
            .width(Length::Fill),
        head(KEY_HEAD, columns.key),
    ]
    .spacing(paint.size(SHEET.cell_gap))
    .align_y(Alignment::End);
    let second_width = line_width(&tr(SECOND_HEAD).to_uppercase(), SHEET.head, paint);
    if second_width > columns.second {
        let borrowed = columns.or + paint.size(SHEET.cell_gap) + columns.second;
        heading = heading.push(head(SECOND_HEAD, borrowed).align_x(end()));
    } else {
        heading = heading
            .push(Space::new().width(Length::Fixed(columns.or)))
            .push(head(SECOND_HEAD, columns.second));
    }
    let rule = tint(ground, palette.outline, SHEET.heading_rule);
    let mut stack = column![row![
        Space::new().width(Length::Fixed(paint.size(MARK_GUTTER))),
        column![
            container(heading).padding(Padding { top, right, bottom, left }),
            folio_horizontal_rule(rule),
        ],
    ]];
    for line in group.lines {
        stack = stack.push(row_view(line, columns, paint));
    }
    stack.into()
}

fn row_view<'a>(line: Line, columns: Columns, paint: Paint<'a>) -> Element<'a, Message> {
    let palette = dimmed(paint.palette, line.shade == Shade::Dim, SHEET.dim);
    let ground = paint.palette.surface;
    let shaded = line.shade == Shade::Lit;
    let plane = if shaded { tint(ground, paint.palette.primary, SELECTED_TINT) } else { ground };
    let focused = lit(paint, line.flat);
    let choice = paint.focus.focused_choice.unwrap_or(0);
    let flat = line.flat;
    let [first, second] = line.chips;
    let chip = |chip: Chip, slot: usize, width: f32| {
        container(chip_cell(ChipCell {
            chip,
            slot,
            width,
            path: &line.path,
            title: &line.title,
            default_text: &line.default_text,
            changed: line.changed,
            side: line.side,
            focused: focused && choice == slot,
            palette,
            plane,
            paint,
        }))
        .width(Length::Fixed(width))
    };
    let or_ink = tint(plane, palette.surface_text, SHEET.or_alpha);
    let content = row![
        title_cell(&line.title, line.clash, columns.title, &palette, paint),
        chip(first, 0, columns.key),
        container(label(tr(OR), SHEET.or, paint.scale, or_ink)).width(Length::Fixed(columns.or)),
        chip(second, 1, columns.second),
    ]
    .spacing(paint.size(SHEET.cell_gap))
    .align_y(Alignment::Center);
    let [down, across] = SHEET.row_pad.map(|px| paint.size(px));
    let rule = tint(ground, paint.palette.outline, SHEET.row_rule);
    let mark_ink = palette.primary;
    let gutter = Length::Fixed(paint.size(MARK_GUTTER - SHEET.strut));
    let mark: Element<'a, Message> = if line.changed {
        container(
            container(Space::new())
                .width(Length::Fixed(paint.size(SHEET.mark)))
                .height(Length::Fill)
                .style(move |_| bg_style(mark_ink)),
        )
        .padding([paint.size(SHEET.mark_inset), 0.0])
        .width(gutter)
        .height(Length::Fill)
        .into()
    } else {
        Space::new().width(gutter).into()
    };
    let strut = Space::new()
        .width(Length::Fixed(paint.size(SHEET.strut)))
        .height(Length::Fixed(paint.size(SHEET.row)));
    let cells = container(
        row![strut, container(content).padding([down, across])].align_y(Alignment::Center),
    )
    .style(move |_| if shaded { bg_style(plane) } else { container::Style::default() });
    let line = row![mark, cells].height(Length::Shrink);
    let indent = Space::new().width(Length::Fixed(paint.size(MARK_GUTTER)));
    container(column![line, row![indent, folio_horizontal_rule(rule)]])
        .id(row_widget_id(paint.owner, flat))
        .into()
}

fn title_cell<'a>(
    title: &str,
    clash: bool,
    width: f32,
    palette: &Palette,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let ink = palette.surface_text;
    let text = label(title.to_string(), SHEET.title, paint.scale, ink)
        .line_height(LineHeight::Relative(SHEET.title_leading));
    let pad = Padding {
        top: paint.size(SHEET.title_pad),
        bottom: paint.size(SHEET.title_pad),
        ..Padding::ZERO
    };
    if !clash {
        return container(text).padding(pad).width(Length::Fill).into();
    }
    let size = paint.size(SHEET.title) * legible_type_scale(1.0);
    let marker = tag(palette, paint);
    let gap = paint.size(SHEET.tag_gap);
    let shown = cached_line_width(title, size, UI_FONT).ceil() + 1.0;
    let room = (width - tag_width(paint) - gap).max(0.0);
    let line = row![container(text).max_width(shown.min(room)), marker]
        .spacing(gap)
        .align_y(Alignment::Center);
    container(line).padding(pad).width(Length::Fill).into()
}

fn tag_width(paint: Paint<'_>) -> f32 {
    cached_line_width(tr(CONFLICT_TAG), paint.size(SHEET.tag) * legible_type_scale(1.0), UI_FONT)
        + paint.size(SHEET.tag_pad) * 2.0
        + 2.0
}

fn tag<'a>(palette: &Palette, paint: Paint<'a>) -> Element<'a, Message> {
    let ink = palette.primary;
    let ground = paint.palette.surface;
    container(
        label(tr(CONFLICT_TAG), SHEET.tag, paint.scale, ink)
            .line_height(LineHeight::Absolute(paint.size(SHEET.tag_height).into())),
    )
    .padding([0.0, paint.size(SHEET.tag_pad)])
    .style(move |_| {
        let mut style = box_style(Color::TRANSPARENT, tint(ground, ink, SHEET.tag_rule));
        style.background = None;
        style
    })
    .into()
}

struct ChipCell<'c, 'a> {
    chip: Chip,
    slot: usize,
    width: f32,
    path: &'c str,
    title: &'c str,
    default_text: &'c str,
    changed: bool,
    side: Side,
    focused: bool,
    palette: Palette,
    plane: Color,
    paint: Paint<'a>,
}

fn chip_cell<'a>(cell: ChipCell<'_, 'a>) -> Element<'a, Message> {
    let ChipCell {
        chip,
        slot,
        width,
        path,
        title,
        default_text,
        changed,
        side,
        focused,
        palette,
        plane,
        paint,
    } = cell;
    let key = token_key(path, slot, TokenSide::Body);
    let listening = chip.look == KeyLook::Listening;
    let open = Message::Settings(SettingsMsg::KeybindCapture(path.to_string(), slot));
    let popover = listening.then(|| Popover {
        key,
        side,
        content: capture_menu(title, slot, changed.then_some(default_text), paint),
    });
    let sentence = calm::sentence(SentenceSpec {
        pieces: chip_pieces(key, chip, width, open, paint),
        size: SHEET.chip,
        leading: SHEET.chip_leading,
        scale: paint.scale,
        palette,
        plane,
        focused: focused.then_some(key),
        ring: focused,
        popover,
        centred: false,
    });
    if !listening {
        return sentence;
    }
    let click = |button| Message::Settings(SettingsMsg::KeybindCaptureClick(button));
    mouse_area(sentence)
        .on_press(click(MouseButton::Left))
        .on_right_press(click(MouseButton::Right))
        .on_middle_press(click(MouseButton::Middle))
        .into()
}

fn chip_split(label: &str, look: KeyLook) -> Option<(&str, &str)> {
    let join = if look == KeyLook::Listening { WORD_JOIN } else { CHORD_JOIN };
    let at = label.rfind(join)?;
    let cut = at + CHORD_JOIN.len_utf8();
    (cut > CHORD_JOIN.len_utf8() && cut < label.len()).then(|| label.split_at(cut))
}

fn chip_pieces(
    key: u64,
    chip: Chip,
    width: f32,
    open: Message,
    paint: Paint<'_>,
) -> Vec<Piece<'static, Message>> {
    let Chip { label, look } = chip;
    let size = paint.size(SHEET.chip) * legible_type_scale(1.0);
    let split = chip_split(&label, look).filter(|_| calm::key_width(&label, look, size) > width);
    let Some((lead, rest)) = split else {
        return vec![Piece::token(key, label, Token::Key(look), open)];
    };
    vec![
        Piece::token(key, lead.to_string(), Token::KeyLead(look), open.clone()),
        Piece::plain(CHORD_BREAK),
        Piece::token(key, rest.to_string(), Token::Key(look), open),
    ]
}

fn capture_menu<'a>(
    title: &str,
    slot: usize,
    default_text: Option<&str>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let heading =
        if slot == 0 { title.to_string() } else { tr_args!(SECOND_TITLE, action => title) };
    let palette = paint.palette;
    let item = |caption: String, message: SettingsMsg| Item {
        caption,
        on: false,
        lit: false,
        ink: palette.surface_text,
        message: Some(Message::Settings(message)),
        ..Item::default()
    };
    let mut items = Vec::new();
    if let Some(keys) = default_text {
        items.push(item(tr_args!(DEFAULT_BOTH, keys => keys), SettingsMsg::KeybindCaptureDefault));
    }
    items.push(item(tr(UNBIND).to_string(), SettingsMsg::KeybindCaptureUnbind));
    items.push(item(tr(CANCEL).to_string(), SettingsMsg::KeybindCaptureCancel));
    fitted_menu(&[&heading, tr(LISTENING_HINT)], items, paint)
}

fn find_field<'a>(query: &'a str, focused: bool, paint: Paint<'a>) -> Element<'a, Message> {
    let palette = *paint.palette;
    let ground = palette.surface;
    let input = text_input(tr(FIND_LABEL), query)
        .id(keys_find_id())
        .on_input(|query| Message::Settings(SettingsMsg::KeysFind(query)))
        .font(UI_FONT)
        .size(paint.size(SHEET.find_size) * legible_type_scale(1.0))
        .padding(Padding::ZERO)
        .width(Length::Fill)
        .style(move |_, _| {
            let mut style =
                crate::frontend::ui::workbench_input_style(palette.surface_text, palette.primary);
            style.background = Color::TRANSPARENT.into();
            style.border = Border::default();
            style
        });
    let field = row![
        label(
            SHEET.find_mark,
            SHEET.find_glyph,
            paint.scale,
            tint(ground, palette.surface_text, SHEET.find_alpha)
        ),
        input,
    ]
    .spacing(paint.size(SHEET.find_gap))
    .align_y(Alignment::Center);
    let [wide, tall] = SHEET.find.map(|px| paint.size(px));
    let ring = paint.size(SHEET.find_ring);
    container(field)
        .padding([0.0, paint.size(SHEET.find_pad)])
        .width(Length::Fixed(wide))
        .height(Length::Fixed(tall))
        .align_y(Alignment::Center)
        .style(move |_| {
            let fill = Color { a: SHEET.find_fill, ..Color::BLACK };
            let rule = tint(ground, palette.surface_text, SHEET.find_rule);
            let mut style = box_style(fill, rule);
            if focused {
                style.border = Border { color: palette.primary, width: ring, ..style.border };
            }
            style
        })
        .into()
}

fn tool<'a>(
    section: &Section,
    map: &InputMap,
    keys: KeysCtx<'a>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let rows: Vec<&Row> = section.rows().collect();
    let finding = paint.focus.keyboard_focus == SettingsFocus::Sections;
    let find = container(find_field(keys.query, finding, paint))
        .height(Length::Fixed(paint.size(SHEET.tool)))
        .align_y(Alignment::Center);
    let (status, needed) = status_line(section, map, keys, paint);
    let mut taken = paint.size(SHEET.find[0]) + nav_width(map, paint);
    let mut reset = None;
    for (flat, row) in rows.iter().enumerate() {
        if let Control::ActionBtn { id: id @ ActionId::ResetKeybinds, label } = &row.control {
            taken += super::control::button_width(label, paint.scale);
            reset = Some(action_button(id.clone(), label.clone(), lit(paint, flat), paint));
        }
    }
    let gaps = paint.size(SHEET.tool_gap) * TOOL_GAPS;
    let room = paint.body + body_inset(paint.scale) - taken - gaps;
    let label = line_width(tr(NAV_LABEL), SHEET.chip, paint);
    let wrapped = (label / 2.0 + paint.size(SHEET.wrap_slack)).ceil();
    let spare = label - wrapped;
    let inline = needed <= room * SHEET.status_lines;
    let wraps = !inline && needed <= (room + spare) * SHEET.status_lines;
    let mut line = row![find].spacing(paint.size(SHEET.tool_gap)).align_y(Alignment::Center);
    for (flat, row) in rows.iter().enumerate() {
        if let Control::ActionChips { items } = &row.control
            && second_keys_row(&row.control)
        {
            let width = wraps.then_some(wrapped);
            line = line.push(second_keys_menu(flat, (items, width), map, paint));
        }
    }
    if inline || wraps {
        line = line.push(container(status).width(Length::Fill));
        return line.extend(reset).into();
    }
    line = line.push(Space::new().width(Length::Fill));
    column![line.extend(reset), status].spacing(paint.size(SHEET.status_gap)).into()
}

fn line_width(text: &str, size: f32, paint: Paint<'_>) -> f32 {
    cached_line_width(text, paint.size(size) * legible_type_scale(1.0), UI_FONT)
}

fn nav_width(map: &InputMap, paint: Paint<'_>) -> f32 {
    let shown = second_keys(map).map_or_else(|| tr(NAV_CUSTOM), |keys| tr(keys.label_key()));
    line_width(tr(NAV_LABEL), SHEET.chip, paint)
        + paint.size(SHEET.label_gap)
        + calm::key_width(shown, KeyLook::Bound, paint.size(SHEET.chip) * legible_type_scale(1.0))
}

fn second_keys_menu<'a>(
    flat: usize,
    (items, label_width): (&[(ActionId, String)], Option<f32>),
    map: &InputMap,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let palette = paint.palette;
    let current = second_keys(map);
    let key = second_keys_token();
    let focused = lit(paint, flat);
    let choice = paint.focus.focused_choice.filter(|_| focused);
    let mut menu = Vec::new();
    if current.is_none() {
        menu.push(Item {
            caption: tr_args!(CUSTOM_KEY, value => second_summary(map)),
            on: true,
            lit: choice.is_none(),
            ink: palette.surface_text,
            message: Some(Message::Settings(SettingsMsg::ClosePop)),
            ..Item::default()
        });
    }
    for (at, (id, caption)) in items.iter().enumerate() {
        let on = matches!(id, ActionId::SetNavKeys(keys) if Some(*keys) == current);
        menu.push(Item {
            caption: caption.clone(),
            on,
            lit: choice.map_or(on, |choice| choice == at),
            ink: palette.surface_text,
            message: Some(Message::Settings(SettingsMsg::Run(id.clone()))),
            ..Item::default()
        });
    }
    let shown = current.map_or_else(|| tr(NAV_CUSTOM), |keys| tr(keys.label_key()));
    let open = paint.calm.pop == Some(key) || choice.is_some();
    let popover = open.then(|| Popover {
        key,
        side: Side::Below,
        content: fitted_menu(&[tr(NAV_HEADING)], menu, paint),
    });
    let token = calm::sentence(SentenceSpec {
        pieces: vec![Piece::token(
            key,
            shown,
            Token::Menu,
            Message::Settings(SettingsMsg::OpenPop(key)),
        )],
        size: SHEET.chip,
        leading: SHEET.chip_leading,
        scale: paint.scale,
        palette: *palette,
        plane: palette.surface,
        focused: focused.then_some(key),
        ring: focused,
        popover,
        centred: false,
    });
    let caption = label(tr(NAV_LABEL), SHEET.chip, paint.scale, palette.surface_text)
        .width(label_width.map_or(Length::Shrink, Length::Fixed));
    row![caption, token].spacing(paint.size(SHEET.label_gap)).align_y(Alignment::Center).into()
}

fn soft(pieces: Vec<Piece<'_, Message>>) -> Vec<Piece<'_, Message>> {
    pieces
        .into_iter()
        .map(|piece| match piece {
            Piece::Plain(text) => Piece::Soft(text),
            other => other,
        })
        .collect()
}

fn line_sentence<'a>(
    pieces: Vec<Piece<'a, Message>>,
    focused: Option<u64>,
    leading: f32,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    calm::sentence(SentenceSpec {
        pieces,
        size: SHEET.line,
        leading,
        scale: paint.scale,
        palette: *paint.palette,
        plane: paint.palette.surface,
        focused,
        ring: focused.is_some(),
        popover: None,
        centred: false,
    })
}

fn link(
    key: u64,
    caption: impl Into<Cow<'static, str>>,
    id: ActionId,
) -> Vec<Piece<'static, Message>> {
    vec![Piece::token(key, caption, Token::Link, Message::Settings(SettingsMsg::Run(id)))]
}

fn status_line<'a>(
    section: &Section,
    map: &InputMap,
    keys: KeysCtx<'a>,
    paint: Paint<'a>,
) -> (Element<'a, Message>, f32) {
    let total = section.rows().count();
    let void = || (Space::new().into(), 0.0);
    if let Some((key, from)) = keys.moved {
        let mut args = marked();
        let Some(undo) = calm::sentinel(0) else {
            return void();
        };
        args.set(KEY_ARG, key.to_string());
        let titles: Vec<_> = from.iter().map(|title| tr(title)).collect();
        args.set(FROM_ARG, listed(LIST_KEY, &titles));
        args.set(UNDO_ARG, undo.to_string());
        let text = crate::i18n::format(MOVED_LINE, &args);
        let token = token_key(UNDO_TOKEN, 0, TokenSide::Body);
        let pieces = calm::split(&text, vec![link(token, tr(UNDO), ActionId::UndoKeyMove)]);
        let focused = lit(paint, total).then_some(token);
        let needed = line_width(&text, SHEET.line, paint) + line_width(tr(UNDO), SHEET.line, paint);
        return (line_sentence(soft(pieces), focused, SHEET.line_leading, paint), needed);
    }
    if let Some(key) = keys.same {
        let text = tr_args!(SAME_LINE, key => key);
        let needed = line_width(&text, SHEET.line, paint);
        let line = calm::worded(
            text,
            SHEET.line,
            paint.scale,
            paint.palette.surface_text,
            SHEET.line_leading,
        );
        return (line, needed);
    }
    let links = section.rows().enumerate().find_map(|(flat, row)| match &row.control {
        Control::ActionChips { items } if links_control(&row.control) => {
            Some((flat, items.clone()))
        }
        _ => None,
    });
    let Some(((flat, items), shared)) = links.zip(shared_key(map)) else {
        let ink = tint(paint.palette.surface, paint.palette.surface_text, SHEET.hint_alpha);
        let calm_line =
            calm::worded(tr(CALM_LINE), SHEET.line, paint.scale, ink, SHEET.hint_leading);
        return (calm_line, line_width(tr(CALM_LINE), SHEET.line, paint));
    };
    let [(first_id, first), (second_id, second)] = items.as_slice() else {
        return void();
    };
    let (Some(first_mark), Some(second_mark)) = (calm::sentinel(0), calm::sentinel(1)) else {
        return void();
    };
    let mut args = marked();
    args.set(FIRST_ARG, first_mark.to_string());
    args.set(SECOND_ARG, second_mark.to_string());
    args.set(KEY_ARG, shared.label());
    args.set(WINNER_ARG, first.clone());
    let text = crate::i18n::format(CONFLICT_LINE, &args);
    let token = |at: usize| token_key(LINK_TOKEN, at, TokenSide::Body);
    let pieces = calm::split(
        &text,
        vec![
            link(token(0), first.clone(), first_id.clone()),
            link(token(1), second.clone(), second_id.clone()),
        ],
    );
    let focused = lit(paint, flat).then(|| token(paint.focus.focused_choice.unwrap_or(0)));
    let needed = tag_width(paint)
        + paint.size(SHEET.label_gap)
        + line_width(&text, SHEET.line, paint)
        + line_width(first, SHEET.line, paint)
        + line_width(second, SHEET.line, paint);
    let view = row![
        tag(paint.palette, paint),
        container(line_sentence(soft(pieces), focused, SHEET.line_leading, paint))
            .width(Length::Fill),
    ]
    .spacing(paint.size(SHEET.label_gap))
    .align_y(Alignment::Center);
    (view.into(), needed)
}

fn fixed_hint(paint: Paint<'_>) -> Element<'_, Message> {
    let mut args = marked();
    args.set(ESCAPE_ARG, tr(FIXED_ESCAPE));
    args.set(SCROLL_ARG, tr(SCROLL_LABEL));
    args.set(HOVER_ARG, tr(HOVER_LABEL));
    let text = crate::i18n::format(FIXED_LINE, &args);
    let pieces = calm::split::<Message>(&text, Vec::new())
        .into_iter()
        .map(|piece| match piece {
            Piece::Plain(text) => Piece::Aside(text),
            other => other,
        })
        .collect();
    line_sentence(pieces, None, SHEET.hint_leading, paint)
}
