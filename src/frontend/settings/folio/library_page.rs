use iced::widget::text::LineHeight;
use iced::widget::{Space, button, column, container, scrollable, stack, text_input};
use iced::{Alignment, Color, Element, Length, Padding};

use super::sections::{
    BODY, Paint, Staggered, TokenSide, action_button, phrase, popover, reveal, row_widget_id, rule,
    section_identity, token_key, token_pieces,
};
use crate::app::Message;
use crate::contracts::settings::{FolderKind, ProfileState};
use crate::frontend::components::with_alpha;
use crate::frontend::settings::{
    ActionId, Control, FolderRow, PageSection, Row, SEGMENT_PREFIX, STATES, SectionId,
    SettingsFocus, SettingsMsg, TagTone, TreeNode, WORKSHOP_TAG, flash_key, folder_ledger,
    page_section, workbench_input_id,
};
use crate::frontend::settings::{
    FILTER_KEY, FOLDER_LIST, PageCtx, TREE_LIST, page_rows, profile_page, state_position,
};
use crate::frontend::ui::calm::{self, Piece};
use crate::frontend::ui::{
    TWISTY_CLOSED, TWISTY_OPEN, UI_FONT, label, legible_type_scale, logical_padding, row, start,
};
use crate::i18n::{tr, tr_args};

struct LineMetrics {
    gap: f32,
    note: f32,
    undo: f32,
    undo_alpha: f32,
    undo_padding: [f32; 2],
    undo_height: f32,
    list_max: f32,
    list_rows: usize,
    flash: f32,
    bottom: f32,
    tag: f32,
    tag_leading: f32,
    tag_padding: f32,
    tag_alpha: f32,
    tag_border: f32,
    tag_lit: f32,
    hint: f32,
    hint_leading: f32,
    hint_alpha: f32,
}

const LINE: LineMetrics = LineMetrics {
    gap: 8.0,
    note: 10.5,
    undo: 11.5,
    undo_alpha: 0.72,
    undo_padding: [4.0, 2.0],
    undo_height: 24.0,
    list_max: 182.0,
    list_rows: 4,
    flash: 0.24,
    bottom: 4.0,
    tag: 9.5,
    tag_leading: 15.0,
    tag_padding: 6.0,
    tag_alpha: 0.72,
    tag_border: 0.5,
    tag_lit: 0.6,
    hint: 11.0,
    hint_leading: 1.4,
    hint_alpha: 0.5,
};

struct PanelMetrics {
    list: f32,
    gap: f32,
    item: f32,
    item_gap: f32,
    item_pad: f32,
    marker: f32,
    heading_top: f32,
    heading_bottom: f32,
    command: f32,
    command_gap: [f32; 2],
    add_top: f32,
    delete_top: f32,
    head: f32,
    head_gap: f32,
    head_bottom: f32,
    name: f32,
    name_alpha: f32,
    note: f32,
    note_leading: f32,
    note_alpha: f32,
    note_padding: [f32; 2],
    note_rule: f32,
    find: [f32; 2],
    find_text: f32,
    find_glyph: f32,
    find_alpha: f32,
    find_fill: f32,
    find_edge: f32,
    find_gap: f32,
    find_pad: f32,
    find_slack: f32,
    no_match: f32,
}

const PANEL: PanelMetrics = PanelMetrics {
    list: 300.0,
    gap: 28.0,
    item: 30.0,
    item_gap: 4.0,
    item_pad: 8.0,
    marker: 2.0,
    heading_top: 12.0,
    heading_bottom: 2.0,
    command: 11.0,
    command_gap: [4.0, 10.0],
    add_top: 4.0,
    delete_top: 12.0,
    head: 30.0,
    head_gap: 10.0,
    head_bottom: 3.0,
    name: 11.0,
    name_alpha: 0.56,
    note: 11.0,
    note_leading: 1.3,
    note_alpha: 0.56,
    note_padding: [1.0, 4.0],
    note_rule: 0.34,
    find: [170.0, 28.0],
    find_text: 12.0,
    find_glyph: 13.0,
    find_alpha: 0.56,
    find_fill: 0.58,
    find_edge: 0.28,
    find_gap: 6.0,
    find_pad: 8.0,
    find_slack: 4.0,
    no_match: 8.0,
};

struct TreeMetrics {
    max: f32,
    rows: usize,
    twisty: [f32; 2],
    twisty_text: f32,
    indent: f32,
    gap: f32,
    pad: f32,
    rule: f32,
    text: f32,
    shown: f32,
    hidden: f32,
    excluded: f32,
    word: f32,
    word_gap: f32,
    word_idle: f32,
    word_line: f32,
    word_pad: f32,
    ring: f32,
    tip: f32,
    tip_pad: f32,
    tip_gap: f32,
    source: f32,
    source_alpha: f32,
    note: f32,
    note_alpha: f32,
    note_leading: f32,
    workshop_pad: f32,
    note_gap: f32,
}

const TREE: TreeMetrics = TreeMetrics {
    max: 262.0,
    rows: 10,
    twisty: [16.0, 20.0],
    twisty_text: 9.0,
    indent: 18.0,
    gap: 12.0,
    pad: 3.0,
    rule: 0.14,
    text: 12.5,
    shown: 1.0,
    hidden: 0.62,
    excluded: 0.38,
    word: 11.5,
    word_gap: 14.0,
    word_idle: 0.42,
    word_line: 2.0,
    word_pad: 1.0,
    ring: 2.0,
    tip: 11.0,
    tip_pad: 6.0,
    tip_gap: 4.0,
    source: 11.5,
    source_alpha: 0.5,
    note: 10.5,
    note_alpha: 0.5,
    note_leading: 1.35,
    workshop_pad: 4.0,
    note_gap: 1.0,
};

const SOURCE_KEY: &str = "settings-paths-removed-source";
const EXCLUDED_KEY: &str = "settings-paths-removed-excluded";
const JOINER: &str = " · ";
const HINT_KEY: &str = "settings-library-excluded-hint";
const FROM_KEY: &str = "settings-library-from";
const TERMINAL_KEY: &str = "settings-library-terminal";
const TREE_NOTE: &str = "settings-library-tree-note";
const EVERYTHING_NOTE: &str = "settings-profiles-everything-desc";
const WORKSHOP_NOTE: &str = "settings-profiles-workshop-desc";
const NAME_KEY: &str = "settings-profiles-name-label";
const ACTIVE_TAG: &str = "settings-profiles-active-tag";
const BUILTIN_TAG: &str = "settings-profiles-builtin-tag";
const NEVER_TAG: &str = "settings-profiles-never-scanned";
const MIXED_TAG: &str = "settings-profiles-mixed";
const HIDDEN_INSIDE: &str = "settings-profiles-hidden-inside";
const EXCLUDED_INSIDE: &str = "settings-library-excluded-inside";
const BOTH_INSIDE: &str = "settings-library-both-inside";
const NO_MATCH: &str = "settings-profiles-no-match";
const DELETE_HINT: &str = "settings-profiles-delete-hint";
const TIPS: [&str; 3] =
    ["settings-library-show-tip", "settings-library-hide-tip", "settings-library-exclude-tip"];
const COMMAND_TIPS: [&str; 2] = ["settings-profiles-helm-desc", "settings-profiles-flag-desc"];
const ADD_KEY: &str = "settings-paths-add-action";
const FILTER_BLANK: &str = "settings-profiles-filter-placeholder";
const FIND_GLYPH: &str = "⌕";
const LINK_MARK: u32 = 0;
const LINK_KEY: u64 = 0;
const BAR_ALPHA: f32 = 0.72;

fn list_id(list: &'static str) -> iced::widget::Id {
    iced::widget::Id::new(list)
}

pub fn reveal_line<T: Send + 'static>(section: SectionId, control: usize) -> iced::Task<T> {
    let owner = section_identity(section);
    iced::Task::batch([FOLDER_LIST, TREE_LIST].map(|list| {
        let targets = (0..=control).map(|flat| row_widget_id(owner, flat)).collect();
        reveal(targets, list_id(list))
    }))
}

struct Walk<'a> {
    rows: std::vec::IntoIter<Row>,
    at: usize,
    owner: u64,
    paint: Paint<'a>,
}

impl<'a> Walk<'a> {
    fn new(rows: Vec<Row>, at: usize, section: SectionId, paint: Paint<'a>) -> Self {
        Self { rows: rows.into_iter(), at, owner: section_identity(section), paint }
    }

    fn lit(&self, at: usize) -> bool {
        self.paint.focus.lit(SettingsFocus::Controls, at)
    }

    fn next(&mut self) -> (Row, usize, bool) {
        let row = self.rows.next().expect("page row");
        let at = self.at;
        self.at += 1;
        let lit = self.lit(at);
        (row, at, lit)
    }

    fn button(&mut self) -> Element<'a, Message> {
        let (row, _, lit) = self.next();
        let Control::ActionBtn { id, label } = row.control else {
            return Space::new().into();
        };
        action_button(id, label, lit, self.paint)
    }

    fn text(&mut self) -> (Element<'a, Message>, usize) {
        let (row, at, lit) = self.next();
        (text_token(&row, lit, self.paint), at)
    }
}

fn text_token<'a>(row: &Row, lit: bool, paint: Paint<'a>) -> Element<'a, Message> {
    let Control::TextField { key, .. } = &row.control else {
        return Space::new().into();
    };
    let side = TokenSide::Body;
    let empty = paint.values.get(key).is_none_or(String::is_empty);
    let mut pieces = token_pieces(&row.control, side, None, paint);
    if empty {
        pieces = pieces.into_iter().map(Piece::muted).collect();
    }
    let open = popover(row, side, paint.open_plane(), paint);
    phrase(pieces, lit.then(|| token_key(key, 0, side)), open, paint)
}

pub(super) fn parts<'a>(
    section: SectionId,
    ctx: PageCtx<'a>,
    base: usize,
    paint: Paint<'a>,
) -> Vec<Element<'a, Message>> {
    let walk = Walk::new(page_rows(section, &ctx), base, section, paint);
    match page_section(section) {
        Some(PageSection::Folders(kind)) => folder_parts(kind, section, &ctx, walk),
        Some(PageSection::Profiles) => vec![panel(section, &ctx, walk)],
        None => Vec::new(),
    }
}

fn hint_ink(paint: Paint<'_>, alpha: f32) -> Color {
    paint.muted(paint.open_plane(), alpha)
}

fn tag(caption: String, tone: TagTone, paint: Paint<'_>) -> Element<'_, Message> {
    let plane = paint.open_plane();
    let palette = paint.palette;
    let (ink, border) = match tone {
        TagTone::Warn => (palette.tertiary, calm::tint(plane, palette.tertiary, LINE.tag_lit)),
        TagTone::Info => (palette.primary, calm::tint(plane, palette.primary, LINE.tag_lit)),
        TagTone::Quiet => (
            calm::tint(plane, palette.surface_text, LINE.tag_alpha),
            calm::tint(plane, palette.outline, LINE.tag_border),
        ),
    };
    container(
        label(caption, LINE.tag, paint.scale, ink)
            .line_height(LineHeight::Absolute(paint.size(LINE.tag_leading).into())),
    )
    .padding([0.0, paint.size(LINE.tag_padding)])
    .style(move |_| crate::frontend::ui::box_style(Color::TRANSPARENT, border))
    .into()
}

fn linked<'a>(
    copy: &str,
    link: (ActionId, String),
    lit: bool,
    size: f32,
    ink: Color,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let (id, name) = link;
    let mark = calm::sentinel(LINK_MARK).expect("slot");
    let (before, after) = copy.split_once(mark).unwrap_or((copy, ""));
    let type_size = paint.size(size) * legible_type_scale(1.0);
    let primary = paint.palette.surface_text;
    let tone = if lit { paint.palette.primary } else { primary };
    let press = Message::Settings(SettingsMsg::Run(id));
    let pieces = vec![
        Piece::inked(before.to_string(), type_size, ink),
        Piece::token(LINK_KEY, name, calm::Token::Underlined(tone), press),
        Piece::inked(after.to_string(), type_size, ink),
    ];
    let spec = calm::prose(pieces, LINE.hint_leading);
    calm::sentence(calm::SentenceSpec { size: type_size, ..spec })
}

fn folder_line<'a>(folder: FolderRow, walk: &mut Walk<'a>) -> Element<'a, Message> {
    let paint = walk.paint;
    let (path, at) = walk.text();
    let mut copy = row![path].spacing(paint.size(LINE.gap)).align_y(Alignment::Center);
    if let Some((caption, tone)) = folder.tag {
        let tone = if tone == TagTone::Warn { tone } else { TagTone::Quiet };
        copy = copy.push(tag(caption, tone, paint));
    }
    if let Some(note) = folder.note {
        copy = copy.push(calm::worded(
            note,
            LINE.note,
            paint.scale,
            hint_ink(paint, BODY.hint_alpha),
            calm::PLAIN_LEADING,
        ));
    }
    let cells = row![container(copy.wrap()).width(Length::Fill), walk.button(), walk.button()]
        .spacing(paint.size(LINE.gap))
        .align_y(Alignment::Center);
    let flash = paint.focus.bar_reveals.get(&flash_key(&folder.key)).map_or(0.0, |tween| tween.x);
    let glow = calm::tint(paint.open_plane(), paint.palette.primary, LINE.flash * flash);
    let [vertical, horizontal] = BODY.line_padding.map(|px| paint.size(px));
    column![
        container(cells)
            .id(row_widget_id(walk.owner, at))
            .padding([vertical, horizontal])
            .width(Length::Fill)
            .style(move |_| crate::frontend::ui::bg_style(glow)),
        rule(paint, BODY.row_rule)
    ]
    .into()
}

struct Listed {
    id: &'static str,
    rows: usize,
    cap: usize,
    max: f32,
}

fn capped<'a>(
    content: Element<'a, Message>,
    spec: &Listed,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let &Listed { id, rows, cap, max } = spec;
    if rows <= cap {
        return content;
    }
    let list = scrollable(content)
        .id(list_id(id))
        .width(Length::Fill)
        .direction(crate::frontend::ui::spaced_vbar(paint.size(BODY.bar_gap)))
        .style(crate::frontend::ui::scroll_style(with_alpha(paint.palette.outline, BAR_ALPHA)));
    container(list).max_height(paint.size(max)).into()
}

fn folder_parts<'a>(
    kind: FolderKind,
    section: SectionId,
    ctx: &PageCtx<'a>,
    mut walk: Walk<'a>,
) -> Vec<Element<'a, Message>> {
    let paint = walk.paint;
    let open = paint.calm.open == Some(section);
    let mut stagger = Staggered { identity: section_identity(section), open, order: 0 };
    let ledger = folder_ledger(ctx.cfg, kind, ctx.watch);
    let count = ledger.rows.len() + usize::from(ledger.rows.iter().any(|row| row.note.is_some()));
    let mut lines = column![];
    for folder in ledger.rows {
        lines = lines.push(folder_line(folder, &mut walk));
    }
    let undone = ctx.lists.undo.filter(|(undone, _)| *undone == kind);
    let lines: Element<'a, Message> = if kind == FolderKind::Source {
        let spec = Listed {
            id: FOLDER_LIST,
            rows: count,
            cap: LINE.list_rows - usize::from(undone.is_some()),
            max: LINE.list_max - if undone.is_some() { LINE.undo_height } else { 0.0 },
        };
        capped(lines.into(), &spec, paint)
    } else {
        lines.into()
    };
    let mut group = column![stagger.wrap(lines, paint)];
    if let Some((_, path)) = undone {
        group = group.push(undo_line(kind, path, &mut walk));
    }
    let (draft, draft_at) = walk.text();
    let choose = walk.button();
    let filled = ctx.lists.drafts.get(kind.key()).is_some_and(|draft| !draft.trim().is_empty());
    let add = if filled { walk.button() } else { disabled_add(paint) };
    let cells = row![container(draft).width(Length::Fill), choose, add]
        .spacing(paint.size(LINE.gap))
        .align_y(Alignment::Center);
    let [vertical, horizontal] = BODY.line_padding.map(|px| paint.size(px));
    let line = column![
        container(cells)
            .id(row_widget_id(section_identity(section), draft_at))
            .padding([vertical, horizontal])
            .width(Length::Fill),
        rule(paint, BODY.row_rule)
    ];
    group = group.push(stagger.wrap(line.into(), paint));
    if kind == FolderKind::Source {
        let padded =
            container(group).padding(Padding { bottom: paint.size(LINE.bottom), ..Padding::ZERO });
        return vec![padded.into()];
    }
    let (link, _, lit) = walk.next();
    let Control::ActionBtn { id, label: name } = link.control else {
        return vec![group.into()];
    };
    let copy = tr_args!(HINT_KEY, link => calm::sentinel(LINK_MARK).expect("slot").to_string());
    let hint = linked(&copy, (id, name), lit, LINE.hint, hint_ink(paint, LINE.hint_alpha), paint);
    vec![group.into(), stagger.wrap(hint, paint)]
}

fn disabled_add(paint: Paint<'_>) -> Element<'_, Message> {
    let caption = tr(ADD_KEY);
    super::control::fixed_button(super::control::ButtonSpec {
        width: super::control::button_width(caption, paint.scale),
        label: caption.to_string(),
        active: false,
        destructive: false,
        message: None,
        focused: false,
        scale: paint.scale,
        palette: paint.palette,
        fade: 1.0,
        height: super::control::ButtonHeight::Compact,
        motion: paint.motion,
    })
}

fn undo_line<'a>(kind: FolderKind, path: &str, walk: &mut Walk<'a>) -> Element<'a, Message> {
    let paint = walk.paint;
    let (undo, _, lit) = walk.next();
    let Control::ActionBtn { id, label: name } = undo.control else {
        return Space::new().into();
    };
    let removed = match kind {
        FolderKind::Source => tr_args!(SOURCE_KEY, path => path),
        FolderKind::Excluded => tr_args!(EXCLUDED_KEY, path => path),
    };
    let mark = calm::sentinel(LINK_MARK).expect("slot");
    let copy = format!("{removed}{JOINER}{mark}");
    let ink = hint_ink(paint, LINE.undo_alpha);
    let [vertical, horizontal] = LINE.undo_padding.map(|px| paint.size(px));
    container(linked(&copy, (id, name), lit, LINE.undo, ink, paint))
        .padding([vertical, horizontal])
        .into()
}

fn group_label<'a>(caption: &str, paint: Paint<'a>) -> Element<'a, Message> {
    container(label(
        caption.to_uppercase(),
        BODY.heading,
        paint.scale,
        hint_ink(paint, BODY.heading_alpha),
    ))
    .padding(Padding {
        top: paint.size(PANEL.heading_top),
        bottom: paint.size(PANEL.heading_bottom),
        ..Padding::ZERO
    })
    .into()
}

fn profile_item<'a>(
    entry: &crate::frontend::settings::ProfileEntry,
    selected: bool,
    walk: &mut Walk<'a>,
) -> Element<'a, Message> {
    let paint = walk.paint;
    let (row, _, lit) = walk.next();
    let palette = *paint.palette;
    let plane = paint.open_plane();
    let marker = if selected { palette.primary } else { Color::TRANSPARENT };
    let name = if entry.name.is_empty() { entry.id.clone() } else { entry.name.clone() };
    let mut line = row![
        Space::new().width(Length::Fixed(paint.size(PANEL.marker))),
        container(label(name, BODY.value, paint.scale, palette.surface_text)).width(Length::Fill),
    ]
    .spacing(paint.size(LINE.gap))
    .align_y(Alignment::Center);
    if entry.builtin() {
        line = line.push(tag(tr(BUILTIN_TAG).to_string(), TagTone::Quiet, paint));
    }
    if entry.active {
        line = line.push(tag(tr(ACTIVE_TAG).to_string(), TagTone::Info, paint));
    }
    let Control::ActionBtn { id, .. } = row.control else {
        return Space::new().into();
    };
    let line = line.push(Space::new().height(Length::Fixed(paint.size(PANEL.item))));
    let bar = container(Space::new())
        .width(Length::Fixed(paint.size(PANEL.marker)))
        .height(Length::Fill)
        .style(move |_| crate::frontend::ui::bg_style(marker));
    let bar = container(bar).width(Length::Fill).height(Length::Fill).align_x(start());
    button(stack![line, bar])
        .width(Length::Fill)
        .padding(logical_padding(0.0, paint.size(PANEL.item_pad), 0.0, 0.0))
        .on_press(Message::Settings(SettingsMsg::Run(id)))
        .style(move |_, status| calm::style::item(plane, &palette, selected || lit, status))
        .into()
}

fn tipped<'a>(content: Element<'a, Message>, tip: &str, paint: Paint<'a>) -> Element<'a, Message> {
    let palette = *paint.palette;
    let ground = palette.surface;
    let bubble = container(label(tip.to_string(), TREE.tip, paint.scale, palette.surface_text))
        .padding(paint.size(TREE.tip_pad))
        .style(move |_| calm::style::popover(ground, &palette));
    iced::widget::tooltip(content, bubble, iced::widget::tooltip::Position::Top)
        .gap(paint.size(TREE.tip_gap))
        .into()
}

fn command_row<'a>(tip: &str, walk: &mut Walk<'a>) -> Element<'a, Message> {
    let paint = walk.paint;
    let (row, _, lit) = walk.next();
    let Control::ActionBtn { id, label: copy } = row.control else {
        return Space::new().into();
    };
    let [down, across] = PANEL.command_gap.map(|px| paint.size(px));
    let line = row![
        label(row.title, PANEL.command, paint.scale, paint.palette.surface_text),
        action_button(id, copy, lit, paint)
    ]
    .spacing(across)
    .align_y(Alignment::Center)
    .wrap()
    .vertical_spacing(down);
    tipped(line.into(), tip, paint)
}

fn profile_list<'a>(
    page: &crate::frontend::settings::ProfilesPage,
    walk: &mut Walk<'a>,
) -> Element<'a, Message> {
    let paint = walk.paint;
    let mut list = column![].spacing(paint.size(PANEL.item_gap)).width(Length::Fill);
    for (at, entry) in page.profiles.iter().enumerate() {
        list = list.push(profile_item(entry, at == page.selected, walk));
    }
    list = list.push(
        container(walk.button())
            .padding(Padding { top: paint.size(PANEL.add_top), ..Padding::ZERO }),
    );
    if page.current().builtin() {
        return list.into();
    }
    list = list.push(group_label(tr(TERMINAL_KEY), paint));
    for tip in COMMAND_TIPS {
        list = list.push(command_row(tr(tip), walk));
    }
    let (delete, _, lit) = walk.next();
    if let Control::ActionBtn { id, label: caption } = delete.control {
        let armed = paint.focus.armed == Some(&id);
        list = list.push(
            container(action_button(id, caption, lit, paint))
                .padding(Padding { top: paint.size(PANEL.delete_top), ..Padding::ZERO }),
        );
        if armed {
            list = list.push(calm::worded(
                tr(DELETE_HINT),
                LINE.hint,
                paint.scale,
                hint_ink(paint, LINE.hint_alpha),
                calm::PLAIN_LEADING,
            ));
        }
    }
    list.into()
}

fn find_field<'a>(ctx: &PageCtx<'a>, lit: bool, paint: Paint<'a>) -> Element<'a, Message> {
    let palette = paint.ink();
    let field = text_input(tr(FILTER_BLANK), ctx.profiles.filter)
        .id(workbench_input_id(FILTER_KEY))
        .on_input(|raw| Message::Settings(SettingsMsg::ProfileFilter(raw)))
        .on_submit(Message::Settings(SettingsMsg::Commit))
        .font(UI_FONT)
        .size(paint.size(PANEL.find_text) * legible_type_scale(1.0))
        .padding(Padding::ZERO)
        .width(Length::Fill)
        .style(move |_, _| {
            let mut style =
                crate::frontend::ui::workbench_input_style(palette.surface_text, palette.primary);
            style.background = Color::TRANSPARENT.into();
            style.border = iced::Border::default();
            style
        });
    let border = if lit {
        palette.primary
    } else {
        calm::tint(paint.open_plane(), palette.surface_text, PANEL.find_edge)
    };
    let fill = Color { a: PANEL.find_fill, ..Color::BLACK };
    let [width, height] = PANEL.find;
    let wanted = crate::frontend::ui::cached_line_width(
        tr(FILTER_BLANK),
        paint.size(PANEL.find_text) * legible_type_scale(1.0),
        UI_FONT,
    ) + paint
        .size(PANEL.find_glyph + PANEL.find_gap + PANEL.find_pad * 2.0 + PANEL.find_slack);
    container(
        row![
            label(FIND_GLYPH, PANEL.find_glyph, paint.scale, hint_ink(paint, PANEL.find_alpha)),
            field
        ]
        .spacing(paint.size(PANEL.find_gap))
        .align_y(Alignment::Center),
    )
    .width(Length::Fixed(paint.size(width).max(wanted)))
    .center_y(Length::Fixed(paint.size(height)))
    .padding([0.0, paint.size(PANEL.find_pad)])
    .style(move |_| crate::frontend::ui::box_style(fill, border))
    .into()
}

fn tree_head<'a>(
    page: &crate::frontend::settings::ProfilesPage,
    ctx: &PageCtx<'a>,
    walk: &mut Walk<'a>,
) -> Element<'a, Message> {
    let paint = walk.paint;
    let entry = page.current();
    let gap = paint.size(PANEL.head_gap);
    let mut line = row![].spacing(gap).align_y(Alignment::Center);
    if entry.builtin() {
        line = line.push(label(
            entry.name.clone(),
            BODY.value,
            paint.scale,
            paint.palette.surface_text,
        ));
    } else {
        line = line.push(label(
            tr(NAME_KEY),
            PANEL.name,
            paint.scale,
            hint_ink(paint, PANEL.name_alpha),
        ));
        line = line.push(walk.text().0);
    }
    line = if entry.active {
        line.push(tag(tr(ACTIVE_TAG).to_string(), TagTone::Info, paint))
    } else {
        line.push(walk.button())
    };
    let (_, _, lit) = walk.next();
    let line = line.wrap().vertical_spacing(gap);
    let head = row![
        container(line).width(Length::Fill),
        Space::new().height(Length::Fixed(paint.size(PANEL.head))),
        find_field(ctx, lit, paint)
    ]
    .spacing(gap)
    .align_y(Alignment::Center);
    container(head)
        .padding(Padding { bottom: paint.size(PANEL.head_bottom), ..Padding::ZERO })
        .width(Length::Fill)
        .into()
}

fn inside(node: &TreeNode) -> Option<String> {
    if !node.collapsed {
        return None;
    }
    let (hidden, excluded) = (node.hidden_inside, node.excluded_inside);
    match (hidden, excluded) {
        (0, 0) => None,
        (_, 0) => Some(tr_args!(HIDDEN_INSIDE, count => hidden)),
        (0, _) => Some(tr_args!(EXCLUDED_INSIDE, count => excluded)),
        _ => Some(tr_args!(BOTH_INSIDE, hidden => hidden, excluded => excluded)),
    }
}

fn twisty<'a>(node: &TreeNode, walk: &mut Walk<'a>) -> Element<'a, Message> {
    let paint = walk.paint;
    let [width, height] = TREE.twisty.map(|px| paint.size(px));
    if !node.children {
        return Space::new().width(Length::Fixed(width)).height(Length::Fixed(height)).into();
    }
    let (row, _, lit) = walk.next();
    let Control::ActionBtn { id, .. } = row.control else {
        return Space::new().into();
    };
    let glyph = if node.collapsed { TWISTY_CLOSED } else { TWISTY_OPEN };
    let primary = paint.palette.primary;
    button(container(label(glyph, TREE.twisty_text, paint.scale, primary)).center(Length::Fill))
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .padding(0)
        .on_press(Message::Settings(SettingsMsg::Run(id)))
        .style(move |_, _| calm::style::bare(primary, primary, lit.then_some(TREE.ring)))
        .into()
}

fn word<'a>(
    caption: String,
    tip: &str,
    message: Option<Message>,
    on: f32,
    lit: bool,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let plane = paint.open_plane();
    let palette = *paint.palette;
    let idle = calm::tint(plane, palette.surface_text, TREE.word_idle);
    let ink = calm::tint(idle, palette.primary, on);
    let hover = palette.surface_text;
    let line = calm::tint(plane, palette.primary, on);
    let underline = container(Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(paint.size(TREE.word_line)))
        .style(move |_| crate::frontend::ui::bg_style(line));
    let typed = paint.size(TREE.word) * legible_type_scale(1.0);
    let hot = message.is_some();
    let word = button(
        column![iced::widget::text(caption).font(UI_FONT).size(typed), underline]
            .spacing(paint.size(TREE.word_pad))
            .width(Length::Shrink),
    )
    .padding(0)
    .on_press_maybe(message)
    .style(move |_, status| {
        let over = hot && matches!(status, button::Status::Hovered | button::Status::Pressed);
        let shown = if over { hover } else { ink };
        calm::style::bare(shown, palette.primary, lit.then_some(TREE.ring))
    });
    tipped(word.into(), tip, paint)
}

fn words<'a>(node: &TreeNode, state: ProfileState, walk: &mut Walk<'a>) -> Element<'a, Message> {
    let paint = walk.paint;
    let (row, _, lit) = walk.next();
    let key = format!("{SEGMENT_PREFIX}{}", node.id);
    let settled = state_position(state);
    let position = paint.focus.bar_reveals.get(&key).map_or(settled, |tween| tween.x);
    let choice = lit.then(|| paint.focus.focused_choice.unwrap_or(settled as usize));
    let mut line = row![].spacing(paint.size(TREE.word_gap));
    for (at, (candidate, name)) in STATES.iter().enumerate() {
        let on = (1.0 - (position - at as f32).abs()).clamp(0.0, 1.0);
        let message = (*candidate != state).then(|| {
            let path = row.control.path().unwrap_or_default().to_string();
            Message::Settings(SettingsMsg::Pick(path, candidate.key().to_string()))
        });
        let caption = tr(name).to_string();
        line = line.push(word(caption, tr(TIPS[at]), message, on, choice == Some(at), paint));
    }
    line.into()
}

fn name_ink(node: &TreeNode, paint: Paint<'_>) -> Color {
    let alpha = match node.state {
        None | Some(ProfileState::Exclude) => TREE.excluded,
        Some(ProfileState::Hide) => TREE.hidden,
        Some(ProfileState::Show) => TREE.shown,
    };
    calm::tint(paint.open_plane(), paint.palette.surface_text, alpha)
}

fn tree_row<'a>(node: &TreeNode, locked: bool, walk: &mut Walk<'a>) -> Element<'a, Message> {
    let paint = walk.paint;
    let workshop = node.id.ends_with(WORKSHOP_TAG) && node.depth == 0;
    let at = walk.at;
    let fold = twisty(node, walk);
    let name_label = label(node.label.clone(), TREE.text, paint.scale, name_ink(node, paint));
    let mut name = column![name_label].spacing(paint.size(TREE.note_gap));
    if workshop {
        name = name.push(calm::worded(
            tr(WORKSHOP_NOTE),
            TREE.note,
            paint.scale,
            hint_ink(paint, TREE.note_alpha),
            TREE.note_leading,
        ));
    }
    let mut cells = row![
        fold,
        container(name).width(Length::Fill).padding(logical_padding(
            0.0,
            0.0,
            0.0,
            paint.size(TREE.indent) * f32::from(node.depth)
        ))
    ]
    .spacing(paint.size(TREE.gap))
    .align_y(Alignment::Center);
    let mut tags = row![].spacing(paint.size(BODY.button_gap));
    if node.state.is_none() {
        tags = tags.push(tag(tr(NEVER_TAG).to_string(), TagTone::Quiet, paint));
    } else if let Some(caption) = inside(node) {
        tags = tags.push(tag(caption, TagTone::Quiet, paint));
    } else if node.mixed && !locked {
        tags = tags.push(tag(tr(MIXED_TAG).to_string(), TagTone::Quiet, paint));
    }
    cells = cells.push(tags);
    match node.state {
        None => {
            let (link, _, lit) = walk.next();
            if let Control::ActionBtn { id, label: name } = link.control {
                let copy = tr_args!(FROM_KEY, link => calm::sentinel(LINK_MARK).expect("slot").to_string());
                cells = cells.push(linked(
                    &copy,
                    (id, name),
                    lit,
                    TREE.source,
                    hint_ink(paint, TREE.source_alpha),
                    paint,
                ));
            }
        }
        Some(state) if !node.path.is_empty() => cells = cells.push(words(node, state, walk)),
        Some(_) => {}
    }
    let pad = if workshop { TREE.workshop_pad } else { TREE.pad };
    container(cells)
        .id(row_widget_id(walk.owner, at))
        .width(Length::Fill)
        .padding([paint.size(pad), 0.0])
        .into()
}

fn tree<'a>(
    page: &crate::frontend::settings::ProfilesPage,
    walk: &mut Walk<'a>,
) -> Element<'a, Message> {
    let paint = walk.paint;
    let entry = page.current();
    let mut list = column![];
    for (at, node) in page.tree.iter().enumerate() {
        if at > 0 {
            list = list.push(rule(paint, TREE.rule));
        }
        list = list.push(tree_row(node, entry.builtin(), walk));
    }
    let workshop = usize::from(page.tree.iter().any(|node| node.id.ends_with(WORKSHOP_TAG)));
    let spec =
        Listed { id: TREE_LIST, rows: page.tree.len() + workshop, cap: TREE.rows, max: TREE.max };
    capped(list.width(Length::Fill).into(), &spec, paint)
}

fn panel<'a>(section: SectionId, ctx: &PageCtx<'a>, mut walk: Walk<'a>) -> Element<'a, Message> {
    let paint = walk.paint;
    let page = profile_page(ctx);
    let open = paint.calm.open == Some(section);
    let mut stagger = Staggered { identity: section_identity(section), open, order: 0 };
    let left = stagger.wrap(profile_list(&page, &mut walk), paint);
    let head = tree_head(&page, ctx, &mut walk);
    let note = if page.current().builtin() { EVERYTHING_NOTE } else { TREE_NOTE };
    let [top, bottom] = PANEL.note_padding.map(|px| paint.size(px));
    let note = column![
        container(calm::worded(
            tr(note),
            PANEL.note,
            paint.scale,
            hint_ink(paint, PANEL.note_alpha),
            PANEL.note_leading,
        ))
        .padding(Padding { top, bottom, ..Padding::ZERO }),
        rule(paint, PANEL.note_rule)
    ];
    let mut right = column![head, note, tree(&page, &mut walk)].width(Length::Fill);
    if page.no_match {
        right = right.push(
            container(label(
                tr(NO_MATCH),
                LINE.hint,
                paint.scale,
                hint_ink(paint, LINE.hint_alpha),
            ))
            .padding([paint.size(PANEL.no_match), 0.0]),
        );
    }
    let right = stagger.wrap(right.into(), paint);
    row![container(left).width(Length::Fixed(paint.size(PANEL.list))), right]
        .spacing(paint.size(PANEL.gap))
        .align_y(Alignment::Start)
        .into()
}

const LEAD_STEPS: usize = 2;

pub(super) fn flush(section: SectionId) -> bool {
    page_section(section) == Some(PageSection::Profiles)
}

pub(super) fn first_stagger(section: SectionId) -> usize {
    if page_section(section).is_some() { LEAD_STEPS } else { 0 }
}
