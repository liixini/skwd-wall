use iced::widget::{column, container, row, scrollable, text};
use iced::{Alignment, Element, Length, Padding};

use crate::app::Message;
use crate::contracts::media::MediaKind;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{UI_FONT, folio_rule, label, with_alpha};
use crate::i18n::tr;

#[derive(Debug, Clone)]
pub enum AudioMsg {
    VolumeStep(i32),
    MonMute(String, bool),
    MonPause(String, bool),
    MonVolume(String, u32),
    MonVolumeRelease(String),
}

#[derive(Clone)]
pub struct AudioMon {
    pub name: String,
    pub label: String,
    pub source: String,
    pub wtype: MediaKind,
    pub mute: bool,
    pub volume: u32,
    pub shared: bool,
    pub paused: bool,
    pub manual_paused: bool,
}

impl AudioMon {
    pub fn has_audio_controls(&self) -> bool {
        self.wtype.has_audio_controls()
    }

    pub fn playing(&self) -> bool {
        self.has_audio_controls() && !self.paused && !self.mute && self.volume > 0
    }

    fn same_source(&self, other: &Self) -> bool {
        self.has_audio_controls()
            && self.wtype == other.wtype
            && !self.source.is_empty()
            && self.source == other.source
    }
}

pub fn align_shared_audio(mons: &mut [AudioMon]) {
    let mut visited = std::collections::HashSet::new();
    for index in 0..mons.len() {
        if visited.contains(&mons[index].name) {
            continue;
        }
        let members: Vec<usize> = (0..mons.len())
            .filter(|&candidate| mons[index].same_source(&mons[candidate]))
            .collect();
        if members.len() < 2 {
            continue;
        }
        let muted = members.iter().all(|&member| mons[member].mute);
        let volume = members
            .iter()
            .find(|&&member| !mons[member].mute)
            .map(|&member| mons[member].volume)
            .or_else(|| members.iter().map(|&member| mons[member].volume).max())
            .unwrap_or(100);
        for member in members {
            mons[member].mute = muted;
            mons[member].volume = volume;
            mons[member].shared = true;
            visited.insert(mons[member].name.clone());
        }
    }
}

pub fn playback_control<'a>(
    output: &str,
    paused: bool,
    manual: bool,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    row![
        label(tr("audio-wallpaper-label"), 9.0, scale, with_alpha(palette.surface_text, 0.46))
            .width(Length::Fixed(72.0 * scale)),
        crate::frontend::ui::folio_action(
            tr(if manual { "audio-resume-wallpaper" } else { "audio-pause-wallpaper" }),
            manual,
            Some(Message::Audio(AudioMsg::MonPause(output.to_string(), !manual))),
            Length::Fixed(132.0 * scale),
            scale * 0.9,
            palette,
        ),
        label(
            tr(if manual {
                "audio-wallpaper-paused"
            } else if paused {
                "audio-wallpaper-held"
            } else {
                "audio-wallpaper-playing"
            }),
            9.0,
            scale,
            with_alpha(palette.surface_text, 0.56)
        ),
    ]
    .spacing(7.0 * scale)
    .align_y(Alignment::Center)
    .into()
}

pub struct AudioPanel {
    pub mons: Vec<AudioMon>,
    pub open: crate::frontend::animation::Tween,
}

impl Default for AudioPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioPanel {
    pub fn new() -> Self {
        Self::new_with_motion(crate::frontend::animation::MotionProfile::default())
    }

    pub fn new_with_motion(motion: crate::frontend::animation::MotionProfile) -> Self {
        let mut open = motion.tween(0.0, crate::frontend::animation::MotionTier::Fast);
        open.retarget(1.0);
        Self { mons: Vec::new(), open }
    }

    pub fn set_motion_profile(&mut self, motion: crate::frontend::animation::MotionProfile) {
        motion.retime_tween(&mut self.open, crate::frontend::animation::MotionTier::Fast);
    }

    pub fn animating(&self) -> bool {
        !self.open.settled()
    }

    pub fn tick(&mut self, dt: f32) {
        self.open.tick(dt);
    }

    fn ease(&self) -> f32 {
        crate::frontend::animation::ease_out_cubic(self.open.x)
    }

    pub fn set_mon_mute(&mut self, name: &str, mute: bool) {
        if let Some(mon) = self.mons.iter_mut().find(|mon| mon.name == name) {
            mon.mute = mute;
        }
    }

    pub fn set_mon_volume(&mut self, name: &str, vol: u32) {
        if let Some(mon) = self.mons.iter_mut().find(|mon| mon.name == name) {
            mon.volume = vol.min(100);
        }
    }

    pub fn any_playing(&self) -> bool {
        self.mons.iter().any(AudioMon::playing)
    }

    pub fn audio_group_outputs(&self, name: &str) -> Vec<String> {
        let Some(source) = self.mons.iter().find(|mon| mon.name == name) else {
            return vec![name.to_string()];
        };
        let mut outputs: Vec<String> = self
            .mons
            .iter()
            .filter(|mon| source.same_source(mon))
            .map(|mon| mon.name.clone())
            .collect();
        if outputs.is_empty() {
            outputs.push(name.to_string());
        }
        outputs.sort();
        outputs
    }

    fn index<'a>(&'a self, scale: f32, palette: &'a Palette) -> Element<'a, Message> {
        let mut channels = column![].spacing(3.0 * scale);
        if self.mons.is_empty() {
            channels = channels.push(label(
                tr("audio-detecting-outputs"),
                10.0,
                scale,
                with_alpha(palette.surface_text, 0.5),
            ));
        } else {
            for (index, mon) in self.mons.iter().enumerate() {
                let available = mon.has_audio_controls();
                let state = if mon.paused {
                    tr("audio-state-paused")
                } else if mon.playing() {
                    tr("audio-state-sound")
                } else if available {
                    tr("audio-state-muted")
                } else {
                    tr("audio-state-none")
                };
                channels = channels.push(
                    container(
                        row![
                            label(
                                format!("{:02}", index + 1),
                                9.0,
                                scale,
                                if mon.playing() {
                                    palette.primary
                                } else {
                                    with_alpha(palette.primary, 0.5)
                                },
                            ),
                            column![
                                label(
                                    if mon.name == "*" {
                                        tr("audio-shared-outputs")
                                    } else {
                                        &mon.name
                                    },
                                    12.0,
                                    scale,
                                    palette.surface_text,
                                ),
                                label(
                                    state,
                                    crate::frontend::ui::TYPE_SMALL,
                                    scale,
                                    with_alpha(palette.surface_text, 0.46)
                                ),
                            ]
                            .spacing(2.0 * scale),
                            container(text("")).width(Length::Fill),
                            label(
                                if mon.playing() { "◆" } else { "◇" },
                                9.0,
                                scale,
                                if mon.playing() {
                                    palette.primary
                                } else {
                                    with_alpha(palette.surface_text, 0.34)
                                },
                            ),
                        ]
                        .spacing(10.0 * scale)
                        .align_y(Alignment::Center),
                    )
                    .padding([9.0 * scale, 10.0 * scale])
                    .style(move |_| {
                        crate::frontend::ui::box_style(
                            with_alpha(palette.surface_container, 0.45),
                            with_alpha(palette.outline, 0.42),
                        )
                    }),
                );
            }
        }

        let audible = self.mons.iter().filter(|mon| mon.playing()).count();
        let available = self.mons.iter().filter(|mon| mon.has_audio_controls()).count();
        let footer = column![
            folio_rule(palette),
            label(tr("audio-live-mix"), 9.0, scale, palette.primary),
            label(
                crate::i18n::audio_live_mix_summary(audible, available),
                9.5,
                scale,
                with_alpha(palette.surface_text, 0.48),
            ),
        ]
        .spacing(8.0 * scale);

        crate::frontend::ui::folio_index_shell(
            tr("audio-index-title"),
            tr("audio-index-desc"),
            vec![channels.into(), container(text("")).height(Length::Fill).into(), footer.into()],
            18.0,
            scale,
            palette,
        )
    }

    fn monitor_row<'a>(
        index: usize,
        mon: &'a AudioMon,
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        let display = if mon.name == "*" { tr("audio-shared-outputs") } else { mon.name.as_str() };
        let kind = match &mon.wtype {
            MediaKind::Video => tr("audio-kind-video"),
            MediaKind::WallpaperEngine => tr("audio-kind-we"),
            MediaKind::Static => tr("audio-kind-static"),
            MediaKind::Other(_) => tr("audio-kind-none"),
        };
        let playing = mon.playing();
        let state = if mon.paused {
            tr("audio-state-paused")
        } else if playing {
            tr("audio-row-sound")
        } else if mon.has_audio_controls() {
            tr("audio-row-muted")
        } else {
            tr("audio-row-none")
        };
        let summary = container(
            row![
                label(
                    format!("{:02}", index + 1),
                    9.0,
                    scale,
                    if playing { palette.primary } else { with_alpha(palette.primary, 0.5) },
                ),
                column![
                    label(display, 17.0, scale, palette.surface_text),
                    label(mon.label.clone(), 10.0, scale, with_alpha(palette.surface_text, 0.52)),
                    label(kind, 9.0, scale, with_alpha(palette.surface_text, 0.4)),
                ]
                .spacing(3.0 * scale),
                container(text("")).width(Length::Fill),
                column![
                    label(
                        state,
                        10.0,
                        scale,
                        if playing {
                            palette.primary
                        } else {
                            with_alpha(palette.surface_text, 0.56)
                        },
                    ),
                    label(
                        if mon.shared {
                            tr("audio-linked-source")
                        } else {
                            tr("audio-independent-source")
                        },
                        8.5,
                        scale,
                        with_alpha(palette.surface_text, 0.38),
                    ),
                ]
                .spacing(3.0 * scale)
                .align_x(Alignment::End),
            ]
            .spacing(12.0 * scale)
            .align_y(Alignment::Center),
        )
        .padding([9.0 * scale, 10.0 * scale])
        .style(move |_| {
            crate::frontend::ui::box_style(
                with_alpha(palette.surface_container, 0.58),
                with_alpha(if playing { palette.primary } else { palette.outline }, 0.52),
            )
        });

        let controls: Element<'a, Message> = if mon.has_audio_controls() {
            let output = mon.name.clone();
            let release = mon.name.clone();
            let slider = crate::frontend::ui::folio_slider(
                0.0,
                100.0,
                f64::from(mon.volume),
                1.0,
                move |value| Message::Audio(AudioMsg::MonVolume(output.clone(), value as u32)),
                Message::Audio(AudioMsg::MonVolumeRelease(release)),
                palette,
            );
            row![
                label(
                    tr("audio-channel-label"),
                    9.0,
                    scale,
                    with_alpha(palette.surface_text, 0.46)
                )
                .width(Length::Fixed(72.0 * scale)),
                crate::frontend::ui::folio_action(
                    if mon.mute { tr("audio-state-muted") } else { tr("audio-state-sound") },
                    !mon.mute,
                    Some(Message::Audio(AudioMsg::MonMute(mon.name.clone(), !mon.mute))),
                    Length::Fixed(76.0 * scale),
                    scale * 0.9,
                    palette,
                ),
                container(slider).width(Length::Fill).height(Length::Fixed(27.0 * scale)),
                label(
                    format!("{}%", mon.volume),
                    10.0,
                    scale,
                    if playing {
                        palette.surface_text
                    } else {
                        with_alpha(palette.surface_text, 0.42)
                    },
                )
                .width(Length::Fixed(42.0 * scale)),
            ]
            .spacing(7.0 * scale)
            .align_y(Alignment::Center)
            .into()
        } else {
            row![
                label(
                    tr("audio-channel-label"),
                    9.0,
                    scale,
                    with_alpha(palette.surface_text, 0.46)
                )
                .width(Length::Fixed(72.0 * scale)),
                label(
                    tr("audio-no-channel"),
                    crate::frontend::ui::TYPE_SMALL,
                    scale,
                    with_alpha(palette.surface_text, 0.46),
                ),
            ]
            .spacing(7.0 * scale)
            .align_y(Alignment::Center)
            .into()
        };

        let mut body = column![summary, controls].spacing(7.0 * scale);
        if mon.has_audio_controls() {
            body = body.push(playback_control(
                &mon.name,
                mon.paused,
                mon.manual_paused,
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
                    with_alpha(palette.surface_container, 0.48),
                    with_alpha(if playing { palette.primary } else { palette.outline }, 0.5),
                )
            })
            .into()
    }

    fn reading_surface<'a>(&'a self, scale: f32, palette: &'a Palette) -> Element<'a, Message> {
        let total = self.mons.len();
        let available = self.mons.iter().filter(|mon| mon.has_audio_controls()).count();
        let sounding = self.mons.iter().filter(|mon| mon.playing()).count();
        let header = column![
            row![
                column![
                    label(
                        tr("audio-outputs-kicker"),
                        9.0,
                        scale,
                        with_alpha(palette.primary, 0.84)
                    ),
                    text(tr("audio-mixer-heading"))
                        .font(UI_FONT)
                        .size(34.0 * scale.clamp(0.88, 1.08))
                        .line_height(iced::widget::text::LineHeight::Relative(1.0))
                        .color(palette.surface_text),
                ]
                .spacing(6.0 * scale),
                container(text("")).width(Length::Fill),
                column![
                    label(
                        tr("audio-mixer-desc"),
                        10.0,
                        scale,
                        with_alpha(palette.surface_text, 0.56),
                    ),
                    label(
                        crate::i18n::audio_outputs_summary(total, available, sounding),
                        9.0,
                        scale,
                        with_alpha(palette.surface_text, 0.42),
                    ),
                ]
                .spacing(6.0 * scale)
                .align_x(Alignment::End),
            ]
            .align_y(Alignment::Center),
            folio_rule(palette),
        ]
        .spacing(10.0 * scale);

        let section = row![
            label("01", 9.0, scale, palette.primary),
            column![
                label(tr("audio-output-channels"), 14.0, scale, palette.surface_text),
                label(
                    tr("audio-output-channels-desc"),
                    10.0,
                    scale,
                    with_alpha(palette.surface_text, 0.52),
                ),
            ]
            .spacing(4.0 * scale),
        ]
        .spacing(9.0 * scale)
        .align_y(Alignment::Start);

        let mut monitors = column![].spacing(9.0 * scale);
        if self.mons.is_empty() {
            monitors = monitors.push(
                container(
                    column![
                        label(tr("audio-looking-displays"), 14.0, scale, palette.surface_text),
                        label(
                            tr("audio-looking-displays-desc"),
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
            for (index, mon) in self.mons.iter().enumerate() {
                monitors = monitors.push(Self::monitor_row(index, mon, scale, palette));
            }
        }
        let channels = scrollable(monitors)
            .id(iced::widget::Id::new("audio-output-channels"))
            .direction(iced::widget::scrollable::Direction::Vertical(
                iced::widget::scrollable::Scrollbar::new().width(3.0).scroller_width(3.0),
            ))
            .style(crate::frontend::ui::scroll_style(palette.primary))
            .width(Length::Fill)
            .height(Length::Fill);

        let footer = row![
            column![
                row![
                    label("02", 9.0, scale, palette.primary),
                    label(tr("audio-live-state"), 13.0, scale, palette.surface_text),
                ]
                .spacing(9.0 * scale),
                label(
                    if sounding == 0 { tr("audio-live-none") } else { tr("audio-live-playing") },
                    9.5,
                    scale,
                    with_alpha(palette.surface_text, 0.5),
                ),
            ]
            .spacing(4.0 * scale),
            container(text("")).width(Length::Fill),
            crate::frontend::ui::folio_action(
                tr("audio-close-mixer"),
                false,
                Some(Message::ToggleAudioPanel),
                Length::Fixed(150.0 * scale),
                scale,
                palette,
            ),
        ]
        .align_y(Alignment::Center);

        container(
            column![header, section, channels, folio_rule(palette), footer].spacing(12.0 * scale),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(crate::frontend::ui::folio_scroll_padding(23.0, 27.0, scale))
        .into()
    }

    pub fn view<'a>(
        &'a self,
        viewport: (f32, f32),
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        let masthead = crate::frontend::ui::folio_masthead(
            tr("audio-masthead").to_string(),
            Message::ToggleAudioPanel,
            scale,
            palette,
        );
        crate::frontend::ui::folio_sheet(
            masthead,
            self.index(scale, palette),
            self.reading_surface(scale, palette),
            Message::Noop,
            viewport,
            scale,
            self.ease(),
            crate::frontend::ui::FOLIO_INDEX_WIDTH,
            palette,
        )
    }
}

pub fn label_for(wtype: &MediaKind, path: &str, we_id: &str) -> String {
    match wtype {
        MediaKind::Video => std::path::Path::new(path).file_name().map_or_else(
            || tr("audio-source-video").to_owned(),
            |stem| stem.to_string_lossy().into_owned(),
        ),
        MediaKind::WallpaperEngine => {
            if we_id.is_empty() {
                tr("audio-source-wallpaper-engine").to_owned()
            } else {
                crate::i18n::tr_args!("audio-source-we-id", id => we_id)
            }
        }
        MediaKind::Static => tr("audio-source-static-image").to_owned(),
        MediaKind::Other(_) => "-".to_string(),
    }
}
