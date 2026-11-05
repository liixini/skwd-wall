use iced::widget::{canvas, column, container, mouse_area, stack, text, text_input};
use iced::{Alignment, Element, Length, Padding};

use crate::app::{SearchMode, tag_query_id};
use crate::domain::library::search::{QueryChip, TagEntry, suggest_completion};
use crate::frontend::theme::Palette;
use crate::frontend::ui::{
    NERD_FONT, TagChips, control_background, control_border, control_text, logical_padding, row,
    with_alpha,
};
use crate::i18n::tr;

pub const MAX_VISIBLE: usize = 400;

#[derive(Debug, Clone)]
pub enum TagMsg {
    InputChanged(String),
    Submit,
    Remove(usize),
    ToggleCardDrawer,
    CloseCardDrawer,
    QueryInput(String),
    SearchMode(SearchMode),
    CycleSearchMode,
    CloudClick(String, bool),
    CloudScroll(f32),
    Autocomplete,
    MatchMode(bool),
    SortAz(bool),
    ToggleMatchingTags,
}

#[derive(Debug, Clone)]
pub enum TagIntent {
    Update(TagMsg),
    Clear,
    ToggleCloud,
}

use self::TagIntent as Message;

pub fn ghost_text(search_text: &str, entries: &[TagEntry], partial: &str) -> String {
    if partial.is_empty() {
        return String::new();
    }
    let suggest = suggest_completion(entries, partial);
    if suggest.chars().count() <= partial.chars().count() {
        return String::new();
    }
    let start = search_text
        .char_indices()
        .map(|(idx, _)| idx)
        .rev()
        .find(|&idx| search_text[idx..].to_lowercase().starts_with(partial));
    match start {
        Some(idx) => format!("{}{}", &search_text[..idx], suggest),
        None => String::new(),
    }
}

pub struct CloudView<'a> {
    pub entries: std::rc::Rc<Vec<TagEntry>>,
    pub db_empty: bool,
    pub selected: &'a [String],
    pub query_chips: Vec<QueryChip>,
    pub match_any: bool,
    pub search_mode: SearchMode,
    pub semantic_model: String,
    pub semantic_enabled: bool,
    pub search_text: &'a str,
    pub partial: &'a str,
    pub semantic_pending: bool,
    pub semantic_error: Option<&'a str>,
    pub semantic_ms: f64,
    pub filtered_count: usize,
    pub sort_az: bool,
    pub matching_tags_open: bool,
    pub entrance: f32,
    pub scroll: f32,
    pub scroll_target: f32,
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    pub pal: &'a Palette,
}

pub fn view(cloud: CloudView<'_>) -> Element<'_, Message> {
    let CloudView {
        entries,
        db_empty,
        selected,
        query_chips,
        match_any,
        search_mode,
        semantic_model,
        semantic_enabled,
        search_text,
        partial,
        semantic_pending,
        semantic_error,
        semantic_ms,
        filtered_count,
        sort_az,
        matching_tags_open,
        entrance,
        scroll,
        scroll_target,
        width,
        height,
        scale,
        pal,
    } = cloud;
    let inner_w = (width - 2.0 * (crate::frontend::ui::PANEL_SKEW + 10.0) * scale).max(120.0);

    let entries = if sort_az {
        let mut sorted = (*entries).clone();
        sort_entries(&mut sorted);
        std::rc::Rc::new(sorted)
    } else {
        entries
    };

    let sel_count = selected.iter().filter(|sel| !sel.is_empty()).count() + query_chips.len();
    let result_label = crate::i18n::tags_wall_count(filtered_count);
    let semantic_label = if search_mode == SearchMode::Tags {
        String::new()
    } else if search_text.trim().is_empty() {
        tr("tags-semantic-ready").to_string()
    } else if semantic_pending {
        tr("tags-semantic-searching").to_string()
    } else if semantic_error.is_some() {
        tr("tags-semantic-unavailable").to_string()
    } else {
        crate::i18n::tr_args!("tags-semantic-ms", ms => format!("{semantic_ms:.1}"))
    };
    let semantic_color = if semantic_error.is_some() {
        pal.destructive()
    } else if semantic_pending {
        pal.primary
    } else {
        with_alpha(pal.surface_text, 0.62)
    };
    let mut toolbar = row![
        text(tr("tags-filter-title"))
            .font(iced::Font::MONOSPACE)
            .size(10.5 * scale)
            .color(with_alpha(pal.surface_text, 0.82)),
        text(result_label)
            .font(iced::Font::MONOSPACE)
            .size(10.5 * scale)
            .color(with_alpha(pal.surface_text, 0.78)),
        text(semantic_label).font(iced::Font::MONOSPACE).size(10.5 * scale).color(semantic_color),
        iced::widget::Space::new().width(Length::Fill),
        text(tr("tags-match"))
            .font(crate::frontend::ui::UI_FONT)
            .size(9.0 * scale)
            .color(with_alpha(pal.surface_text, 0.62)),
        tag_control_chip(
            tr("tags-match-all"),
            !match_any,
            false,
            Message::Update(TagMsg::MatchMode(false)),
            scale,
            pal,
        ),
        tag_control_chip(
            tr("tags-match-any"),
            match_any,
            false,
            Message::Update(TagMsg::MatchMode(true)),
            scale,
            pal,
        ),
        text(tr("tags-order"))
            .font(crate::frontend::ui::UI_FONT)
            .size(9.0 * scale)
            .color(with_alpha(pal.surface_text, 0.62)),
        tag_control_chip(
            tr("tags-order-popular"),
            !sort_az,
            false,
            Message::Update(TagMsg::SortAz(false)),
            scale,
            pal,
        ),
        tag_control_chip(
            tr("tags-order-alphabetical"),
            sort_az,
            false,
            Message::Update(TagMsg::SortAz(true)),
            scale,
            pal,
        ),
    ]
    .spacing(7.0 * scale)
    .align_y(Alignment::Center);
    if sel_count > 0 {
        let clear_label = crate::i18n::tr_args!("tags-clear-count", count => sel_count);
        toolbar =
            toolbar.push(tag_control_chip(clear_label, false, true, Message::Clear, scale, pal));
    }
    toolbar = toolbar
        .push(tag_control_chip(
            if matching_tags_open { tr("tags-toggle-close") } else { tr("tags-toggle-open") },
            matching_tags_open,
            false,
            Message::Update(TagMsg::ToggleMatchingTags),
            scale,
            pal,
        ))
        .push(
            mouse_area(
                text(tr("tags-close-shortcut"))
                    .font(iced::Font::MONOSPACE)
                    .size(10.0 * scale)
                    .color(with_alpha(pal.surface_text, 0.68)),
            )
            .on_press(Message::ToggleCloud),
        );

    let icon = text("\u{f0349}")
        .font(NERD_FONT)
        .size(16.0 * scale)
        .color(with_alpha(pal.surface_text, 0.82));

    let ghost_str = if search_mode == SearchMode::Tags {
        ghost_text(search_text, &entries, partial)
    } else {
        String::new()
    };

    let placeholder = if search_mode == SearchMode::Tags {
        tr("tags-search-tags-placeholder")
    } else {
        tr("tags-search-describe-placeholder")
    };
    let input = text_input(placeholder, search_text)
        .id(tag_query_id())
        .on_input(|value| Message::Update(TagMsg::QueryInput(value)))
        .on_submit(Message::Update(TagMsg::Autocomplete))
        .padding(Padding::from([7.0, 10.0]))
        .size(13.0 * scale)
        .width(Length::Fill)
        .style(move |_t, _s| text_input::Style {
            background: iced::Color::TRANSPARENT.into(),
            border: iced::Border { radius: 0.0.into(), ..Default::default() },
            icon: pal.surface_text,
            placeholder: with_alpha(pal.surface_text, 0.52),
            value: pal.surface_text,
            selection: with_alpha(pal.primary, 0.4),
        });
    let field_bg = with_alpha(pal.surface_variant, 0.94);
    let field_border = with_alpha(pal.primary, 0.5);
    let field_h = 46.0 * scale;
    let input_stack = stack![
        container(
            text(ghost_str)
                .font(iced::Font::MONOSPACE)
                .size(13.0 * scale)
                .color(with_alpha(pal.surface_text, 0.3)),
        )
        .padding(Padding { left: 10.0 * scale, ..Padding::ZERO })
        .center_y(Length::Fixed(field_h)),
        container(input).center_y(Length::Fixed(field_h)),
    ]
    .width(Length::Fill)
    .height(Length::Fixed(field_h));
    let mut search_controls = row![tag_control_chip(
        tr("tags-search-mode-tags"),
        search_mode == SearchMode::Tags,
        false,
        Message::Update(TagMsg::SearchMode(SearchMode::Tags)),
        scale,
        pal,
    ),]
    .spacing(7.0 * scale)
    .align_y(Alignment::Center);
    if semantic_enabled {
        search_controls = search_controls.push(tag_control_chip(
            tr("tags-search-mode-describe"),
            search_mode == SearchMode::Describe,
            false,
            Message::Update(TagMsg::SearchMode(SearchMode::Describe)),
            scale,
            pal,
        ));
    }
    if search_mode == SearchMode::Describe {
        search_controls = search_controls.push(
            text(semantic_model)
                .font(iced::Font::MONOSPACE)
                .size(9.5 * scale)
                .color(with_alpha(pal.surface_text, 0.62)),
        );
    }
    search_controls = search_controls.push(icon).push(input_stack);
    let search_row = container(search_controls)
        .width(Length::Fill)
        .height(Length::Fixed(field_h))
        .padding(logical_padding(0.0, 5.0 * scale, 0.0, 12.0 * scale))
        .style(move |_theme| crate::frontend::ui::box_style(field_bg, field_border));
    let mut sections: Vec<Element<'_, Message>> = vec![toolbar.into(), search_row.into()];
    if search_mode == SearchMode::Tags {
        let help = if query_chips.is_empty() {
            tr("tags-search-range-help")
        } else {
            tr("tags-search-predicates")
        };
        let mut query_help = row![
            text(help)
                .font(iced::Font::MONOSPACE)
                .size(8.5 * scale)
                .color(with_alpha(pal.surface_text, 0.62)),
            iced::widget::Space::new().width(Length::Fill),
        ]
        .spacing(6.0 * scale)
        .align_y(Alignment::Center);
        for chip in query_chips.iter().take(2) {
            query_help = query_help.push(query_chip(chip, scale, pal));
        }
        if query_chips.len() > 2 {
            query_help = query_help.push(
                text(format!("+{}", query_chips.len() - 2))
                    .font(iced::Font::MONOSPACE)
                    .size(8.5 * scale)
                    .color(with_alpha(pal.surface_text, 0.68)),
            );
        }
        sections.push(query_help.into());
    }
    if matching_tags_open {
        let chrome_h = 134.0;
        let chips_max_h = (height - chrome_h * scale).max(27.0 * scale);
        let body_inner = chips_body(
            TagChips {
                entries,
                pal,
                width: inner_w,
                max_h: chips_max_h,
                scale,
                entrance,
                scroll,
                scroll_target,
            },
            db_empty,
        );
        sections.push(
            container(body_inner)
                .width(Length::Fill)
                .height(Length::Fixed(chips_max_h))
                .align_y(Alignment::Start)
                .into(),
        );
    }

    let h_pad = 18.0 * scale;
    let content = container(column(sections).spacing(8.0 * scale))
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .padding(Padding { top: 10.0 * scale, bottom: 10.0 * scale, left: h_pad, right: h_pad })
        .style(move |_theme| {
            crate::frontend::ui::box_style(
                with_alpha(pal.surface, 0.97),
                with_alpha(pal.primary, 0.48),
            )
        });
    content.into()
}

fn query_chip<'a>(chip: &QueryChip, scale: f32, pal: &'a Palette) -> Element<'a, Message> {
    let foreground = if chip.invalid { pal.destructive() } else { pal.surface_text };
    let border = if chip.invalid {
        with_alpha(pal.destructive(), 0.72)
    } else {
        with_alpha(pal.primary, 0.42)
    };
    container(
        text(chip.label.clone()).font(iced::Font::MONOSPACE).size(8.5 * scale).color(foreground),
    )
    .padding([2.0 * scale, 6.0 * scale])
    .style(move |_theme| {
        crate::frontend::ui::box_style(with_alpha(pal.surface_variant, 0.7), border)
    })
    .into()
}

fn tag_control_chip<'a>(
    label: impl Into<String>,
    active: bool,
    destructive: bool,
    message: Message,
    scale: f32,
    pal: &Palette,
) -> Element<'a, Message> {
    let fill = if destructive {
        with_alpha(pal.destructive(), 0.16)
    } else {
        control_background(pal, active, false)
    };
    let foreground = if destructive { pal.destructive() } else { control_text(pal, active, false) };
    let border = if destructive {
        with_alpha(pal.destructive(), 0.72)
    } else {
        control_border(pal, active, false)
    };
    mouse_area(
        container(
            text(label.into()).font(iced::Font::MONOSPACE).size(9.5 * scale).color(foreground),
        )
        .height(Length::Fixed(26.0 * scale.max(1.0)))
        .padding([4.0 * scale, 10.0 * scale])
        .center_y(Length::Fixed(26.0 * scale.max(1.0)))
        .style(move |_theme| crate::frontend::ui::box_style(fill, border)),
    )
    .on_press(message)
    .into()
}

fn sort_entries(entries: &mut [TagEntry]) {
    entries.sort_by(|lhs, rhs| {
        let lhs_pin = lhs.selected || lhs.excluded;
        let rhs_pin = rhs.selected || rhs.excluded;
        if lhs_pin != rhs_pin {
            return if lhs_pin { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater };
        }
        lhs.tag.cmp(&rhs.tag)
    });
}

fn chips_body(chips: TagChips<'_>, db_empty: bool) -> Element<'_, Message> {
    let scale = chips.scale;
    if chips.entries.is_empty() {
        return container(
            text(if db_empty { tr("tags-empty") } else { tr("tags-no-matches") })
                .font(iced::Font::MONOSPACE)
                .size(13.0 * scale)
                .color(with_alpha(chips.pal.surface_text, 0.72)),
        )
        .padding(Padding::from([8.0, 2.0]))
        .into();
    }
    let (w, h) = (chips.width, chips.max_h);
    canvas(chips).width(Length::Fixed(w)).height(Length::Fixed(h)).into()
}
