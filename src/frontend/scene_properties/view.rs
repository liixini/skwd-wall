use iced::widget::{column, container, scrollable, text};
use iced::{Alignment, Color, Element, Length};

use super::state::{ScenePropMsg, SceneProperties};
use crate::app::Message;
use crate::domain::scene_properties::{SceneProperty, ScenePropertyKind, format_number};
use crate::frontend::theme::Palette;
use crate::frontend::ui::{
    FOLIO_INDEX_WIDTH, TYPE_SMALL, folio_action, folio_action_width, folio_action_wrap,
    folio_ghost_field, folio_index_shell, folio_masthead, folio_rule, folio_scroll_padding,
    folio_sheet, folio_sheet_dims, folio_slider, label, row, with_alpha,
};
use crate::i18n::{tr, tr_args};

const SECTION_SPACING: f32 = 14.0;
const LABEL_WIDTH: f32 = 218.0;

fn wrap(message: ScenePropMsg) -> Message {
    Message::SceneProps(message)
}

fn swatch<'a>(property: &SceneProperty, scale: f32, palette: &Palette) -> Element<'a, Message> {
    let colour = property
        .value
        .colour()
        .map_or(palette.surface, |[red, green, blue]| Color::from_rgb(red, green, blue));
    container(text(""))
        .width(Length::Fixed(38.0 * scale))
        .height(Length::Fixed(20.0 * scale))
        .style(move |_| crate::frontend::ui::box_style(colour, with_alpha(Color::BLACK, 0.38)))
        .into()
}

fn control<'a>(
    property: &'a SceneProperty,
    panel: &'a SceneProperties,
    available: f32,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    let name = property.name.clone();
    match property.kind {
        ScenePropertyKind::Flag => {
            let on = property.value.flag();
            let caption = if on { tr("scene-props-on") } else { tr("scene-props-off") };
            folio_action(
                caption,
                on,
                Some(wrap(ScenePropMsg::Toggle(name))),
                Length::Shrink,
                scale,
                palette,
            )
        }
        ScenePropertyKind::Range => {
            let (min, max, step) = property.range();
            let value = property.value.number().clamp(min, max);
            let slide_name = name.clone();
            row![
                container(folio_slider(
                    min,
                    max,
                    value,
                    step,
                    move |next| wrap(ScenePropMsg::Slide(slide_name.clone(), next)),
                    wrap(ScenePropMsg::Commit(name)),
                    palette,
                ))
                .width(Length::Fill),
                label(format_number(value), 11.0, scale, palette.surface_text),
            ]
            .spacing(10.0 * scale)
            .align_y(Alignment::Center)
            .into()
        }
        ScenePropertyKind::Choice => {
            let selected = property.value.number();
            let actions = property
                .choices
                .iter()
                .map(|choice| {
                    let caption = if choice.label.is_empty() {
                        format_number(choice.value)
                    } else {
                        choice.label.clone()
                    };
                    let active = (choice.value - selected).abs() < f64::EPSILON;
                    (caption, active, wrap(ScenePropMsg::Choose(name.clone(), choice.value)))
                })
                .collect();
            folio_action_wrap(actions, available, scale, palette)
        }
        ScenePropertyKind::Colour => {
            let draft = panel.colour_draft(&property.name).unwrap_or_default();
            let input_name = name.clone();
            let mut editor = column![
                row![
                    swatch(property, scale, palette),
                    folio_ghost_field(
                        draft,
                        "1.000 1.000 1.000",
                        move |next| wrap(ScenePropMsg::ColourInput(input_name.clone(), next)),
                        wrap(ScenePropMsg::ColourCommit(name.clone())),
                        Length::Fill,
                        scale,
                        palette,
                    ),
                    folio_action(
                        tr("theme-designer-set-colour"),
                        false,
                        Some(wrap(ScenePropMsg::ColourCommit(name.clone()))),
                        Length::Fixed(folio_action_width(tr("theme-designer-set-colour"), scale)),
                        scale,
                        palette,
                    ),
                ]
                .spacing(8.0 * scale)
                .align_y(Alignment::Center)
            ]
            .spacing(5.0 * scale);
            for (channel, (caption, value)) in ["browser-red", "browser-green", "browser-blue"]
                .into_iter()
                .zip(property.value.colour().unwrap_or([1.0; 3]))
                .enumerate()
            {
                let slide_name = name.clone();
                editor = editor.push(
                    row![
                        container(label(tr(caption), TYPE_SMALL, scale, palette.surface_text))
                            .width(Length::Fixed(58.0 * scale)),
                        container(folio_slider(
                            0.0,
                            1.0,
                            f64::from(value).clamp(0.0, 1.0),
                            0.001,
                            move |next| wrap(ScenePropMsg::ColourSlide(
                                slide_name.clone(),
                                channel,
                                next
                            )),
                            wrap(ScenePropMsg::Commit(name.clone())),
                            palette,
                        ))
                        .width(Length::Fill),
                    ]
                    .spacing(8.0 * scale)
                    .align_y(Alignment::Center),
                );
            }
            editor.into()
        }
        ScenePropertyKind::Group | ScenePropertyKind::Unsupported => label(
            tr("scene-props-unsupported"),
            TYPE_SMALL,
            scale,
            with_alpha(palette.surface_text, 0.48),
        )
        .into(),
    }
}

fn notice(message: &str, colour: Color, scale: f32) -> Element<'_, Message> {
    container(label(message, 12.0, scale, colour)).padding(24.0 * scale).into()
}

fn reading<'a>(
    panel: &'a SceneProperties,
    available: f32,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    if panel.loading {
        return notice(tr("scene-props-loading"), palette.surface_text, scale);
    }
    let mut body = column![].spacing(SECTION_SPACING * scale);
    if let Some(global) = panel.global_fps {
        let mut actions = vec![(
            tr("scene-props-fps-default").to_string(),
            panel.fps.is_none(),
            wrap(ScenePropMsg::Fps(None)),
        )];
        actions.extend([15, 24, 30, 60, 90, 120, 144, 240].map(|fps| {
            (fps.to_string(), panel.fps == Some(fps), wrap(ScenePropMsg::Fps(Some(fps))))
        }));
        body = body.push(
            column![
                label(tr("scene-props-fps"), 13.0, scale, palette.surface_text),
                label(
                    tr_args!("scene-props-fps-note", fps => global),
                    TYPE_SMALL,
                    scale,
                    with_alpha(palette.surface_text, 0.58)
                ),
                folio_action_wrap(actions, available, scale, palette),
                row![
                    container(folio_slider(
                        1.0,
                        240.0,
                        f64::from(panel.fps.unwrap_or(global)),
                        1.0,
                        |fps| wrap(ScenePropMsg::FpsSlide(fps as u32)),
                        wrap(ScenePropMsg::FpsCommit),
                        palette,
                    ))
                    .width(Length::Fill),
                    label(
                        panel.fps.unwrap_or(global).to_string(),
                        11.0,
                        scale,
                        palette.surface_text
                    ),
                ]
                .spacing(10.0 * scale)
                .align_y(Alignment::Center),
            ]
            .spacing(9.0 * scale),
        );
    }
    if panel.rows.is_empty() {
        body = body.push(notice(tr("scene-props-empty"), palette.surface_text, scale));
    }
    if let Some(error) = &panel.error {
        body = body.push(notice(error.as_str(), palette.primary, scale));
    }
    for property in panel.shown_rows() {
        if property.is_group() {
            body = body.push(
                container(label(property.label.as_str(), 12.0, scale, palette.primary))
                    .padding([10.0 * scale, 8.0 * scale]),
            );
            continue;
        }
        let caption = if property.label == "ui_browse_properties_scheme_color" {
            tr("scene-props-scheme-colour")
        } else {
            property.label.as_str()
        };
        let mut title =
            column![label(caption, 13.0, scale, palette.surface_text)].spacing(5.0 * scale);
        if property.changed() {
            title = title.push(label(
                tr_args!("scene-props-default", value => property.default.display()),
                TYPE_SMALL,
                scale,
                with_alpha(palette.surface_text, 0.58),
            ));
        }
        let stacked = available < 590.0 * scale;
        let controls_width =
            if stacked { available } else { available - (LABEL_WIDTH + 20.0) * scale };
        let controls = control(property, panel, controls_width, scale, palette);
        let contents: Element<'a, Message> = if stacked {
            column![title, controls].spacing(9.0 * scale).into()
        } else {
            row![
                container(title).width(Length::Fixed(LABEL_WIDTH * scale)),
                container(controls).width(Length::Fill),
            ]
            .spacing(20.0 * scale)
            .align_y(Alignment::Center)
            .into()
        };
        body = body.push(column![folio_rule(palette), contents].spacing(12.0 * scale));
    }

    container(scrollable(body.padding(folio_scroll_padding(23.0, 27.0, scale))))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn index_column<'a>(
    panel: &'a SceneProperties,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    let note = tr_args!(
        "scene-props-count",
        editable => panel.editable_count(),
        changed => panel.changed_count()
    );
    let reset = folio_action(
        tr("scene-props-reset"),
        false,
        (panel.fps.is_some() || panel.rows.iter().any(|row| row.overridden))
            .then(|| wrap(ScenePropMsg::Reset)),
        Length::Fill,
        scale,
        palette,
    );
    folio_index_shell(panel.title.as_str(), note, vec![reset], 12.0 * scale, scale, palette)
}

pub fn view<'a>(
    panel: &'a SceneProperties,
    viewport: (f32, f32),
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    folio_sheet(
        folio_masthead(
            tr("scene-props-crumb").to_string(),
            wrap(ScenePropMsg::Close),
            scale,
            palette,
        ),
        index_column(panel, scale, palette),
        reading(
            panel,
            folio_sheet_dims(viewport, scale).0 - FOLIO_INDEX_WIDTH * scale.max(0.9) - 54.0 * scale,
            scale,
            palette,
        ),
        wrap(ScenePropMsg::Close),
        viewport,
        scale,
        1.0,
        FOLIO_INDEX_WIDTH,
        palette,
    )
}
