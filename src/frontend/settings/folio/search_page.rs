use iced::widget::text::Wrapping;
use iced::widget::{column, container, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Length, Task};

use crate::app::Message;
use crate::frontend::theme::Palette;
use crate::frontend::ui::calm::{self, Piece, tint};
use crate::frontend::ui::{
    UI_FONT, box_style, label, legible_type_scale, logical_padding, row, start,
};
use crate::i18n::tr;

use super::super::places::SUMMARY_JOINER;
use super::super::{
    Cell, Control, Row, SettingsKey, SettingsMsg, SettingsSearchResult, Slot, settings_reading_id,
    settings_search_input_id,
};
use super::control;
use super::sections::{self, BODY, Paint, SURFACE, TokenSide};

pub(super) const INDEX_NOTE: &str = "settings-search-index-note";
const TITLE: &str = "settings-search-title";
const QUOTED: &str = "settings-search-heading";
const NOTE_EMPTY: &str = "settings-search-note-empty";
const NOTE_NONE: &str = "settings-search-note-none";
const NOTE_FOUND: &str = "settings-search-note-found";
const NOTE_TOP: &str = "settings-search-note-top";
const OPEN: &str = "settings-search-open";
const MATCHED: &str = "settings-search-matched";
const DEPENDS: &str = "settings-search-depends";
const KEY_DOWN: &str = "↓";
const KEY_SPACE: &str = "settings-search-key-space";
const KEY_ENTER: &str = "settings-search-key-enter";
const KEY_ESC: &str = "settings-search-key-esc";
const FROM_FIELD: &str = "settings-search-hint-field";
const IN_RESULTS: &str = "settings-search-hint-results";
const TOGGLES: &str = "settings-search-hint-toggle";
const OPENS: &str = "settings-search-hint-open";
const CLEARS: &str = "settings-search-hint-clear";
const SEARCH_GLYPH: &str = "⌕";
const CLEAR_GLYPH: &str = "×";
const ROW_ID_PREFIX: &str = "settings-search-row:";

pub(super) const SUGGESTIONS: [&str; 6] = [
    "settings-search-try-sound",
    "settings-search-try-fps",
    "settings-search-try-blur",
    "settings-search-try-lock",
    "settings-search-try-gpu",
    "settings-search-try-colour",
];

struct PageMetrics {
    pad: f32,
    crumb: f32,
    crumb_alpha: f32,
    crumb_gap: f32,
    why: f32,
    why_alpha: f32,
    why_rule: f32,
    why_pad: [f32; 2],
    title_share: f32,
    suggest_gap: f32,
}

const PAGE: PageMetrics = PageMetrics {
    pad: 10.0,
    crumb: 10.5,
    crumb_alpha: 0.5,
    crumb_gap: 6.0,
    why: 9.5,
    why_alpha: 0.6,
    why_rule: 0.4,
    why_pad: [0.0, 5.0],
    title_share: 0.6,
    suggest_gap: 7.0,
};

struct FieldMetrics {
    height: f32,
    pad_start: f32,
    pad_end: f32,
    gap: f32,
    glyph: f32,
    input: f32,
    clear: f32,
    clear_scale: f32,
    rule: f32,
    fill: f32,
    placeholder: f32,
}

const FIELD: FieldMetrics = FieldMetrics {
    height: 32.0,
    pad_start: 8.0,
    pad_end: 4.0,
    gap: 7.0,
    glyph: 15.0,
    input: 12.0,
    clear: 24.0,
    clear_scale: 0.8,
    rule: 0.28,
    fill: 0.58,
    placeholder: 0.48,
};

struct KeyMetrics {
    gap: f32,
    word_gap: f32,
    hint: f32,
    hint_alpha: f32,
    chip: f32,
    chip_alpha: f32,
    chip_rule: f32,
    chip_pad: [f32; 2],
}

const KEYS: KeyMetrics = KeyMetrics {
    gap: 7.0,
    word_gap: 4.0,
    hint: 10.5,
    hint_alpha: 0.56,
    chip: 10.0,
    chip_alpha: 0.72,
    chip_rule: 0.4,
    chip_pad: [2.0, 6.0],
};

#[derive(Clone, Copy)]
pub(super) struct Found<'a> {
    pub query: &'a str,
    pub results: &'a [SettingsSearchResult],
    pub total: usize,
    pub at: Option<usize>,
}

pub(super) fn result_id(at: usize) -> iced::widget::Id {
    format!("{ROW_ID_PREFIX}{at}").into()
}

pub fn reveal_result<T>(at: usize) -> Task<T> {
    sections::reveal(vec![result_id(at)], settings_reading_id())
}

fn title_of(query: &str) -> String {
    if query.is_empty() {
        tr(TITLE).to_string()
    } else {
        crate::i18n::tr_args!(QUOTED, query => query)
    }
}

fn note_of(found: Found<'_>) -> String {
    if found.query.trim().is_empty() {
        tr(NOTE_EMPTY).to_string()
    } else if found.results.is_empty() {
        tr(NOTE_NONE).to_string()
    } else if found.total > found.results.len() {
        crate::i18n::tr_args!(NOTE_TOP, shown => found.results.len(), count => found.total)
    } else {
        crate::i18n::tr_args!(NOTE_FOUND, count => found.total)
    }
}

fn heading<'a>(found: Found<'a>, paint: Paint<'a>) -> Element<'a, Message> {
    let room = paint.body + sections::body_inset(paint.scale);
    let title = sections::title_text(title_of(found.query.trim()), 1.0, paint)
        .wrapping(Wrapping::WordOrGlyph);
    row![
        container(title).max_width(room * PAGE.title_share),
        container(label(
            note_of(found),
            SURFACE.note,
            paint.scale,
            paint.muted(paint.surface(), SURFACE.note_alpha),
        ))
        .padding(iced::Padding { bottom: paint.size(SURFACE.note_drop), ..iced::Padding::ZERO })
        .width(Length::Fill),
    ]
    .spacing(paint.size(SURFACE.head_spacing))
    .align_y(Alignment::End)
    .width(Length::Fill)
    .into()
}

fn suggestions(paint: Paint<'_>) -> Element<'_, Message> {
    let buttons = SUGGESTIONS.map(|key| {
        let word = tr(key);
        let message = Message::Settings(SettingsMsg::SearchInput(word.to_string()));
        plain_button(word, message, paint)
    });
    crate::frontend::ui::Flow::new(buttons.into_iter().collect())
        .spacing(paint.size(PAGE.suggest_gap))
        .wrap()
        .into()
}

fn plain_button<'a>(caption: &str, message: Message, paint: Paint<'a>) -> Element<'a, Message> {
    let width = control::button_width(caption, paint.scale);
    control::fixed_button(paint.compact(caption.to_string(), width, Some(message), false))
}

pub(super) fn page<'a>(found: Found<'a>, paint: Paint<'a>) -> Element<'a, Message> {
    let head = row![
        heading(found, paint),
        sections::close_button(paint),
        crate::frontend::ui::folio_icon_action(
            crate::frontend::ui::EYE_CLOSED,
            Message::Stash,
            paint.scale,
            paint.palette
        ),
    ]
    .spacing(paint.size(SURFACE.head_spacing))
    .align_y(Alignment::Start);
    let head = column![head, sections::rule(paint, SURFACE.head_rule)]
        .spacing(paint.size(SURFACE.head_gap));
    let rows = found
        .results
        .iter()
        .enumerate()
        .map(|(at, result)| {
            let line = result_row(result, found.at == Some(at), found.query, paint);
            container(line).id(result_id(at)).into()
        })
        .collect();
    let mut list = column![].spacing(paint.size(SURFACE.gap));
    if found.query.trim().is_empty() {
        list = list.push(suggestions(paint));
    }
    list = list.push(sections::sections_column(rows, paint));
    let scroll = scrollable(list)
        .id(settings_reading_id())
        .direction(crate::frontend::ui::spaced_vbar(paint.size(BODY.bar_gap)))
        .style(crate::frontend::ui::outline_scroll(paint.palette.outline))
        .width(Length::Fill)
        .height(Length::Fill);
    let [top, right, bottom, left] = SURFACE.padding.map(|px| paint.size(px));
    column![head, container(scroll).height(Length::Fill)]
        .spacing(paint.size(SURFACE.gap))
        .padding(logical_padding(top, right, bottom, left))
        .height(Length::Fill)
        .into()
}

fn result_row<'a>(
    result: &'a SettingsSearchResult,
    selected: bool,
    query: &str,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let plane = if selected { paint.open_plane() } else { paint.surface() };
    let open = Message::Settings(SettingsMsg::OpenSearchResult(
        result.tab.clone(),
        result.section,
        result.row,
    ));
    let line = row![
        name(result, selected, query, plane, paint),
        container(say(result, plane, paint)).width(Length::Fill).align_x(start()),
    ]
    .push_maybe(action(result, selected, paint))
    .push(plain_button(tr(OPEN), open, paint))
    .spacing(paint.size(SURFACE.columns))
    .align_y(Alignment::Center);
    let [top, end, bottom] = [PAGE.pad, SURFACE.pad_end, PAGE.pad].map(|px| paint.size(px));
    let line = container(line)
        .padding(logical_padding(top, end, bottom, 0.0))
        .width(Length::Fill)
        .style(move |_| box_style(plane, plane));
    column![sections::rule(paint, SURFACE.secs_rule), line].into()
}

fn name<'a>(
    result: &'a SettingsSearchResult,
    selected: bool,
    query: &str,
    plane: Color,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let palette = paint.palette;
    let crumb = label(
        format!(
            "{}{SUMMARY_JOINER}{}",
            crate::frontend::ui::sentence_case(&result.tab_label),
            result.section_title
        ),
        PAGE.crumb,
        paint.scale,
        paint.muted(plane, PAGE.crumb_alpha),
    )
    .wrapping(Wrapping::WordOrGlyph);
    let mut trail = vec![Element::from(crumb)];
    if result.elsewhere {
        let terms = query.split_whitespace().collect::<Vec<_>>().join(" ");
        let [vertical, horizontal] = PAGE.why_pad.map(|px| paint.size(px));
        let edge = tint(plane, palette.outline, PAGE.why_rule);
        trail.push(
            container(label(
                crate::i18n::tr_args!(MATCHED, terms => terms.as_str()),
                PAGE.why,
                paint.scale,
                paint.muted(plane, PAGE.why_alpha),
            ))
            .padding([vertical, horizontal])
            .style(move |_| box_style(Color::TRANSPARENT, edge))
            .into(),
        );
    }
    let trail = crate::frontend::ui::Flow::new(trail)
        .spacing(paint.size(PAGE.crumb_gap))
        .align_y(Alignment::Center)
        .wrap();
    let copy = column![
        text(result.title.as_str())
            .font(UI_FONT)
            .size(paint.size(SURFACE.title) * legible_type_scale(1.0))
            .wrapping(Wrapping::WordOrGlyph)
            .color(if selected { palette.primary } else { palette.surface_text }),
        trail,
    ]
    .spacing(paint.size(SURFACE.name_gap))
    .width(Length::Fill)
    .align_x(start());
    container(copy)
        .padding(logical_padding(0.0, 0.0, 0.0, paint.size(SURFACE.name_inset)))
        .width(Length::Fixed(paint.size(SURFACE.name)))
        .into()
}

fn plain<'a>(
    result: &SettingsSearchResult,
    plane: Color,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    match described(result) {
        Some(copy) => quiet(copy, plane, paint),
        None => sections::spoken(vec![Piece::plain(own_words(result))], plane, None, None, paint),
    }
}

fn described(result: &SettingsSearchResult) -> Option<&str> {
    if result.wake.is_some() {
        return None;
    }
    let words = match &result.setting {
        Some(row) => [row.hint(), row.desc.as_str()].into_iter().find(|words| !words.is_empty()),
        None => Some(result.desc.as_str()),
    };
    words.filter(|words| !words.is_empty())
}

fn quiet<'a>(copy: &str, plane: Color, paint: Paint<'a>) -> Element<'a, Message> {
    calm::worded(
        copy.to_string(),
        SURFACE.hint,
        paint.scale,
        paint.muted(plane, SURFACE.hint_alpha),
        SURFACE.hint_leading,
    )
}

pub(super) fn own_words(result: &SettingsSearchResult) -> String {
    if let Some(wake) = &result.wake {
        return crate::i18n::tr_args!(DEPENDS, setting => wake.as_str());
    }
    let Some(row) = &result.setting else {
        return result.desc.clone();
    };
    let value = if matches!(row.control, Control::Static) { "" } else { result.value.as_str() };
    [row.hint(), row.desc.as_str(), value]
        .into_iter()
        .find(|words| !words.is_empty())
        .unwrap_or_default()
        .to_string()
}

fn worded(piece: Piece<'_, Message>) -> Piece<'_, Message> {
    match piece {
        Piece::Token { label, .. } => Piece::Value(label),
        other => other,
    }
}

fn own_sentence<'a>(
    result: &SettingsSearchResult,
    paint: Paint<'a>,
) -> Option<Vec<Piece<'a, Message>>> {
    let home = result.home.as_ref().filter(|home| !home.say.text.is_empty())?;
    let own = home
        .say
        .tokens
        .iter()
        .position(|slot| matches!(slot, Slot::Value { row, .. } if *row == result.row));
    if own.is_none() && (home.master.is_none() || result.row > 0) {
        return None;
    }
    let rows: Vec<&Row> = home.rows().collect();
    let tokens = sections::say_tokens(home, &rows, paint)
        .into_iter()
        .enumerate()
        .map(
            |(at, pieces)| {
                if own == Some(at) { pieces } else { pieces.into_iter().map(worded).collect() }
            },
        )
        .collect();
    Some(calm::split(&home.say.text, tokens))
}

fn say<'a>(
    result: &'a SettingsSearchResult,
    plane: Color,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let Some(row) = result.live() else {
        return plain(result, plane, paint);
    };
    let own = own_sentence(result, paint);
    match &row.control {
        Control::Toggle { .. }
        | Control::ToggleAction { .. }
        | Control::ActionBtn { .. }
        | Control::ActionChips { .. } => own.map_or_else(
            || plain(result, plane, paint),
            |pieces| sections::spoken(pieces, plane, None, None, paint),
        ),
        Control::Colour { .. } => noted(sections::colours(&[row], None, paint), row, plane, paint),
        control => {
            let popover = sections::popover(row, TokenSide::Sentence, plane, paint);
            if let Some(pieces) = own {
                return sections::spoken(pieces, plane, popover, None, paint);
            }
            let pieces =
                sections::token_pieces(control, TokenSide::Sentence, sections::caption(row), paint);
            noted(sections::spoken(pieces, plane, popover, None, paint), row, plane, paint)
        }
    }
}

fn noted<'a>(
    value: Element<'a, Message>,
    row: &Row,
    plane: Color,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    if row.hint().is_empty() {
        return value;
    }
    column![value, quiet(row.hint(), plane, paint)].width(Length::Fill).align_x(start()).into()
}

fn action<'a>(
    result: &'a SettingsSearchResult,
    selected: bool,
    paint: Paint<'a>,
) -> Option<Element<'a, Message>> {
    let row = result.live()?;
    match &row.control {
        Control::Toggle { .. } | Control::ToggleAction { .. } => {
            Some(sections::toggle(row.control.clone(), selected, selected, paint))
        }
        Control::ActionBtn { id, label } => {
            Some(sections::action_button(id.clone(), label.clone(), selected, paint))
        }
        Control::ActionChips { items } if row.look.cell == Cell::Menu => {
            Some(sections::choose(row, items, false, paint))
        }
        Control::ActionChips { items } => {
            let buttons = items
                .iter()
                .map(|(id, label)| sections::action_button(id.clone(), label.clone(), false, paint))
                .collect();
            Some(sections::lay(buttons, true, paint))
        }
        _ => None,
    }
}

pub(super) fn field<'a>(query: &'a str, scale: f32, palette: &'a Palette) -> Element<'a, Message> {
    let (ink, accent) = (palette.surface_text, palette.primary);
    let fill = crate::frontend::components::with_alpha(Color::BLACK, FIELD.fill);
    let edge = tint(palette.background, ink, FIELD.rule);
    let placeholder = tint(palette.background, ink, FIELD.placeholder);
    let input = text_input(tr("settings-nav-search"), query)
        .id(settings_search_input_id())
        .font(UI_FONT)
        .on_input(|value| Message::Settings(SettingsMsg::SearchInput(value)))
        .on_submit(Message::Settings(SettingsMsg::Key(SettingsKey::Activate)))
        .size(FIELD.input * legible_type_scale(scale))
        .padding(0.0)
        .width(Length::Fill)
        .style(move |_, _| {
            let mut style = crate::frontend::ui::workbench_input_style(ink, accent);
            style.background = Color::TRANSPARENT.into();
            style.border.width = 0.0;
            style.placeholder = placeholder;
            style
        });
    let clear = crate::frontend::ui::folio_action(
        CLEAR_GLYPH,
        false,
        Some(Message::Settings(SettingsMsg::SearchClose)),
        Length::Fixed(FIELD.clear * scale),
        scale * FIELD.clear_scale,
        palette,
    );
    let line = row![label(SEARCH_GLYPH, FIELD.glyph, scale, accent), input, clear]
        .spacing(FIELD.gap * scale)
        .align_y(Alignment::Center);
    container(line)
        .padding(logical_padding(0.0, FIELD.pad_end * scale, 0.0, FIELD.pad_start * scale))
        .height(Length::Fixed(FIELD.height * scale))
        .center_y(Length::Fixed(FIELD.height * scale))
        .width(Length::Fill)
        .style(move |_| box_style(fill, edge))
        .into()
}

fn chip<'a>(caption: &'a str, scale: f32, palette: &Palette) -> Element<'a, Message> {
    let plane = palette.background;
    let edge = tint(plane, palette.outline, KEYS.chip_rule);
    let [vertical, horizontal] = KEYS.chip_pad.map(|px| px * scale);
    container(label(caption, KEYS.chip, scale, tint(plane, palette.surface_text, KEYS.chip_alpha)))
        .padding([vertical, horizontal])
        .style(move |_| box_style(Color::TRANSPARENT, edge))
        .into()
}

fn hint<'a>(caption: &'a str, scale: f32, palette: &Palette) -> Element<'a, Message> {
    let ink = tint(palette.background, palette.surface_text, KEYS.hint_alpha);
    label(caption, KEYS.hint, scale, ink).into()
}

pub(super) fn keys<'a>(scale: f32, palette: &Palette) -> Element<'a, Message> {
    let line = |parts: Vec<Element<'a, Message>>| {
        Element::from(
            crate::frontend::ui::Flow::new(parts)
                .spacing(KEYS.word_gap * scale)
                .align_y(Alignment::Center)
                .wrap(),
        )
    };
    let word = |key: &'static str| hint(tr(key), scale, palette);
    let key = |key: &'static str| chip(tr(key), scale, palette);
    column![
        line(vec![chip(KEY_DOWN, scale, palette), word(FROM_FIELD)]),
        line(vec![word(IN_RESULTS), key(KEY_SPACE), word(TOGGLES), key(KEY_ENTER), word(OPENS)]),
        line(vec![key(KEY_ESC), word(CLEARS)]),
    ]
    .spacing(KEYS.gap * scale)
    .into()
}
