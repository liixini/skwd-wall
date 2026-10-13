use iced::widget::{button, column, container, image, mouse_area, text};
use iced::{Alignment, ContentFit, Element, Length, Padding};

use crate::app::Message;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{label, row, with_alpha};
use crate::i18n::tr;

use super::super::state::{Effects, EffectsMsg, FILL_MODES, MonitorInfo, TileAudio};
use super::hud::thumb_placeholder;

const PREVIEW_MAX_WIDTH: f32 = 132.0;
const PREVIEW_MAX_HEIGHT: f32 = 88.0;
const WIDE_PREVIEW_MAX_WIDTH: f32 = 220.0;
const SELECTION_MIN_WIDTH: f32 = 360.0;
const TILE_SIDE_PADDING: f32 = 10.0;
const READING_SIDE_PADDING: f32 = 27.0;
const SELECTION_PADDING: [f32; 2] = [8.0, 10.0];
const ROW_SPACING: f32 = 7.0;
const ROW_LABEL_WIDTH: f32 = 72.0;

pub(super) fn tile_is_wide(viewport: (f32, f32), scale: f32) -> bool {
    let (panel, _) = crate::frontend::ui::folio_sheet_dims(viewport, scale);
    let card = panel
        - crate::frontend::ui::FOLIO_INDEX_WIDTH * scale.max(0.9)
        - 2.0 * (READING_SIDE_PADDING + TILE_SIDE_PADDING) * scale;
    card >= controls_width(scale) + (SELECTION_MIN_WIDTH + 14.0) * scale
}

fn controls_width(scale: f32) -> f32 {
    ((ROW_LABEL_WIDTH + 6.0 * (69.0 + 6.0)) * scale)
        .max(crate::frontend::settings::background::BackgroundControl::choices_width(scale))
}

impl Effects {
    pub(super) fn monitor_tile<'a>(
        &'a self,
        monitor: &'a MonitorInfo,
        scale: f32,
        wide: bool,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        let selected = self.displays.selected_outputs.contains(&monitor.target);
        let inactive = self.activity.inactive();
        let mut controls = vec![
            column![
                Self::placement_row(monitor, scale, palette),
                monitor.background.view(
                    monitor.background_reveal,
                    monitor.background_picker,
                    scale,
                    palette
                )
            ]
            .into(),
            Self::lock_row(monitor, scale, palette),
        ];
        if inactive {
            controls.clear();
        }
        if monitor.connected && !inactive {
            controls.push(Self::theme_row(monitor, scale, palette));
        }
        let audio = self.tile_audio(monitor, selected);
        if audio != TileAudio::None && !inactive {
            controls.push(self.audio_row(monitor, audio == TileAudio::PreApply, scale, palette));
        }
        if monitor.connected
            && !inactive
            && matches!(
                monitor.kind,
                crate::domain::library::catalog::WallpaperKind::Video
                    | crate::domain::library::catalog::WallpaperKind::We
            )
        {
            controls.push(
                crate::frontend::display_control::DisplayControl::Playback {
                    target: monitor.target.clone(),
                    paused: monitor.paused,
                    manual: monitor.manual_paused,
                }
                .view(scale, palette),
            );
        }
        let rows = controls.len();
        let wide = wide && !inactive;
        let selection = self.selection(monitor, selected, wide.then_some(rows), scale, palette);
        let stack = iced::widget::Column::with_children(controls)
            .spacing(ROW_SPACING * scale)
            .align_x(crate::frontend::ui::start());
        let body: Element<'a, Message> = if wide {
            row![selection, stack.width(Length::Fixed(controls_width(scale)))]
                .spacing(14.0 * scale)
                .into()
        } else {
            column![selection, stack].spacing(ROW_SPACING * scale).into()
        };
        container(body)
            .width(Length::Fill)
            .padding(Padding {
                top: if wide { 9.0 } else { 3.0 } * scale,
                right: TILE_SIDE_PADDING * scale,
                bottom: 9.0 * scale,
                left: TILE_SIDE_PADDING * scale,
            })
            .style(move |_| {
                crate::frontend::ui::box_style(
                    with_alpha(palette.surface_container, if selected { 0.7 } else { 0.48 }),
                    with_alpha(
                        if selected { palette.primary } else { palette.outline },
                        if selected { 0.62 } else { 0.5 },
                    ),
                )
            })
            .into()
    }

    fn selection<'a>(
        &'a self,
        monitor: &'a MonitorInfo,
        selected: bool,
        wide_rows: Option<usize>,
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        let thumb = self.tile_thumb(monitor);
        let thumbnail: Element<Message> = if let Some(rows) = wide_rows {
            let row_height = crate::frontend::ui::FOLIO_BUTTON_HEIGHT * (scale * 0.9).max(1.0);
            let stack_height =
                rows as f32 * row_height + rows.saturating_sub(1) as f32 * ROW_SPACING * scale;
            let height = stack_height - 2.0 * SELECTION_PADDING[0] * scale;
            let width =
                monitor.preview_dimensions(f32::MAX, height).0.min(WIDE_PREVIEW_MAX_WIDTH * scale);
            match thumb {
                Some(path) => image(image::Handle::from_path(path))
                    .width(Length::Fixed(width))
                    .height(Length::Fill)
                    .content_fit(ContentFit::Cover)
                    .into(),
                None => thumb_placeholder(width, Length::Fill, palette),
            }
        } else {
            let (preview_width, preview_height) =
                monitor.preview_dimensions(PREVIEW_MAX_WIDTH, PREVIEW_MAX_HEIGHT);
            let thumbnail: Element<Message> = match thumb {
                Some(path) => image(image::Handle::from_path(path))
                    .width(Length::Fixed(preview_width * scale))
                    .height(Length::Fixed(preview_height * scale))
                    .content_fit(ContentFit::Cover)
                    .into(),
                None => thumb_placeholder(preview_width * scale, preview_height * scale, palette),
            };
            container(thumbnail)
                .center_x(Length::Fixed(PREVIEW_MAX_WIDTH * scale))
                .center_y(Length::Fixed(PREVIEW_MAX_HEIGHT * scale))
                .into()
        };
        let resolution = if monitor.width > 0 {
            format!(
                "{} × {}  ·  {}",
                monitor.width,
                monitor.height,
                tr(monitor.orientation_label_key())
            )
        } else {
            tr("effects-resolution-pending").to_string()
        };
        let details = column![
            label(monitor.name.clone(), 17.0, scale, palette.surface_text),
            label(
                if monitor.connected {
                    resolution
                } else {
                    format!("{}  ·  {}", resolution, tr("effects-display-offline"))
                },
                10.0,
                scale,
                with_alpha(palette.surface_text, 0.5),
            ),
            label(
                match (selected, self.activity.supported && monitor.current.is_empty()) {
                    (true, _) => tr("effects-incoming-preview"),
                    (false, true) => tr("effects-activity-unassigned"),
                    _ => tr("effects-current-wallpaper"),
                },
                9.0,
                scale,
                with_alpha(palette.surface_text, 0.4),
            ),
        ]
        .spacing(4.0 * scale)
        .align_x(crate::frontend::ui::start());
        let target = label(
            if selected { tr("effects-target-selected") } else { tr("effects-target-include") },
            10.0,
            scale,
            if selected { palette.primary } else { with_alpha(palette.surface_text, 0.58) },
        );
        let content = if wide_rows.is_some() {
            row![
                thumbnail,
                column![details, container(text("")).height(Length::Fill), target]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(crate::frontend::ui::start()),
            ]
            .spacing(12.0 * scale)
            .height(Length::Fill)
        } else {
            row![thumbnail, details, container(text("")).width(Length::Fill), target]
                .spacing(12.0 * scale)
                .align_y(Alignment::Center)
        };
        let selection = button(content)
            .width(Length::Fill)
            .height(if wide_rows.is_some() { Length::Fill } else { Length::Shrink })
            .padding([SELECTION_PADDING[0] * scale, SELECTION_PADDING[1] * scale])
            .on_press(Message::Effects(EffectsMsg::MonitorToggle(monitor.target.clone())))
            .style(move |_theme, status| {
                crate::frontend::ui::folio_line_button_style(selected, false, palette, 1.0, status)
            });
        mouse_area(selection)
            .on_enter(Message::Effects(EffectsMsg::MonitorHover(Some(monitor.target.clone()))))
            .on_exit(Message::Effects(EffectsMsg::MonitorHover(None)))
            .into()
    }

    fn placement_row<'a>(
        monitor: &MonitorInfo,
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        let mut controls = row![
            label(
                tr("effects-placement-label"),
                9.0,
                scale,
                with_alpha(palette.surface_text, 0.46)
            )
            .width(Length::Fixed(ROW_LABEL_WIDTH * scale)),
        ]
        .spacing(6.0 * scale)
        .align_y(Alignment::Center);
        for (mode, label_key) in FILL_MODES {
            controls = controls.push(crate::frontend::ui::folio_action(
                tr(label_key),
                monitor.fill == mode,
                Some(Message::MonFill(monitor.target.clone(), mode.to_string())),
                Length::Fixed(69.0 * scale),
                scale * 0.9,
                palette,
            ));
        }
        controls.into()
    }

    fn audio_row<'a>(
        &'a self,
        monitor: &'a MonitorInfo,
        source_preview: bool,
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        let (muted, volume) = if source_preview {
            (self.preview.mute, self.preview.volume)
        } else {
            (monitor.mute, monitor.volume)
        };
        crate::frontend::display_control::DisplayControl::Audio {
            target: monitor.target.clone(),
            muted,
            volume,
            source_preview,
        }
        .view(scale, palette)
    }

    fn theme_row<'a>(
        monitor: &MonitorInfo,
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        crate::frontend::display_control::DisplayControl::Colours {
            target: monitor.target.clone(),
            active: monitor.theme_source,
        }
        .view(scale, palette)
    }

    fn lock_row<'a>(
        monitor: &MonitorInfo,
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        row![
            label(
                tr("settings-displays-lock-label"),
                9.0,
                scale,
                with_alpha(palette.surface_text, 0.46)
            )
            .width(Length::Fixed(ROW_LABEL_WIDTH * scale)),
            crate::frontend::ui::folio_action(
                if monitor.locked {
                    tr("settings-control-enabled")
                } else {
                    tr("settings-control-disabled")
                },
                monitor.locked,
                Some(Message::Effects(EffectsMsg::MonitorLock(
                    monitor.target.clone(),
                    !monitor.locked,
                ))),
                Length::Fixed(92.0 * scale),
                scale * 0.9,
                palette,
            ),
        ]
        .spacing(6.0 * scale)
        .align_y(Alignment::Center)
        .into()
    }
}

#[cfg(test)]
mod tests;
