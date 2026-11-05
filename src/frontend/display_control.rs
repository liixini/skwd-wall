use iced::widget::container;
use iced::{Alignment, Element, Length};

use crate::app::Message;
use crate::frontend::effects::EffectsMsg;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{label, row, with_alpha};
use crate::i18n::tr;

#[derive(Debug, Clone)]
pub enum DisplayControl {
    Colours { target: String, active: bool },
    Audio { target: String, muted: bool, volume: u32, source_preview: bool },
    Playback { target: String, paused: bool, manual: bool },
}

impl DisplayControl {
    pub fn label(&self) -> &'static str {
        tr(match self {
            Self::Colours { .. } => "effects-colours-label",
            Self::Audio { .. } => "effects-audio-label",
            Self::Playback { .. } => "audio-wallpaper-label",
        })
    }

    pub fn summary(&self) -> String {
        match self {
            Self::Colours { active, .. } => {
                tr(if *active { "effects-colours-source" } else { "effects-colours-use" }).into()
            }
            Self::Audio { muted, volume, .. } => {
                format!("{} {volume}%", tr(if *muted { "effects-muted" } else { "effects-sound" }))
            }
            Self::Playback { paused, manual, .. } => tr(if *manual {
                "audio-wallpaper-paused"
            } else if *paused {
                "audio-wallpaper-held"
            } else {
                "audio-wallpaper-playing"
            })
            .into(),
        }
    }

    pub fn view(self, scale: f32, palette: &Palette) -> Element<'_, Message> {
        match self {
            Self::Colours { target, active } => {
                let (source, use_for) = (tr("effects-colours-source"), tr("effects-colours-use"));
                let action_scale = scale * 0.9;
                let width = crate::frontend::ui::folio_action_width(source, action_scale)
                    .max(crate::frontend::ui::folio_action_width(use_for, action_scale));
                row![
                    label(
                        tr("effects-colours-label"),
                        9.0,
                        scale,
                        with_alpha(palette.surface_text, 0.46)
                    )
                    .width(Length::Fixed(72.0 * scale)),
                    crate::frontend::ui::folio_action(
                        if active { source } else { use_for },
                        active,
                        Some(Message::Effects(EffectsMsg::MonitorTheme(target))),
                        Length::Fixed(width),
                        action_scale,
                        palette
                    ),
                ]
                .spacing(6.0 * scale)
                .align_y(Alignment::Center)
                .into()
            }
            Self::Audio { target, muted, volume, source_preview } => {
                audio_row(target, muted, volume, source_preview, scale, palette)
            }
            Self::Playback { target, paused, manual } => {
                crate::frontend::audio_panel::playback_control(
                    &target, paused, manual, scale, palette,
                )
            }
        }
    }
}

fn audio_row(
    target: String,
    muted: bool,
    volume: u32,
    source_preview: bool,
    scale: f32,
    palette: &Palette,
) -> Element<'_, Message> {
    let mute_message = if source_preview {
        Message::Effects(EffectsMsg::SrcMute(!muted))
    } else {
        Message::Audio(crate::frontend::audio_panel::AudioMsg::MonMute(target.clone(), !muted))
    };
    let volume_output = target.clone();
    let release_output = target;
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
            Message::Audio(crate::frontend::audio_panel::AudioMsg::MonVolumeRelease(release_output))
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
