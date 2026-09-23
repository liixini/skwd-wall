use iced::widget::{button, column, container, image, scrollable, text};
use iced::{Alignment, Background, ContentFit, Element, Length, Padding};

use crate::app::Message;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{UI_FONT, folio_rule, label, logical_padding, row, with_alpha};
use crate::i18n::{tr, tr_args};

use super::filter::source_colors;
use super::model::{PlMsg, Playlists};

fn playlist_index<'a>(
    playlists: &'a Playlists,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    let mut items = column![].spacing(4.0 * scale);
    for playlist in &playlists.lists {
        let id = playlist.id;
        let selected = playlists.selected == Some(id);
        let outputs = playlists.active_outputs(id);
        let playing = !outputs.is_empty();
        let name = if playlist.name.is_empty() {
            tr("playlists-unnamed").to_string()
        } else {
            playlist.name.clone()
        };
        let kind = if playlist.kind.is_smart() {
            tr("playlists-type-smart")
        } else {
            tr("playlists-type-curated")
        };
        let detail = if playing {
            tr_args!("playlists-index-live-detail", kind => kind, outputs => outputs.join(", "))
        } else {
            crate::i18n::playlists_count_detail(kind, playlist.count)
        };
        let content = row![
            label(
                tr_args!("playlists-id", id => id.to_string()),
                9.0,
                scale,
                if selected || playing {
                    palette.primary
                } else {
                    with_alpha(palette.primary, 0.46)
                },
            ),
            column![
                label(name, 12.0, scale, palette.surface_text),
                label(
                    detail,
                    8.5,
                    scale,
                    if playing {
                        with_alpha(palette.primary, 0.86)
                    } else {
                        with_alpha(palette.surface_text, 0.44)
                    },
                ),
            ]
            .spacing(3.0 * scale)
            .width(Length::Fill),
            label(
                if playing {
                    "●"
                } else if selected {
                    "◆"
                } else {
                    "◇"
                },
                9.0,
                scale,
                if selected || playing {
                    palette.primary
                } else {
                    with_alpha(palette.surface_text, 0.32)
                },
            ),
        ]
        .spacing(9.0 * scale)
        .align_y(Alignment::Center);
        items = items.push(
            button(content)
                .width(Length::Fill)
                .padding([9.0 * scale, 10.0 * scale])
                .on_press(Message::Pl(PlMsg::Select(id)))
                .style(move |_theme, status| {
                    crate::frontend::ui::folio_line_button_style(
                        selected, false, palette, 1.0, status,
                    )
                }),
        );
    }
    if playlists.lists.is_empty() {
        items = items.push(
            container(
                column![
                    label(tr("playlists-index-empty-title"), 13.0, scale, palette.surface_text),
                    label(
                        tr("playlists-index-empty-hint"),
                        9.5,
                        scale,
                        with_alpha(palette.surface_text, 0.46),
                    ),
                ]
                .spacing(5.0 * scale),
            )
            .width(Length::Fill)
            .padding(14.0 * scale),
        );
    }

    let create = column![
        folio_rule(palette),
        row![
            label(tr("playlists-new-playlist"), 9.0, scale, palette.primary),
            container(text("")).width(Length::Fill),
            label(
                format!("{:02}", playlists.lists.len()),
                9.0,
                scale,
                with_alpha(palette.surface_text, 0.42),
            ),
        ]
        .align_y(Alignment::Center),
        row![
            crate::frontend::ui::folio_ghost_field(
                &playlists.new_buf,
                tr("playlists-new-name"),
                |value| Message::Pl(PlMsg::NewInput(value)),
                Message::Pl(PlMsg::NewSubmit),
                Length::Fill,
                scale,
                palette,
            ),
            crate::frontend::ui::folio_action(
                "+",
                !playlists.new_buf.trim().is_empty(),
                (!playlists.new_buf.trim().is_empty()).then_some(Message::Pl(PlMsg::NewSubmit)),
                Length::Fixed(42.0 * scale),
                scale,
                palette,
            ),
        ]
        .spacing(6.0 * scale)
        .align_y(Alignment::Center),
    ]
    .spacing(9.0 * scale);

    let footer = if playlists.any_active() {
        column![
            folio_rule(palette),
            label(tr("playlists-live-routing"), 9.0, scale, palette.primary),
            label(
                crate::i18n::playlists_active_assignments(playlists.assign.len()),
                9.5,
                scale,
                with_alpha(palette.surface_text, 0.48),
            ),
            crate::frontend::ui::folio_destructive_action(
                tr("playlists-stop-all"),
                false,
                Some(Message::Pl(PlMsg::Stop(0))),
                Length::Fill,
                scale * 0.9,
                palette,
            ),
        ]
        .spacing(7.0 * scale)
    } else {
        column![
            folio_rule(palette),
            label(tr("playlists-library"), 9.0, scale, palette.primary),
            label(
                crate::i18n::playlists_library_stats(playlists.lists.len()),
                9.5,
                scale,
                with_alpha(palette.surface_text, 0.48),
            ),
        ]
        .spacing(7.0 * scale)
    };

    crate::frontend::ui::folio_index_shell(
        tr("playlists-index-title"),
        tr("playlists-index-subtitle"),
        vec![
            scrollable(items)
                .id(iced::widget::Id::new("folio-playlist-index"))
                .direction(iced::widget::scrollable::Direction::Vertical(
                    iced::widget::scrollable::Scrollbar::new().width(3.0).scroller_width(3.0),
                ))
                .style(crate::frontend::ui::scroll_style(palette.primary))
                .height(Length::Fill)
                .into(),
            create.into(),
            footer.into(),
        ],
        14.0,
        scale,
        palette,
    )
}

fn colour_picker<'a>(
    id: i64,
    selected: &[i64],
    available: f32,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    let gap = 3.0 * scale;
    let width = ((available - gap * 12.0) / 13.0).clamp(17.0 * scale, 38.0 * scale);
    let mut swatches = row![].spacing(gap).align_y(Alignment::Center);
    for index in 0..13 {
        let bucket = crate::frontend::ui::strip_bucket(index);
        let active = selected.contains(&bucket);
        let colour = crate::frontend::ui::swatch_color(index, active);
        swatches = swatches.push(
            button(text(""))
                .width(Length::Fixed(width))
                .height(Length::Fixed(21.0 * scale.max(1.0)))
                .padding(0)
                .on_press(Message::Pl(PlMsg::ColorPick(id, bucket)))
                .style(move |_theme, status| {
                    crate::frontend::ui::flat_button_style(
                        colour,
                        palette.surface_text,
                        with_alpha(
                            if active { palette.surface_text } else { palette.outline },
                            if active { 0.9 } else { 0.42 },
                        ),
                        1.0,
                        status,
                    )
                }),
        );
    }
    swatches.into()
}

fn wallpaper_type_mark(kind: &str) -> &'static str {
    match kind {
        "video" => "▶",
        "we" | "scene" => "◆",
        "static" | "image" => "▧",
        _ => "◇",
    }
}

fn sequence_rail<'a>(
    playlists: &'a Playlists,
    id: i64,
    smart: bool,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    let heading = column![
        row![
            label(
                if smart { tr("playlists-live-results") } else { tr("playlists-the-sequence") },
                9.0,
                scale,
                palette.primary
            ),
            container(text("")).width(Length::Fill),
            label(
                format!("{:02}", playlists.members.len()),
                9.0,
                scale,
                with_alpha(palette.surface_text, 0.44),
            ),
        ]
        .align_y(Alignment::Center),
        label(
            if smart {
                tr("playlists-sequence-smart-hint")
            } else {
                tr("playlists-sequence-curated-hint")
            },
            9.5,
            scale,
            with_alpha(palette.surface_text, 0.48),
        ),
        folio_rule(palette),
    ]
    .spacing(8.0 * scale);

    let list: Element<'a, Message> = if playlists.members.is_empty() {
        container(
            column![
                label(
                    if smart { tr("playlists-empty-smart") } else { tr("playlists-empty-curated") },
                    12.0,
                    scale,
                    palette.surface_text,
                ),
                label(
                    if smart {
                        tr("playlists-empty-smart-hint")
                    } else {
                        tr("playlists-empty-curated-hint")
                    },
                    9.0,
                    scale,
                    with_alpha(palette.surface_text, 0.46),
                )
                .line_height(iced::widget::text::LineHeight::Relative(1.4)),
            ]
            .spacing(6.0 * scale),
        )
        .height(Length::Fill)
        .align_y(iced::alignment::Vertical::Center)
        .into()
    } else {
        let total = playlists.members.len();
        let mut rows = column![].spacing(0.0);
        for (index, wallpaper) in playlists.members.iter().enumerate() {
            let key = wallpaper.key.clone().unwrap_or_default();
            let kind = wallpaper.kind.as_deref().unwrap_or("wall");
            let preview = wallpaper
                .thumb_sm
                .as_deref()
                .filter(|path| !path.is_empty())
                .or_else(|| wallpaper.thumb.as_deref().filter(|path| !path.is_empty()))
                .or_else(|| wallpaper.preview.as_deref().filter(|path| !path.is_empty()));
            let artwork: Element<'a, Message> = if let Some(path) = preview {
                image(image::Handle::from_path(path))
                    .content_fit(ContentFit::Cover)
                    .width(Length::Fill)
                    .height(Length::Fixed(158.0 * scale))
                    .into()
            } else {
                container(
                    text(kind.chars().next().unwrap_or('W').to_uppercase().to_string())
                        .font(UI_FONT)
                        .size(18.0 * scale)
                        .color(with_alpha(palette.surface_text, 0.4)),
                )
                .width(Length::Fill)
                .height(Length::Fixed(158.0 * scale))
                .center(Length::Fill)
                .style(move |_| {
                    crate::frontend::ui::box_style(
                        with_alpha(palette.background, 0.72),
                        with_alpha(palette.outline, 0.4),
                    )
                })
                .into()
            };
            let controls: Element<'a, Message> = if smart {
                label(tr("playlists-live-badge"), 8.0, scale, with_alpha(palette.primary, 0.72))
                    .into()
            } else {
                column![
                    crate::frontend::ui::folio_action(
                        "\u{2c4}",
                        false,
                        (index > 0).then_some(Message::Pl(PlMsg::MoveMember(id, key.clone(), -1,))),
                        Length::Fixed(26.0 * scale),
                        scale * 0.78,
                        palette,
                    ),
                    crate::frontend::ui::folio_action(
                        "\u{2c5}",
                        false,
                        (index + 1 < total).then_some(Message::Pl(PlMsg::MoveMember(
                            id,
                            key.clone(),
                            1,
                        ))),
                        Length::Fixed(26.0 * scale),
                        scale * 0.78,
                        palette,
                    ),
                    crate::frontend::ui::folio_destructive_action(
                        "×",
                        false,
                        Some(Message::Pl(PlMsg::RemoveMember(id, key))),
                        Length::Fixed(26.0 * scale),
                        scale * 0.74,
                        palette,
                    ),
                ]
                .spacing(4.0 * scale)
                .align_x(Alignment::Center)
                .into()
            };
            let spine = column![
                label(format!("{:02}", index + 1), 14.0, scale, palette.primary),
                label(
                    wallpaper_type_mark(kind),
                    11.0,
                    scale,
                    with_alpha(palette.surface_text, 0.54),
                ),
                container(text("")).height(Length::Fill),
                controls,
            ]
            .spacing(5.0 * scale)
            .width(Length::Fixed(26.0 * scale))
            .height(Length::Fixed(158.0 * scale))
            .align_x(Alignment::Center);
            rows = rows.push(
                column![
                    container(row![spine, artwork].spacing(8.0 * scale).align_y(Alignment::Start))
                        .width(Length::Fill)
                        .padding([10.0 * scale, 0.0]),
                    folio_rule(palette),
                ]
                .spacing(0.0),
            );
        }
        scrollable(rows)
            .id(crate::frontend::ui::pane_id("folio.playlist.members"))
            .direction(iced::widget::scrollable::Direction::Vertical(
                iced::widget::scrollable::Scrollbar::new()
                    .width(2.0)
                    .scroller_width(2.0)
                    .margin(3.0)
                    .spacing(7.0),
            ))
            .style(crate::frontend::ui::scroll_style(palette.primary))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };
    container(column![heading, list].spacing(12.0 * scale))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding {
            top: 30.0 * scale,
            right: 14.0 * scale,
            bottom: 24.0 * scale,
            left: 14.0 * scale,
        })
        .style(move |_| {
            crate::frontend::ui::bg_style(Background::Color(with_alpha(
                palette.surface_container,
                0.14,
            )))
        })
        .into()
}

fn empty_reading_surface(scale: f32, palette: &Palette) -> Element<'_, Message> {
    container(
        column![
            label(tr("playlists-empty-eyebrow"), 9.0, scale, with_alpha(palette.primary, 0.84)),
            text(tr("playlists-empty-title"))
                .font(UI_FONT)
                .size(34.0 * scale.clamp(0.88, 1.08))
                .color(palette.surface_text),
            folio_rule(palette),
            container(
                column![
                    label(tr("playlists-empty-choose"), 19.0, scale, palette.surface_text),
                    label(
                        tr("playlists-empty-desc"),
                        10.5,
                        scale,
                        with_alpha(palette.surface_text, 0.5),
                    ),
                ]
                .spacing(7.0 * scale),
            )
            .height(Length::Fill)
            .align_y(iced::alignment::Vertical::Center),
        ]
        .spacing(10.0 * scale),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(crate::frontend::ui::folio_scroll_padding(23.0, 27.0, scale))
    .into()
}

fn reading_surface<'a>(
    playlists: &'a Playlists,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    let Some(definition) = playlists.selected_def() else {
        return empty_reading_surface(scale, palette);
    };
    let id = definition.id;
    let smart = definition.kind.is_smart();
    let kind = if smart { tr("playlists-kind-smart") } else { tr("playlists-kind-curated") };
    let playing_on = playlists.active_outputs(id);
    let playing = !playing_on.is_empty();
    let display_name =
        if definition.name.is_empty() { tr("playlists-unnamed") } else { definition.name.as_str() };

    let state_copy = if playing {
        tr_args!("playlists-state-live", outputs => playing_on.join("  +  "))
    } else if smart && playlists.members.is_empty() {
        tr("playlists-state-query-ready").into()
    } else {
        crate::i18n::playlists_state_ready(playlists.members.len())
    };
    let header = column![
        row![
            label(tr("playlists-edition-eyebrow"), 9.0, scale, with_alpha(palette.primary, 0.84)),
            label(
                tr_args!("playlists-id", id => id.to_string()),
                9.0,
                scale,
                with_alpha(palette.primary, 0.84)
            ),
            container(text("")).width(Length::Fill),
            label(
                state_copy,
                9.0,
                scale,
                if playing { palette.primary } else { with_alpha(palette.surface_text, 0.46) },
            ),
        ]
        .spacing(10.0 * scale)
        .align_y(Alignment::Center),
        text(display_name)
            .font(UI_FONT)
            .size(42.0 * scale.clamp(0.88, 1.08))
            .line_height(iced::widget::text::LineHeight::Relative(0.98))
            .color(palette.surface_text),
        row![
            label(
                tr_args!("playlists-kind-tagline", kind => kind),
                10.5,
                scale,
                with_alpha(palette.surface_text, 0.5),
            ),
            container(text("")).width(Length::Fill),
            crate::frontend::ui::folio_action(
                if playing { tr("playlists-stop-edition") } else { tr("playlists-play-now") },
                true,
                Some(Message::Pl(if playing { PlMsg::Stop(id) } else { PlMsg::PlayNow(id) })),
                Length::Fixed(116.0 * scale),
                scale,
                palette,
            ),
            crate::frontend::ui::folio_destructive_action(
                tr("playlists-delete"),
                false,
                Some(Message::Pl(PlMsg::Delete(id))),
                Length::Fixed(68.0 * scale),
                scale,
                palette,
            ),
        ]
        .spacing(7.0 * scale)
        .align_y(Alignment::Center),
    ]
    .spacing(10.0 * scale);

    let source = definition.source.clone().unwrap_or_default();
    let feature: Option<Element<'a, Message>> = if smart {
        let selected_colours = source_colors(&playlists.source_buf);
        let filter = column![
            row![
                crate::frontend::ui::folio_action_wrap(
                    vec![
                        (
                            tr("playlists-all").into(),
                            source == "all",
                            Message::Pl(PlMsg::SetProp(id, "source".into(), "all".into())),
                        ),
                        (
                            tr("playlists-favourites").into(),
                            source == "favourites",
                            Message::Pl(PlMsg::SetProp(id, "source".into(), "favourites".into(),)),
                        ),
                    ],
                    260.0 * scale,
                    scale,
                    palette,
                ),
                container(text("")).width(Length::Fixed(14.0 * scale)),
                crate::frontend::ui::folio_ghost_field(
                    &playlists.source_buf,
                    tr("playlists-filter-example"),
                    |value| Message::Pl(PlMsg::EditSource(value)),
                    Message::Pl(PlMsg::SetProp(id, "source".into(), playlists.source_buf.clone(),)),
                    Length::Fill,
                    scale,
                    palette,
                ),
                crate::frontend::ui::folio_action(
                    tr("playlists-filter-help"),
                    false,
                    Some(Message::Pl(PlMsg::FilterHelp(true))),
                    Length::Fixed(crate::frontend::ui::folio_action_width(
                        tr("playlists-filter-help"),
                        scale
                    )),
                    scale,
                    palette,
                ),
            ]
            .align_y(Alignment::Center),
            row![
                label(
                    tr("playlists-colour-notes"),
                    8.5,
                    scale,
                    with_alpha(palette.surface_text, 0.44)
                ),
                colour_picker(id, &selected_colours, 420.0 * scale, scale, palette),
            ]
            .spacing(13.0 * scale)
            .align_y(Alignment::Center),
        ]
        .spacing(14.0 * scale);
        Some(crate::frontend::ui::folio_field(
            "01",
            tr("playlists-live-collection-title"),
            tr("playlists-live-collection-desc"),
            filter.into(),
            scale,
            palette,
        ))
    } else {
        None
    };

    let definition_field = crate::frontend::ui::folio_field(
        if smart { "02" } else { "01" },
        tr("playlists-definition-title"),
        tr("playlists-definition-desc"),
        column![
            crate::frontend::ui::folio_ghost_field(
                &playlists.name_buf,
                tr("playlists-name"),
                |value| Message::Pl(PlMsg::EditName(value)),
                Message::Pl(PlMsg::SetProp(id, "name".into(), playlists.name_buf.clone())),
                Length::Fill,
                scale,
                palette,
            ),
            crate::frontend::ui::folio_action_wrap(
                vec![
                    (
                        tr("playlists-type-curated").into(),
                        !smart,
                        Message::Pl(PlMsg::SetProp(id, "kind".into(), "curated".into())),
                    ),
                    (
                        tr("playlists-type-smart").into(),
                        smart,
                        Message::Pl(PlMsg::SetProp(id, "kind".into(), "smart".into())),
                    ),
                ],
                360.0 * scale,
                scale,
                palette,
            ),
        ]
        .spacing(11.0 * scale)
        .into(),
        scale,
        palette,
    );
    let order = definition.order.as_str();
    let playback_body = column![
        crate::frontend::ui::folio_action_wrap(
            vec![
                (
                    tr("playlists-order-shuffle").into(),
                    order != "sequential",
                    Message::Pl(PlMsg::SetProp(id, "order".into(), "shuffle".into())),
                ),
                (
                    tr("playlists-order-sequential").into(),
                    order == "sequential",
                    Message::Pl(PlMsg::SetProp(id, "order".into(), "sequential".into())),
                ),
            ],
            280.0 * scale,
            scale,
            palette,
        ),
        row![
            label(
                tr("playlists-every"),
                crate::frontend::ui::TYPE_SMALL,
                scale,
                with_alpha(palette.surface_text, 0.48)
            ),
            crate::frontend::ui::folio_ghost_field(
                &playlists.dwell_buf,
                "600",
                |value| Message::Pl(PlMsg::EditDwell(value)),
                Message::Pl(PlMsg::SetProp(id, "dwell".into(), playlists.dwell_buf.clone())),
                Length::Fixed(72.0 * scale),
                scale,
                palette,
            ),
            label(
                tr("playlists-seconds-minimum"),
                8.5,
                scale,
                with_alpha(palette.surface_text, 0.42),
            ),
        ]
        .spacing(6.0 * scale)
        .align_y(Alignment::Center),
    ]
    .spacing(11.0 * scale);
    let playback = crate::frontend::ui::folio_field(
        if smart { "03" } else { "02" },
        tr("playlists-playback-title"),
        tr("playlists-playback-desc"),
        playback_body.into(),
        scale,
        palette,
    );
    let divider =
        container(text("")).width(Length::Fixed(1.0)).height(Length::Fill).style(move |_| {
            crate::frontend::ui::bg_style(Background::Color(with_alpha(palette.outline, 0.34)))
        });
    let settings = row![
        container(definition_field).width(Length::FillPortion(1)),
        divider,
        container(playback).width(Length::FillPortion(1)),
    ]
    .spacing(20.0 * scale)
    .align_y(Alignment::Start);

    let all_outputs = playlists.assigned_id("*") == Some(id);
    let mut assignments =
        vec![(tr("playlists-all").into(), all_outputs, Message::Pl(PlMsg::ToggleAll(id)))];
    for output in &playlists.outputs {
        assignments.push((
            output.clone(),
            all_outputs || playlists.assigned_id(output) == Some(id),
            Message::Pl(PlMsg::ToggleOutput(output.clone(), id)),
        ));
    }
    let routing_state = if all_outputs {
        tr("playlists-routing-all")
    } else if playing {
        tr("playlists-routing-live")
    } else {
        tr("playlists-routing-none")
    };
    let routing = crate::frontend::ui::folio_field(
        if smart { "04" } else { "03" },
        tr("playlists-routing-title"),
        tr("playlists-routing-desc"),
        row![
            crate::frontend::ui::folio_action_wrap(assignments, 500.0 * scale, scale, palette),
            container(text("")).width(Length::Fill),
            label(
                routing_state,
                9.0,
                scale,
                if playing { palette.primary } else { with_alpha(palette.surface_text, 0.44) },
            ),
        ]
        .align_y(Alignment::Center)
        .into(),
        scale,
        palette,
    );

    let mut page = column![header].spacing(28.0 * scale);
    if let Some(feature) = feature {
        page = page.push(feature);
    }
    page = page.push(settings).push(routing);
    let editor = scrollable(container(page).width(Length::Fill).padding(logical_padding(
        30.0 * scale,
        30.0 * scale,
        44.0 * scale,
        34.0 * scale,
    )))
    .id(iced::widget::Id::new("folio-playlist-editor"))
    .direction(iced::widget::scrollable::Direction::Vertical(
        iced::widget::scrollable::Scrollbar::hidden(),
    ))
    .width(Length::Fill)
    .height(Length::Fill);
    let rail_divider =
        container(text("")).width(Length::Fixed(1.0)).height(Length::Fill).style(move |_| {
            crate::frontend::ui::bg_style(Background::Color(with_alpha(palette.outline, 0.42)))
        });
    row![
        container(editor).width(Length::Fill).height(Length::Fill),
        rail_divider,
        container(sequence_rail(playlists, id, smart, scale, palette))
            .width(Length::Fixed(380.0 * scale.max(0.9)))
            .height(Length::Fill),
    ]
    .spacing(0.0)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

pub fn view<'a>(
    playlists: &'a Playlists,
    viewport: (f32, f32),
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    if playlists.picker {
        return super::picker::view(playlists, viewport, scale, palette);
    }
    let selected = playlists
        .selected_def()
        .map(|playlist| playlist.name.as_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(tr("playlists-masthead-fallback"));
    let masthead = crate::frontend::ui::folio_masthead(
        tr_args!("playlists-masthead", selected => selected),
        Message::ClosePlaylists,
        scale,
        palette,
    );
    let sheet = crate::frontend::ui::folio_sheet(
        masthead,
        playlist_index(playlists, scale, palette),
        reading_surface(playlists, scale, palette),
        Message::ClosePlaylists,
        viewport,
        scale,
        1.0,
        crate::frontend::ui::FOLIO_INDEX_WIDTH,
        palette,
    );
    if playlists.filter_help {
        iced::widget::stack![sheet, super::help::view(viewport, scale, palette)].into()
    } else {
        sheet
    }
}
