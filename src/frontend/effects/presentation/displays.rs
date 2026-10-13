use iced::widget::{column, container, image, scrollable, text};
use iced::{Alignment, Element, Length};

use crate::app::Message;
use crate::domain::library::catalog::WallpaperKind;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{
    FOLIO_INDEX_WIDTH, UI_FONT, folio_horizontal_rule, folio_rule, label, row, with_alpha,
};
use crate::i18n::tr;

use super::super::state::{Effects, EffectsMsg};

struct ActivitySpec {
    gap: f32,
    heading: f32,
    body: f32,
    muted_alpha: f32,
}

const ACTIVITY: ActivitySpec =
    ActivitySpec { gap: 8.0, heading: 14.0, body: 10.0, muted_alpha: 0.52 };

impl Effects {
    pub(super) fn displays_view<'a>(
        &'a self,
        viewport: (f32, f32),
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        let ease = self.open_ease();
        let source = self.source_stem();
        let count = self.displays.selected_outputs.len();
        let total = self.displays.monitors.len();

        let masthead: Element<'a, Message> = container(text("")).height(0.0).into();
        let index = self.apply_index(source, count, total, scale, palette);
        let reading = self.display_reading_surface(count, viewport, scale, palette);
        crate::frontend::ui::folio_sheet(
            masthead,
            index,
            reading,
            Message::ToggleEffects,
            viewport,
            scale,
            ease,
            FOLIO_INDEX_WIDTH,
            palette,
        )
    }

    fn apply_index<'a>(
        &'a self,
        source: String,
        count: usize,
        total: usize,
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        let source_path = self.display_source();
        let preview: Element<'a, Message> = if source_path.is_empty() {
            super::hud::thumb_placeholder(270.0 * scale, 152.0 * scale, palette)
        } else {
            image(image::Handle::from_path(source_path))
                .width(Length::Fill)
                .height(Length::Fixed(152.0 * scale))
                .content_fit(iced::ContentFit::Cover)
                .into()
        };
        let kind = match self.preview.kind {
            WallpaperKind::Static => tr("effects-kind-static"),
            WallpaperKind::Video => tr("effects-kind-video"),
            WallpaperKind::We => tr("effects-kind-we"),
        };
        let source_card = column![
            preview,
            label(tr("effects-incoming-wallpaper"), 9.0, scale, palette.primary),
            label(source, 14.0, scale, palette.surface_text),
            label(kind, 9.5, scale, with_alpha(palette.surface_text, 0.48)),
        ]
        .spacing(7.0 * scale);

        let selection_action = if self.all_selected() {
            tr("effects-clear-selection")
        } else {
            tr("effects-select-all")
        };
        let selection = column![
            folio_horizontal_rule(with_alpha(palette.outline, 0.48)),
            label(tr("effects-targets-label"), 9.0, scale, palette.primary),
            row![
                text(count.to_string())
                    .font(UI_FONT)
                    .size(30.0 * scale.clamp(0.9, 1.05))
                    .color(palette.surface_text),
                column![
                    label(
                        crate::i18n::effects_of_total_displays(total),
                        11.0,
                        scale,
                        palette.surface_text,
                    ),
                    label(
                        tr("effects-target-note"),
                        9.0,
                        scale,
                        with_alpha(palette.surface_text, 0.46),
                    ),
                ]
                .spacing(3.0 * scale),
            ]
            .spacing(10.0 * scale)
            .align_y(Alignment::Center),
            crate::frontend::ui::folio_action(
                selection_action,
                self.all_selected(),
                (!self.displays.monitors.is_empty())
                    .then_some(Message::Effects(EffectsMsg::MonitorToggleAll)),
                Length::Fill,
                scale,
                palette,
            ),
        ]
        .spacing(10.0 * scale);

        crate::frontend::ui::folio_index_shell(
            tr("effects-apply-index-title"),
            tr("effects-apply-index-desc"),
            vec![
                source_card.into(),
                selection.into(),
                container(text("")).height(Length::Fill).into(),
            ],
            18.0,
            scale,
            palette,
        )
    }

    fn display_reading_surface<'a>(
        &'a self,
        count: usize,
        viewport: (f32, f32),
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        let header = column![
            row![
                column![
                    text(tr("effects-choose-displays"))
                        .font(UI_FONT)
                        .size(34.0 * scale.clamp(0.88, 1.08))
                        .line_height(iced::widget::text::LineHeight::Relative(1.0))
                        .color(palette.surface_text),
                    label(
                        tr("effects-choose-displays-desc"),
                        10.0,
                        scale,
                        with_alpha(palette.surface_text, 0.56),
                    ),
                ]
                .spacing(6.0 * scale),
                container(text("")).width(Length::Fill),
                crate::frontend::ui::folio_action(
                    "×",
                    false,
                    Some(Message::ToggleEffects),
                    Length::Shrink,
                    scale,
                    palette,
                ),
            ]
            .align_y(Alignment::Start),
            folio_rule(palette),
        ]
        .spacing(10.0 * scale);

        let section = column![
            label(tr("effects-target-displays"), 14.0, scale, palette.surface_text),
            label(
                tr("effects-target-displays-desc"),
                10.0,
                scale,
                with_alpha(palette.surface_text, 0.52),
            ),
        ]
        .spacing(4.0 * scale);

        let mut monitors = column![].spacing(9.0 * scale);
        if self.displays.monitors.is_empty() {
            monitors = monitors.push(
                container(
                    column![
                        label(tr("effects-looking-displays"), 14.0, scale, palette.surface_text),
                        label(
                            tr("effects-looking-displays-desc"),
                            10.0,
                            scale,
                            with_alpha(palette.surface_text, 0.5),
                        ),
                    ]
                    .spacing(6.0 * scale),
                )
                .padding(18.0 * scale)
                .width(Length::Fill)
                .style(move |_| {
                    crate::frontend::ui::box_style(
                        with_alpha(palette.surface_container, 0.54),
                        with_alpha(palette.outline, 0.52),
                    )
                }),
            );
        } else {
            let wide = super::monitor::tile_is_wide(viewport, scale);
            for monitor in &self.displays.monitors {
                monitors = monitors.push(self.monitor_tile(monitor, scale, wide, palette));
            }
        }
        let targets = scrollable(monitors)
            .id(iced::widget::Id::new("effects-display-targets"))
            .direction(iced::widget::scrollable::Direction::Vertical(
                iced::widget::scrollable::Scrollbar::new().width(3.0).scroller_width(3.0),
            ))
            .style(crate::frontend::ui::scroll_style(palette.primary))
            .width(Length::Fill)
            .height(Length::Fill);

        let apply_label = if count == 0 {
            tr("effects-select-to-apply").to_string()
        } else {
            crate::i18n::effects_apply_count(count)
        };
        let apply = crate::frontend::ui::folio_action(
            apply_label,
            count > 0 && self.activity.can_apply(),
            (count > 0 && self.activity.can_apply())
                .then_some(Message::Effects(EffectsMsg::MonitorApplySelected)),
            Length::Fixed(210.0 * scale),
            scale,
            palette,
        );
        let commit = row![
            column![
                label(tr("effects-commit-selection"), 13.0, scale, palette.surface_text),
                label(
                    if count == 0 { tr("effects-commit-none") } else { tr("effects-commit-some") },
                    9.5,
                    scale,
                    with_alpha(palette.surface_text, 0.5),
                ),
            ]
            .spacing(4.0 * scale),
            container(text("")).width(Length::Fill),
            apply,
        ]
        .align_y(Alignment::Center);

        let mut content = column![header].spacing(12.0 * scale);
        if let Some(activity) = self.activity_section(scale, palette) {
            content = content.push(activity);
        }
        container(content.push(section).push(targets).push(folio_rule(palette)).push(commit))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(crate::frontend::ui::folio_scroll_padding(23.0, 27.0, scale))
            .into()
    }

    fn activity_section<'a>(
        &'a self,
        scale: f32,
        palette: &'a Palette,
    ) -> Option<Element<'a, Message>> {
        if !self.activity.supported && self.activity.error.is_none() {
            return None;
        }
        let mut choices = row![].spacing(ACTIVITY.gap * scale);
        for activity in &self.activity.activities {
            let name = if activity.current {
                format!("{} · {}", activity.name, tr("effects-activity-current"))
            } else {
                activity.name.clone()
            };
            let width = crate::frontend::ui::folio_action_width_in(&name, scale, 0.0, f32::MAX);
            choices = choices.push(crate::frontend::ui::folio_action(
                name,
                activity.id == self.activity.selected,
                (self.activity.list_request == 0)
                    .then(|| Message::Effects(EffectsMsg::ActivitySelect(activity.id.clone()))),
                Length::Fixed(width),
                scale,
                palette,
            ));
        }
        let mut section = column![
            label(tr("effects-activity-label"), ACTIVITY.heading, scale, palette.surface_text),
            label(
                tr("effects-activity-desc"),
                ACTIVITY.body,
                scale,
                with_alpha(palette.surface_text, ACTIVITY.muted_alpha)
            ),
            scrollable(choices)
                .direction(iced::widget::scrollable::Direction::Horizontal(
                    iced::widget::scrollable::Scrollbar::new(),
                ))
                .width(Length::Fill),
        ]
        .spacing(ACTIVITY.gap * scale);
        if self.activity.inactive() {
            section = section.push(label(
                tr("effects-activity-inactive-desc"),
                ACTIVITY.body,
                scale,
                with_alpha(palette.surface_text, ACTIVITY.muted_alpha),
            ));
        }
        if let Some(error) = &self.activity.error {
            section = section.push(label(error.clone(), ACTIVITY.body, scale, palette.primary));
        }
        Some(section.into())
    }
}
