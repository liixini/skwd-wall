#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use iced::widget::canvas::{self, Frame, Path, Stroke};
use iced::widget::{
    button, column, container, image, mouse_area, scrollable, stack, text, text_input,
};
use iced::{Alignment, Background, Color, Element, Length, Padding, Point, Rectangle};

use crate::app::Message;
use crate::contracts::settings::SettingsSource;
use crate::frontend::animation::{MotionProfile, Tween};
use crate::frontend::components::with_alpha;
use crate::frontend::theme::Palette;
use crate::frontend::ui::calm::{self, tint};
use crate::frontend::ui::{
    FOLIO_RULE_ALPHA, TWISTY_CLOSED, TWISTY_OPEN, TYPE_SMALL, end, folio_horizontal_rule, label,
    legible_type_scale, logical_padding, mirror_x, motion_slot, row, start,
};
use crate::i18n::tr;

use super::super::SETTINGS_MOTION;
use super::super::{
    ActionId, Card, Control, HOME_LEAD, KEYS_SHEET, KeysCtx, Place, PlaceSource, Row, Section,
    SectionId, SettingsFocus, SettingsMsg, SettingsSearchResult, place_keys, place_title,
};
use super::appearance::{INDEX, METRICS, navigation_icon};
use super::control;
use super::search_page;
use super::sections::{self, Block, CalmCtx, Paint, SURFACE};

const GLYPH_FADE: f32 = 1.0;

pub struct SourceCtx<'a> {
    pub layout_premade: Option<usize>,
    pub config: &'a dyn SettingsSource,
    pub values: &'a HashMap<String, String>,
    pub themes: &'a [String],
    pub folders: &'a [String],
    pub backends: &'a [String],
    pub outputs: &'a [String],
    pub output_statuses: &'a [crate::contracts::daemon::OutputStatus],
    pub output_previews: &'a HashMap<String, String>,
    pub library_watch: Option<&'a crate::contracts::daemon::LibraryWatchStatus>,
    pub app_themes: Option<&'a crate::contracts::daemon::AppThemesResult>,
    pub playback: Option<&'a crate::contracts::daemon::PlaybackStatus>,
    pub weather: Option<&'a crate::contracts::daemon::WeatherStatus>,
    pub weather_pending: bool,
    pub playlists: &'a [crate::contracts::playlists::Playlist],
    pub sddm_themes: &'a [String],
    pub drawing: Option<&'a crate::contracts::settings::drawing::Status>,
    pub drawing_selection: Option<&'a str>,
    pub drawing_view_count: usize,
    pub drawing_library: bool,
    pub connected: bool,
    pub thumbnail_task: Option<&'a crate::contracts::daemon::TaskStatus>,
    pub drawing_task: Option<&'a crate::contracts::daemon::TaskStatus>,
    pub analysis: String,
}

impl SourceCtx<'_> {
    fn motion(&self) -> MotionProfile {
        MotionProfile::new(
            self.config.motion_fast_ms(),
            self.config.motion_standard_ms(),
            self.config.motion_slow_ms(),
        )
    }

    fn place_source(&self) -> PlaceSource<'_> {
        PlaceSource {
            cfg: self.config,
            themes: self.themes,
            folders: self.folders,
            analysis: &self.analysis,
            backends: self.backends,
            runtime: super::super::RuntimeStatus {
                outputs: self.outputs,
                statuses: self.output_statuses,
                previews: Some(self.output_previews),
                library_watch: self.library_watch,
                playback: self.playback,
                app_themes: self.app_themes,
                weather: self.weather,
                weather_pending: self.weather_pending,
                playlists: self.playlists,
                sddm_themes: self.sddm_themes,
                drawing: self.drawing,
                drawing_selection: self.drawing_selection,
                drawing_view_count: self.drawing_view_count,
                drawing_job: super::super::DrawingJob {
                    library: self.drawing_library,
                    running: self.drawing_task.is_some_and(|task| task.state.is_active()),
                },
                connected: self.connected,
                layout_premade: self.layout_premade,
            },
        }
    }

    pub(crate) fn cards(&self, category: &str) -> Vec<(Card, Vec<Row>)> {
        self.place_source().cards(category)
    }

    pub(crate) fn place(&self, key: &str) -> Place {
        let mut place = self.place_source().place(key);
        let rows = place.sections.iter_mut().flat_map(|section| &mut section.groups);
        for row in rows.flat_map(|group| &mut group.rows) {
            let super::super::Control::ActionBtn { id, label } = &mut row.control else {
                continue;
            };
            let (task, stop, skipped) = match id {
                ActionId::CaptureWeThumbnails => {
                    (self.thumbnail_task, tr("settings-performance-capture-we-stop"), 0)
                }
                ActionId::PrepareAllDrawings => (
                    self.drawing_task,
                    tr("settings-drawing-library-stop"),
                    self.drawing.map_or(0, |status| status.failed),
                ),
                _ => continue,
            };
            let Some(task) = task else {
                continue;
            };
            row.desc = task_progress(task);
            if skipped > 0 {
                row.desc = format!(
                    "{} · {}",
                    row.desc,
                    crate::i18n::tr_args!(super::super::tabs::DRAWING_SKIPPED, count => skipped)
                );
            }
            if task.state.is_active() {
                *label = stop.into();
            }
        }
        place
    }
}

fn task_progress(task: &crate::contracts::daemon::TaskStatus) -> String {
    use crate::contracts::daemon::TaskState;

    let state = match &task.state {
        TaskState::Running => tr("filter-bar-state-running"),
        TaskState::Paused => tr("filter-bar-state-paused"),
        TaskState::Completed => tr("filter-bar-state-done"),
        TaskState::Failed => tr("filter-bar-state-failed"),
        TaskState::Cancelled => tr("filter-bar-state-stopped"),
        TaskState::Other(state) => state.as_str(),
    };
    let progress = format!("{state} · {} / {}", task.progress, task.total);
    if task.detail.is_empty() { progress } else { format!("{progress} · {}", task.detail) }
}

#[derive(Clone, Copy)]
pub struct FocusCtx<'a> {
    pub keyboard_focus: SettingsFocus,
    pub focused_control: usize,
    pub focused_choice: Option<usize>,
    pub expanded_details: &'a HashSet<String>,
    pub bar_reveals: &'a HashMap<String, Tween>,
    pub active_input: Option<&'a str>,
    pub armed: Option<&'a ActionId>,
}

impl<'a> FocusCtx<'a> {
    pub(super) fn lit(&self, area: SettingsFocus, index: usize) -> bool {
        self.keyboard_focus == area && self.focused_control == index
    }

    fn on(&self, area: SettingsFocus, index: usize) -> (bool, Option<usize>) {
        let focused = self.lit(area, index);
        (focused, if focused { self.focused_choice } else { None })
    }

    pub(super) fn reveals(&self) -> control::Reveals<'a> {
        control::Reveals { bars: self.bar_reveals }
    }

    fn bar(&self, id: &str) -> (bool, f32) {
        self.bar_reveals.get(id).map_or((false, 0.0), |reveal| (reveal.target > 0.5, reveal.x))
    }
}

#[derive(Clone, Copy)]
pub struct ChromeCtx<'a> {
    pub viewport: (f32, f32),
    pub scale: f32,
    pub entrance: f32,
    pub palette: &'a Palette,
}

pub struct ProcessPickerView {
    pub targets: Vec<String>,
    pub running: Vec<String>,
    pub query: String,
    pub manual: String,
    pub scroll: f32,
}

#[derive(Clone, Copy)]
pub struct ProfilesCtx<'a> {
    pub selected: usize,
    pub filter: &'a str,
}

#[derive(Clone, Copy)]
pub struct FoldersCtx<'a> {
    pub drafts: &'a HashMap<&'static str, String>,
    pub undo: Option<(crate::contracts::settings::FolderKind, &'a str)>,
}

pub struct WorkbenchInput<'a> {
    pub layouts: super::super::LayoutsUi,
    pub source: SourceCtx<'a>,
    pub focus: FocusCtx<'a>,
    pub chrome: ChromeCtx<'a>,
    pub calm: CalmCtx,
    pub profiles: ProfilesCtx<'a>,
    pub folders: FoldersCtx<'a>,
    pub preview_path: &'a str,
    pub transition_preview: Option<iced::advanced::image::Handle>,
    pub tab_transition: f32,
    pub page_transition: f32,
    pub emblem_transition: f32,
    pub previous: Option<&'a super::super::PageSnapshot>,
    pub tab: &'a str,
    pub search_open: bool,
    pub search_query: &'a str,
    pub search_results: &'a [SettingsSearchResult],
    pub search_total: usize,
    pub search_at: Option<usize>,
    pub keys: KeysCtx<'a>,
    pub process_picker: Option<ProcessPickerView>,
    pub weather_review: Option<super::weather_review::WeatherReviewView>,
}

pub fn settings_workbench<'a>(input: WorkbenchInput<'a>) -> Element<'a, Message> {
    let WorkbenchInput {
        layouts,
        source,
        focus,
        chrome,
        calm,
        profiles,
        folders,
        preview_path,
        transition_preview,
        tab_transition,
        page_transition,
        emblem_transition,
        previous,
        tab,
        search_open,
        search_query,
        search_results,
        search_total,
        search_at,
        keys,
        process_picker,
        weather_review,
    } = input;
    let ChromeCtx { viewport, scale, entrance, palette } = chrome;
    let scale = super::appearance::sheet_scale(viewport, scale);
    let place = source.place(tab);
    let reveal = crate::frontend::animation::smoothstep(entrance.clamp(0.0, 1.0));
    let (panel_width, panel_height) = super::appearance::sheet_size(viewport, scale);
    let surface_reveal = (entrance * SETTINGS_MOTION.open_ms / SETTINGS_MOTION.surface_ms).min(1.0);
    let trace_reveal = SETTINGS_MOTION
        .stroke
        .at((entrance * SETTINGS_MOTION.open_ms / SETTINGS_MOTION.trace_ms).min(1.0));
    let places = place_keys().map(|key| (key, place_title(key)));
    let overview = super::super::home::overview(&source.place_source());
    let index = navigation(&NavigationInput {
        places: &places,
        overview: &overview,
        selected: place.key,
        note: if search_open { tr(search_page::INDEX_NOTE) } else { place.index_note },
        previous: previous.map(|old| old.place.key),
        travel: SETTINGS_MOTION.roll.at(tab_transition),
        scale,
        palette,
        entrance,
        search_open,
        search_query,
    });
    let pages = PageInput { source: &source, focus, profiles, folders, keys };
    let paint = Paint {
        values: source.values,
        focus,
        calm,
        scale,
        palette,
        motion: source.motion(),
        body: panel_width - METRICS.index * scale - reading_inset(scale),
        dim: false,
        reading: sections::reading_height(panel_height, scale),
        owner: 0,
        undivided: false,
        trim: crate::frontend::settings::Trim::None,
    };
    let found = search_open.then_some(search_page::Found {
        query: search_query,
        results: search_results,
        total: search_total,
        at: search_at,
    });
    let reading = reading_surface(ReadingInput {
        found,
        layouts,
        place,
        pages,
        preview: transition_preview,
        entrance,
        transition: SETTINGS_MOTION.roll.at(page_transition),
        emblem_transition,
        previous,
        paint,
    });
    let atmosphere: Element<'a, Message> = if preview_path.is_empty() {
        iced::widget::Space::new().width(Length::Fill).height(Length::Fill).into()
    } else {
        image(image::Handle::from_path(preview_path))
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(iced::ContentFit::Cover)
            .opacity(METRICS.atmosphere * surface_reveal)
            .into()
    };
    let blueprint = iced::widget::canvas(Blueprint { palette: *palette, fade: surface_reveal })
        .width(Length::Fill)
        .height(Length::Fill);
    let page = container(stack![atmosphere, blueprint, reading])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| super::appearance::surface(palette, surface_reveal));
    let body = mouse_area(
        row![container(index).width(Length::Fixed(METRICS.index * scale)), page]
            .height(Length::Fill),
    )
    .on_press(Message::Settings(SettingsMsg::ClosePop));
    let panel = container(stack![
        body,
        iced::widget::canvas(super::appearance::Ornament::Frame {
            palette: *palette,
            progress: trace_reveal,
            index_width: METRICS.index * scale,
        })
        .width(Length::Fill)
        .height(Length::Fill),
    ])
    .width(Length::Fixed(panel_width))
    .height(Length::Fixed(panel_height))
    .clip(true);
    let mut layers = stack![
        mouse_area(
            container(text(""))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| crate::frontend::ui::folio_scrim_style(reveal)),
        )
        .on_press(Message::ToggleSettings),
        container(iced::widget::opaque(panel))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    ];
    if let Some(picker) = process_picker {
        layers = layers.push(process_picker_layer(picker, scale, palette, reveal));
    }
    if let Some(review) = weather_review {
        layers = layers.push(super::weather_review::layer(review, scale, palette, reveal));
    }
    layers.into()
}

fn process_picker_layer(
    picker: ProcessPickerView,
    scale: f32,
    palette: &Palette,
    reveal: f32,
) -> Element<'_, Message> {
    let ProcessPickerView { targets, running, query, manual, scroll } = picker;
    let rail_count = running.len().max(2);
    let rail_active = (scroll.clamp(0.0, 1.0) * (rail_count - 1) as f32).round() as usize;
    let target_rows: Element<'_, Message> = if targets.is_empty() {
        container(label(
            tr("settings-playback-targets-empty"),
            TYPE_SMALL,
            scale,
            with_alpha(palette.surface_text, 0.58),
        ))
        .width(Length::Fill)
        .padding([12.0 * scale, 14.0 * scale])
        .style(move |_| {
            crate::frontend::ui::box_style(
                with_alpha(palette.surface_container, 0.46),
                with_alpha(palette.outline, 0.3),
            )
        })
        .into()
    } else {
        targets
            .into_iter()
            .fold(column![].spacing(6.0 * scale), |rows, target| {
                let remove = crate::frontend::ui::folio_destructive_action(
                    tr("settings-playback-target-remove"),
                    false,
                    Some(Message::Settings(SettingsMsg::ProcessPickerRemove(target.clone()))),
                    Length::Fixed(80.0 * scale),
                    scale * 0.76,
                    palette,
                );
                rows.push(
                    container(
                        row![
                            label(
                                target,
                                14.0,
                                scale,
                                with_alpha(palette.surface_text, GLYPH_FADE)
                            ),
                            container(text("")).width(Length::Fill),
                            remove,
                        ]
                        .align_y(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .padding([8.0 * scale, 10.0 * scale])
                    .style(move |_| {
                        crate::frontend::ui::box_style(
                            with_alpha(palette.surface_container, 0.7),
                            with_alpha(palette.primary, 0.46),
                        )
                    }),
                )
            })
            .into()
    };
    let running_rows: Element<'_, Message> = if running.is_empty() {
        container(label(
            tr("settings-playback-running-empty"),
            TYPE_SMALL,
            scale,
            with_alpha(palette.surface_text, 0.58),
        ))
        .width(Length::Fill)
        .padding([12.0 * scale, 14.0 * scale])
        .style(move |_| {
            crate::frontend::ui::box_style(
                with_alpha(palette.surface_container, 0.46),
                with_alpha(palette.outline, 0.3),
            )
        })
        .into()
    } else {
        running
            .into_iter()
            .fold(column![].spacing(6.0 * scale), |rows, process| {
                let add = crate::frontend::ui::folio_action(
                    tr("settings-playback-target-add"),
                    true,
                    Some(Message::Settings(SettingsMsg::ProcessPickerAdd(process.clone()))),
                    Length::Fixed(68.0 * scale),
                    scale * 0.76,
                    palette,
                );
                rows.push(
                    container(
                        row![
                            label(
                                process,
                                14.0,
                                scale,
                                with_alpha(palette.surface_text, GLYPH_FADE)
                            ),
                            container(text("")).width(Length::Fill),
                            add,
                        ]
                        .align_y(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .padding([8.0 * scale, 10.0 * scale])
                    .style(move |_| {
                        crate::frontend::ui::box_style(
                            with_alpha(palette.surface_container, 0.7),
                            with_alpha(palette.outline, 0.38),
                        )
                    }),
                )
            })
            .into()
    };
    let close = crate::frontend::ui::folio_action(
        tr("settings-playback-target-close"),
        true,
        Some(Message::Settings(SettingsMsg::ProcessPickerClose)),
        Length::Fixed(76.0 * scale),
        scale * 0.82,
        palette,
    );
    let body = column![
        row![
            label(
                tr("settings-playback-target-dialog-title"),
                20.0,
                scale,
                with_alpha(palette.surface_text, GLYPH_FADE),
            ),
            container(text("")).width(Length::Fill),
            label(
                tr("settings-playback-process-enabled"),
                TYPE_SMALL,
                scale,
                with_alpha(palette.primary, GLYPH_FADE),
            ),
        ]
        .align_y(Alignment::Center),
        label(
            tr("settings-playback-target-dialog-desc"),
            TYPE_SMALL,
            scale,
            with_alpha(palette.surface_text, 0.65),
        )
        .line_height(iced::widget::text::LineHeight::Relative(1.35)),
        crate::frontend::ui::folio_horizontal_rule(with_alpha(palette.outline, 0.46)),
        label(
            tr("settings-playback-targets"),
            12.0,
            scale,
            with_alpha(palette.primary, GLYPH_FADE)
        ),
        target_rows,
        label(
            tr("settings-playback-running"),
            12.0,
            scale,
            with_alpha(palette.primary, GLYPH_FADE)
        ),
        row![
            label("⌕", 15.0, scale, with_alpha(palette.primary, GLYPH_FADE)),
            text_input(tr("browser-search-placeholder"), &query)
                .font(crate::frontend::ui::UI_FONT)
                .on_input(|query| Message::Settings(SettingsMsg::ProcessPickerSearch(query)))
                .size(12.0 * crate::frontend::ui::legible_type_scale(scale))
                .padding([8.0 * scale, 9.0 * scale])
                .width(Length::Fill)
                .style(move |_theme, _status| {
                    crate::frontend::ui::workbench_input_style(
                        palette.surface_text,
                        palette.primary,
                    )
                }),
        ]
        .spacing(8.0 * scale)
        .align_y(Alignment::Center),
        row![
            text_input(tr("settings-playback-target-manual-placeholder"), &manual)
                .font(crate::frontend::ui::UI_FONT)
                .on_input(|process| Message::Settings(SettingsMsg::ProcessPickerManualInput(
                    process
                )))
                .on_submit(Message::Settings(SettingsMsg::ProcessPickerManualAdd))
                .size(12.0 * crate::frontend::ui::legible_type_scale(scale))
                .padding([8.0 * scale, 9.0 * scale])
                .width(Length::Fill)
                .style(move |_theme, _status| {
                    crate::frontend::ui::workbench_input_style(
                        palette.surface_text,
                        palette.primary,
                    )
                }),
            crate::frontend::ui::folio_action(
                tr("settings-playback-target-add"),
                true,
                Some(Message::Settings(SettingsMsg::ProcessPickerManualAdd)),
                Length::Fixed(68.0 * scale),
                scale * 0.76,
                palette,
            ),
        ]
        .spacing(8.0 * scale)
        .align_y(Alignment::Center),
        row![
            scrollable(running_rows)
                .height(Length::Fixed(142.0 * scale))
                .width(Length::Fill)
                .direction(iced::widget::scrollable::Direction::Vertical(
                    iced::widget::scrollable::Scrollbar::new().width(0.0).scroller_width(0.0),
                ))
                .on_scroll(|viewport| {
                    Message::Settings(SettingsMsg::ProcessPickerScroll(
                        viewport.relative_offset().y,
                    ))
                }),
            iced::widget::canvas(IndexRail {
                active: rail_active as f32,
                count: rail_count,
                notches: false,
                palette: *palette,
                fade: GLYPH_FADE,
            })
            .width(Length::Fixed(16.0 * scale))
            .height(Length::Fixed(142.0 * scale)),
        ]
        .spacing(8.0 * scale),
        crate::frontend::ui::folio_horizontal_rule(with_alpha(palette.outline, 0.46)),
        row![container(text("")).width(Length::Fill), close].align_y(Alignment::Center),
    ]
    .spacing(12.0 * scale);
    let panel = mouse_area(
        container(body)
            .width(Length::Fixed(500.0 * scale.max(0.9)))
            .padding([20.0 * scale, 22.0 * scale])
            .style(move |_| crate::frontend::ui::folio_sheet_panel_style(palette, reveal)),
    )
    .on_press(Message::Noop);
    mouse_area(
        container(panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_| crate::frontend::ui::folio_scrim_style(0.6 * reveal)),
    )
    .on_press(Message::Settings(SettingsMsg::ProcessPickerClose))
    .into()
}

pub fn picker_layout_workbench<'a>(
    source: &SourceCtx<'a>,
    focus: FocusCtx<'a>,
    chrome: ChromeCtx<'a>,
) -> Element<'a, Message> {
    const LAYOUT_SECTION: usize = 1;
    let ChromeCtx { viewport, scale, entrance, palette } = chrome;
    let motion = source.motion();
    let rows = source
        .cards("picker")
        .into_iter()
        .nth(LAYOUT_SECTION)
        .map_or_else(Vec::new, |(_, rows)| rows);
    let reveal = crate::frontend::animation::smoothstep(entrance.clamp(0.0, 1.0));
    let panel_width = super::super::picker_layout_studio_width(viewport.0, scale);
    let panel_height = (viewport.1 - 36.0 * scale).max(360.0);

    let mut pinned = column![].spacing(10.0 * scale);
    let mut controls = column![].spacing(13.0 * scale);
    for (index, setting) in rows.into_iter().enumerate() {
        let is_pinned = setting.title == tr("settings-selector-display-mode-label");
        let field = picker_layout_field(
            setting,
            index,
            source.values,
            focus,
            panel_width - 42.0 * scale,
            scale,
            palette,
            GLYPH_FADE,
            motion,
        );
        if is_pinned {
            pinned = pinned.push(field);
        } else {
            controls = controls.push(field);
        }
    }

    let mode = tr(match source.config.display_mode().as_str() {
        "grid" | "wall" => "settings-selector-mode-wall",
        "hex" => "settings-selector-mode-hex",
        "sandy" | "nova" => "settings-selector-mode-sandy",
        "hand" => "settings-selector-mode-hand",
        "depth" => "settings-selector-mode-depth",
        "collection" => "settings-selector-mode-collection",
        _ => "settings-selector-mode-slices",
    });
    let page_actions = row![
        crate::frontend::ui::folio_action(
            tr("settings-studio-back"),
            false,
            Some(Message::Settings(SettingsMsg::LeaveLayoutStudio)),
            Length::Fixed(104.0 * scale.max(1.0)),
            scale,
            palette,
        ),
        container(text("")).width(Length::Fill),
        crate::frontend::ui::folio_action(
            "×",
            false,
            Some(Message::ToggleSettings),
            Length::Shrink,
            scale,
            palette,
        ),
    ]
    .align_y(Alignment::Center);
    let intro = column![
        page_actions,
        label(
            crate::i18n::tr_args!("settings-studio-title", mode => mode),
            31.0,
            scale,
            with_alpha(palette.surface_text, GLYPH_FADE)
        ),
        label(
            tr("settings-studio-intro"),
            TYPE_SMALL,
            scale,
            with_alpha(palette.surface_text, 0.58),
        )
        .line_height(iced::widget::text::LineHeight::Relative(1.35)),
    ]
    .spacing(7.0 * scale);
    let scroll = scrollable(controls.padding(Padding {
        top: 1.0,
        right: 10.0 * scale,
        bottom: 20.0 * scale,
        left: 0.0,
    }))
    .direction(crate::frontend::ui::thin_vbar())
    .style(crate::frontend::ui::outline_scroll(palette.outline))
    .height(Length::Fill);
    let page = column![intro, pinned, scroll]
        .spacing(15.0 * scale)
        .height(Length::Fill)
        .padding([17.0 * scale, 19.0 * scale]);
    let surface = stack![
        container(text("")).width(Length::Fill).height(Length::Fill).style(move |_| {
            crate::frontend::ui::bg_style(Background::Color(with_alpha(
                palette.surface,
                0.985 * reveal,
            )))
        }),
        iced::widget::canvas(Blueprint { palette: *palette, fade: 0.55 * reveal })
            .width(Length::Fill)
            .height(Length::Fill),
        page,
    ]
    .width(Length::Fill)
    .height(Length::Fill);
    let panel = mouse_area(
        container(surface)
            .width(Length::Fixed(panel_width))
            .height(Length::Fixed(panel_height))
            .clip(true)
            .style(move |_| {
                let mut style = crate::frontend::ui::box_style(
                    with_alpha(palette.surface, 0.99 * reveal),
                    with_alpha(palette.outline, 0.72 * reveal),
                );
                style.shadow = iced::Shadow {
                    color: with_alpha(palette.shadow, 0.4 * reveal),
                    offset: iced::Vector::new(8.0, 14.0),
                    blur_radius: 36.0,
                };
                style
            }),
    )
    .on_press(Message::Noop);
    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(start())
        .align_y(iced::alignment::Vertical::Center)
        .padding(logical_padding(0.0, 0.0, 0.0, 18.0 * scale))
        .into()
}

fn picker_layout_field<'a>(
    setting: Row,
    index: usize,
    values: &'a HashMap<String, String>,
    focus: FocusCtx<'_>,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'a, Message> {
    if setting.control.is_inline_editor() {
        let (focused, focused_choice) = focus.on(SettingsFocus::Controls, index);
        return compact_inline_field(
            setting,
            index,
            values,
            focused,
            focused_choice,
            available_width,
            scale,
            palette,
            fade,
            motion,
        );
    }
    if setting.control.is_group_editor() {
        return compact_group_field(
            setting,
            index,
            values,
            focus,
            available_width,
            scale,
            palette,
            fade,
            motion,
        );
    }
    let (focused, focused_choice) = focus.on(SettingsFocus::Controls, index);
    if setting.control.is_compact_action() {
        return compact_action_field(ActionField {
            setting,
            index,
            focused,
            available_width,
            scale,
            palette,
            fade,
            motion,
        });
    }
    let locked = focus.active_input.is_some_and(|key| match &setting.control {
        Control::Number { key: control_key, .. }
        | Control::TextField { key: control_key, .. }
        | Control::Resolution { key: control_key, .. } => control_key == key,
        _ => false,
    });
    let title = if setting.title.is_empty() {
        tr("settings-selector-saved-styles-label").to_string()
    } else {
        setting.title
    };
    let widget = control::widget(
        setting.control,
        values,
        focused,
        focused_choice,
        available_width,
        scale * 0.94,
        palette,
        fade,
        focus.armed,
        motion,
        Some(focus.reveals()),
    );
    let line = if focused || locked { palette.primary } else { palette.outline };
    mouse_area(
        container(
            column![
                container(text(""))
                    .width(Length::Fill)
                    .height(Length::Fixed(if focused || locked { 2.0 } else { 1.0 }))
                    .style(move |_| crate::frontend::ui::bg_style(Background::Color(with_alpha(
                        line,
                        if focused || locked { 0.9 } else { 0.42 },
                    )))),
                label(title, 13.0, scale, with_alpha(palette.surface_text, fade)),
                label(setting.desc, TYPE_SMALL, scale, with_alpha(palette.surface_text, 0.54))
                    .line_height(iced::widget::text::LineHeight::Relative(1.3)),
                container(widget).padding(Padding { top: 4.0 * scale, ..Padding::ZERO }),
            ]
            .spacing(7.0 * scale),
        )
        .width(Length::Fill)
        .padding([0.0, 3.0 * scale]),
    )
    .on_press(Message::Settings(SettingsMsg::FocusControl(index)))
    .into()
}

#[derive(Clone, Copy)]
struct NavigationInput<'a, 'b> {
    places: &'b [(&'static str, &'static str)],
    overview: &'b crate::contracts::settings::Overview,
    selected: &'a str,
    note: &'a str,
    previous: Option<&'a str>,
    travel: f32,
    scale: f32,
    palette: &'a Palette,
    entrance: f32,
    search_open: bool,
    search_query: &'a str,
}

struct PlaceEntry {
    key: &'static str,
    title: &'static str,
    active: bool,
    fade: f32,
    badge: Option<String>,
}

fn place_entry(entry: PlaceEntry, scale: f32, palette: &Palette) -> Element<'_, Message> {
    let PlaceEntry { key, title, active, fade, badge } = entry;
    let twisty = if active { TWISTY_OPEN } else { TWISTY_CLOSED };
    let line = row![
        container(label(twisty, INDEX.glyph, scale, with_alpha(palette.surface_text, fade)))
            .width(Length::Fixed(INDEX.glyph_width * scale)),
        container(label(title, INDEX.row_size, scale, with_alpha(palette.surface_text, fade)))
            .width(Length::Fill),
    ]
    .push_maybe(
        badge.map(|badge| label(badge, INDEX.badge, scale, with_alpha(palette.primary, fade))),
    )
    .spacing(INDEX.row_gap * scale)
    .align_y(Alignment::Center);
    button(line)
        .width(Length::Fill)
        .padding(INDEX.row_padding * scale)
        .on_press(Message::Settings(SettingsMsg::SelectTab(key.to_owned())))
        .style(move |_theme, status| calm::style::item(palette.background, palette, active, status))
        .into()
}

fn navigation<'a>(input: &NavigationInput<'a, '_>) -> Element<'a, Message> {
    let NavigationInput {
        places,
        overview,
        selected,
        note,
        previous,
        travel,
        scale,
        palette,
        entrance,
        search_open,
        search_query,
    } = *input;
    let search = search_control(search_open, search_query, scale, palette, GLYPH_FADE);
    let mut list = column![].spacing(1.0);
    for (order, (key, title)) in places.iter().copied().enumerate() {
        let active = key == selected && !search_open;
        let fade = SETTINGS_MOTION.entrance(entrance, order, true);
        let badge = super::super::home::badge(key, overview);
        let entry = PlaceEntry { key, title, active, fade, badge };
        list = list.push(motion_slot(
            place_entry(entry, scale, palette),
            None,
            fade,
            Some(INDEX.travel * scale),
        ));
    }
    let position = |key: &str| places.iter().position(|(place, _)| *place == key).unwrap_or(0);
    let target = position(selected) as f32;
    let from = previous.map_or(target, |key| position(key) as f32);
    let content: Element<'a, Message> = row![
        container(list).width(Length::Fill),
        iced::widget::canvas(IndexRail {
            active: if search_open { 0.0 } else { from + (target - from) * travel },
            count: places.len(),
            notches: true,
            palette: *palette,
            fade: GLYPH_FADE,
        })
        .width(Length::Fixed(INDEX.rail * scale))
        .height(Length::Fill),
    ]
    .spacing(INDEX.rail_gap * scale)
    .height(Length::Fill)
    .into();
    let surface_fade = (entrance * SETTINGS_MOTION.open_ms / SETTINGS_MOTION.surface_ms).min(1.0);
    let head = column![
        label(
            tr("settings-index-title"),
            INDEX.title,
            scale,
            with_alpha(palette.surface_text, surface_fade)
        ),
        label(
            note,
            INDEX.note,
            scale,
            tint(palette.background, palette.surface_text, INDEX.note_alpha * surface_fade)
        )
        .line_height(iced::widget::text::LineHeight::Relative(INDEX.note_leading)),
    ]
    .spacing(INDEX.head_gap * scale);
    let [top, right, bottom, left] = INDEX.padding.map(|px| px * scale);
    let mut lines = column![head, search, content];
    if search_open {
        lines = lines.push(search_page::keys(scale, palette));
    }
    container(lines.spacing(INDEX.gap * scale).height(Length::Fill))
        .padding(Padding { top, right, bottom, left })
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| {
            crate::frontend::ui::bg_style(Background::Color(with_alpha(
                palette.background,
                INDEX.fill * surface_fade,
            )))
        })
        .into()
}

fn search_control<'a>(
    open: bool,
    query: &'a str,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    if open {
        return search_page::field(query, scale, palette);
    }
    button(
        row![
            label("⌕", INDEX.search_glyph, scale, with_alpha(palette.primary, fade)),
            label(
                tr("settings-nav-search"),
                INDEX.search_size,
                scale,
                with_alpha(palette.surface_text, fade)
            ),
            container(text("")).width(Length::Fill),
            label(
                tr("settings-nav-search-shortcut"),
                INDEX.shortcut,
                scale,
                tint(palette.background, palette.surface_text, INDEX.shortcut_alpha * fade)
            ),
        ]
        .spacing(INDEX.row_gap * scale)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(INDEX.row_padding * scale)
    .on_press(Message::Settings(SettingsMsg::SearchOpen))
    .style(move |_theme, status| calm::style::item(palette.background, palette, false, status))
    .into()
}

const STAGE_HEIGHT: f32 = 9.0 / 16.0;

fn transition_preview_stage<'a>(
    handle: Option<iced::advanced::image::Handle>,
    width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    let height = width * STAGE_HEIGHT;
    let frame: Element<'a, Message> = if let Some(handle) = handle {
        image(handle)
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(iced::ContentFit::Cover)
            .into()
    } else {
        container(label(
            tr("settings-control-live-backdrop"),
            11.0,
            scale,
            with_alpha(palette.primary, 0.78 * fade),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    };
    container(frame)
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .clip(true)
        .style(move |_| {
            crate::frontend::ui::box_style(
                with_alpha(Color::BLACK, 0.9 * fade),
                with_alpha(palette.primary, 0.58 * fade),
            )
        })
        .into()
}

struct PageInput<'a, 'b> {
    source: &'b SourceCtx<'a>,
    focus: FocusCtx<'a>,
    profiles: ProfilesCtx<'a>,
    folders: FoldersCtx<'a>,
    keys: KeysCtx<'a>,
}

impl<'a> PageInput<'a, '_> {
    fn outgoing(&self, old: &Section, index: usize, paint: Paint<'a>) -> Element<'a, Message> {
        if old.id == KEYS_SHEET {
            let map = self.source.config.bindings();
            return super::keybinds_page::sheet(old, &map, KeysCtx::default(), paint);
        }
        if old.id == HOME_LEAD {
            return super::home_page::lead(old, paint);
        }
        sections::header(old, index, paint)
    }

    fn parts(
        &self,
        section: SectionId,
        base: usize,
        paint: Paint<'a>,
    ) -> Vec<Element<'a, Message>> {
        let ctx = super::super::PageCtx {
            cfg: self.source.config,
            folders: self.source.folders,
            watch: self.source.library_watch,
            profiles: self.profiles,
            lists: self.folders,
            folds: self.focus.expanded_details,
        };
        super::library_page::parts(section, ctx, base, paint)
    }
}

struct ReadingInput<'a, 'b> {
    found: Option<search_page::Found<'a>>,
    layouts: super::super::LayoutsUi,
    place: Place,
    pages: PageInput<'a, 'b>,
    preview: Option<iced::advanced::image::Handle>,
    entrance: f32,
    transition: f32,
    emblem_transition: f32,
    previous: Option<&'a super::super::PageSnapshot>,
    paint: Paint<'a>,
}

fn reading_inset(scale: f32) -> f32 {
    let [_, right, _, left] = SURFACE.padding;
    (right + left) * scale + sections::body_inset(scale)
}

fn emblem<'a>(key: &str, elapsed_ms: f32, paint: Paint<'a>) -> Element<'a, Message> {
    let size = METRICS.emblem * paint.scale;
    stack![
        iced::widget::canvas(super::appearance::Ornament::Emblem {
            palette: *paint.palette,
            elapsed_ms,
        })
        .width(Length::Fixed(size))
        .height(Length::Fixed(size)),
        container(navigation_icon(
            super::appearance::place_icon(key),
            super::appearance::EMBLEM_ICON_SIZE * paint.scale,
            with_alpha(paint.palette.primary, GLYPH_FADE)
        ))
        .center_x(Length::Fill)
        .center_y(Length::Fill),
    ]
    .width(Length::Fixed(size))
    .height(Length::Fixed(size))
    .into()
}

fn same_header(
    old: &Section,
    new: &Section,
    old_inputs: &HashMap<String, String>,
    paint: Paint<'_>,
) -> bool {
    old.title == new.title
        && old.desc == new.desc
        && old.say.text == new.say.text
        && old.say.notes == new.say.notes
        && old.head.len() == new.head.len()
        && old
            .head
            .iter()
            .zip(&new.head)
            .all(|(old_row, new_row)| unchanged_row(old_row, new_row, old_inputs, paint.values))
        && old.say.tokens.len() == new.say.tokens.len()
        && old.say.tokens == new.say.tokens
        && old.say.rows().all(|at| match (old.row(at), new.row(at)) {
            (Some(old_row), Some(new_row)) => {
                unchanged_row(old_row, new_row, old_inputs, paint.values)
            }
            _ => false,
        })
}

fn section_blocks<'a>(
    place: Place,
    pages: &PageInput<'a, '_>,
    preview: Option<&iced::advanced::image::Handle>,
    paint: Paint<'a>,
) -> Vec<(Section, Element<'a, Message>)> {
    place
        .sections
        .into_iter()
        .enumerate()
        .map(|(index, mut section)| {
            let header = section.clone();
            if section.id == KEYS_SHEET {
                let map = pages.source.config.bindings();
                return (header, super::keybinds_page::sheet(&section, &map, pages.keys, paint));
            }
            if section.id == HOME_LEAD {
                return (header, super::home_page::lead(&section, paint));
            }
            let mut lead = Vec::new();
            if paint.calm.open == Some(section.id) || paint.calm.closing == Some(section.id) {
                lead.extend(pages.parts(section.id, section.rows().count(), paint));
            }
            let mut gutter = super::super::is_picker_layout_section(Some(section.id))
                .then(|| sections::layout_entry(paint));
            for group in &mut section.groups {
                let before = group.rows.len();
                group.rows.retain(|row| !matches!(row.control, Control::Preview));
                if group.rows.len() != before {
                    gutter = Some(transition_preview_stage(
                        preview.cloned(),
                        sections::stage_width(paint.scale),
                        paint.scale,
                        paint.palette,
                        GLYPH_FADE,
                    ));
                }
            }
            (header, sections::block(Block { index, section, lead, gutter }, paint))
        })
        .collect()
}

fn page_spacing(key: &str) -> (f32, f32) {
    match key {
        super::super::place::KEYBINDS => {
            (super::keybinds_page::SHEET_GAP, super::keybinds_page::MARK_GUTTER)
        }
        super::super::place::HOME => (super::home_page::LEAD.gap, 0.0),
        _ => (SURFACE.gap, 0.0),
    }
}

fn reading_surface<'a>(input: ReadingInput<'a, '_>) -> Element<'a, Message> {
    let ReadingInput {
        found,
        layouts,
        place,
        pages,
        preview,
        entrance,
        transition,
        emblem_transition,
        previous,
        paint,
    } = input;
    if let Some(found) = found {
        return search_page::page(found, paint);
    }
    let previous = previous.filter(|_| transition < 1.0);
    let unchanged = previous.is_none_or(|old| old.place.key == place.key);
    let progress = if unchanged { 1.0 } else { transition };
    let current = if place.key == super::super::place::LAYOUT {
        let strip =
            super::strip::strip(&pages.source.config.saved_layouts(), layouts, progress, paint);
        sections::strip_heading(place.title, strip, progress, paint)
    } else {
        sections::heading(place.title, place.note, progress, paint)
    };
    let old_heading = previous.filter(|_| !unchanged).map(|old| {
        let note = if old.place.key == super::super::place::LAYOUT { "" } else { old.place.note };
        sections::heading(old.place.title, note, 1.0 - transition, paint)
    });
    let emblem_time = if entrance < 1.0 {
        entrance * SETTINGS_MOTION.open_ms
    } else {
        emblem_transition * SETTINGS_MOTION.emblem_ms
    };
    let head = row![
        emblem(place.key, emblem_time, paint),
        container(motion_slot(current, old_heading, progress, None)).width(Length::Fill),
    ]
    .push_maybe(place.strip.then(|| sections::palette_strip(paint)))
    .push(sections::close_button(paint))
    .push(crate::frontend::ui::folio_icon_action(
        crate::frontend::ui::EYE_CLOSED,
        Message::Stash,
        paint.scale,
        paint.palette,
    ))
    .spacing(SURFACE.head_spacing * paint.scale)
    .align_y(Alignment::Start);
    let head = column![head, sections::rule(paint, SURFACE.head_rule)]
        .spacing(SURFACE.head_gap * paint.scale);
    let sheet = place.key == super::super::place::KEYBINDS;
    let spacing = page_spacing(place.key);
    let blocks = section_blocks(place, &pages, preview.as_ref(), paint);
    let count = blocks.len();
    let old_sections = previous.map_or(&[][..], |old| old.place.sections.as_slice());
    let old_paint = |snapshot: &'a super::super::PageSnapshot| Paint {
        values: &snapshot.inputs,
        calm: CalmCtx { open: None, busy: 0.0, pop: None, edit: None, ..paint.calm },
        ..paint
    };
    let mut rows = Vec::new();
    for (index, (section, block)) in blocks.into_iter().enumerate() {
        let old = old_sections.get(index);
        let same = previous.is_some_and(|snapshot| {
            old.is_some_and(|old| same_header(old, &section, &snapshot.inputs, paint))
        });
        let progress = match previous {
            Some(_) if !same => transition,
            Some(_) => 1.0,
            None => SETTINGS_MOTION.entrance(entrance, index, false),
        };
        let outgoing = previous
            .filter(|_| !same)
            .and_then(|snapshot| old.map(|old| pages.outgoing(old, index, old_paint(snapshot))));
        let open = [paint.calm.open, paint.calm.closing].contains(&Some(section.id));
        rows.push((motion_slot(block, outgoing, progress, None).into(), open));
    }
    if let Some(snapshot) = previous {
        for (index, old) in old_sections.iter().enumerate().skip(count) {
            let outgoing = pages.outgoing(old, index, old_paint(snapshot));
            let empty = iced::widget::Space::new().height(Length::Fixed(0.0)).into();
            rows.push((motion_slot(empty, Some(outgoing), transition, None).into(), false));
        }
    }
    if !sheet {
        rows.push((sections::rule(paint, SURFACE.secs_rule), false));
    }
    let scroll = scrollable(calm::page(rows, paint.size(SURFACE.foot)))
        .id(super::super::settings_reading_id())
        .direction(iced::widget::scrollable::Direction::Vertical(
            iced::widget::scrollable::Scrollbar::hidden(),
        ))
        .width(Length::Fill)
        .height(Length::Fill);
    let [top, right, bottom, left] = SURFACE.padding.map(|px| px * paint.scale);
    let (gap, gutter) = spacing;
    let gutter = gutter * paint.scale;
    column![container(head).padding(logical_padding(0.0, 0.0, 0.0, gutter)), scroll]
        .spacing(gap * paint.scale)
        .padding(logical_padding(top, right, bottom, left - gutter))
        .height(Length::Fill)
        .into()
}

fn unchanged_row(
    old: &Row,
    new: &Row,
    old_inputs: &HashMap<String, String>,
    new_inputs: &HashMap<String, String>,
) -> bool {
    if old.title != new.title || old.desc != new.desc || old.look != new.look {
        return false;
    }
    match (&old.control, &new.control) {
        (Control::Toggle { path: old_path, value: old_value }, Control::Toggle { path, value }) => {
            old_path == path && old_value == value
        }
        (
            Control::Number { key: old_key, path: old_path, unit: old_unit },
            Control::Number { key, path, unit },
        ) => old_path == path && old_unit == unit && old_inputs.get(old_key) == new_inputs.get(key),
        (
            Control::TextField { key: old_key, path: old_path, placeholder: old_hint, .. },
            Control::TextField { key, path, placeholder, .. },
        ) => {
            old_path == path
                && old_hint == placeholder
                && old_inputs.get(old_key) == new_inputs.get(key)
        }
        (
            Control::Dropdown { path: old_path, options: old_options, current: old_value, .. },
            Control::Dropdown { path, options, current, .. },
        ) => old_path == path && old_options == options && old_value == current,
        (
            Control::Chips {
                path: old_path,
                options: old_options,
                current: old_value,
                disabled: old_disabled,
                ..
            },
            Control::Chips { path, options, current, disabled, .. },
        ) => {
            old_path == path
                && old_options == options
                && old_value == current
                && old_disabled == disabled
        }
        (Control::ActionBtn { id: old_id, label: old_label }, Control::ActionBtn { id, label }) => {
            old_id == id && old_label == label
        }
        (Control::Static, Control::Static) => true,
        _ => false,
    }
}

fn compact_field_copy(
    title: &str,
    desc: &str,
    copy_width: f32,
    scale: f32,
    palette: &Palette,
    fade: f32,
) -> Element<'static, Message> {
    let title = label(title.trim().to_owned(), 12.0, scale, with_alpha(palette.surface_text, fade))
        .width(Length::Fixed(copy_width))
        .wrapping(iced::widget::text::Wrapping::WordOrGlyph);
    if desc.trim().is_empty() {
        title.into()
    } else {
        column![
            title,
            label(
                desc.trim().to_owned(),
                TYPE_SMALL,
                scale,
                with_alpha(palette.surface_text, 0.52 * fade)
            )
            .width(Length::Fixed(copy_width))
            .line_height(iced::widget::text::LineHeight::Relative(1.12))
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
        ]
        .spacing(2.0 * scale)
        .into()
    }
}

#[allow(clippy::too_many_arguments)]
fn compact_inline_field<'a>(
    setting: Row,
    index: usize,
    values: &'a HashMap<String, String>,
    focused: bool,
    focused_choice: Option<usize>,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'a, Message> {
    let Row { title, desc, control, .. } = setting;
    let editor_width = match &control {
        Control::Number { .. } => (available_width * 0.34).clamp(118.0 * scale, 176.0 * scale),
        Control::TextField { .. } => (available_width * 0.42).clamp(142.0 * scale, 248.0 * scale),
        Control::KeyBinding { .. } => (available_width * 0.5).clamp(176.0 * scale, 300.0 * scale),
        _ => unreachable!("compact inline field requires one scalar editor"),
    };
    let copy_width = (available_width - editor_width - 44.0 * scale).max(72.0 * scale);
    let copy = compact_field_copy(&title, &desc, copy_width, scale, palette, fade);
    let editor = control::widget(
        control,
        values,
        focused,
        focused_choice,
        editor_width,
        scale * 0.78,
        palette,
        fade,
        None,
        motion,
        None,
    );
    crate::frontend::ui::folio_inline_bar(
        copy,
        focused,
        Message::Settings(SettingsMsg::FocusControl(index)),
        container(editor).width(Length::Fixed(editor_width)).into(),
        scale,
        palette,
        fade,
    )
}

struct ActionField<'a> {
    setting: Row,
    index: usize,
    focused: bool,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
}

fn compact_action_field(input: ActionField<'_>) -> Element<'_, Message> {
    let ActionField { setting, index, focused, available_width, scale, palette, fade, motion } =
        input;
    let Row { title, desc, control, .. } = setting;
    let (value, identity, message) = match control {
        Control::Toggle { path, value } => (
            value,
            super::diamond_toggle::ToggleIdentity::Path(path.clone()),
            SettingsMsg::Toggle(path, !value),
        ),
        Control::ToggleAction { id, value } => {
            (value, super::diamond_toggle::ToggleIdentity::Action(id.clone()), SettingsMsg::Run(id))
        }
        _ => unreachable!("toggle field"),
    };
    let copy_width = (available_width - METRICS.editor * scale - METRICS.row_gap * scale).max(1.0);
    let copy = compact_field_copy(&title, &desc, copy_width, scale, palette, fade);
    let action = super::diamond_toggle::diamond_toggle(super::diamond_toggle::ToggleSpec {
        identity,
        value,
        focused,
        scale,
        fade,
        palette: *palette,
        message: Message::Settings(message),
        motion,
    });
    crate::frontend::ui::folio_action_bar(
        copy,
        focused,
        Message::Settings(SettingsMsg::FocusControl(index)),
        action,
        scale,
        palette,
        fade,
    )
}

struct SelectionMetrics {
    preview_width: f32,
    preview_height: f32,
    identity_share: f32,
    identity_min: f32,
    identity_max: f32,
    side_gap: f32,
    summary_min: f32,
    bar: f32,
    tray: f32,
    live_row: f32,
    background_row: f32,
    picker: f32,
}

const SELECTION: SelectionMetrics = SelectionMetrics {
    preview_width: 86.0,
    preview_height: 48.0,
    identity_share: 0.24,
    identity_min: 110.0,
    identity_max: 185.0,
    side_gap: 96.0,
    summary_min: 100.0,
    bar: 68.0,
    tray: 176.0,
    live_row: 48.0,
    background_row: 52.0,
    picker: 192.0,
};

#[allow(clippy::too_many_arguments)]
fn compact_group_field<'a>(
    setting: Row,
    index: usize,
    values: &'a HashMap<String, String>,
    focus: FocusCtx<'_>,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'a, Message> {
    let Row { title, desc, control, .. } = setting;
    let title = if title.is_empty() {
        tr("settings-selector-saved-styles-label").to_string()
    } else {
        title
    };
    let id = control.compact_bar_id().expect("compact group requires a bar identity");
    let (expanded, reveal) = focus.bar(&id);
    let Control::StackBar { summary, preview, rows, .. } = control else {
        unreachable!("compact group requires a stack bar")
    };
    let content: Element<'a, Message> = {
        let preview_width = SELECTION.preview_width * scale;
        let preview_height = SELECTION.preview_height * scale;
        let identity_width = (available_width * SELECTION.identity_share)
            .clamp(SELECTION.identity_min * scale, SELECTION.identity_max * scale);
        let wallpaper_width =
            (available_width - preview_width - identity_width - SELECTION.side_gap * scale)
                .max(SELECTION.summary_min * scale);
        let preview: Element<'a, Message> = preview.as_ref().map_or_else(
            || {
                container(label("◇", 11.0, scale, with_alpha(palette.surface_text, 0.36 * fade)))
                    .width(Length::Fixed(preview_width))
                    .height(Length::Fixed(preview_height))
                    .center(Length::Fill)
                    .style(move |_| {
                        crate::frontend::ui::box_style(
                            with_alpha(palette.surface_container, 0.32 * fade),
                            with_alpha(palette.outline, 0.34 * fade),
                        )
                    })
                    .into()
            },
            |path| {
                container(
                    image(image::Handle::from_path(path))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .content_fit(iced::ContentFit::Cover),
                )
                .width(Length::Fixed(preview_width))
                .height(Length::Fixed(preview_height))
                .clip(true)
                .style(move |_| {
                    crate::frontend::ui::box_style(
                        with_alpha(palette.surface_container, 0.32 * fade),
                        with_alpha(palette.outline, 0.52 * fade),
                    )
                })
                .into()
            },
        );
        row![
            preview,
            column![
                label(
                    title.trim().to_string(),
                    12.5,
                    scale,
                    with_alpha(palette.surface_text, fade)
                ),
                label(
                    desc.trim().to_string(),
                    TYPE_SMALL,
                    scale,
                    with_alpha(palette.surface_text, 0.52 * fade),
                ),
            ]
            .width(Length::Fixed(identity_width))
            .spacing(1.0 * scale),
            container(text("")).width(Length::Fill),
            label(summary.trim().to_string(), TYPE_SMALL, scale, with_alpha(palette.primary, fade))
                .width(Length::Fixed(wallpaper_width))
                .align_x(end()),
        ]
        .spacing(12.0 * scale)
        .align_y(Alignment::Center)
        .into()
    };
    let live_height = rows.iter().filter(|row| matches!(row.control, Control::Display(_))).count()
        as f32
        * SELECTION.live_row;
    let background_height: f32 = rows
        .iter()
        .filter_map(|row| {
            let Control::Chips { path, current, background: Some(background), .. } = &row.control
            else {
                return None;
            };
            let reveal = focus
                .bar_reveals
                .get(&super::super::background::BackgroundControl::reveal_id(path))
                .map_or(
                    if matches!(current.as_str(), "center" | "fit") { 1.0 } else { 0.0 },
                    |value| value.x,
                );
            let picker =
                focus.bar_reveals.get(&background.picker_id()).map_or(0.0, |value| value.x);
            Some(
                (SELECTION.background_row + SELECTION.picker * picker)
                    * crate::frontend::animation::smoothstep(reveal),
            )
        })
        .sum();
    let options = detail_rows(
        rows,
        index,
        values,
        available_width - 18.0 * scale,
        scale,
        palette,
        fade,
        focus,
        motion,
    );
    let options_height = SELECTION.tray + background_height + live_height;
    crate::frontend::ui::folio_stack_bar(
        content,
        expanded,
        expanded,
        reveal,
        Message::Settings(SettingsMsg::ToggleBar(id, index)),
        None,
        options,
        SELECTION.bar,
        options_height,
        scale,
        palette,
    )
}

pub(super) fn field<'a>(
    setting: Row,
    index: usize,
    values: &'a HashMap<String, String>,
    focus: FocusCtx<'_>,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'a, Message> {
    if setting.control.is_inline_editor() {
        let (focused, focused_choice) = focus.on(SettingsFocus::Controls, index);
        return compact_inline_field(
            setting,
            index,
            values,
            focused,
            focused_choice,
            available_width,
            scale,
            palette,
            fade,
            motion,
        );
    }
    if setting.control.is_group_editor() {
        return compact_group_field(
            setting,
            index,
            values,
            focus,
            available_width,
            scale,
            palette,
            fade,
            motion,
        );
    }
    let (focused, focused_choice) = focus.on(SettingsFocus::Controls, index);
    let Row { title, desc, control, .. } = setting;
    if matches!(control, Control::Segment) {
        return column![
            text(title)
                .font(crate::frontend::ui::UI_FONT)
                .size(22.0 * legible_type_scale(scale))
                .color(with_alpha(palette.surface_text, fade)),
            label(desc, TYPE_SMALL, scale, with_alpha(palette.surface_text, 0.56 * fade))
                .line_height(iced::widget::text::LineHeight::Relative(1.38)),
            folio_horizontal_rule(with_alpha(palette.outline, FOLIO_RULE_ALPHA * fade)),
        ]
        .spacing(9.0 * scale)
        .padding(Padding { top: 10.0 * scale, ..Padding::ZERO })
        .into();
    }
    if let Control::Details { id, summary, rows } = control {
        let expanded = focus.expanded_details.contains(&id);
        let body =
            detail_rows(rows, index, values, available_width, scale, palette, fade, focus, motion);
        return crate::frontend::ui::folio_details(
            title,
            summary,
            desc,
            expanded,
            focused,
            Message::Settings(SettingsMsg::ToggleDetails(id, index)),
            body,
            scale,
            palette,
        );
    }
    let locked = focus.active_input.is_some_and(|key| match &control {
        Control::Number { key: control_key, .. }
        | Control::TextField { key: control_key, .. }
        | Control::Resolution { key: control_key, .. } => control_key == key,
        _ => false,
    }) || focused && focused_choice.is_some();
    let title = if title.is_empty() {
        tr("settings-selector-saved-styles-label").to_string()
    } else {
        title
    };
    let widget = control::widget(
        control,
        values,
        focused,
        focused_choice,
        available_width,
        scale,
        palette,
        fade,
        focus.armed,
        motion,
        Some(focus.reveals()),
    );
    let body = column![
        container(text(""))
            .width(Length::Fill)
            .height(Length::Fixed(if focused || locked { 2.0 } else { 1.0 }))
            .style(move |_| crate::frontend::ui::bg_style(Background::Color(
                if focused || locked {
                    with_alpha(palette.primary, fade)
                } else {
                    with_alpha(palette.outline, 0.48 * fade)
                }
            ))),
        label(title, 14.0, scale, with_alpha(palette.surface_text, fade)),
        label(desc, TYPE_SMALL, scale, with_alpha(palette.surface_text, 0.56 * fade))
            .line_height(iced::widget::text::LineHeight::Relative(1.38)),
        container(widget).width(Length::Fill).padding(Padding {
            top: 7.0 * scale,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }),
    ]
    .spacing(8.0 * scale);
    mouse_area(container(body).width(Length::Fill).padding(Padding {
        top: 0.0,
        right: 6.0 * scale,
        bottom: 4.0 * scale,
        left: 6.0 * scale,
    }))
    .on_press(Message::Settings(SettingsMsg::FocusControl(index)))
    .into()
}

#[allow(clippy::too_many_arguments)]
#[expect(clippy::too_many_lines, reason = "composes nested settings rows and app theme controls")]
fn detail_rows<'a>(
    rows: Vec<Row>,
    parent_index: usize,
    values: &'a HashMap<String, String>,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    focus: FocusCtx<'_>,
    motion: MotionProfile,
) -> Element<'a, Message> {
    let mut content = column![].spacing(15.0 * scale);
    for row in rows {
        if let Control::Display(control) = row.control {
            content = content.push(control.view(scale * 0.94, palette));
            continue;
        }
        if let Control::Details { id, summary, rows } = row.control {
            let expanded = focus.expanded_details.contains(&id);
            let body = detail_rows(
                rows,
                parent_index,
                values,
                (available_width - 24.0 * scale).max(160.0 * scale),
                scale * 0.94,
                palette,
                fade,
                focus,
                motion,
            );
            content = content.push(crate::frontend::ui::folio_details(
                row.title,
                summary,
                row.desc,
                expanded,
                false,
                Message::Settings(SettingsMsg::ToggleDetails(id, parent_index)),
                body,
                scale * 0.94,
                palette,
            ));
            continue;
        }
        if row.control.is_inline_editor() {
            content = content.push(compact_inline_field(
                row,
                parent_index,
                values,
                false,
                None,
                (available_width - 24.0 * scale).max(160.0 * scale),
                scale * 0.94,
                palette,
                fade,
                motion,
            ));
            continue;
        }
        if row.control.is_group_editor() {
            let nested_focus =
                FocusCtx { focused_control: usize::MAX, focused_choice: None, ..focus };
            content = content.push(compact_group_field(
                row,
                parent_index,
                values,
                nested_focus,
                (available_width - 24.0 * scale).max(160.0 * scale),
                scale * 0.94,
                palette,
                fade,
                motion,
            ));
            continue;
        }
        if row.control.is_compact_action() {
            content = content.push(compact_action_field(ActionField {
                setting: row,
                index: parent_index,
                focused: false,
                available_width: (available_width - 24.0 * scale).max(160.0 * scale),
                scale: scale * 0.94,
                palette,
                fade,
                motion,
            }));
            continue;
        }
        let Row { title, desc, control, .. } = row;
        let widget = control::widget(
            control,
            values,
            false,
            None,
            (available_width - 24.0 * scale).max(160.0 * scale),
            scale * 0.94,
            palette,
            fade,
            focus.armed,
            motion,
            Some(focus.reveals()),
        );
        content = content.push(
            column![
                label(title, 12.0, scale, with_alpha(palette.surface_text, fade)),
                label(desc, TYPE_SMALL, scale, with_alpha(palette.surface_text, 0.5 * fade))
                    .line_height(iced::widget::text::LineHeight::Relative(1.32)),
                container(widget)
                    .width(Length::Fill)
                    .padding(Padding { top: 3.0 * scale, ..Padding::ZERO }),
            ]
            .spacing(6.0 * scale),
        );
    }
    content.into()
}

fn tracker_position(active: f32, count: usize) -> f32 {
    if count <= 1 { 0.0 } else { active.clamp(0.0, (count - 1) as f32) / (count - 1) as f32 }
}

struct IndexRail {
    active: f32,
    count: usize,
    notches: bool,
    palette: Palette,
    fade: f32,
}

impl canvas::Program<Message> for IndexRail {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let x = bounds.width * 0.5;
        let top = 8.0;
        let bottom = (bounds.height - 8.0).max(top);
        frame.stroke(
            &Path::line(Point::new(x, top), Point::new(x, bottom)),
            Stroke::default()
                .with_color(with_alpha(self.palette.outline, 0.48 * self.fade))
                .with_width(1.0),
        );
        if self.notches {
            for index in 0..self.count {
                let y = top + (bottom - top) * tracker_position(index as f32, self.count);
                frame.stroke(
                    &Path::line(Point::new(x - 2.5, y), Point::new(x + 2.5, y)),
                    Stroke::default()
                        .with_color(with_alpha(self.palette.outline, 0.58 * self.fade))
                        .with_width(1.0),
                );
            }
        }
        let y = top + (bottom - top) * tracker_position(self.active, self.count);
        let diamond = crate::frontend::ui::diamond(Point::new(x, y), 5.0);
        frame.fill(&diamond, with_alpha(self.palette.background, self.fade));
        frame.stroke(
            &diamond,
            Stroke::default()
                .with_color(with_alpha(self.palette.primary, self.fade))
                .with_width(1.5),
        );
        vec![frame.into_geometry()]
    }
}

struct Blueprint {
    palette: Palette,
    fade: f32,
}

impl canvas::Program<Message> for Blueprint {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let line = tint(self.palette.surface, self.palette.outline, 0.075 * self.fade);
        let accent = tint(self.palette.surface, self.palette.primary, 0.09 * self.fade);
        let centre =
            Point::new(mirror_x(bounds.width * 0.72, 0.0, bounds.width), bounds.height * 0.56);
        for radius in [90.0, 176.0, 264.0] {
            frame.stroke(
                &Path::circle(centre, radius),
                Stroke::default().with_color(line).with_width(1.0),
            );
        }
        frame.stroke(
            &Path::line(Point::new(centre.x, 0.0), Point::new(centre.x, bounds.height)),
            Stroke::default().with_color(line).with_width(1.0),
        );
        frame.stroke(
            &Path::line(Point::new(0.0, centre.y), Point::new(bounds.width, centre.y)),
            Stroke::default().with_color(line).with_width(1.0),
        );
        let marker = crate::frontend::ui::diamond(centre, 7.0);
        frame.stroke(&marker, Stroke::default().with_color(accent).with_width(1.3));
        vec![frame.into_geometry()]
    }
}
