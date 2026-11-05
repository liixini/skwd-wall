#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use iced::advanced::widget::{Operation, operation};
use iced::widget::text::LineHeight;
use iced::widget::{Space, button, column, container, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Length, Padding, Rectangle, Vector};

use crate::app::Message;
use crate::contracts::daemon::{APP_THEME_BUSY, APP_THEME_INTERRUPTED};
use crate::contracts::settings::keys;
use crate::domain::library::filter::resolution_parts;
use crate::frontend::animation::{CALM, CSS_EASE, MotionProfile};
use crate::frontend::theme::Palette;
use crate::frontend::ui::calm::{
    self, OptionsSpec, Piece, PlaneSpec, Popover, ROW_TINT, RevealSpec, SAY_LEADING, SAY_SIZE,
    Scrub, SentenceSpec, Side, StaggerSpec, ToggleSpec, Token,
};
use crate::frontend::ui::{
    UI_FONT, bg_style, folio_horizontal_rule, label, legible_type_scale, logical_padding, row,
    start,
};
use crate::i18n::tr;

use super::super::background::BackgroundControl;
use super::super::bar::BAND_TO;
use super::super::places::{self, LIST_KEY};
use super::super::{
    ActionId, Cell, Control, DIMENSIONS, Form, Group, GroupKind, Row, Section, SectionId,
    SettingsFocus, SettingsMsg, Slot, Trim, workbench_input_id,
};
use super::control::{ButtonHeight, ButtonSpec};
use super::folio::{FocusCtx, field};

pub struct SurfaceMetrics {
    pub padding: [f32; 4],
    pub gap: f32,
    pub head_gap: f32,
    pub heading: f32,
    pub note: f32,
    pub note_alpha: f32,
    pub note_drop: f32,
    pub head_spacing: f32,
    pub head_rule: f32,
    pub close: f32,
    pub secs_rule: f32,
    pub name: f32,
    pub name_inset: f32,
    pub columns: f32,
    pub pad: f32,
    pub busy_pad: f32,
    pub pad_end: f32,
    pub title: f32,
    pub desc: f32,
    pub desc_leading: f32,
    pub desc_alpha: f32,
    pub hint: f32,
    pub hint_alpha: f32,
    pub hint_leading: f32,
    pub name_gap: f32,
    pub say_gap: f32,
    pub foot: f32,
}

pub(super) const SURFACE: SurfaceMetrics = SurfaceMetrics {
    padding: [34.0, 38.0, 0.0, 38.0],
    gap: 18.0,
    head_gap: 12.0,
    heading: 46.0,
    note: 11.0,
    note_alpha: 0.56,
    note_drop: 5.0,
    head_spacing: 20.0,
    head_rule: 0.58,
    close: 30.0,
    secs_rule: 0.34,
    name: 240.0,
    name_inset: 12.0,
    columns: 20.0,
    pad: 13.0,
    busy_pad: 8.0,
    pad_end: 4.0,
    title: 15.0,
    desc: 11.0,
    desc_leading: 1.35,
    desc_alpha: 0.56,
    hint: 11.0,
    hint_alpha: 0.5,
    hint_leading: 1.4,
    name_gap: 3.0,
    say_gap: 14.0,
    foot: 15.0,
};

pub(super) struct BodyMetrics {
    padding: [f32; 4],
    gap: f32,
    column_gap: f32,
    divider_alpha: f32,
    pub heading: f32,
    pub heading_alpha: f32,
    heading_padding: [f32; 4],
    row_padding: [f32; 2],
    pub row_rule: f32,
    row_gap: f32,
    label_share: f32,
    label_limit: f32,
    pub value: f32,
    hint: f32,
    pub hint_alpha: f32,
    hint_gap: &'static str,
    dim_alpha: f32,
    line_gap: f32,
    pub line_padding: [f32; 2],
    line_hint: f32,
    field_padding: [f32; 2],
    line_height: f32,
    lines_visible: usize,
    ring: f32,
    pub note_leading: f32,
    columns: usize,
    divider: f32,
    pair: &'static str,
    beside: &'static str,
    pub button_gap: f32,
    button_max: f32,
    text_max: f32,
    token_max: f32,
    process_max: f32,
    stage: f32,
    swatch: f32,
    swatch_gap: f32,
    pub wrap_slack: f32,
    pub reading_margin: f32,
    room_floor: f32,
    pub bar_gap: f32,
    wide_inset: f32,
    corner_side: f32,
    corner_box: [f32; 2],
    corner_max: f32,
    corner_gap: f32,
    nudge: f32,
    nudge_inset: f32,
    foot_gap: f32,
    pub snug_trim: f32,
    lean_trim: f32,
}

pub(super) const BODY: BodyMetrics = BodyMetrics {
    padding: [2.0, 30.0, 18.0, 260.0],
    gap: 10.0,
    column_gap: 17.0,
    divider_alpha: 0.34,
    heading: 10.0,
    heading_alpha: 0.5,
    heading_padding: [2.0, 2.0, 4.0, 2.0],
    row_padding: [9.0, 2.0],
    row_rule: 0.2,
    row_gap: 14.0,
    label_share: 0.3,
    label_limit: 0.5,
    value: 12.5,
    hint: 10.5,
    hint_alpha: 0.5,
    hint_gap: "\u{2002}",
    dim_alpha: 0.42,
    line_gap: 10.0,
    line_padding: [6.0, 2.0],
    line_hint: 11.0,
    field_padding: [5.0, 8.0],
    line_height: 43.0,
    lines_visible: 3,
    ring: 2.0,
    note_leading: 1.17,
    columns: 2,
    divider: 1.0,
    pair: "\u{2002}×\u{2002}",
    beside: "\u{2002}·\u{2002}",
    button_gap: 6.0,
    button_max: 170.0,
    text_max: 190.0,
    token_max: 235.0,
    process_max: 110.0,
    stage: 216.0,
    swatch: 10.0,
    swatch_gap: 5.0,
    wrap_slack: 1.15,
    reading_margin: 18.0,
    room_floor: 96.0,
    bar_gap: 4.0,
    wide_inset: 20.0,
    corner_side: 64.0,
    corner_box: [120.0, 64.0],
    corner_max: 30.0,
    corner_gap: 6.0,
    nudge: 28.0,
    nudge_inset: 16.0,
    foot_gap: 10.0,
    snug_trim: 8.0,
    lean_trim: 1.0,
};

struct MenuMetrics {
    heading: f32,
    heading_alpha: f32,
    heading_padding: [f32; 4],
    padding: f32,
    width: f32,
    item: [f32; 2],
    gap: f32,
    glyph: f32,
    size: f32,
    disabled_alpha: f32,
    editor: f32,
    editor_min: f32,
    unit_gap: f32,
    hint_width: f32,
    wide: f32,
    rows: usize,
    columns: usize,
    panel_gap: f32,
    panel_padding: f32,
    panel_scale: f32,
    slack: f32,
    striped: f32,
    striped_wide: f32,
    strip_colours: usize,
    strip: [f32; 2],
}

const MENU: MenuMetrics = MenuMetrics {
    heading: 10.0,
    heading_alpha: 0.5,
    heading_padding: [6.0, 10.0, 2.0, 10.0],
    padding: 4.0,
    width: 180.0,
    item: [8.0, 10.0],
    gap: 9.0,
    glyph: 8.0,
    size: 12.0,
    disabled_alpha: 0.4,
    editor: 12.5,
    editor_min: 36.0,
    unit_gap: 4.0,
    hint_width: 320.0,
    wide: 300.0,
    rows: 15,
    columns: 2,
    panel_gap: 8.0,
    panel_padding: 10.0,
    panel_scale: 0.94,
    slack: 8.0,
    striped: 230.0,
    striped_wide: 420.0,
    strip_colours: 4,
    strip: [40.0, 8.0],
};

struct PaletteStrip {
    size: [f32; 2],
    drop: f32,
}

const PALETTE_STRIP: PaletteStrip = PaletteStrip { size: [220.0, 12.0], drop: 25.0 };
const APP_PANEL_WIDTH: f32 = 380.0;
const EVERY_SLOT: usize = 1;
const MORE_MENU: &str = "settings-menu-more";
const DETAILS_KEY: &str = "settings-app-themes-details";
const APP_TOKEN: &str = "app-theme:";
const APP_UNDO: &str = "settings-app-themes-undo";

struct SizeScrub {
    min: f64,
    max: [f64; 2],
    step: f64,
}

const SIZE_SCRUB: SizeScrub = SizeScrub { min: 1.0, max: [15360.0, 8640.0], step: 10.0 };

const DESC_FADE: std::time::Duration = std::time::Duration::from_millis(200);
const SELECTED_MARK: &str = "◆";
pub(super) const CUSTOM_KEY: &str = "settings-menu-custom";
const CHECKS_KEY: &str = "settings-checks-count";
const CHOOSE_KEY: &str = "settings-menu-choose";
const MORE_KEY: &str = "settings-wallpapers-display-more";
const LIVE_TOKEN: &str = "display-live:";
const NO_MAXIMUM: &str = "settings-bar-no-limit";
const LIMITS: [&str; 2] =
    ["settings-filter-resolution-clear-limit", "settings-filter-resolution-set-limit"];
const OPEN_MARK: &str = "◇";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenSide {
    Sentence,
    Body,
}

pub fn token_key(identity: &str, slot: usize, side: TokenSide) -> u64 {
    let mut hasher = DefaultHasher::new();
    identity.hash(&mut hasher);
    slot.hash(&mut hasher);
    side.hash(&mut hasher);
    hasher.finish()
}

pub(super) fn token_key_for(identity: &str) -> u64 {
    token_key(identity, 0, TokenSide::Sentence)
}

pub fn live_key(output: &str) -> u64 {
    token_key(&format!("{LIVE_TOKEN}{output}"), 0, TokenSide::Body)
}

pub fn token_side(section: &Section, control: usize) -> TokenSide {
    if control < section.header_rows() { TokenSide::Sentence } else { TokenSide::Body }
}

const SECTION_ID_PREFIX: &str = "settings-section:";
const BODY_ID_PREFIX: &str = "settings-body:";
const ROW_ID_PREFIX: &str = "settings-row:";
const REVEAL_MARGIN: f32 = 48.0;
const ECHOED: usize = usize::MAX;

pub fn section_widget_id(id: SectionId) -> iced::widget::Id {
    format!("{SECTION_ID_PREFIX}{}:{}:{}", id.category, id.key, id.nth).into()
}

pub(super) fn body_scroll_id(id: SectionId) -> iced::widget::Id {
    format!("{BODY_ID_PREFIX}{}:{}:{}", id.category, id.key, id.nth).into()
}

const LABEL_ID_PREFIX: &str = "row-label:";

pub(super) fn label_widget_id(owner: u64, flat: usize) -> iced::widget::Id {
    format!("{LABEL_ID_PREFIX}{owner}:{flat}").into()
}

pub(super) fn row_widget_id(owner: u64, flat: usize) -> iced::widget::Id {
    format!("{ROW_ID_PREFIX}{owner}:{flat}").into()
}

fn reveal_offset(top: f32, content: f32, scrolled: f32, view: f32) -> Option<f32> {
    let offset = top - content;
    let shown = offset >= scrolled && offset + REVEAL_MARGIN <= scrolled + view;
    (!shown).then_some(offset)
}

struct Reveal {
    targets: Vec<iced::widget::Id>,
    scroller: iced::widget::Id,
    top: Option<(usize, f32)>,
    view: Option<(Rectangle, Rectangle, f32)>,
}

impl Operation for Reveal {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn container(&mut self, id: Option<&iced::widget::Id>, bounds: Rectangle) {
        let Some(at) = id.and_then(|id| self.targets.iter().position(|target| target == id)) else {
            return;
        };
        if self.top.is_none_or(|(best, _)| at >= best) {
            self.top = Some((at, bounds.y));
        }
    }

    fn scrollable(
        &mut self,
        id: Option<&iced::widget::Id>,
        bounds: Rectangle,
        content: Rectangle,
        translation: Vector,
        _state: &mut dyn operation::Scrollable,
    ) {
        if id == Some(&self.scroller) {
            self.view = Some((bounds, content, translation.y));
        }
    }

    fn finish(&self) -> operation::Outcome<()> {
        let (Some((_, top)), Some((bounds, content, scrolled))) = (self.top, self.view) else {
            return operation::Outcome::None;
        };
        let Some(offset) = reveal_offset(top, content.y, scrolled, bounds.height) else {
            return operation::Outcome::None;
        };
        let to = operation::scrollable::AbsoluteOffset { x: None, y: Some(offset) };
        operation::Outcome::Chain(Box::new(operation::scrollable::scroll_to(
            self.scroller.clone(),
            to,
        )))
    }
}

pub(super) fn reveal<T>(
    targets: Vec<iced::widget::Id>,
    scroller: iced::widget::Id,
) -> iced::Task<T> {
    iced_runtime::task::effect(iced_runtime::Action::widget(Reveal {
        targets,
        scroller,
        top: None,
        view: None,
    }))
}

pub fn reveal_section<T>(id: SectionId) -> iced::Task<T> {
    reveal(vec![section_widget_id(id)], super::super::settings_reading_id())
}

pub fn reveal_row<T>(id: SectionId, control: usize) -> iced::Task<T> {
    let owner = section_identity(id);
    reveal((0..=control).map(|flat| row_widget_id(owner, flat)).collect(), body_scroll_id(id))
}

pub(super) fn section_identity(id: SectionId) -> u64 {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone, Copy)]
pub struct CalmCtx {
    pub open: Option<SectionId>,
    pub section: usize,
    pub busy: f32,
    pub pop: Option<u64>,
    pub edit: Option<u64>,
    pub instant: bool,
    pub closing: Option<SectionId>,
}

#[derive(Clone, Copy)]
pub(super) struct Paint<'a> {
    pub values: &'a HashMap<String, String>,
    pub focus: FocusCtx<'a>,
    pub calm: CalmCtx,
    pub scale: f32,
    pub palette: &'a Palette,
    pub motion: MotionProfile,
    pub body: f32,
    pub dim: bool,
    pub reading: f32,
    pub owner: u64,
    pub undivided: bool,
    pub trim: Trim,
}

impl<'a> Paint<'a> {
    pub(super) fn compact(
        &self,
        label: String,
        width: f32,
        message: Option<Message>,
        focused: bool,
    ) -> ButtonSpec<'a> {
        ButtonSpec {
            width,
            label,
            active: false,
            destructive: false,
            message,
            focused,
            scale: self.scale,
            palette: self.palette,
            fade: 1.0,
            height: ButtonHeight::Compact,
            motion: self.motion,
        }
    }

    pub(super) fn ink(&self) -> Palette {
        let palette = *self.palette;
        if !self.dim {
            return palette;
        }
        let plane = self.open_plane();
        let fade = |color| calm::tint(plane, color, BODY.dim_alpha);
        Palette {
            primary: fade(palette.primary),
            primary_text: fade(palette.primary_text),
            surface_text: fade(palette.surface_text),
            surface_container: fade(palette.surface_container),
            outline: fade(palette.outline),
            tertiary: fade(palette.tertiary),
            ..palette
        }
    }

    pub(super) fn surface(&self) -> Color {
        self.palette.surface
    }

    pub(super) fn open_plane(&self) -> Color {
        calm::tint(self.palette.surface, self.palette.primary, ROW_TINT)
    }

    pub(super) fn muted(&self, plane: Color, alpha: f32) -> Color {
        calm::tint(plane, self.ink().surface_text, alpha)
    }

    pub(super) fn size(&self, px: f32) -> f32 {
        px * self.scale
    }
}

pub(super) struct Block<'a> {
    pub index: usize,
    pub section: Section,
    pub lead: Vec<Element<'a, Message>>,
    pub gutter: Option<Element<'a, Message>>,
}

pub(super) fn stage_width(scale: f32) -> f32 {
    BODY.stage * scale
}

pub(super) fn reading_height(sheet: f32, scale: f32) -> f32 {
    let [top, ..] = SURFACE.padding;
    sheet - (top + SURFACE.heading + SURFACE.head_gap + BODY.divider + SURFACE.gap) * scale
}

fn wide_gain(scale: f32) -> f32 {
    body_inset(scale) - BODY.wide_inset * 2.0 * scale
}

pub(super) fn body_inset(scale: f32) -> f32 {
    let [_, right, _, left] = BODY.padding;
    (right + left) * scale
}

pub(super) fn title_text<'a>(
    title: impl iced::advanced::text::IntoFragment<'a>,
    fade: f32,
    paint: Paint<'a>,
) -> iced::widget::Text<'a> {
    text(title)
        .font(UI_FONT)
        .size(paint.size(SURFACE.heading) * legible_type_scale(1.0))
        .line_height(LineHeight::Relative(1.0))
        .color(paint.muted(paint.surface(), fade))
}

pub(super) fn heading<'a>(
    title: &'a str,
    note: &'a str,
    fade: f32,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    row![
        title_text(title, fade, paint),
        container(calm::worded(
            note,
            SURFACE.note,
            paint.scale,
            paint.muted(paint.surface(), SURFACE.note_alpha * fade),
            calm::PLAIN_LEADING,
        ))
        .padding(Padding { bottom: paint.size(SURFACE.note_drop), ..Padding::ZERO })
        .width(Length::Fill),
    ]
    .spacing(paint.size(SURFACE.head_spacing))
    .align_y(Alignment::End)
    .width(Length::Fill)
    .into()
}

pub(super) fn strip_heading<'a>(
    title: &'a str,
    strip: Element<'a, Message>,
    fade: f32,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    row![title_text(title, fade, paint), strip]
        .spacing(paint.size(SURFACE.head_spacing))
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
}

pub(super) fn close_button(paint: Paint<'_>) -> Element<'_, Message> {
    crate::frontend::ui::folio_action(
        "×",
        false,
        Some(Message::ToggleSettings),
        Length::Fixed(paint.size(SURFACE.close)),
        paint.scale,
        paint.palette,
    )
}

pub(super) fn rule(paint: Paint<'_>, alpha: f32) -> Element<'_, Message> {
    folio_horizontal_rule(calm::tint(paint.surface(), paint.palette.outline, alpha))
}

fn name<'a>(section: &Section, index: usize, quiet: f32, paint: Paint<'a>) -> Element<'a, Message> {
    let palette = paint.palette;
    let focused =
        paint.focus.keyboard_focus == SettingsFocus::Sections && paint.calm.section == index;
    let desc_height = desc_height(section, paint) * (1.0 - CALM.at(quiet));
    let fold = calm::GROW.duration.as_secs_f32() / DESC_FADE.as_secs_f32();
    let desc_fade = 1.0 - CSS_EASE.at((quiet * fold).min(1.0));
    let mut copy = column![
        text(section.title.clone())
            .font(UI_FONT)
            .size(paint.size(SURFACE.title) * legible_type_scale(1.0))
            .color(if focused { palette.primary } else { palette.surface_text })
    ]
    .spacing(paint.size(SURFACE.name_gap));
    if !section.desc.is_empty() && desc_height > 0.0 {
        let note = container(calm::worded(
            section.desc.clone(),
            SURFACE.desc,
            paint.scale,
            paint.muted(paint.surface(), SURFACE.desc_alpha * desc_fade),
            SURFACE.desc_leading,
        ));
        copy = copy.push(if quiet > 0.0 { note.max_height(desc_height).clip(true) } else { note });
    }
    let width = Length::Fixed(paint.size(SURFACE.name));
    let inset = logical_padding(0.0, 0.0, 0.0, paint.size(SURFACE.name_inset));
    if !section.has_body() {
        return container(copy).padding(inset).width(width).into();
    }
    button(copy)
        .padding(inset)
        .width(width)
        .on_press(Message::Settings(SettingsMsg::ToggleSection(index)))
        .style(button::text)
        .into()
}

fn options<'a>(
    section: &Section,
    index: usize,
    open: bool,
    focused: bool,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    calm::options_button(OptionsSpec {
        identity: section_identity(section.id),
        label: section.opens.unwrap_or_else(|| tr("settings-section-options")),
        open,
        focused,
        message: Message::Settings(SettingsMsg::ToggleSection(index)),
        instant: paint.calm.instant,
        scale: paint.scale,
        palette: *paint.palette,
        plane: if open { paint.open_plane() } else { paint.surface() },
    })
}

pub(super) fn say_tokens<'a>(
    section: &Section,
    rows: &[&Row],
    paint: Paint<'a>,
) -> Vec<Vec<Piece<'a, Message>>> {
    let caption = |blank: &Option<String>, form: Form, control: &Control| match (control, form) {
        (Control::Number { key, .. }, Form::Duration) => {
            super::number_editor::duration_caption(paint.values.get(key).map_or("", String::as_str))
        }
        _ => blank.clone(),
    };
    section
        .say
        .tokens
        .iter()
        .map(|slot| match slot {
            Slot::Checks { .. } => slot_identity(slot, rows).map_or_else(Vec::new, |identity| {
                let key = token_key(&identity, 0, TokenSide::Sentence);
                let open = Message::Settings(SettingsMsg::OpenPop(key));
                vec![Piece::token(key, checked(slot, rows), Token::Menu, open)]
            }),
            Slot::Value { row, blank, form } => rows.get(*row).map_or_else(Vec::new, |row| {
                if let Some(link) = link_piece(row, *form) {
                    return vec![link];
                }
                let caption = caption(blank, *form, &row.control);
                token_pieces(&row.control, TokenSide::Sentence, caption.as_deref(), paint)
            }),
        })
        .collect()
}

pub(super) fn spoken<'a>(
    pieces: Vec<Piece<'a, Message>>,
    plane: Color,
    popover: Option<Popover<'a, Message>>,
    focused: Option<u64>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    calm::sentence(SentenceSpec {
        pieces,
        size: SAY_SIZE,
        leading: SAY_LEADING,
        scale: paint.scale,
        palette: *paint.palette,
        plane,
        focused,
        ring: focused.is_some(),
        popover,
        centred: false,
    })
}

pub(super) fn caption(row: &Row) -> Option<&str> {
    row.look.blank.as_deref().filter(|_| matches!(row.control, Control::TextField { .. }))
}

pub(super) fn parked<'a>(
    row: &Row,
    plane: Color,
    open: bool,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let tokens = token_pieces(&row.control, TokenSide::Sentence, caption(row), paint);
    let focused = row.control.token().map(|key| token_key(&key, 0, TokenSide::Sentence));
    let mark = calm::sentinel(0).map(String::from).unwrap_or_default();
    let pieces = if tokens.is_empty() {
        vec![Piece::plain(row.title.clone())]
    } else {
        calm::split(&places::lead(&row.title, &mark), vec![tokens])
    };
    let popover = popover(row, TokenSide::Sentence, plane, paint);
    let button: Option<Element<'a, Message>> = match &row.control {
        Control::Toggle { .. } | Control::ToggleAction { .. } => {
            Some(toggle(row.control.clone(), open, true, paint))
        }
        Control::ActionBtn { id, label } => {
            Some(action_button(id.clone(), label.clone(), true, paint))
        }
        _ => None,
    };
    row![container(spoken(pieces, plane, popover, focused, paint)).width(Length::Fill)]
        .push_maybe(button)
        .spacing(paint.size(SURFACE.say_gap))
        .align_y(Alignment::Center)
        .into()
}

fn link_piece<'a>(row: &Row, form: Form) -> Option<Piece<'a, Message>> {
    let Control::ActionBtn { id, label } = &row.control else {
        return None;
    };
    matches!(form, Form::Link | Form::Mention).then(|| {
        let key = token_key(label, 0, TokenSide::Sentence);
        let press = Message::Settings(SettingsMsg::Run(id.clone()));
        let kind = Token::Mention { underlined: form == Form::Link };
        Piece::token(key, label.clone(), kind, press)
    })
}

fn say<'a>(section: &Section, index: usize, quiet: f32, paint: Paint<'a>) -> Element<'a, Message> {
    let open = paint.calm.open == Some(section.id);
    let plane = if open { paint.open_plane() } else { paint.surface() };
    let rows: Vec<&Row> = section.rows().collect();
    let lit =
        |at: usize| paint.calm.section == index && paint.focus.lit(SettingsFocus::Controls, at);
    let focused = section
        .say
        .tokens
        .iter()
        .find(|slot| {
            slot.rows().any(|at| token_side(section, at) == TokenSide::Sentence && lit(at))
        })
        .and_then(|slot| slot_identity(slot, &rows))
        .map(|identity| token_key(&identity, 0, TokenSide::Sentence));
    let tokens = say_tokens(section, &rows, paint);
    let popover = section.say.tokens.iter().find_map(|slot| match slot {
        Slot::Checks { heading, .. } => slot_identity(slot, &rows).and_then(|identity| {
            let members: Vec<&Row> = picked(slot, &rows).collect();
            let key = token_key(&identity, 0, TokenSide::Sentence);
            checks(key, heading, &members, Side::Below, paint)
        }),
        Slot::Value { row, .. } => {
            rows.get(*row).and_then(|row| popover(row, TokenSide::Sentence, plane, paint))
        }
    });
    let mut pieces = calm::split(&section.say.text, tokens);
    if let Some(aside) = section.say.aside.as_ref().filter(|_| quiet == 0.0) {
        pieces.extend([Piece::plain(calm::ASIDE_GAP), Piece::aside(aside.clone())]);
    }
    let sentence = spoken(pieces, plane, popover, focused, paint);
    let mut line = row![container(sentence).width(Length::Fill).align_x(start())]
        .spacing(paint.size(SURFACE.say_gap))
        .align_y(Alignment::Center);
    let lead = usize::from(section.master.is_some());
    for (at, row) in section.head.iter().enumerate() {
        if section.say.parks(lead + at).is_some() || section.say.binds(lead + at) {
            continue;
        }
        if let Control::ActionBtn { id, label } = &row.control {
            line = line.push(action_button(id.clone(), label.clone(), lit(lead + at), paint));
        }
    }
    let mut copy = column![line].width(Length::Fill);
    let landed = rows
        .iter()
        .enumerate()
        .find(|(at, _)| lit(*at) && section.say.parks(*at).is_some() && !section.say.binds(*at));
    if let Some((_, row)) = landed {
        copy = copy.push(parked(row, plane, open, paint));
    }
    if quiet == 0.0 {
        for note in &section.say.notes {
            copy = copy.push(calm::worded(
                note.clone(),
                SURFACE.hint,
                paint.scale,
                paint.muted(plane, SURFACE.hint_alpha),
                SURFACE.hint_leading,
            ));
        }
    }
    copy.into()
}

pub(super) fn action_button(
    id: ActionId,
    label: String,
    focused: bool,
    paint: Paint<'_>,
) -> Element<'_, Message> {
    let confirming = paint.focus.armed == Some(&id);
    let label = if confirming {
        crate::i18n::tr_args!(super::control::CONFIRM_KEY, label => &label)
    } else {
        label
    };
    let (width, height) =
        super::control::fitted_button(&label, paint.size(BODY.button_max), paint.scale);
    let message = Some(Message::Settings(SettingsMsg::Run(id)));
    super::control::fixed_button(ButtonSpec {
        active: confirming,
        destructive: confirming,
        height,
        ..paint.compact(label, width, message, focused)
    })
}

pub(super) fn header<'a>(
    section: &Section,
    index: usize,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let open = paint.calm.open == Some(section.id);
    let quiet = if open { 0.0 } else { paint.calm.busy };
    let pad = paint.size(SURFACE.pad + (SURFACE.busy_pad - SURFACE.pad) * CALM.at(quiet));
    let lit =
        |focus: SettingsFocus| paint.focus.keyboard_focus == focus && paint.calm.section == index;
    let master = section.master.as_ref().map_or_else(
        || Space::new().into(),
        |master| {
            let focused = lit(SettingsFocus::Controls) && paint.focus.focused_control == 0;
            toggle(master.control.clone(), open, focused, paint)
        },
    );
    let more = if section.has_body() {
        options(section, index, open, lit(SettingsFocus::Sections), paint)
    } else {
        Space::new().into()
    };
    let head =
        row![name(section, index, quiet, paint), say(section, index, quiet, paint), master, more]
            .spacing(paint.size(SURFACE.columns))
            .align_y(Alignment::Center);
    container(head)
        .padding(Padding { top: pad, right: paint.size(SURFACE.pad_end), bottom: pad, left: 0.0 })
        .width(Length::Fill)
        .into()
}

pub(super) fn block<'a>(block: Block<'a>, paint: Paint<'a>) -> Element<'a, Message> {
    let Block { index, section, lead, gutter } = block;
    #[cfg(test)]
    let section = tests::untrimmed(section);
    let id = section.id;
    let open = paint.calm.open == Some(id);
    let identity = section_identity(section.id);
    let head = header(&section, index, paint);
    let floor = paint.size(BODY.room_floor + BODY.padding[0] + bottom_padding(section.trim));
    let body = if open || paint.calm.closing == Some(id) {
        body(section, BodyParts { lead, gutter }, open, paint)
    } else {
        Space::new().into()
    };
    let instant = paint.calm.instant;
    let content = column![
        rule(paint, SURFACE.secs_rule),
        head,
        calm::reveal(body, RevealSpec { identity, open, instant, floor }),
    ];
    container(calm::plane(
        content,
        PlaneSpec {
            identity,
            open,
            instant: paint.calm.instant,
            planes: [paint.surface(), paint.open_plane()],
        },
    ))
    .id(section_widget_id(id))
    .into()
}

pub(super) struct Staggered {
    pub identity: u64,
    pub open: bool,
    pub order: usize,
}

impl Staggered {
    pub(super) fn wrap<'a>(
        &mut self,
        cell: Element<'a, Message>,
        paint: Paint<'a>,
    ) -> Element<'a, Message> {
        let index = self.order;
        self.order += 1;
        calm::stagger(
            cell,
            StaggerSpec {
                identity: self.identity ^ index as u64,
                index,
                open: self.open,
                instant: paint.calm.instant,
                planes: [paint.surface(), paint.open_plane()],
            },
        )
    }
}

fn band<'a>(columns: Vec<Element<'a, Message>>, paint: Paint<'a>) -> Element<'a, Message> {
    let mut band = row![].spacing(paint.size(BODY.column_gap));
    for (at, column) in columns.into_iter().enumerate() {
        if at > 0 {
            let edge = if paint.undivided { 0.0 } else { BODY.divider_alpha };
            let divider = calm::tint(paint.open_plane(), paint.palette.outline, edge);
            band = band.push(
                container(Space::new())
                    .width(Length::Fixed(1.0))
                    .height(Length::Fill)
                    .style(move |_| bg_style(divider)),
            );
        }
        band = band.push(container(column).width(Length::FillPortion(1)));
    }
    band.height(Length::Shrink).into()
}

fn stacked<'a>(
    heading: Option<&str>,
    note: Option<&str>,
    cells: Vec<Element<'a, Message>>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let mut column = column![].width(Length::Fill);
    if let Some(heading) = heading {
        column = column.push(group_heading(heading, note, paint));
    }
    for cell in cells {
        column = column.push(cell);
    }
    column.into()
}

fn pack_cells<'a>(
    rows: Vec<Row>,
    blank: Option<&str>,
    width: f32,
    flat: &mut usize,
    stagger: &mut Staggered,
    paint: Paint<'a>,
) -> Vec<Element<'a, Message>> {
    let mut cells: Vec<Element<'a, Message>> = Vec::new();
    let mut rows = rows.into_iter().peekable();
    while let Some(row) = rows.next() {
        let many = row.look.cell == Cell::Checks;
        let mut partners: Vec<Row> = Vec::new();
        while let Some(next) = rows.next_if(|next| match next.look.cell {
            Cell::Beside => true,
            Cell::Joined => many || partners.is_empty(),
            _ => false,
        }) {
            partners.push(next);
        }
        let joined = partners.len();
        let cell = Value { row, partners, blank: blank.map(str::to_string) };
        cells.push(stagger.wrap(value_row(cell, *flat, width, paint), paint));
        *flat += 1 + joined;
    }
    cells
}

struct BodyParts<'a> {
    lead: Vec<Element<'a, Message>>,
    gutter: Option<Element<'a, Message>>,
}

fn desc_lines(section: &Section, paint: Paint<'_>) -> f32 {
    let line =
        crate::frontend::ui::cached_line_width(&section.desc, paint.size(SURFACE.desc), UI_FONT);
    let width = paint.size(SURFACE.name - SURFACE.name_inset);
    (line * BODY.wrap_slack / width).ceil()
}

fn desc_height(section: &Section, paint: Paint<'_>) -> f32 {
    let line = paint.size(SURFACE.desc) * legible_type_scale(1.0) * SURFACE.desc_leading;
    line * desc_lines(section, paint)
}

fn bottom_padding(trim: Trim) -> f32 {
    match trim {
        Trim::None => BODY.padding[2],
        Trim::Bottom | Trim::Rows => BODY.padding[2] - BODY.snug_trim,
    }
}

fn body<'a>(
    section: Section,
    parts: BodyParts<'a>,
    open: bool,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let BodyParts { lead, gutter } = parts;
    let wide = section.wide;
    let scroll = body_scroll_id(section.id);
    let paint = Paint { owner: section_identity(section.id), ..paint };
    let order = super::library_page::first_stagger(section.id);
    let mut stagger = Staggered { identity: section_identity(section.id), open, order };
    let flush = super::library_page::flush(section.id);
    let mut flat = section.header_rows();
    let dim = section.switched_off();
    let columns = section
        .groups
        .iter()
        .filter(|group| group.kind == GroupKind::Column && !group.is_empty())
        .count()
        .max(section.min_columns)
        .min(BODY.columns);
    let gaps =
        (paint.size(BODY.column_gap) * 2.0 + BODY.divider) * columns.saturating_sub(1) as f32;
    let gain = if section.wide { wide_gain(paint.scale) } else { 0.0 };
    let paint = Paint {
        body: paint.body + gain,
        undivided: section.undivided,
        trim: section.trim,
        ..paint
    };
    let width_px = (paint.body - gaps) / columns.max(1) as f32;
    let mut content = column![].spacing(paint.size(BODY.gap));
    for part in lead {
        content = content.push(part);
    }
    let mut pending: Vec<Element<'a, Message>> = Vec::new();
    for group in section.groups {
        if group.is_empty() {
            continue;
        }
        let paint = Paint { dim: group.follows_master && dim, ..paint };
        let Group { heading, note, rows, kind, blank, echoes, .. } = group;
        let mut cells = |rows: Vec<Row>, stagger: &mut Staggered| {
            pack_cells(rows, blank.as_deref(), width_px, &mut flat, stagger, paint)
        };
        match kind {
            GroupKind::Column => {
                if pending.len() == 2 {
                    content = content.push(band(std::mem::take(&mut pending), paint));
                }
                let mut cells = cells(rows, &mut stagger);
                for (at, echo) in echoes {
                    let cell = Value { row: echo, partners: Vec::new(), blank: None };
                    let echoed = value_row(cell, ECHOED, width_px, paint);
                    cells.insert(at.min(cells.len()), stagger.wrap(echoed, paint));
                }
                pending.push(stacked(heading.as_deref(), note.as_deref(), cells, paint));
            }
            GroupKind::Spread => {
                if !pending.is_empty() {
                    content = content.push(band(std::mem::take(&mut pending), paint));
                }
                let mut left = rows;
                let right = left.split_off(left.len().div_ceil(2));
                let left = stacked(None, None, cells(left, &mut stagger), paint);
                let right = stacked(None, None, cells(right, &mut stagger), paint);
                content = content.push(stacked(
                    heading.as_deref(),
                    note.as_deref(),
                    vec![band(vec![left, right], paint)],
                    paint,
                ));
            }
            GroupKind::Wide => {
                if !pending.is_empty() {
                    content = content.push(band(std::mem::take(&mut pending), paint));
                }
                let cells = cells(rows, &mut stagger);
                content = content.push(stacked(heading.as_deref(), note.as_deref(), cells, paint));
            }
            GroupKind::Foot => {
                if !pending.is_empty() {
                    content = content.push(band(std::mem::take(&mut pending), paint));
                }
                content = content.push(stagger.wrap(foot_line(&rows, &mut flat, paint), paint));
            }
            GroupKind::Lines => {
                if !pending.is_empty() {
                    content = content.push(band(std::mem::take(&mut pending), paint));
                }
                let lines = line_cells(rows, &mut flat, blank.as_deref(), &mut stagger, paint);
                content = content.push(listed(heading.as_deref(), note.as_deref(), lines, paint));
            }
        }
    }
    if !pending.is_empty() {
        if pending.len() < section.min_columns {
            pending.push(Space::new().into());
        }
        content = content.push(band(pending, paint));
    }
    if let Some(foot) = section.foot {
        content = content.push(hint(foot, paint));
    }
    framed(content, gutter, Frame { scroll, flush, wide, trim: section.trim }, paint)
}

fn line_cells<'a>(
    rows: Vec<Row>,
    flat: &mut usize,
    blank: Option<&str>,
    stagger: &mut Staggered,
    paint: Paint<'a>,
) -> Vec<Element<'a, Message>> {
    lines(rows)
        .map(|line| {
            let banded = line.iter().any(|row| matches!(row.control, Control::Resolution { .. }));
            let cell = if banded {
                band_line(&line, flat, blank, paint)
            } else {
                line_row(line, flat, paint)
            };
            stagger.wrap(cell, paint)
        })
        .collect()
}

fn hint(copy: String, paint: Paint<'_>) -> Element<'_, Message> {
    calm::worded(
        copy,
        SURFACE.hint,
        paint.scale,
        paint.muted(paint.open_plane(), SURFACE.hint_alpha),
        SURFACE.hint_leading,
    )
}

fn foot_line<'a>(rows: &[Row], flat: &mut usize, paint: Paint<'a>) -> Element<'a, Message> {
    let mut line = row![].spacing(paint.size(BODY.foot_gap)).align_y(Alignment::Center);
    for row in rows {
        if let Control::ActionBtn { id, label } = &row.control {
            let lit = paint.focus.lit(SettingsFocus::Controls, *flat);
            line = line.push(action_button(id.clone(), label.clone(), lit, paint));
        }
        if !row.desc.is_empty() {
            line = line.push(container(hint(row.desc.clone(), paint)).width(Length::Fill));
        }
        *flat += 1;
    }
    container(line).padding(Padding { top: paint.size(BODY.foot_gap), ..Padding::ZERO }).into()
}

struct Frame {
    scroll: iced::widget::Id,
    flush: bool,
    wide: bool,
    trim: Trim,
}

fn framed<'a>(
    content: iced::widget::Column<'a, Message>,
    gutter: Option<Element<'a, Message>>,
    frame: Frame,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let Frame { scroll, flush, wide, trim } = frame;
    let [top, mut right, _, mut left] = BODY.padding.map(|px| paint.size(px));
    let bottom = paint.size(bottom_padding(trim));
    if flush {
        left = paint.size(SURFACE.name_inset);
    }
    if wide {
        (right, left) = (paint.size(BODY.wide_inset), paint.size(BODY.wide_inset));
    }
    let content = scrollable(content)
        .id(scroll)
        .width(Length::Fill)
        .direction(crate::frontend::ui::spaced_vbar(paint.size(BODY.bar_gap)))
        .style(crate::frontend::ui::outline_scroll(paint.palette.outline));
    let Some(gutter) = gutter else {
        return container(content).padding(Padding { top, right, bottom, left }).into();
    };
    let inset = paint.size(SURFACE.name_inset);
    let stage = paint.size(BODY.stage);
    let side = container(gutter).width(Length::Fixed(stage));
    container(row![side, content].spacing(left - inset - stage))
        .padding(Padding { top, right, bottom, left: inset })
        .into()
}

fn listed<'a>(
    heading: Option<&str>,
    note: Option<&str>,
    lines: Vec<Element<'a, Message>>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let overflow = lines.len() > BODY.lines_visible;
    let list = stacked(None, None, lines, paint);
    if !overflow {
        return stacked(heading, note, vec![list], paint);
    }
    let height = paint.size(BODY.line_height) * BODY.lines_visible as f32;
    let scroll = scrollable(list)
        .height(Length::Fixed(height))
        .direction(crate::frontend::ui::thin_vbar())
        .style(crate::frontend::ui::outline_scroll(paint.palette.outline));
    stacked(heading, note, vec![scroll.into()], paint)
}

fn group_heading<'a>(heading: &str, note: Option<&str>, paint: Paint<'a>) -> Element<'a, Message> {
    let [top, right, bottom, left] = BODY.heading_padding.map(|px| paint.size(px));
    let plane = paint.open_plane();
    let mut line = row![label(
        heading.to_uppercase(),
        BODY.heading,
        paint.scale,
        paint.muted(plane, BODY.heading_alpha),
    )]
    .spacing(paint.size(BODY.row_gap))
    .align_y(Alignment::Start);
    if let Some(note) = note {
        let hint = calm::worded(
            note.to_string(),
            BODY.hint,
            paint.scale,
            paint.muted(plane, BODY.hint_alpha),
            BODY.note_leading,
        );
        line = line.push(container(hint).width(Length::Fill));
    }
    container(line).padding(Padding { top, right, bottom, left }).into()
}

fn copy<'a>(row: &Row, paint: Paint<'a>) -> Element<'a, Message> {
    let title = paint.size(BODY.value) * legible_type_scale(1.0);
    let mut pieces = vec![Piece::inked(row.title.clone(), title, paint.ink().surface_text)];
    if !row.hint().is_empty() {
        let gap = if row.title.is_empty() { "" } else { BODY.hint_gap };
        pieces.push(Piece::inked(
            format!("{gap}{}", row.hint()),
            paint.size(BODY.hint) * legible_type_scale(1.0),
            paint.muted(paint.open_plane(), BODY.hint_alpha),
        ));
    }
    calm::sentence(calm::prose(pieces, calm::PLAIN_LEADING))
}

struct Value {
    row: Row,
    partners: Vec<Row>,
    blank: Option<String>,
}

fn value_row<'a>(cell: Value, flat: usize, width: f32, paint: Paint<'a>) -> Element<'a, Message> {
    let Value { row, partners, blank } = cell;
    if let Cell::Corners { cut } = row.look.cell {
        let values: Vec<&Row> = std::iter::once(&row).chain(&partners).collect();
        return corners_row(&values, cut, flat, paint);
    }
    let value = value_cell(&row, &partners, blank.as_deref(), flat, width, paint);
    let rule = rule(paint, BODY.row_rule);
    let [vertical, horizontal] = BODY.row_padding.map(|px| paint.size(px));
    let vertical =
        if paint.trim == Trim::Rows { vertical - paint.size(BODY.lean_trim) } else { vertical };
    let cell: Element<'a, Message> = match value {
        Some(value) => {
            let spent = paint.size(BODY.row_gap + BODY.row_padding[1] * 2.0);
            let floor = label_floor(&row, width, paint);
            let label = container(copy(&row, paint)).width(Length::Fill).align_x(start());
            let label =
                if flat == ECHOED { label } else { label.id(label_widget_id(paint.owner, flat)) };
            #[cfg(test)]
            tests::note_floor(
                &label_widget_id(paint.owner, flat),
                longest_word(&row, paint).min(floor),
            );
            let side =
                (width - spent - floor - crate::frontend::ui::BAR_WIDTH * paint.scale).max(0.0);
            row![label, container(value).max_width(side)]
                .width(Length::Fill)
                .spacing(paint.size(BODY.row_gap))
                .align_y(Alignment::Center)
                .into()
        }
        None if matches!(row.control, Control::Segment) => group_heading(&row.title, None, paint),
        None => field(
            row,
            flat,
            paint.values,
            paint.focus,
            width,
            paint.scale,
            paint.palette,
            1.0,
            paint.motion,
        ),
    };
    let cell = container(cell).padding([vertical, horizontal]).width(Length::Fill);
    let cell = if flat == ECHOED { cell } else { cell.id(row_widget_id(paint.owner, flat)) };
    column![cell, rule].into()
}

fn action_cell<'a>(
    row: &Row,
    values: &[&Row],
    flat: usize,
    width: f32,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let lit = |at: usize| paint.focus.lit(SettingsFocus::Controls, at);
    let actions: Vec<_> = values
        .iter()
        .enumerate()
        .filter_map(|(at, value)| match &value.control {
            Control::ActionBtn { id, label } => Some((at, id, label)),
            _ => None,
        })
        .collect();
    let across = fits(row, actions.iter().map(|(_, _, label)| label.as_str()), width, paint);
    let buttons = actions
        .into_iter()
        .map(|(at, id, label)| action_button(id.clone(), label.clone(), lit(flat + at), paint))
        .collect();
    lay(buttons, across, paint)
}

fn nudge_pad<'a>(
    items: &[(ActionId, String)],
    axes: &[&Row],
    flat: usize,
    choice: Option<usize>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let lit = |at: usize| paint.focus.lit(SettingsFocus::Controls, at);
    let mut line = row![].spacing(paint.size(BODY.bar_gap)).align_y(Alignment::Center);
    for (at, (id, caption)) in items.iter().enumerate() {
        let measured = super::control::label_width(caption, paint.scale);
        let width = paint.size(BODY.nudge).max(measured + paint.size(BODY.nudge_inset));
        let message = Some(Message::Settings(SettingsMsg::Run(id.clone())));
        let focused = lit(flat) && choice == Some(at);
        line = line.push(super::control::fixed_button(paint.compact(
            caption.clone(),
            width,
            message,
            focused,
        )));
    }
    let shown = (0..axes.len()).find(|at| lit(flat + 1 + at)).map(|at| (at, 0));
    if !axes.is_empty() {
        line = line.push(Space::new().width(Length::Fixed(paint.size(BODY.bar_gap))));
        line = line.push(tokens(axes, shown, None, paint));
    }
    line.into()
}

fn corners_row<'a>(
    values: &[&Row],
    cut: bool,
    flat: usize,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let lit = |at: usize| paint.focus.lit(SettingsFocus::Controls, flat + at).then_some((0, 0));
    let side = paint.size(BODY.corner_side);
    let cell = |at: usize| -> Element<'a, Message> {
        values.get(at).map_or_else(
            || Space::new().into(),
            |row| {
                let token = tokens(&[row], lit(at), None, paint);
                container(token).id(row_widget_id(paint.owner, flat + at)).into()
            },
        )
    };
    let radius = |at: usize| -> f32 {
        let row = values.get(at).or_else(|| values.first());
        let key = row.and_then(|row| match &row.control {
            Control::Number { key, .. } => Some(key.as_str()),
            _ => None,
        });
        let raw = key.and_then(|key| paint.values.get(key)).map_or("", String::as_str);
        raw.trim().parse::<f32>().unwrap_or_default().clamp(0.0, BODY.corner_max) * paint.scale
    };
    let edge = calm::tint(paint.surface(), paint.palette.outline, SURFACE.head_rule);
    let [top_left, top_right, bottom_left, bottom_right] = std::array::from_fn(|at| at);
    let sizes = [top_left, top_right, bottom_right, bottom_left].map(radius);
    let [box_width, box_height] = BODY.corner_box.map(|px| paint.size(px));
    let outline = iced::widget::canvas(super::corner_outline::Outline { sizes, cut, edge })
        .width(Length::Fixed(box_width))
        .height(Length::Fixed(box_height));
    let gap = paint.size(BODY.corner_gap);
    let mirrored = crate::frontend::ui::rtl();
    let (upper, lower) = if mirrored {
        ([top_right, top_left], [bottom_right, bottom_left])
    } else {
        ([top_left, top_right], [bottom_left, bottom_right])
    };
    let content: Element<'a, Message> = if values.len() == 1 {
        let token = tokens(&[values[0]], lit(0), None, paint);
        iced::widget::stack![
            outline,
            container(token)
                .width(Length::Fixed(box_width))
                .height(Length::Fixed(box_height))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        ]
        .into()
    } else {
        let flank = |top: usize, bottom: usize, end: bool| {
            let floor = Space::new().width(side).height(0.0);
            column![floor, cell(top), Space::new().height(Length::Fill), cell(bottom)]
                .align_x(if end { Alignment::End } else { Alignment::Start })
                .height(Length::Fixed(box_height))
        };
        row![flank(upper[0], lower[0], !mirrored), outline, flank(upper[1], lower[1], mirrored)]
            .spacing(gap)
            .into()
    };
    let [vertical, horizontal] = BODY.row_padding.map(|px| paint.size(px));
    column![
        container(content).padding([vertical, horizontal]).center_x(Length::Fill),
        rule(paint, BODY.row_rule),
    ]
    .into()
}

fn value_cell<'a>(
    row: &Row,
    partners: &[Row],
    blank: Option<&str>,
    flat: usize,
    width: f32,
    paint: Paint<'a>,
) -> Option<Element<'a, Message>> {
    let lit = |at: usize| paint.focus.lit(SettingsFocus::Controls, at);
    let (focused, choice) = (lit(flat), paint.focus.focused_choice);
    let values: Vec<&Row> = std::iter::once(row).chain(partners).collect();
    let lit_slot = |slot: usize| (0..values.len()).find(|at| lit(flat + at)).map(|at| (at, slot));
    match (&row.control, row.look.cell) {
        (Control::ActionBtn { id: id @ ActionId::ResetSetting(change), label }, _) => {
            Some(super::home_page::shift(change, id, label, focused, paint))
        }
        (Control::ActionChips { items }, Cell::Nudge) => {
            Some(nudge_pad(items, &values[1..], flat, choice, paint))
        }
        (
            Control::Dropdown { path, current, .. } | Control::Chips { path, current, .. },
            Cell::Buttons(labels),
        ) => {
            let options =
                labels.iter().map(|(value, key)| ((*value).to_string(), tr(key).to_string()));
            let options: Vec<_> = options.collect();
            let across = fits(row, options.iter().map(|(_, label)| label.as_str()), width, paint);
            let choices = Choices { path, current, options, choice, focused, across };
            Some(picks(choices, paint))
        }
        (_, Cell::Checks) => Some(tally(row, &values[1..], focused, paint)),
        (Control::AppTheme { .. }, _) => {
            Some(app_value(&values, lit_slot(0).map(|(at, _)| at), paint))
        }
        (Control::Dropdown { .. }, Cell::Menu) => Some(tokens(&values, lit_slot(0), blank, paint)),
        (Control::ActionChips { items }, Cell::Menu) => Some(choose(row, items, focused, paint)),
        (Control::Chips { background: Some(background), .. }, _) => {
            let lit = lit_slot(0).map(|(at, _)| at);
            Some(placement(&values, background, lit, paint))
        }
        (Control::Colour { .. }, _) => Some(colours(&values, lit_slot(0).map(|(at, _)| at), paint)),
        (Control::ActionBtn { id: ActionId::ChooseRunningProcess, label }, _) => {
            Some(processes(row, label, focused, paint))
        }
        _ if mixed(&values) => Some(mixed_cell(&values, flat, blank, paint)),
        (Control::TextField { key, placeholder, .. }, Cell::Field) => {
            let blank = blank.unwrap_or(placeholder);
            let field = line_field(key, blank, focused, paint);
            Some(container(field).width(Length::Fixed(paint.size(BODY.text_max))).into())
        }
        (Control::Toggle { .. } | Control::ToggleAction { .. }, _) => {
            Some(toggle(row.control.clone(), true, focused, paint))
        }
        (control, _) if super::super::tokenable(control) => {
            Some(tokens(&values, lit_slot(0), blank, paint))
        }
        (Control::ActionBtn { .. }, _) => Some(action_cell(row, &values, flat, width, paint)),
        (Control::ActionChips { items }, _) => {
            let mut buttons = row![].spacing(paint.size(BODY.line_gap));
            for (at, (id, label)) in items.iter().enumerate() {
                let lit = focused && choice.unwrap_or(0) == at;
                buttons = buttons.push(action_button(id.clone(), label.clone(), lit, paint));
            }
            Some(buttons.into())
        }
        (Control::Static, _) => Some(Space::new().into()),
        _ => None,
    }
}

fn mixed(values: &[&Row]) -> bool {
    let driven = |row: &&Row| {
        matches!(
            row.control,
            Control::ActionBtn { .. } | Control::Toggle { .. } | Control::ToggleAction { .. }
        )
    };
    let tokened = |row: &&Row| super::super::tokenable(&row.control);
    values.len() > 1 && values.iter().any(tokened) && values.iter().any(driven)
}

fn mixed_cell<'a>(
    values: &[&Row],
    flat: usize,
    blank: Option<&str>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let lit = |at: usize| paint.focus.lit(SettingsFocus::Controls, flat + at);
    let tokened: Vec<&Row> =
        values.iter().copied().filter(|row| super::super::tokenable(&row.control)).collect();
    let shown = values
        .iter()
        .enumerate()
        .filter(|(_, row)| super::super::tokenable(&row.control))
        .position(|(at, _)| lit(at))
        .map(|at| (at, paint.focus.focused_choice.unwrap_or(0)));
    let mut line = row![tokens(&tokened, shown, blank, paint)]
        .spacing(paint.size(BODY.button_gap))
        .align_y(Alignment::Center);
    for (at, value) in values.iter().enumerate() {
        if let Control::ActionBtn { id, label } = &value.control {
            line = line.push(action_button(id.clone(), label.clone(), lit(at), paint));
        }
    }
    for (at, value) in values.iter().enumerate() {
        if value.control.switched_on().is_some() {
            line = line.push(toggle(value.control.clone(), true, lit(at), paint));
        }
    }
    line.into()
}

fn placement<'a>(
    values: &[&Row],
    background: &BackgroundControl,
    lit: Option<usize>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let lead = values[0];
    let live: Vec<_> = values[1..]
        .iter()
        .filter_map(|row| match &row.control {
            Control::Display(control) => Some(control.clone()),
            _ => None,
        })
        .collect();
    let key = live_key(&background.output);
    let mut pieces = token_pieces(&lead.control, TokenSide::Body, None, paint);
    if background.placement_visible || !live.is_empty() {
        let open = Message::Settings(SettingsMsg::OpenPop(key));
        pieces.push(Piece::plain(BODY.beside));
        pieces.push(Piece::token(key, tr(MORE_KEY), Token::Menu, open));
    }
    let popover = (paint.calm.pop == Some(key))
        .then(|| Popover {
            key,
            side: Side::BelowEnd,
            content: live_panel(lead, background, live, paint),
        })
        .or_else(|| popover(lead, TokenSide::Body, paint.open_plane(), paint));
    let focused = lit.map(|at| {
        if at == 0 {
            lead.control.token().map_or(key, |token| token_key(&token, 0, TokenSide::Body))
        } else {
            key
        }
    });
    phrase(pieces, focused, popover, paint)
}

fn live_panel<'a>(
    lead: &Row,
    background: &BackgroundControl,
    live: Vec<crate::frontend::display_control::DisplayControl>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let palette = paint.palette;
    let mut panel = column![].spacing(paint.size(MENU.panel_gap));
    if background.placement_visible {
        let reveal = lead.control.path().map_or(1.0, |path| {
            paint
                .focus
                .bar_reveals
                .get(&BackgroundControl::reveal_id(path))
                .map_or(1.0, |value| value.x)
        });
        let picker =
            paint.focus.bar_reveals.get(&background.picker_id()).map_or(0.0, |value| value.x);
        panel = panel.push(background.view(reveal, picker, paint.scale, palette));
    }
    for control in live {
        panel = panel.push(control.view(paint.scale * MENU.panel_scale, palette));
    }
    let ground = palette.surface;
    let palette = *palette;
    container(panel)
        .padding(paint.size(MENU.panel_padding))
        .width(Length::Fixed(BackgroundControl::choices_width(paint.scale)))
        .style(move |_| calm::style::popover(ground, &palette))
        .into()
}

pub(super) fn colours<'a>(
    rows: &[&Row],
    lit: Option<usize>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let mut line = row![].spacing(paint.size(BODY.swatch_gap)).align_y(Alignment::Center);
    for (at, row) in rows.iter().enumerate() {
        let Control::Colour { path, hex } = &row.control else {
            continue;
        };
        if at > 0 {
            line = line.push(label(
                BODY.beside,
                BODY.value,
                paint.scale,
                paint.muted(paint.open_plane(), BODY.hint_alpha),
            ));
        }
        let key = token_key(path, 0, TokenSide::Body);
        let open = Message::Settings(SettingsMsg::OpenPop(key));
        let popover = (paint.calm.pop == Some(key)).then(|| Popover {
            key,
            side: Side::BelowEnd,
            content: wheel_panel(path, hex, paint),
        });
        let token = Piece::token(key, hex.clone(), Token::Menu, open);
        let swatch = super::super::colour::swatch(hex, paint.size(BODY.swatch), paint.palette);
        line = line.push(swatch).push(phrase(
            vec![token],
            (lit == Some(at)).then_some(key),
            popover,
            paint,
        ));
    }
    line.into()
}

fn wheel_panel<'a>(path: &str, hex: &str, paint: Paint<'a>) -> Element<'a, Message> {
    let changed = path.to_string();
    let wheel = super::super::colour::wheel(
        hex,
        paint.scale,
        move |hex| Message::Settings(SettingsMsg::Colour(changed.clone(), hex)),
        Message::Settings(SettingsMsg::ColourCommit(path.to_string())),
    );
    let ground = paint.palette.surface;
    let palette = *paint.palette;
    container(wheel)
        .padding(paint.size(MENU.panel_padding))
        .style(move |_| calm::style::popover(ground, &palette))
        .into()
}

fn processes<'a>(row: &Row, action: &str, focused: bool, paint: Paint<'a>) -> Element<'a, Message> {
    let path = keys::playback::PROCESSES;
    let raw = paint.values.get(path).cloned().unwrap_or_default();
    let key = token_key(path, 0, TokenSide::Body);
    let plane = paint.open_plane();
    let shown = if raw.trim().is_empty() {
        row.look.blank.clone().unwrap_or_default()
    } else {
        raw.clone()
    };
    let edit = Message::Settings(SettingsMsg::EditToken(key, path.to_string()));
    let token = Piece::token(key, shown, Token::Text, edit);
    let popover = (paint.calm.edit == Some(key)).then(|| Popover {
        key,
        side: Side::Over,
        content: editor(Field { input: path, raw, unit: "", width: None }, None, plane, paint),
    });
    let phrase = container(phrase(vec![token], None, popover, paint))
        .max_width(paint.size(BODY.process_max));
    let Control::ActionBtn { id, .. } = &row.control else {
        return phrase.into();
    };
    row![phrase, action_button(id.clone(), action.to_string(), focused, paint)]
        .spacing(paint.size(BODY.button_gap))
        .align_y(Alignment::Center)
        .into()
}

fn lines(rows: Vec<Row>) -> impl Iterator<Item = Vec<Row>> {
    let action = |row: &Row| matches!(row.control, Control::ActionBtn { .. });
    let mut lines = vec![Vec::new()];
    let mut rows = rows.into_iter().peekable();
    while let Some(row) = rows.next() {
        let chips = matches!(row.control, Control::ActionChips { .. });
        let ends = chips || (action(&row) && !rows.peek().is_some_and(action));
        lines.last_mut().expect("one line").push(row);
        if ends {
            lines.push(Vec::new());
        }
    }
    lines.into_iter().filter(|line| !line.is_empty())
}

fn line_row<'a>(line: Vec<Row>, flat: &mut usize, paint: Paint<'a>) -> Element<'a, Message> {
    let lone = line.len() == 1;
    let hint = |copy: String| {
        label(copy, BODY.line_hint, paint.scale, paint.muted(paint.open_plane(), BODY.hint_alpha))
    };
    let mut cells = row![].spacing(paint.size(BODY.line_gap)).align_y(Alignment::Center);
    for row in line {
        let focused = paint.focus.lit(SettingsFocus::Controls, *flat);
        *flat += 1;
        let titled = if row.title.is_empty() || matches!(row.control, Control::ActionBtn { .. }) {
            cells
        } else {
            cells.push(hint(row.title.clone()))
        };
        cells = match &row.control {
            Control::TextField { key, placeholder, .. } => {
                titled.push(line_field(key, placeholder, focused, paint))
            }
            Control::ActionBtn { id, label } => {
                let cells = titled.push(action_button(id.clone(), label.clone(), focused, paint));
                if lone && !row.hint().is_empty() {
                    cells.push(hint(row.hint().to_string()))
                } else {
                    cells
                }
            }
            Control::Toggle { .. } => {
                titled.push(toggle(row.control.clone(), true, focused, paint))
            }
            Control::ActionChips { items } => {
                let mut cells =
                    titled.push(container(hint(row.hint().to_string())).width(Length::Fill));
                for (at, (id, label)) in items.iter().enumerate() {
                    let lit = focused && paint.focus.focused_choice.unwrap_or(0) == at;
                    cells = cells.push(action_button(id.clone(), label.clone(), lit, paint));
                }
                cells
            }
            control if super::super::tokenable(control) => {
                titled.push(tokens(&[&row], focused.then_some((0, 0)), None, paint))
            }
            _ => titled.push(container(hint(row.hint().to_string())).width(Length::Fill)),
        };
    }
    let [vertical, horizontal] = BODY.line_padding.map(|px| paint.size(px));
    column![
        container(cells).padding([vertical, horizontal]).width(Length::Fill),
        rule(paint, BODY.row_rule)
    ]
    .into()
}

fn band_line<'a>(
    line: &[Row],
    flat: &mut usize,
    blank: Option<&str>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let hint = |copy: &str| {
        label(
            copy.to_string(),
            BODY.line_hint,
            paint.scale,
            paint.muted(paint.open_plane(), BODY.hint_alpha),
        )
    };
    let choice = paint.focus.focused_choice;
    let mut left = row![].spacing(paint.size(BODY.line_gap)).align_y(Alignment::Center);
    let mut right = row![].spacing(paint.size(BODY.button_gap)).align_y(Alignment::Center);
    let mut limit = None;
    for row in line {
        let focused = paint.focus.lit(SettingsFocus::Controls, *flat);
        *flat += 1;
        match &row.control {
            Control::TextField { .. } => {
                left = left.push(tokens(&[row], focused.then_some((0, 0)), blank, paint));
            }
            Control::Resolution { key, .. } => {
                let set = paint.values.get(key).is_some_and(|value| !value.is_empty());
                left = if set {
                    left.push(hint(&row.title)).push(tokens(
                        &[row],
                        focused.then(|| (0, choice.unwrap_or(0))),
                        None,
                        paint,
                    ))
                } else {
                    left.push(hint(tr(NO_MAXIMUM)))
                };
                if key.rsplit_once('.').is_some_and(|(_, leaf)| leaf == BAND_TO) {
                    let toggle = SettingsMsg::ResolutionLimit(key.clone(), !set);
                    let caption = tr(LIMITS[usize::from(!set)]).to_string();
                    limit = Some(pressed(
                        caption,
                        Message::Settings(toggle),
                        false,
                        !set && focused,
                        paint,
                    ));
                }
            }
            Control::Chips { path, current, options, .. } => {
                let options = options.clone();
                let choices = Choices { path, current, options, choice, focused, across: true };
                right = right.push(picks(choices, paint));
            }
            Control::ActionBtn { id, label } => {
                right = right.push_maybe(limit.take());
                right = right.push(action_button(id.clone(), label.clone(), focused, paint));
            }
            _ => {}
        }
    }
    let cells = row![container(left.wrap()).width(Length::Fill), right.push_maybe(limit)]
        .spacing(paint.size(BODY.row_gap))
        .align_y(Alignment::Center);
    let [vertical, horizontal] = BODY.line_padding.map(|px| paint.size(px));
    column![
        container(cells).padding([vertical, horizontal]).width(Length::Fill),
        rule(paint, BODY.row_rule)
    ]
    .into()
}

fn line_field<'a>(
    key: &str,
    placeholder: &str,
    focused: bool,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let raw = paint.values.get(key).map_or("", String::as_str);
    let ink = paint.ink();
    let input = key.to_string();
    let [vertical, horizontal] = BODY.field_padding.map(|px| paint.size(px));
    let ring = paint.size(BODY.ring);
    text_input(placeholder, raw)
        .id(workbench_input_id(key))
        .on_input(move |raw| Message::Settings(SettingsMsg::Input(input.clone(), raw)))
        .on_submit(Message::Settings(SettingsMsg::Commit))
        .font(UI_FONT)
        .size(paint.size(BODY.value) * legible_type_scale(1.0))
        .padding([vertical, horizontal])
        .width(Length::Fill)
        .style(move |_, _| {
            let mut style =
                crate::frontend::ui::workbench_input_style(ink.surface_text, ink.primary);
            if focused {
                style.border.color = ink.primary;
                style.border.width = ring;
            }
            style
        })
        .into()
}

fn switch_identity(control: &Control) -> Option<String> {
    match control {
        Control::Toggle { path, .. } => Some(path.clone()),
        Control::ToggleAction { id, .. } => Some(format!("{:?}", id.steady())),
        _ => None,
    }
}

pub(super) fn toggle(
    control: Control,
    open: bool,
    focused: bool,
    paint: Paint<'_>,
) -> Element<'_, Message> {
    let Some(identity) = switch_identity(&control) else {
        return Space::new().into();
    };
    let (on, message) = match control {
        Control::Toggle { path, value } => (value, SettingsMsg::Toggle(path, !value)),
        Control::ToggleAction { id, value } => (value, SettingsMsg::Run(id)),
        _ => return Space::new().into(),
    };
    calm::wipe_toggle(ToggleSpec {
        identity: token_key(&identity, 0, TokenSide::Body),
        on,
        focused,
        labels: [tr("settings-toggle-off"), tr("settings-toggle-on")],
        message: Message::Settings(message),
        instant: paint.calm.instant,
        scale: paint.scale,
        palette: paint.ink(),
        plane: if open { paint.open_plane() } else { paint.surface() },
    })
}

struct Choices<'p> {
    path: &'p str,
    current: &'p str,
    options: Vec<(String, String)>,
    choice: Option<usize>,
    focused: bool,
    across: bool,
}

fn picks<'a>(choices: Choices<'_>, paint: Paint<'a>) -> Element<'a, Message> {
    let Choices { path, current, options, choice, focused, across } = choices;
    let on = options.iter().position(|(value, _)| value == current);
    let lit = choice.or(on).filter(|_| focused);
    let buttons: Vec<_> = options
        .into_iter()
        .enumerate()
        .map(|(at, (value, label))| {
            let message = Message::Settings(SettingsMsg::Pick(path.to_string(), value));
            pressed(label, message, on == Some(at), lit == Some(at), paint)
        })
        .collect();
    lay(buttons, across, paint)
}

pub(super) fn lay<'a>(
    buttons: Vec<Element<'a, Message>>,
    across: bool,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let gap = paint.size(BODY.button_gap);
    if across {
        return crate::frontend::ui::Flow::new(buttons)
            .spacing(gap)
            .align_y(Alignment::Center)
            .into();
    }
    iced::widget::Column::with_children(buttons).spacing(gap).align_x(Alignment::End).into()
}

fn pressed(
    label: String,
    message: Message,
    active: bool,
    focused: bool,
    paint: Paint<'_>,
) -> Element<'_, Message> {
    let width = super::control::button_width(&label, paint.scale);
    super::control::fixed_button(ButtonSpec {
        active,
        ..paint.compact(label, width, Some(message), focused)
    })
}

fn natural(row: &Row, paint: Paint<'_>) -> f32 {
    let measure = |text: &str, size: f32| {
        crate::frontend::ui::cached_line_width(
            text,
            paint.size(size) * legible_type_scale(1.0),
            UI_FONT,
        )
    };
    let hint = row.hint();
    let note =
        if hint.is_empty() { 0.0 } else { measure(&format!("{}{hint}", BODY.hint_gap), BODY.hint) };
    measure(&row.title, BODY.value) + note
}

fn breaks_anywhere(ch: char) -> bool {
    matches!(ch, '\u{3040}'..='\u{30FF}' | '\u{3400}'..='\u{9FFF}' | '\u{AC00}'..='\u{D7AF}')
}

fn unbreakable(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for word in text.split_whitespace() {
        let mut run = String::new();
        for ch in word.chars() {
            if breaks_anywhere(ch) {
                found.extend((!run.is_empty()).then(|| std::mem::take(&mut run)));
                found.push(ch.to_string());
            } else {
                run.push(ch);
            }
        }
        found.extend((!run.is_empty()).then_some(run));
    }
    found
}

fn longest_word(row: &Row, paint: Paint<'_>) -> f32 {
    let word = |text: &str, size: f32| {
        unbreakable(text)
            .iter()
            .map(|piece| {
                crate::frontend::ui::cached_line_width(
                    piece,
                    paint.size(size) * legible_type_scale(1.0),
                    UI_FONT,
                )
            })
            .fold(0.0, f32::max)
    };
    word(&row.title, BODY.value).max(word(row.hint(), BODY.hint))
}

fn label_floor(row: &Row, width: f32, paint: Paint<'_>) -> f32 {
    let share = natural(row, paint).min(width * BODY.label_share);
    (longest_word(row, paint) * BODY.wrap_slack).max(share).min(width * BODY.label_limit)
}

fn fits<'l>(
    row: &Row,
    labels: impl Iterator<Item = &'l str>,
    width: f32,
    paint: Paint<'_>,
) -> bool {
    let mut count = 0;
    let buttons: f32 = labels
        .inspect(|_| count += 1)
        .map(|label| super::control::button_width(label, paint.scale))
        .sum();
    let gaps = paint.size(BODY.button_gap) * (count as f32 - 1.0).max(0.0);
    natural(row, paint) + buttons + gaps + paint.size(BODY.row_gap + BODY.row_padding[1] * 2.0)
        <= width
}

fn tokens<'a>(
    rows: &[&Row],
    focused: Option<(usize, usize)>,
    blank: Option<&str>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let plane = paint.open_plane();
    let mut pieces = Vec::new();
    for (at, row) in rows.iter().enumerate() {
        if at > 0 {
            let joiner = if row.look.cell == Cell::Beside { BODY.beside } else { BODY.pair };
            pieces.push(Piece::plain(joiner));
        }
        let caption = row
            .look
            .blank
            .as_deref()
            .or(blank)
            .filter(|_| matches!(row.control, Control::TextField { .. }));
        pieces.extend(token_pieces(&row.control, TokenSide::Body, caption, paint));
    }
    let focused = focused.and_then(|(at, slot)| {
        let identity = rows.get(at)?.control.token()?;
        Some(token_key(&identity, slot, TokenSide::Body))
    });
    let popover = rows.iter().find_map(|row| popover(row, TokenSide::Body, plane, paint));
    let sentence = phrase(pieces, focused, popover, paint);
    if !rows.iter().any(|row| matches!(row.control, Control::TextField { .. })) {
        return sentence;
    }
    container(sentence).max_width(paint.size(BODY.token_max)).into()
}

pub(super) fn phrase<'a>(
    pieces: Vec<Piece<'a, Message>>,
    focused: Option<u64>,
    popover: Option<Popover<'a, Message>>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    calm::sentence(SentenceSpec {
        pieces,
        size: BODY.value,
        leading: SAY_LEADING,
        scale: paint.scale,
        palette: paint.ink(),
        plane: paint.open_plane(),
        focused,
        ring: focused.is_some(),
        popover,
        centred: false,
    })
}

fn tally<'a>(
    lead: &Row,
    members: &[&Row],
    focused: bool,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let Some(identity) = members.first().and_then(|row| row.control.path()) else {
        return Space::new().into();
    };
    let key = token_key(identity, 0, TokenSide::Body);
    let shown = members.iter().filter(|row| row.control.ticked() == Some(true)).count();
    let label = if lead.desc.is_empty() {
        crate::i18n::tr_args!(CHECKS_KEY, shown => shown, all => members.len())
    } else {
        lead.desc.clone()
    };
    let open = Message::Settings(SettingsMsg::OpenPop(key));
    let popover = checks(key, &lead.title, members, Side::BelowEnd, paint);
    phrase(
        vec![Piece::token(key, label, Token::Menu, open)],
        focused.then_some(key),
        popover,
        paint,
    )
}

pub(super) fn choose<'a>(
    row: &Row,
    items: &[(ActionId, String)],
    focused: bool,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let Some(identity) = row.control.token() else {
        return Space::new().into();
    };
    let key = token_key(&identity, 0, TokenSide::Body);
    let choice = paint.focus.focused_choice.filter(|_| focused);
    let ink = paint.palette.surface_text;
    let popover = (paint.calm.pop == Some(key) || choice.is_some()).then(|| {
        let items = items
            .iter()
            .enumerate()
            .map(|(at, (id, caption))| Item {
                caption: caption.clone(),
                on: false,
                lit: choice == Some(at),
                ink,
                message: Some(Message::Settings(SettingsMsg::Run(id.clone()))),
                ..Item::default()
            })
            .collect();
        Popover { key, side: Side::BelowEnd, content: menu_frame(&row.title, items, paint) }
    });
    let open = Message::Settings(SettingsMsg::OpenPop(key));
    let token = Piece::token(key, tr(CHOOSE_KEY), Token::Menu, open);
    phrase(vec![token], focused.then_some(key), popover, paint)
}

pub(super) fn slot_identity(slot: &Slot, rows: &[&Row]) -> Option<String> {
    let control = &rows.get(slot.lead())?.control;
    match slot {
        Slot::Value { form: Form::Link | Form::Mention, .. } => match control {
            Control::ActionBtn { label, .. } => Some(label.clone()),
            _ => None,
        },
        Slot::Checks { .. } => control.path().map(str::to_string),
        Slot::Value { .. } => control.token(),
    }
}

fn picked<'r>(slot: &Slot, rows: &[&'r Row]) -> impl Iterator<Item = &'r Row> {
    slot.rows().filter_map(|at| rows.get(at).copied())
}

fn checked(slot: &Slot, rows: &[&Row]) -> String {
    let on: Vec<&str> = picked(slot, rows)
        .filter(|row| row.control.switched_on() == Some(true))
        .map(|row| row.title.as_str())
        .collect();
    if on.is_empty() {
        let Slot::Checks { blank, .. } = slot else {
            return String::new();
        };
        blank.clone()
    } else {
        places::listed(LIST_KEY, &on)
    }
}

fn checks<'a>(
    key: u64,
    heading: &str,
    members: &[&Row],
    side: Side,
    paint: Paint<'a>,
) -> Option<Popover<'a, Message>> {
    if paint.calm.pop != Some(key) {
        return None;
    }
    let ink = paint.palette.surface_text;
    let choice = paint.focus.focused_choice;
    let items: Vec<_> = members
        .iter()
        .enumerate()
        .filter_map(|(at, row)| {
            let on = row.control.ticked()?;
            let message = tick(&row.control)?;
            let lit = choice.map_or(on, |choice| choice == at);
            Some(Item {
                caption: row.title.clone(),
                on,
                lit,
                ink,
                message: Some(message),
                ..Item::default()
            })
        })
        .collect();
    Some(Popover { key, side, content: menu_frame(heading, items, paint) })
}

fn tick(control: &Control) -> Option<Message> {
    let message = match control {
        Control::Toggle { path, value } => SettingsMsg::Toggle(path.clone(), !value),
        Control::Dropdown { path, options, current, .. } => {
            let other = options.iter().find(|(value, _)| value != current)?;
            SettingsMsg::Pick(path.clone(), other.0.clone())
        }
        _ => return None,
    };
    Some(Message::Settings(message))
}

pub(super) fn token_pieces<'a>(
    control: &Control,
    side: TokenSide,
    caption: Option<&str>,
    paint: Paint<'a>,
) -> Vec<Piece<'a, Message>> {
    let Some(identity) = control.token() else {
        return Vec::new();
    };
    let key = token_key(&identity, 0, side);
    let message = |message| Message::Settings(message);
    match control {
        Control::Number { key: input, path, unit } => {
            vec![dial_piece(key, &Dial { input, path, unit }, caption, paint)]
        }
        Control::TextField { key: input, path, .. } if super::number_editor::scrubs_text(path) => {
            let raw = paint.values.get(input).map_or("", String::as_str);
            let (_, _, step) = super::number_editor::scrub_bounds(path);
            let value = raw.trim().parse::<f64>().unwrap_or_default();
            let shown = super::number_editor::number_step_text(value, step, None);
            vec![dial_piece(key, &Dial { input, path, unit: "" }, shown.as_deref(), paint)]
        }
        Control::TextField { key: input, placeholder, secret, .. } => {
            let raw = paint.values.get(input).map_or("", String::as_str);
            let shown = [raw, caption.unwrap_or(placeholder)]
                .into_iter()
                .find(|text| !text.is_empty())
                .unwrap_or_else(|| tr("settings-control-none"));
            let shown = if *secret && !raw.is_empty() {
                super::super::masked(raw)
            } else {
                shown.to_string()
            };
            vec![Piece::token(
                key,
                shown,
                Token::Text,
                message(SettingsMsg::EditToken(key, input.clone())),
            )]
        }
        Control::Dropdown { current, .. } | Control::Chips { current, .. } => {
            let shown = caption.or(control.current_label()).map_or_else(
                || crate::i18n::tr_args!(CUSTOM_KEY, value => current.as_str()),
                str::to_string,
            );
            vec![Piece::token(key, shown, Token::Menu, message(SettingsMsg::OpenPop(key)))]
        }
        Control::Resolution { key: input, width_placeholder, height_placeholder, .. } => {
            let blanks = [*width_placeholder, *height_placeholder];
            resolution_pieces(input, blanks, &identity, side, paint)
        }
        _ => Vec::new(),
    }
}

#[derive(Clone, Copy)]
struct Dial<'d> {
    input: &'d str,
    path: &'d str,
    unit: &'d str,
}

fn dial_piece<'a>(
    key: u64,
    dial: &Dial<'_>,
    caption: Option<&str>,
    paint: Paint<'a>,
) -> Piece<'a, Message> {
    let Dial { input, path, unit } = *dial;
    let message = |message| Message::Settings(message);
    let raw = paint.values.get(input).map_or("", String::as_str);
    let value = raw.trim().parse::<f64>().unwrap_or_default();
    let (min, max, step) = super::number_editor::scrub_bounds(path);
    let scrub = Scrub { value, min, max, step };
    let change = input.to_string();
    let previous = raw.to_string();
    let shown = caption
        .map_or_else(|| super::number_editor::number_caption(input, raw, unit), str::to_string);
    let stage = move |next: f64| {
        let text =
            super::number_editor::number_step_text(next, step, Some(&previous)).unwrap_or_default();
        message(SettingsMsg::Input(change.clone(), text))
    };
    let token =
        Token::Number { scrub, change: Box::new(stage), commit: message(SettingsMsg::Commit) };
    Piece::token(key, shown, token, message(SettingsMsg::EditToken(key, input.to_string())))
}

fn resolution_pieces<'a>(
    input: &str,
    blanks: [&str; 2],
    identity: &str,
    side: TokenSide,
    paint: Paint<'a>,
) -> Vec<Piece<'a, Message>> {
    let message = |message| Message::Settings(message);
    let raw = paint.values.get(input).map_or("", String::as_str);
    let mut pieces = Vec::new();
    for (slot, width) in DIMENSIONS.into_iter().enumerate() {
        if slot > 0 {
            pieces.push(Piece::plain(BODY.pair));
        }
        let half = dimension(raw, width);
        let scrub = Scrub {
            value: half.parse().unwrap_or_default(),
            min: SIZE_SCRUB.min,
            max: SIZE_SCRUB.max[slot],
            step: SIZE_SCRUB.step,
        };
        let change = input.to_string();
        let previous = half.to_string();
        let stage = move |next: f64| {
            let text = super::number_editor::number_step_text(next, scrub.step, Some(&previous))
                .unwrap_or_default();
            message(SettingsMsg::ResolutionInput(change.clone(), width, text))
        };
        let dial =
            Token::Number { scrub, change: Box::new(stage), commit: message(SettingsMsg::Commit) };
        let token = token_key(identity, slot, side);
        let shown = if half.is_empty() { blanks[slot] } else { half };
        let edit = message(SettingsMsg::EditToken(token, input.to_string()));
        pieces.push(Piece::token(token, shown.to_string(), dial, edit));
    }
    pieces
}

pub(super) fn popover<'a>(
    row: &Row,
    side: TokenSide,
    plane: Color,
    paint: Paint<'a>,
) -> Option<Popover<'a, Message>> {
    let control = &row.control;
    let identity = control.token()?;
    let key = token_key(&identity, 0, side);
    let every = token_key(&identity, EVERY_SLOT, side);
    if paint.calm.pop == Some(key) || paint.calm.pop == Some(every) {
        let side = if side == TokenSide::Body { Side::BelowEnd } else { Side::Below };
        let trimmed = (paint.calm.pop == Some(key)).then_some(every);
        return menu(row, trimmed, paint).map(|content| Popover { key, side, content });
    }
    let hint = (side == TokenSide::Sentence).then_some(row.hint()).filter(|hint| !hint.is_empty());
    let raw = |input: &str| paint.values.get(input).cloned().unwrap_or_default();
    let (key, field) = match control {
        Control::Number { key: input, unit, .. } => {
            (key, Field { input, raw: raw(input), unit, width: None })
        }
        Control::TextField { key: input, .. } => {
            (key, Field { input, raw: raw(input), unit: "", width: None })
        }
        Control::Resolution { key: input, .. } => {
            let slot = (0..DIMENSIONS.len())
                .find(|slot| paint.calm.edit == Some(token_key(&identity, *slot, side)))?;
            let width = DIMENSIONS[slot];
            let raw = dimension(&raw(input), width).to_string();
            (token_key(&identity, slot, side), Field { input, raw, unit: "", width: Some(width) })
        }
        _ => return None,
    };
    (paint.calm.edit == Some(key)).then(|| Popover {
        key,
        side: Side::Over,
        content: editor(field, hint, plane, paint),
    })
}

fn dimension(raw: &str, width: bool) -> &str {
    let (across, down) = resolution_parts(raw);
    if width { across } else { down }
}

struct Field<'f> {
    input: &'f str,
    raw: String,
    unit: &'static str,
    width: Option<bool>,
}

#[derive(Default)]
pub(super) struct Item {
    pub caption: String,
    pub on: bool,
    pub lit: bool,
    pub ink: Color,
    pub message: Option<Message>,
    pub strip: Vec<Color>,
    pub group: Option<String>,
}

fn strip_colours(palettes: &[(String, Vec<String>)], value: &str) -> Vec<Color> {
    palettes
        .iter()
        .find(|(name, _)| name == value)
        .into_iter()
        .flat_map(|(_, colours)| colours)
        .filter_map(|hex| crate::contracts::settings::parse_rgb(hex))
        .take(MENU.strip_colours)
        .map(|[r, g, b]| Color::from_rgb8(r, g, b))
        .collect()
}

fn menu<'a>(row: &Row, every: Option<u64>, paint: Paint<'a>) -> Option<Element<'a, Message>> {
    let (path, options, current, disabled, palettes) = match &row.control {
        Control::Dropdown { path, options, current, palettes } => {
            (path, options, current, &[][..], palettes.as_slice())
        }
        Control::Chips { path, options, current, disabled, .. } => {
            (path, options, current, disabled.as_slice(), &[][..])
        }
        _ => return None,
    };
    let more =
        row.look.more.as_ref().filter(|_| every.is_some() && paint.focus.focused_choice.is_none());
    let custom = usize::from(row.control.unlisted());
    let palette = *paint.palette;
    let focused = paint.focus.focused_choice;
    let mut items = Vec::new();
    if custom > 0 {
        items.push(Item {
            caption: crate::i18n::tr_args!(CUSTOM_KEY, value => current.as_str()),
            on: true,
            lit: focused.is_none_or(|choice| choice == 0),
            ink: palette.surface_text,
            message: Some(Message::Settings(SettingsMsg::ClosePop)),
            ..Item::default()
        });
    }
    for (at, (value, caption)) in options.iter().enumerate() {
        if more.is_some_and(|more| !more.short.contains(value)) {
            continue;
        }
        let on = value == current;
        let enabled = !disabled.contains(value);
        let pick = Message::Settings(SettingsMsg::Pick(path.clone(), value.clone()));
        items.push(Item {
            caption: caption.clone(),
            on,
            lit: focused.map_or(on, |choice| choice == at + custom),
            ink: if enabled {
                palette.surface_text
            } else {
                calm::tint(palette.surface, palette.surface_text, MENU.disabled_alpha)
            },
            message: enabled.then_some(pick),
            strip: strip_colours(palettes, value),
            ..Item::default()
        });
    }
    if let (Some(more), Some(every)) = (more, every) {
        items.push(Item {
            caption: more.label.clone(),
            on: false,
            lit: false,
            ink: palette.surface_text,
            message: Some(Message::Settings(SettingsMsg::OpenPop(every))),
            group: Some(tr(MORE_MENU).to_string()),
            ..Item::default()
        });
    }
    Some(menu_frame(row.look.heading.as_deref().unwrap_or(&row.title), items, paint))
}

pub(super) fn menu_item<'a>(
    item: Item,
    glyph: Option<&'static str>,
    striped: bool,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let palette = *paint.palette;
    let ground = palette.surface;
    let [vertical, horizontal] = MENU.item.map(|px| paint.size(px));
    let Item { caption, on, lit, ink, message, strip, group } = item;
    let mark = glyph.unwrap_or(match (group.is_some(), on) {
        (true, _) => "",
        (false, true) => SELECTED_MARK,
        (false, false) => OPEN_MARK,
    });
    let caption_width = if striped { Length::Fill } else { Length::Shrink };
    let mark_size = MENU.glyph * legible_type_scale(paint.scale);
    let mark_room = crate::frontend::ui::cached_line_width(SELECTED_MARK, mark_size, UI_FONT);
    button(
        row![
            container(label(mark, MENU.glyph, paint.scale, palette.primary))
                .width(Length::Fixed(mark_room)),
            container(label(caption, MENU.size, paint.scale, ink)).width(caption_width),
        ]
        .push_maybe((!strip.is_empty()).then(|| swatches(strip.iter().copied(), MENU.strip, paint)))
        .spacing(paint.size(MENU.gap))
        .align_y(Alignment::Center),
    )
    .padding([vertical, horizontal])
    .width(Length::Fill)
    .on_press_maybe(message)
    .style(move |_, status| calm::style::item(ground, &palette, lit, status))
    .into()
}

pub(super) fn menu_heading<'a>(heading: &str, paint: Paint<'a>) -> Element<'a, Message> {
    let palette = *paint.palette;
    let [top, right, bottom, left] = MENU.heading_padding.map(|px| paint.size(px));
    container(label(
        heading.to_string(),
        MENU.heading,
        paint.scale,
        calm::tint(palette.surface, palette.surface_text, MENU.heading_alpha),
    ))
    .padding(Padding { top, right, bottom, left })
    .into()
}

fn menu_frame<'a>(heading: &str, items: Vec<Item>, paint: Paint<'a>) -> Element<'a, Message> {
    let columns = items.len().div_ceil(MENU.rows).clamp(1, MENU.columns);
    let striped = items.iter().any(|item| !item.strip.is_empty());
    let width = match (columns > 1, striped) {
        (true, true) => MENU.striped_wide,
        (true, false) => MENU.wide,
        (false, true) => MENU.striped,
        (false, false) => MENU.width,
    };
    menu_panel(&[heading], items, columns, width, paint)
}

pub(super) fn fitted_menu<'a>(
    headings: &[&str],
    items: Vec<Item>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let frame = MENU.glyph + MENU.gap + MENU.item[1] * 2.0 + MENU.padding * 2.0 + MENU.slack;
    let width = items
        .iter()
        .map(|item| {
            crate::frontend::ui::cached_line_width(
                &item.caption,
                MENU.size * legible_type_scale(1.0),
                UI_FONT,
            ) + frame
        })
        .fold(MENU.width, f32::max);
    menu_panel(headings, items, 1, width, paint)
}

fn menu_panel<'a>(
    headings: &[&str],
    items: Vec<Item>,
    columns: usize,
    width: f32,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let palette = *paint.palette;
    let ground = palette.surface;
    let striped = items.iter().any(|item| !item.strip.is_empty());
    let item = |item: Item| menu_item(item, None, striped, paint);
    let [top, right, bottom, left] = MENU.heading_padding.map(|px| paint.size(px));
    let title = |caption: &str, top: f32| {
        container(label(
            caption.to_uppercase(),
            MENU.heading,
            paint.scale,
            calm::tint(ground, palette.surface_text, MENU.heading_alpha),
        ))
        .padding(Padding { top, right, bottom, left })
    };
    let mut list = column![];
    for (at, heading) in headings.iter().enumerate() {
        list = list.push(title(heading, if at > 0 { 0.0 } else { top }));
    }
    let mut items = items.into_iter().peekable();
    while items.peek().is_some() {
        if let Some(group) = items.peek().and_then(|entry| entry.group.clone()) {
            list = list.push(title(&group, top));
        }
        let mut line = row![];
        for entry in items.by_ref().take(columns) {
            line = line.push(container(item(entry)).width(Length::FillPortion(1)));
        }
        list = list.push(line);
    }
    container(list)
        .padding(paint.size(MENU.padding))
        .width(Length::Fixed(paint.size(width)))
        .style(move |_| calm::style::popover(ground, &palette))
        .into()
}

fn swatches(
    colours: impl IntoIterator<Item = Color>,
    size: [f32; 2],
    paint: Paint<'_>,
) -> Element<'_, Message> {
    let [width, height] = size.map(|px| paint.size(px));
    let mut strip = row![].width(Length::Fixed(width)).height(Length::Fixed(height));
    for colour in colours {
        strip = strip.push(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| bg_style(colour)),
        );
    }
    strip.into()
}

pub fn app_key(id: &str) -> u64 {
    token_key(&format!("{APP_TOKEN}{id}"), 0, TokenSide::Body)
}

fn app_slot(index: usize) -> u8 {
    u8::try_from(index).unwrap_or(u8::MAX)
}

fn app_value<'a>(values: &[&Row], lit: Option<usize>, paint: Paint<'a>) -> Element<'a, Message> {
    let Control::AppTheme { app, index } = &values[0].control else {
        return Space::new().into();
    };
    let key = app_key(&app.id);
    let profile = values.get(1).copied();
    let open = Message::Settings(SettingsMsg::OpenPop(key));
    let plane = paint.open_plane();
    let popover = if paint.calm.pop == Some(key) {
        Some(Popover { key, side: Side::BelowEnd, content: app_panel(app, *index, profile, paint) })
    } else {
        profile
            .and_then(|row| popover(row, TokenSide::Body, plane, paint))
            .map(|found| Popover { key, ..found })
    };
    let details = phrase(
        vec![Piece::token(key, tr(DETAILS_KEY), Token::Menu, open)],
        (lit == Some(1)).then_some(key),
        popover,
        paint,
    );
    let id = ActionId::SetAppTheme(app_slot(*index), !app.enabled && !app.can_disable);
    let focused = lit == Some(0);
    let action = match app.state.as_str() {
        APP_THEME_BUSY => None,
        APP_THEME_INTERRUPTED => Some(action_button(id, tr(APP_UNDO).to_string(), focused, paint)),
        _ if app.can_disable || app.can_enable => {
            Some(toggle(Control::ToggleAction { id, value: app.enabled }, true, focused, paint))
        }
        _ => None,
    };
    row![details]
        .push_maybe(action)
        .spacing(paint.size(BODY.row_gap))
        .align_y(Alignment::Center)
        .into()
}

fn app_panel<'a>(
    app: &crate::contracts::daemon::AppThemeStatus,
    index: usize,
    profile: Option<&Row>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let palette = *paint.palette;
    let ground = palette.surface;
    let parts = super::app_theme::panel_parts(app, app_slot(index));
    let stop = paint.focus.focused_choice;
    let mut panel =
        column![group_heading(&app.name, None, paint)].spacing(paint.size(MENU.panel_gap));
    for line in parts.lines {
        panel = panel.push(label(
            line,
            BODY.hint,
            paint.scale,
            calm::tint(ground, palette.surface_text, SURFACE.desc_alpha),
        ));
    }
    if let Some(row) = profile {
        let shown = row.control.current_label().map(str::to_string);
        let token = row.control.token();
        if let (Some(shown), Some(token)) = (shown, token) {
            let open = token_key(&token, 0, TokenSide::Body);
            let pieces = vec![
                Piece::plain(row.title.clone()),
                Piece::plain(BODY.hint_gap),
                Piece::token(
                    open,
                    shown,
                    Token::Menu,
                    Message::Settings(SettingsMsg::OpenPop(open)),
                ),
            ];
            panel = panel.push(phrase(pieces, (stop == Some(0)).then_some(open), None, paint));
        } else if !row.desc.is_empty() {
            panel =
                panel.push(label(row.desc.clone(), BODY.hint, paint.scale, palette.surface_text));
        }
    }
    let buttons = parts
        .actions
        .into_iter()
        .enumerate()
        .map(|(at, (id, caption))| action_button(id, caption, stop == Some(at + 1), paint))
        .collect();
    panel = panel
        .push(container(lay(buttons, false, paint)).width(Length::Fill).align_x(Alignment::End));
    container(panel)
        .padding(paint.size(MENU.panel_padding))
        .width(Length::Fixed(paint.size(APP_PANEL_WIDTH)))
        .style(move |_| calm::style::popover(ground, &palette))
        .into()
}

pub(super) fn palette_strip(paint: Paint<'_>) -> Element<'_, Message> {
    let palette = paint.palette;
    let roles = [
        palette.primary,
        palette.secondary,
        palette.tertiary,
        palette.hover,
        palette.surface,
        palette.surface_container,
        palette.surface_text,
        palette.error,
    ];
    container(swatches(roles, PALETTE_STRIP.size, paint))
        .padding(Padding { top: paint.size(PALETTE_STRIP.drop), ..Padding::ZERO })
        .into()
}

fn editor<'a>(
    field: Field<'_>,
    hint: Option<&str>,
    plane: Color,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let palette = *paint.palette;
    let Field { input, raw, unit, width } = field;
    let key = input.to_string();
    let typed = move |raw| {
        Message::Settings(match width {
            Some(width) => SettingsMsg::ResolutionInput(key.clone(), width, raw),
            None => SettingsMsg::Input(key.clone(), raw),
        })
    };
    let field =
        text_input("", &raw)
            .id(workbench_input_id(input))
            .on_input(typed)
            .on_submit(Message::Settings(SettingsMsg::Commit))
            .font(UI_FONT)
            .size(paint.size(MENU.editor) * legible_type_scale(1.0))
            .padding(Padding::ZERO)
            .width(Length::Fixed(paint.size(MENU.editor_min).max(
                crate::frontend::ui::cached_line_width(&raw, paint.size(MENU.editor), UI_FONT),
            )))
            .style(move |_, _| calm::style::editor(plane, &palette));
    let mut line = row![field].spacing(paint.size(MENU.unit_gap)).align_y(Alignment::Center);
    if !unit.is_empty() {
        line = line.push(label(unit, MENU.editor, paint.scale, palette.primary));
    }
    let Some(hint) = hint else {
        return container(line).style(move |_| bg_style(plane)).into();
    };
    let note = label(hint.to_string(), BODY.hint, paint.scale, paint.muted(plane, BODY.hint_alpha));
    container(column![line, note].spacing(paint.size(MENU.unit_gap)))
        .padding(paint.size(MENU.padding))
        .max_width(paint.size(MENU.hint_width))
        .style(move |_| calm::style::popover(plane, &palette))
        .into()
}

pub(super) fn sections_column<'a>(
    rows: Vec<Element<'a, Message>>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let mut list = column![];
    for row in rows {
        list = list.push(row);
    }
    list.push(rule(paint, SURFACE.secs_rule)).width(Length::Fill).into()
}

pub(super) fn layout_entry(paint: Paint<'_>) -> Element<'_, Message> {
    crate::frontend::ui::folio_action(
        tr("settings-studio-preview"),
        false,
        Some(Message::Settings(SettingsMsg::EnterLayoutStudio)),
        Length::Fixed(super::control::button_width(tr("settings-studio-preview"), paint.scale)),
        paint.scale,
        paint.palette,
    )
}
