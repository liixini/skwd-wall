use iced::widget::{button, column, container, image, mouse_area, row, text};
use iced::{Alignment, ContentFit, Element, Length, Padding};

use crate::app::Message;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{label, with_alpha};
use crate::i18n::tr;

use super::super::state::{Effects, EffectsMsg, FILL_MODES, MonitorInfo, TileAudio};
use super::hud::thumb_placeholder;

const PREVIEW_MAX_WIDTH: f32 = 132.0;
const PREVIEW_MAX_HEIGHT: f32 = 88.0;

impl Effects {
    pub(super) fn monitor_tile<'a>(
        &'a self,
        monitor: &'a MonitorInfo,
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        let selected = self.displays.selected_outputs.contains(&monitor.target);
        let (preview_width, preview_height) =
            monitor.preview_dimensions(PREVIEW_MAX_WIDTH, PREVIEW_MAX_HEIGHT);
        let thumbnail: Element<Message> = match self.tile_thumb(monitor) {
            Some(path) => image(image::Handle::from_path(path))
                .width(Length::Fixed(preview_width * scale))
                .height(Length::Fixed(preview_height * scale))
                .content_fit(ContentFit::Cover)
                .into(),
            None => thumb_placeholder(preview_width * scale, preview_height * scale, palette),
        };
        let thumbnail = container(thumbnail)
            .center_x(Length::Fixed(PREVIEW_MAX_WIDTH * scale))
            .center_y(Length::Fixed(PREVIEW_MAX_HEIGHT * scale));
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
        let selection = button(
            row![
                thumbnail,
                column![
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
                        if selected {
                            tr("effects-incoming-preview")
                        } else {
                            tr("effects-current-wallpaper")
                        },
                        9.0,
                        scale,
                        with_alpha(palette.surface_text, 0.4),
                    ),
                ]
                .spacing(4.0 * scale),
                container(text("")).width(Length::Fill),
                label(
                    if selected {
                        tr("effects-target-selected")
                    } else {
                        tr("effects-target-include")
                    },
                    10.0,
                    scale,
                    if selected { palette.primary } else { with_alpha(palette.surface_text, 0.58) },
                ),
            ]
            .spacing(12.0 * scale)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([8.0 * scale, 10.0 * scale])
        .on_press(Message::Effects(EffectsMsg::MonitorToggle(monitor.target.clone())))
        .style(move |_theme, status| {
            crate::frontend::ui::folio_line_button_style(selected, false, palette, 1.0, status)
        });
        let selection = mouse_area(selection)
            .on_enter(Message::Effects(EffectsMsg::MonitorHover(Some(monitor.target.clone()))))
            .on_exit(Message::Effects(EffectsMsg::MonitorHover(None)));

        let mut body = column![
            selection,
            Self::placement_row(monitor, scale, palette),
            Self::lock_row(monitor, scale, palette),
        ]
        .spacing(7.0 * scale);
        let audio = self.tile_audio(monitor, selected);
        if audio != TileAudio::None {
            body = body.push(self.audio_row(monitor, audio == TileAudio::PreApply, scale, palette));
        }
        if monitor.connected
            && matches!(
                monitor.kind,
                crate::domain::library::catalog::WallpaperKind::Video
                    | crate::domain::library::catalog::WallpaperKind::We
            )
        {
            body = body.push(crate::frontend::audio_panel::playback_control(
                &monitor.target,
                monitor.paused,
                monitor.manual_paused,
                scale,
                palette,
            ));
        }
        container(body)
            .width(Length::Fill)
            .padding(Padding {
                top: 3.0 * scale,
                right: 10.0 * scale,
                bottom: 9.0 * scale,
                left: 10.0 * scale,
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
            .width(Length::Fixed(72.0 * scale)),
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
        let mute_message = if source_preview {
            Message::Effects(EffectsMsg::SrcMute(!muted))
        } else {
            Message::Audio(crate::frontend::audio_panel::AudioMsg::MonMute(
                monitor.target.clone(),
                !muted,
            ))
        };
        let volume_output = monitor.target.clone();
        let release_output = monitor.target.clone();
        let gauge = crate::frontend::ui::folio_slider(
            0.0,
            100.0,
            f64::from(volume),
            1.0,
            move |value| {
                if source_preview {
                    Message::Effects(EffectsMsg::SrcVolume(value as u32))
                } else {
                    Message::Audio(crate::frontend::audio_panel::AudioMsg::MonVolume(
                        volume_output.clone(),
                        value as u32,
                    ))
                }
            },
            if source_preview {
                Message::Noop
            } else {
                Message::Audio(crate::frontend::audio_panel::AudioMsg::MonVolumeRelease(
                    release_output,
                ))
            },
            palette,
        );
        row![
            label(tr("effects-audio-label"), 9.0, scale, with_alpha(palette.surface_text, 0.46))
                .width(Length::Fixed(72.0 * scale)),
            crate::frontend::ui::folio_action(
                if muted { tr("effects-muted") } else { tr("effects-sound") },
                !muted,
                Some(mute_message),
                Length::Fixed(74.0 * scale),
                scale * 0.9,
                palette,
            ),
            container(gauge).width(Length::Fill).height(Length::Fixed(27.0 * scale)),
            label(
                format!("{volume}%"),
                10.0,
                scale,
                if muted { with_alpha(palette.surface_text, 0.42) } else { palette.surface_text },
            )
            .width(Length::Fixed(42.0 * scale)),
        ]
        .spacing(7.0 * scale)
        .align_y(Alignment::Center)
        .into()
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
            .width(Length::Fixed(132.0 * scale)),
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
