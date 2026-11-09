use iced::widget::{
    Space, Stack, button, canvas, column, container, mouse_area, scrollable, stack, text,
    text_input,
};
use iced::{Alignment, Element, Length, Padding, Rectangle};

use crate::frontend::scene::layout::Mode;
use crate::frontend::ui::row;

#[allow(clippy::wildcard_imports)]
use super::super::*;

const CLOUD_RISE_POWER: i32 = 3;
const CLOUD_RISE: f32 = 16.0;
const CLOUD_INNER_PAD: f32 = 36.0;
const CLOUD_INNER_MIN: f32 = 120.0;
const CLOUD_CARD_GAP: f32 = 14.0;
const CLOUD_BOTTOM_MARGIN: f32 = 16.0;
const CLOUD_WIDTH_GROWTH: f32 = 1.15;
const CLOUD_AUTO_WIDTH_MAX: f32 = 920.0;

pub(crate) fn tag_intent_message(intent: crate::frontend::tagcloud::TagIntent) -> Message {
    match intent {
        crate::frontend::tagcloud::TagIntent::Update(message) => Message::Tag(message),
        crate::frontend::tagcloud::TagIntent::Clear => Message::ClearTags,
        crate::frontend::tagcloud::TagIntent::ToggleCloud => Message::OpenTagCloud,
    }
}

pub(crate) fn cloud_fit_width(
    override_w: Option<f32>,
    content_w: Option<f32>,
    cfg_w: f32,
    vw: f32,
) -> f32 {
    let w = match (override_w, content_w) {
        (Some(w), _) => w,
        (None, Some(cw)) => {
            cw.clamp(cfg_w, (cfg_w * CLOUD_WIDTH_GROWTH).min(CLOUD_AUTO_WIDTH_MAX).max(cfg_w))
        }
        (None, None) => cfg_w,
    };
    w.min(vw - 40.0).max(200.0)
}

pub(crate) fn cloud_fit_height(
    matching_tags_open: bool,
    _describe_mode: bool,
    visible_rows: usize,
    content_rows: usize,
    vh: f32,
    scale: f32,
) -> f32 {
    const COLLAPSED_H: f32 = 126.0;
    const EXPANDED_CHROME_H: f32 = 134.0;
    if !matching_tags_open {
        return (COLLAPSED_H * scale).min((vh - 80.0).max(90.0));
    }
    let actual_rows = content_rows.clamp(1, visible_rows.clamp(1, 3));
    (EXPANDED_CHROME_H * scale + crate::frontend::ui::tag_cloud_body_height(actual_rows, scale))
        .min((vh - 80.0).max(160.0))
}

pub(super) fn selection_marks(app: &App) -> Option<Element<'_, Message>> {
    let marks: Vec<(f32, f32, f32, f32, f32)> = app
        .scene
        .render
        .hits
        .iter()
        .filter_map(|hit| {
            let si = *app.library_session.filtered.get(hit.index)?;
            app.tags.select.contains(&si).then_some((hit.cx, hit.cy, hit.hw, hit.hh, hit.skew))
        })
        .collect();
    (!marks.is_empty()).then(|| {
        canvas(crate::frontend::ui::SelectionMarks { marks, pal: app.theme.palette })
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    })
}

pub(super) fn tag_add_overlay(app: &App) -> Option<Element<'_, Message>> {
    if !app.tags.editing || app.tags.card_drawer_open || app.scene.flipped().is_none() {
        return None;
    }
    let bp = app.scene.render.back.as_ref().filter(|bp| bp.add_open > 0.85)?;
    let lay = crate::frontend::ui::back_layout(bp);
    let (left, top, width, _) = crate::frontend::ui::back_bounds(
        bp,
        &lay,
        (lay.add.0 + 8.0, lay.add.1 - 2.0, (lay.add.2 - 16.0).max(80.0), lay.add.3 + 4.0),
    );
    let ip = app.theme.palette;
    let input = text_input(crate::i18n::tr("tags-input-placeholder"), &app.tags.input)
        .id(tag_input_id())
        .on_input(|text| Message::Tag(crate::frontend::tagcloud::TagMsg::InputChanged(text)))
        .on_submit(Message::Tag(crate::frontend::tagcloud::TagMsg::Submit))
        .padding(Padding { top: 7.0, bottom: 7.0, left: 6.0, right: 6.0 })
        .size(13)
        .width(Length::Fixed(width.max(80.0)))
        .style(move |_theme, _status| crate::frontend::ui::ghost_input_style(&ip));
    Some(container(input).padding(Padding { top, left, ..Padding::ZERO }).into())
}

pub(super) fn card_tag_remove_overlay(app: &App) -> Option<Element<'_, Message>> {
    app.scene.flipped()?;
    let panel = app.scene.render.back.as_ref()?;
    if panel.tags.is_empty() {
        return None;
    }
    let layout = crate::frontend::ui::back_layout(panel);
    let mut targets: Vec<Element<'_, Message>> = layout
        .tags
        .iter()
        .enumerate()
        .map(|(index, &(x, y, width, height))| {
            let target_w = 30.0_f32.min(width);
            let (left, top, target_w, target_h) = crate::frontend::ui::back_bounds(
                panel,
                &layout,
                (x + width - target_w, y, target_w, height),
            );
            container(
                mouse_area(
                    Space::new().width(Length::Fixed(target_w)).height(Length::Fixed(target_h)),
                )
                .on_press(Message::Tag(crate::frontend::tagcloud::TagMsg::Remove(index)))
                .interaction(iced::mouse::Interaction::Pointer),
            )
            .padding(Padding { top: top.max(0.0), right: 0.0, bottom: 0.0, left: left.max(0.0) })
            .into()
        })
        .collect();
    if let Some((x, y, width, height, _)) = layout.tag_overflow {
        let (x, y, width, height) =
            crate::frontend::ui::back_bounds(panel, &layout, (x, y, width, height));
        targets.push(
            container(
                mouse_area(Space::new().width(Length::Fixed(width)).height(Length::Fixed(height)))
                    .on_press(Message::Tag(crate::frontend::tagcloud::TagMsg::ToggleCardDrawer))
                    .interaction(iced::mouse::Interaction::Pointer),
            )
            .padding(Padding { top: y.max(0.0), left: x.max(0.0), ..Padding::ZERO })
            .into(),
        );
    }
    Some(Stack::with_children(targets).width(Length::Fill).height(Length::Fill).into())
}

pub(super) fn card_tag_drawer(app: &App) -> Option<Element<'_, Message>> {
    if !app.tags.card_drawer_open || app.scene.flipped().is_none() {
        return None;
    }
    let panel = app.scene.render.back.as_ref()?;
    let layout = crate::frontend::ui::back_layout(panel);
    let drawer = card_tag_drawer_bounds(panel, &layout);
    let (left, top, width, height) = crate::frontend::ui::back_bounds(panel, &layout, drawer);
    let scale = app.config.ui_scale();
    let pal = app.theme.palette;

    let close_pal = pal;
    let close = button(text("×").size(15.0 * scale))
        .on_press(Message::Tag(crate::frontend::tagcloud::TagMsg::CloseCardDrawer))
        .padding(Padding::from([3.0 * scale, 9.0 * scale]))
        .style(move |_theme, status| {
            crate::frontend::ui::folio_button_style(false, false, &close_pal, status)
        });
    let header = row![
        text(crate::i18n::tr("tags-drawer-title"))
            .font(iced::Font::MONOSPACE)
            .size(10.0 * scale)
            .color(pal.primary),
        Space::new().width(Length::Fill),
        text(format!("{:02}", app.tags.card_locked.len()))
            .font(iced::Font::MONOSPACE)
            .size(10.0 * scale)
            .color(crate::frontend::ui::with_alpha(pal.surface_text, 0.56)),
        close,
    ]
    .spacing(8.0 * scale)
    .align_y(Alignment::Center);

    let mut entries: Vec<Element<'_, Message>> = Vec::with_capacity(app.tags.card_locked.len());
    for (index, tag) in app.tags.card_locked.iter().enumerate() {
        let remove_pal = pal;
        let remove = button(text("×").size(13.0 * scale))
            .on_press(Message::Tag(crate::frontend::tagcloud::TagMsg::Remove(index)))
            .padding(Padding::from([3.0 * scale, 10.0 * scale]))
            .style(move |_theme, status| {
                crate::frontend::ui::folio_button_style(false, true, &remove_pal, status)
            });
        let line_bg = crate::frontend::ui::with_alpha(pal.surface_variant, 0.34);
        let line_border = crate::frontend::ui::with_alpha(pal.outline, 0.28);
        entries.push(
            container(
                row![
                    text(format!("{:02}", index + 1))
                        .font(iced::Font::MONOSPACE)
                        .size(9.5 * scale)
                        .color(crate::frontend::ui::with_alpha(pal.primary, 0.72)),
                    text(tag.to_uppercase()).size(11.0 * scale).color(pal.surface_text),
                    Space::new().width(Length::Fill),
                    remove,
                ]
                .spacing(10.0 * scale)
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([5.0 * scale, 8.0 * scale]))
            .width(Length::Fill)
            .style(move |_| crate::frontend::ui::box_style(line_bg, line_border))
            .into(),
        );
    }
    if entries.is_empty() {
        entries.push(
            container(
                text(crate::i18n::tr("tags-none-remain"))
                    .font(iced::Font::MONOSPACE)
                    .size(10.0 * scale)
                    .color(crate::frontend::ui::with_alpha(pal.surface_text, 0.5)),
            )
            .padding(12.0 * scale)
            .into(),
        );
    }

    let scroll_pal = pal;
    let list = scrollable(column(entries).spacing(5.0 * scale))
        .direction(crate::frontend::ui::thin_vbar())
        .style(crate::frontend::ui::scroll_style(scroll_pal.primary));
    let input_pal = pal;
    let input = text_input(crate::i18n::tr("tags-input-placeholder"), &app.tags.input)
        .id(tag_input_id())
        .on_input(|value| Message::Tag(crate::frontend::tagcloud::TagMsg::InputChanged(value)))
        .on_submit(Message::Tag(crate::frontend::tagcloud::TagMsg::Submit))
        .padding(Padding::from([7.0 * scale, 8.0 * scale]))
        .size(11.0 * scale)
        .width(Length::Fill)
        .style(move |_theme, _status| crate::frontend::ui::ghost_input_style(&input_pal));
    let hint = text(crate::i18n::tr("tags-drawer-hint"))
        .font(iced::Font::MONOSPACE)
        .size(8.5 * scale)
        .color(crate::frontend::ui::with_alpha(pal.surface_text, 0.48));
    let drawer_bg = crate::frontend::ui::with_alpha(pal.surface_container, 0.97);
    let drawer_border = crate::frontend::ui::with_alpha(pal.primary, 0.72);
    let drawer = container(
        column![header, list.height(Length::Fill), input, hint]
            .spacing(10.0 * scale)
            .height(Length::Fill),
    )
    .padding(12.0 * scale)
    .width(Length::Fixed(width))
    .height(Length::Fixed(height))
    .style(move |_| crate::frontend::ui::box_style(drawer_bg, drawer_border));

    Some(
        container(mouse_area(drawer).on_press(Message::Noop))
            .padding(Padding { top, left, ..Padding::ZERO })
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
    )
}

fn card_tag_drawer_bounds(
    panel: &crate::frontend::scene::BackPanel,
    layout: &crate::frontend::ui::BackLayout,
) -> (f32, f32, f32, f32) {
    let card_right = layout.card.0 + layout.card.2;
    let card_bottom = layout.card.1 + layout.card.3;
    if !panel.embedded {
        let left = layout.sheet.0 + 12.0;
        let preferred_top = (layout.tags_label_cy - 10.0).max(layout.sheet.1 + 10.0);
        let top = if card_bottom - preferred_top >= 182.0 {
            preferred_top
        } else {
            layout.sheet.1 + 10.0
        };
        return (
            left,
            top,
            (layout.sheet.2 - 24.0).max(100.0),
            (card_bottom - top - 12.0).clamp(80.0, 440.0),
        );
    }

    let width = (layout.sheet.2 * 0.56).clamp(290.0, 520.0).min(layout.card.2 - 20.0);
    let left = layout.content_left.clamp(layout.card.0 + 10.0, card_right - width - 10.0);
    let masthead_bottom = layout.masthead.1 + layout.masthead.3 + 8.0;
    let available_above = layout.sheet.1 - masthead_bottom - 8.0;
    if available_above >= 170.0 {
        let height = available_above.min(360.0);
        return (left, layout.sheet.1 - height - 8.0, width, height);
    }
    let height = (layout.card.3 * 0.54).clamp(170.0, 340.0).min(layout.card.3 - 20.0);
    let top = (layout.card.1 + (layout.card.3 - height) * 0.5)
        .clamp(layout.card.1 + 10.0, card_bottom - height - 10.0);
    (left, top, width, height)
}

pub(crate) fn tag_cloud_bounds(app: &App, context: &super::layout::LayoutContext<'_>) -> Rectangle {
    let scale = context.config.ui_scale();
    let (vw, vh) = context.viewport;
    let content_w = match context.params.mode {
        Mode::Grid => Some(context.params.grid.total_w()),
        Mode::Hex => Some(context.params.hex.visible_band()),
        _ => None,
    };
    let cloud_w = cloud_fit_width(
        context.config.tag_cloud_width_override().map(|w| w as f32 * scale),
        content_w,
        context.config.tag_cloud_width() * scale,
        vw,
    )
    .min(vw.max(1.0));
    let (cox, coy) = context.config.tag_cloud_offset();
    let ent = context.entrance;
    let rise = (1.0 - ent).powi(CLOUD_RISE_POWER) * CLOUD_RISE * scale;
    let entries = app.tag_cloud_entries();
    let tag_inner_width = (cloud_w - CLOUD_INNER_PAD * scale).max(CLOUD_INNER_MIN);
    let content_rows = crate::frontend::ui::tag_cloud_row_count(&entries, tag_inner_width, scale);
    let visible_rows = context.config.tag_cloud_rows();
    let cloud_h = cloud_fit_height(
        app.tags.matching_tags_open,
        app.tags.search_mode == SearchMode::Describe,
        visible_rows,
        content_rows,
        vh,
        scale,
    )
    .min(vh.max(1.0));
    let default_top = if matches!(context.params.mode, Mode::Sandy) {
        let strip_top =
            crate::frontend::scene::sandy::strip_bottom(vh) - context.params.extra.sandy.strip_h();
        (strip_top - crate::frontend::scene::sandy::BAR_GAP - cloud_h + coy).max(0.0)
    } else {
        (((vh + context.card_height()) * 0.5 + CLOUD_CARD_GAP + coy)
            .min(vh - cloud_h - CLOUD_BOTTOM_MARGIN))
        .max(0.0)
    };
    let (cloud_left, cloud_top) = if let Some((x, y)) = context.config.tag_cloud_position() {
        (x * vw, y * vh)
    } else {
        (((vw - cloud_w) * 0.5 + cox).max(0.0), default_top)
    };
    Rectangle {
        x: cloud_left.clamp(0.0, (vw - cloud_w).max(0.0)),
        y: (cloud_top + rise).clamp(0.0, (vh - cloud_h).max(0.0)),
        width: cloud_w,
        height: cloud_h,
    }
}

pub(crate) fn tag_cloud_target(app: &App, vh: f32) -> Rectangle {
    let context = super::layout::LayoutContext::current(app, (app.scene.viewport.0, vh));
    let rect = tag_cloud_bounds(app, &context);
    crate::app::update::arrange::placement::resolve(
        app,
        crate::frontend::arrange::Target::Search,
        rect,
    )
}

pub(crate) fn tag_cloud_geometry(app: &App, vh: f32) -> Rectangle {
    let rect = tag_cloud_target(app, vh);
    app.panels
        .arrange
        .as_ref()
        .and_then(|session| session.editor.shown(crate::frontend::arrange::Target::Search))
        .map_or(rect, |shown| Rectangle { x: shown.x, y: shown.y, width: shown.w, height: shown.h })
}

pub(super) fn tag_cloud_layer(app: &App, vh: f32) -> Element<'_, Message> {
    let scale = app.config.ui_scale();
    let rect = tag_cloud_geometry(app, vh);
    let entries = app.tag_cloud_entries();
    let cloud = crate::frontend::tagcloud::view(crate::frontend::tagcloud::CloudView {
        entries,
        db_empty: app.library_session.library.catalog().tags.is_empty(),
        selected: &app.library_session.filters.tags,
        query_chips: app.library_session.filters.numeric.chips(),
        match_any: app.library_session.filters.tags_match_any,
        search_mode: app.tags.search_mode,
        semantic_model: semantic_model_name(app),
        semantic_enabled: app.config.flag_default_true(skwd_config::keys::semantic::ENABLED),
        search_text: if app.tags.search_mode == SearchMode::Tags {
            &app.tags.tag_search
        } else {
            &app.tags.semantic.search
        },
        partial: if app.tags.search_mode == SearchMode::Tags {
            crate::app::tag_search_partial(&app.tags.tag_search)
        } else {
            ""
        },
        semantic_pending: app.tags.semantic.pending,
        semantic_error: app.tags.semantic.error.as_deref(),
        semantic_ms: app.tags.semantic.query_ms + app.tags.semantic.search_ms,
        filtered_count: app.library_session.visible_count,
        sort_az: app.tags.sort_az,
        matching_tags_open: app.tags.matching_tags_open,
        entrance: if app.panels.arrange.is_some() {
            1.0
        } else {
            app.tags.cloud_entrance.x * app.scene.open_fade()
        },
        scroll: app.tags.cloud_scroll.x,
        scroll_target: app.tags.cloud_scroll.target,
        width: rect.width,
        height: rect.height,
        scale,
        pal: &app.theme.palette,
    })
    .map(tag_intent_message);
    container(iced::widget::mouse_area(cloud).on_press(Message::Noop))
        .width(Length::Fill)
        .align_x(Alignment::Start)
        .padding(Padding { top: rect.y, left: rect.x, ..Padding::ZERO })
        .into()
}

#[cfg(test)]
mod tests;

fn semantic_model_name(app: &App) -> String {
    let selected = app.config.str_path(skwd_config::keys::semantic::MANIFEST);
    app.config
        .array_values(skwd_config::keys::semantic::MODELS)
        .into_iter()
        .find(|model| model.get("manifest").and_then(serde_json::Value::as_str) == Some(&selected))
        .and_then(|model| model.get("name").and_then(serde_json::Value::as_str).map(str::to_string))
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| crate::i18n::tr("tags-search-model-default").to_string())
}

pub(super) fn tag_mode_panel(app: &App) -> Element<'_, Message> {
    let scale = app.config.ui_scale();
    let pal = app.theme.palette;
    let chip_h = 24.0 * scale;
    let count = app.tags.select.len();

    let mut top_row: Vec<Element<'_, Message>> = vec![
        container(
            text(format!("\u{f04fb} {}", crate::i18n::tags_selected_count(count)))
                .size(12.0 * scale)
                .color(pal.primary),
        )
        .center_y(Length::Fixed(chip_h))
        .into(),
    ];
    for (idx, tag) in app.tags.mass_tags.iter().enumerate() {
        let lbl = format!("{tag} \u{00d7}");
        top_row.push(crate::frontend::ui::skwd_chip(
            lbl.clone(),
            false,
            11.0,
            true,
            Length::Fixed(crate::frontend::ui::chip_width(&lbl, 11.0, chip_h)),
            chip_h,
            Message::MassTagRemove(idx),
            &pal,
        ));
    }
    let apply_lbl = crate::i18n::tr_args!("tags-apply-count", count => count);
    top_row.push(crate::frontend::ui::skwd_chip(
        apply_lbl.clone(),
        false,
        11.0,
        count > 0 && !app.tags.mass_tags.is_empty(),
        Length::Fixed(crate::frontend::ui::chip_width(&apply_lbl, 11.0, chip_h)),
        chip_h,
        Message::MassTagApply,
        &pal,
    ));
    top_row.push(crate::frontend::ui::skwd_chip(
        crate::i18n::tr("tags-done"),
        false,
        11.0,
        false,
        Length::Fixed(crate::frontend::ui::chip_width(crate::i18n::tr("tags-done"), 11.0, chip_h)),
        chip_h,
        Message::ToggleTagMode,
        &pal,
    ));

    let msugg = crate::app::mass_tag_suggestions(app);
    let ghost_str = ghost_completion(&app.tags.mass_input, &msugg);
    let ip = pal;
    let input = text_input(crate::i18n::tr("tags-add-placeholder"), &app.tags.mass_input)
        .id(mass_tag_input_id())
        .on_input(Message::MassTagInput)
        .on_submit(Message::MassTagAdd(app.tags.mass_input.clone()))
        .padding(Padding { top: 6.0, bottom: 6.0, left: 8.0, right: 8.0 })
        .size(13)
        .width(Length::Fixed(170.0 * scale))
        .style(move |_t, _s| crate::frontend::ui::ghost_input_style(&ip));
    let ghost = container(
        text(ghost_str).size(13).color(crate::frontend::ui::with_alpha(pal.surface_text, 0.28)),
    )
    .padding(Padding { top: 6.0, left: 8.0, ..Padding::ZERO });
    let field_bg = crate::frontend::ui::with_alpha(pal.surface, 0.55);
    let field_bd = crate::frontend::ui::with_alpha(pal.primary, 0.3);
    let field = container(stack![ghost, input])
        .style(move |_| crate::frontend::ui::box_style(field_bg, field_bd));
    let mut input_row: Vec<Element<Message>> = vec![field.into()];
    for (tag, num) in msugg.iter().skip(1).take(3) {
        let label = format!("{tag}  {num}");
        input_row.push(crate::frontend::ui::skwd_chip(
            label.clone(),
            false,
            11.0,
            false,
            Length::Fixed(crate::frontend::ui::chip_width(&label, 11.0, chip_h)),
            chip_h,
            Message::MassTagAdd(tag.clone()),
            &pal,
        ));
    }

    let panel_bg = crate::frontend::ui::with_alpha(pal.surface_container, 0.97);
    let panel_bd = crate::frontend::ui::with_alpha(pal.primary, 0.5);
    let panel = container(
        column(vec![
            crate::frontend::ui::Flow::new(top_row).spacing(6.0).align_y(Alignment::Center).into(),
            crate::frontend::ui::Flow::new(input_row)
                .spacing(6.0)
                .align_y(Alignment::Center)
                .into(),
        ])
        .spacing(8.0),
    )
    .padding(Padding::from([10.0, 14.0]))
    .style(move |_| crate::frontend::ui::box_style(panel_bg, panel_bd));

    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::End)
        .padding(Padding { bottom: 96.0 * scale, ..Padding::ZERO })
        .into()
}
