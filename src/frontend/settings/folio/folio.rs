#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use iced::widget::canvas::{self, Frame, Path, Stroke};
use iced::widget::{
    button, column, container, image, mouse_area, row, scrollable, stack, text, text_input,
};
use iced::{Alignment, Background, Color, Element, Length, Padding, Point, Rectangle};

use crate::app::Message;
use crate::contracts::settings::SettingsSource;
use crate::frontend::animation::{MotionProfile, Tween};
use crate::frontend::components::with_alpha;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{
    FOLIO_INDEX_WIDTH, FOLIO_RULE_ALPHA, TYPE_SMALL, folio_horizontal_rule, label,
    legible_type_scale, sentence_case,
};
use crate::i18n::tr;

use super::super::{
    ActionId, Card, Control, Row, SettingsFocus, SettingsMsg, SettingsSearchResult,
    build_tab_with_runtime_status, visible_tabs,
};
use super::control;

const GLYPH_FADE: f32 = 1.0;

fn category_note(tab: &str) -> &'static str {
    tr(match tab {
        "picker" => "settings-tab-note-picker",
        "filter" => "settings-tab-note-filter",
        "position" => "settings-tab-note-position",
        "displays" => "settings-tab-note-displays",
        "motion" => "settings-tab-note-motion",
        "playback" => "settings-tab-note-playback",
        "performance" => "settings-tab-note-performance",
        "library" => "settings-tab-note-library",
        "sources" => "settings-tab-note-sources",
        "automation" => "settings-tab-note-automation",
        "theme" => "settings-tab-note-theme",
        "integrations" => "settings-tab-note-integrations",
        "language" => "settings-tab-note-language",
        _ => "settings-tab-note-default",
    })
}

pub struct SourceCtx<'a> {
    pub config: &'a dyn SettingsSource,
    pub values: &'a HashMap<String, String>,
    pub themes: &'a [String],
    pub folders: &'a [String],
    pub backends: &'a [String],
    pub outputs: &'a [String],
    pub output_statuses: &'a [crate::contracts::daemon::OutputStatus],
    pub output_previews: &'a HashMap<String, String>,
    pub library_watch: Option<&'a crate::contracts::daemon::LibraryWatchStatus>,
    pub playback: Option<&'a crate::contracts::daemon::PlaybackStatus>,
    pub thumbnail_task: Option<&'a crate::contracts::daemon::TaskStatus>,
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

    fn cards(&self, tab: &str) -> Vec<(Card, Vec<Row>)> {
        let mut cards = build_tab_with_runtime_status(
            tab,
            self.config,
            self.themes,
            self.folders,
            &self.analysis,
            self.backends,
            self.outputs,
            self.output_statuses,
            self.output_previews,
            self.library_watch,
            self.playback,
        );
        if let Some(task) = self.thumbnail_task {
            for row in cards.iter_mut().flat_map(|(_, rows)| rows) {
                if let super::super::Control::ActionBtn {
                    id: ActionId::CaptureWeThumbnails,
                    label,
                } = &mut row.control
                {
                    let state = match &task.state {
                        crate::contracts::daemon::TaskState::Running => {
                            tr("filter-bar-state-running")
                        }
                        crate::contracts::daemon::TaskState::Paused => {
                            tr("filter-bar-state-paused")
                        }
                        crate::contracts::daemon::TaskState::Completed => {
                            tr("filter-bar-state-done")
                        }
                        crate::contracts::daemon::TaskState::Failed => {
                            tr("filter-bar-state-failed")
                        }
                        crate::contracts::daemon::TaskState::Cancelled => {
                            tr("filter-bar-state-stopped")
                        }
                        crate::contracts::daemon::TaskState::Other(state) => state.as_str(),
                    };
                    row.desc =
                        format!("{state} · {} / {} · {}", task.progress, task.total, task.detail);
                    if task.state.is_active() {
                        *label = tr("settings-performance-capture-we-stop").into();
                    }
                }
            }
        }
        cards
    }
}

#[derive(Clone, Copy)]
pub struct FocusCtx<'a> {
    pub keyboard_focus: SettingsFocus,
    pub focused_control: usize,
    pub focused_choice: Option<usize>,
    pub expanded_details: &'a HashSet<String>,
    pub bar_reveals: &'a HashMap<String, Tween>,
    pub active_input: Option<&'a str>,
    pub armed: Option<ActionId>,
}

impl FocusCtx<'_> {
    fn on(self, area: SettingsFocus, index: usize) -> (bool, Option<usize>) {
        let focused = self.keyboard_focus == area && self.focused_control == index;
        (focused, if focused { self.focused_choice } else { None })
    }

    fn bar(self, id: &str) -> (bool, f32) {
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

pub struct KeybindCaptureView {
    pub title: String,
    pub binding: String,
    pub edited: bool,
    pub conflict: Option<String>,
}

pub struct WorkbenchInput<'a> {
    pub source: SourceCtx<'a>,
    pub focus: FocusCtx<'a>,
    pub chrome: ChromeCtx<'a>,
    pub preview_path: &'a str,
    pub transition_preview: Option<iced::advanced::image::Handle>,
    pub tab_transition: f32,
    pub tab: &'a str,
    pub selected_section: usize,
    pub search_open: bool,
    pub search_query: &'a str,
    pub search_results: &'a [SettingsSearchResult],
    pub keybind_capture: Option<KeybindCaptureView>,
}

pub fn settings_workbench<'a>(input: WorkbenchInput<'a>) -> Element<'a, Message> {
    let WorkbenchInput {
        source,
        focus,
        chrome,
        preview_path,
        transition_preview,
        tab_transition,
        tab,
        selected_section,
        search_open,
        search_query,
        search_results,
        keybind_capture,
    } = input;
    let ChromeCtx { viewport, scale, entrance, palette } = chrome;
    let motion = source.motion();
    let cards = source.cards(tab);
    let selected_section = selected_section.min(cards.len().saturating_sub(1));
    let sections: Vec<(usize, &'static str)> =
        cards.iter().enumerate().map(|(index, (card, _))| (index, card.title)).collect();
    let (card, settings) = cards.into_iter().nth(selected_section).unwrap_or_else(|| {
        (Card { title: tr("settings-section-general"), subtitle: "" }, Vec::new())
    });
    let has_selected_preset = settings.iter().any(|setting| {
        matches!(&setting.control, Control::Presets { items, .. } if items.iter().any(|(_, active)| *active))
    });
    let reveal = crate::frontend::animation::smoothstep(entrance.clamp(0.0, 1.0));
    let branch_reveal = crate::frontend::animation::smoothstep(tab_transition.clamp(0.0, 1.0));
    let (panel_width, panel_height) = crate::frontend::ui::folio_sheet_dims(viewport, scale);
    let tabs = visible_tabs(source.config);
    let active_tab = tabs.iter().position(|(key, _)| *key == tab).unwrap_or(0);

    let index = navigation(
        &tabs,
        active_tab,
        tab,
        &sections,
        selected_section,
        focus.keyboard_focus == SettingsFocus::Index,
        focus.keyboard_focus == SettingsFocus::Sections,
        scale,
        palette,
        GLYPH_FADE,
        branch_reveal,
        search_open,
        search_query,
        search_results,
    );
    let reading = reading_surface(
        ReadingInput {
            tab,
            card,
            settings,
            transition_preview,
            values: source.values,
            has_selected_preset,
            available_width: panel_width - FOLIO_INDEX_WIDTH * scale.max(0.9),
            scale,
            palette,
            fade: GLYPH_FADE,
            motion,
        },
        focus,
    );
    let atmosphere: Element<'a, Message> = if preview_path.is_empty() {
        iced::widget::Space::new().width(Length::Fill).height(Length::Fill).into()
    } else {
        image(image::Handle::from_path(preview_path))
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(iced::ContentFit::Cover)
            .opacity(0.045 * reveal)
            .into()
    };
    let page = container(stack![
        atmosphere,
        iced::widget::canvas(Blueprint { palette: *palette, fade: reveal })
            .width(Length::Fill)
            .height(Length::Fill),
        reading,
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_| {
        crate::frontend::ui::bg_style(Background::Color(with_alpha(
            palette.surface,
            0.965 * reveal,
        )))
    });
    let body = row![
        container(index).width(Length::Fixed(FOLIO_INDEX_WIDTH * scale.max(0.9))),
        container(page).width(Length::Fill),
    ]
    .spacing(0.0)
    .height(Length::Fill);
    let panel = container(body)
        .width(Length::Fixed(panel_width))
        .height(Length::Fixed(panel_height))
        .clip(true)
        .style(move |_| crate::frontend::ui::folio_sheet_panel_style(palette, reveal));
    let mut layers = stack![
        container(text(""))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| crate::frontend::ui::folio_scrim_style(reveal)),
        container(panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    ];
    if let Some(capture) = keybind_capture {
        layers = layers.push(keybind_capture_layer(capture, scale, palette, reveal));
    }
    layers.into()
}

fn keybind_capture_layer(
    capture: KeybindCaptureView,
    scale: f32,
    palette: &Palette,
    reveal: f32,
) -> Element<'_, Message> {
    let KeybindCaptureView { title, binding, edited, conflict } = capture;
    let action_scale = scale * 0.82;
    let action = |label_key: &'static str, active: bool, message: Message| {
        crate::frontend::ui::folio_action(
            tr(label_key),
            active,
            Some(message),
            Length::Fixed(crate::frontend::ui::folio_action_width(tr(label_key), action_scale)),
            action_scale,
            palette,
        )
    };
    let pad = mouse_area(
        container(label(
            binding,
            22.0,
            scale,
            with_alpha(if edited { palette.primary } else { palette.surface_text }, GLYPH_FADE),
        ))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .padding([20.0 * scale, 12.0 * scale])
        .style(move |_| {
            crate::frontend::ui::box_style(
                with_alpha(palette.surface_container, 0.55),
                with_alpha(if edited { palette.primary } else { palette.outline }, 0.75),
            )
        }),
    )
    .on_press(Message::Settings(SettingsMsg::KeybindCaptureClick(
        crate::domain::input::MouseButton::Left,
    )))
    .on_right_press(Message::Settings(SettingsMsg::KeybindCaptureClick(
        crate::domain::input::MouseButton::Right,
    )))
    .on_middle_press(Message::Settings(SettingsMsg::KeybindCaptureClick(
        crate::domain::input::MouseButton::Middle,
    )));
    let mut body = column![
        label(title, 14.0, scale, with_alpha(palette.surface_text, GLYPH_FADE)),
        pad,
        label(
            tr("settings-keybind-capture-hint"),
            TYPE_SMALL,
            scale,
            with_alpha(palette.surface_text, 0.58),
        )
        .line_height(iced::widget::text::LineHeight::Relative(1.35)),
    ]
    .spacing(10.0 * scale);
    if let Some(conflict) = conflict {
        body =
            body.push(label(conflict, TYPE_SMALL, scale, with_alpha(palette.tertiary, GLYPH_FADE)));
    }
    body = body.push(
        row![
            action("settings-keybind-capture-default", false, {
                Message::Settings(SettingsMsg::KeybindCaptureDefault)
            }),
            action("settings-keybind-capture-unbind", false, {
                Message::Settings(SettingsMsg::KeybindCaptureUnbind)
            }),
            container(text("")).width(Length::Fill),
            action("settings-keybind-capture-cancel", false, {
                Message::Settings(SettingsMsg::KeybindCaptureCancel)
            }),
            action("settings-keybind-capture-ok", true, {
                Message::Settings(SettingsMsg::KeybindCaptureApply)
            }),
        ]
        .spacing(8.0 * scale)
        .align_y(Alignment::Center),
    );
    let panel = mouse_area(
        container(body)
            .width(Length::Fixed(390.0 * scale.max(0.9)))
            .padding([18.0 * scale, 20.0 * scale])
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
    .on_press(Message::Settings(SettingsMsg::KeybindCaptureCancel))
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
    let has_selected_preset = rows.iter().any(|row| {
        matches!(&row.control, Control::Presets { items, .. } if items.iter().any(|(_, active)| *active))
    });
    let reveal = crate::frontend::animation::smoothstep(entrance.clamp(0.0, 1.0));
    let panel_width = super::super::picker_layout_studio_width(viewport.0, scale);
    let panel_height = (viewport.1 - 36.0 * scale).max(360.0);

    let mut pinned = column![].spacing(10.0 * scale);
    let mut controls = column![].spacing(13.0 * scale);
    for (index, setting) in rows.into_iter().enumerate() {
        let is_pinned = setting.title == tr("settings-selector-display-mode-label")
            || matches!(
                &setting.control,
                Control::TextField { key, .. } if key == super::super::PRESET_NAME_KEY
            )
            || matches!(&setting.control, Control::Presets { .. });
        let field = picker_layout_field(
            setting,
            index,
            source.values,
            has_selected_preset,
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
    .style(crate::frontend::ui::scroll_style(with_alpha(palette.outline, 0.72)))
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
                    color: with_alpha(Color::BLACK, 0.4 * reveal),
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
        .align_x(iced::alignment::Horizontal::Left)
        .align_y(iced::alignment::Vertical::Center)
        .padding(Padding { top: 0.0, right: 0.0, bottom: 0.0, left: 18.0 * scale })
        .into()
}

fn picker_layout_field<'a>(
    setting: Row,
    index: usize,
    values: &'a HashMap<String, String>,
    has_selected_preset: bool,
    focus: FocusCtx<'_>,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'a, Message> {
    if setting.control.is_inline_editor() {
        let (focused, _) = focus.on(SettingsFocus::Controls, index);
        return compact_inline_field(
            setting,
            index,
            values,
            has_selected_preset,
            focused,
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
            has_selected_preset,
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
        return compact_action_field(
            setting,
            index,
            focused,
            available_width,
            scale,
            palette,
            fade,
        );
    }
    let locked = focus.active_input.is_some_and(|key| match &setting.control {
        Control::Number { key: control_key, .. } | Control::TextField { key: control_key, .. } => {
            control_key == key
        }
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
        has_selected_preset,
        focused,
        focused_choice,
        available_width,
        scale * 0.94,
        palette,
        fade,
        focus.armed,
        motion,
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

fn navigation<'a>(
    tabs: &[(&'static str, &'static str)],
    active_tab: usize,
    selected_tab: &str,
    sections: &[(usize, &'static str)],
    selected_section: usize,
    index_focused: bool,
    section_focused: bool,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    branch_reveal: f32,
    search_open: bool,
    search_query: &'a str,
    search_results: &'a [SettingsSearchResult],
) -> Element<'a, Message> {
    let search = search_control(search_open, search_query, search_results, scale, palette, fade);
    let content: Element<'a, Message> = if search_open {
        search_result_list(search_query, search_results, scale, palette, fade)
    } else {
        let mut tree = column![].spacing(1.0 * scale);
        for (key, label) in tabs.iter().copied() {
            let active = key == selected_tab;
            let category = button(
                row![
                    text(if active { "▾" } else { "▸" })
                        .font(crate::frontend::ui::UI_FONT)
                        .size(10.0 * legible_type_scale(scale)),
                    text(sentence_case(label))
                        .font(crate::frontend::ui::UI_FONT)
                        .size(12.5 * legible_type_scale(scale)),
                    container(text("")).width(Length::Fill),
                ]
                .spacing(9.0 * scale)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding([8.0 * scale, 8.0 * scale])
            .on_press(Message::Settings(SettingsMsg::SelectTab(key.to_string())))
            .style(move |_theme, status| {
                line_button_style(active, index_focused && active, palette, fade, status)
            });
            tree = tree.push(category);
            if active {
                tree = tree.push(section_branch(
                    sections,
                    selected_section,
                    section_focused,
                    scale,
                    palette,
                    fade,
                    branch_reveal,
                ));
            }
        }
        tree.into()
    };
    let tree = scrollable(content)
        .direction(iced::widget::scrollable::Direction::Vertical(
            iced::widget::scrollable::Scrollbar::hidden(),
        ))
        .height(Length::Fill);
    let tracker: Element<'a, Message> = if search_open {
        container(text("")).width(Length::Fixed(14.0 * scale)).into()
    } else {
        iced::widget::canvas(IndexRail {
            active: active_tab,
            count: tabs.len(),
            palette: *palette,
            fade,
        })
        .width(Length::Fixed(14.0 * scale.max(0.9)))
        .height(Length::Fill)
        .into()
    };
    crate::frontend::ui::folio_index_shell(
        tr("settings-nav-heading"),
        category_note(selected_tab),
        vec![search, row![tree, tracker].spacing(9.0 * scale).height(Length::Fill).into()],
        17.0,
        scale,
        palette,
    )
}

fn search_control<'a>(
    open: bool,
    query: &'a str,
    results: &[SettingsSearchResult],
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    if open {
        let submit = results.first().map_or(Message::Noop, |result| {
            Message::Settings(SettingsMsg::OpenSearchResult(
                result.tab.clone(),
                result.section,
                result.row,
            ))
        });
        row![
            label("⌕", 15.0, scale, with_alpha(palette.primary, fade)),
            text_input(tr("settings-nav-search"), query)
                .id(super::super::settings_search_input_id())
                .font(crate::frontend::ui::UI_FONT)
                .on_input(|value| Message::Settings(SettingsMsg::SearchInput(value)))
                .on_submit(submit)
                .size(12.0 * legible_type_scale(scale))
                .padding([7.0 * scale, 6.0 * scale])
                .width(Length::Fill)
                .style(move |_theme, _status| {
                    crate::frontend::ui::workbench_input_style(
                        with_alpha(palette.surface_text, fade),
                        with_alpha(palette.primary, fade),
                    )
                }),
            button(
                text("×").font(crate::frontend::ui::UI_FONT).size(14.0 * legible_type_scale(scale))
            )
            .padding([5.0 * scale, 8.0 * scale])
            .on_press(Message::Settings(SettingsMsg::SearchClose))
            .style(move |_theme, status| {
                line_button_style(false, false, palette, fade, status)
            }),
        ]
        .spacing(7.0 * scale)
        .align_y(Alignment::Center)
        .into()
    } else {
        button(
            row![
                label("⌕", 15.0, scale, with_alpha(palette.primary, fade)),
                text(tr("settings-nav-search"))
                    .font(crate::frontend::ui::UI_FONT)
                    .size(11.5 * legible_type_scale(scale)),
                container(text("")).width(Length::Fill),
                label(
                    tr("settings-nav-search-shortcut"),
                    9.0,
                    scale,
                    with_alpha(palette.surface_text, 0.42 * fade)
                ),
            ]
            .spacing(8.0 * scale)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([8.0 * scale, 8.0 * scale])
        .on_press(Message::Settings(SettingsMsg::SearchOpen))
        .style(move |_theme, status| line_button_style(false, false, palette, fade, status))
        .into()
    }
}

fn search_result_list<'a>(
    query: &str,
    results: &'a [SettingsSearchResult],
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    if results.is_empty() {
        return container(label(
            if query.trim().is_empty() {
                tr("settings-nav-search-empty-hint")
            } else {
                tr("settings-nav-search-no-results")
            },
            10.5,
            scale,
            with_alpha(palette.surface_text, 0.5 * fade),
        ))
        .padding([12.0 * scale, 8.0 * scale])
        .into();
    }
    let mut list = column![].spacing(1.0 * scale);
    for result in results {
        list = list.push(
            button(
                column![
                    text(&result.title)
                        .font(crate::frontend::ui::UI_FONT)
                        .size(12.0 * legible_type_scale(scale)),
                    label(
                        format!(
                            "{}  /  {}",
                            sentence_case(&result.tab_label),
                            result.section_title,
                        ),
                        9.0,
                        scale,
                        with_alpha(palette.primary, 0.72 * fade),
                    ),
                ]
                .spacing(3.0 * scale),
            )
            .width(Length::Fill)
            .padding([9.0 * scale, 8.0 * scale])
            .on_press(Message::Settings(SettingsMsg::OpenSearchResult(
                result.tab.clone(),
                result.section,
                result.row,
            )))
            .style(move |_theme, status| line_button_style(false, false, palette, fade, status)),
        );
    }
    list.into()
}

fn section_branch<'a>(
    sections: &[(usize, &'static str)],
    selected: usize,
    keyboard_focused: bool,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    reveal: f32,
) -> Element<'a, Message> {
    let mut children = column![].spacing(1.0 * scale);
    for (index, title) in sections.iter().copied() {
        let active = index == selected;
        children = children.push(
            button(
                row![
                    label(
                        if active { "◆" } else { "◇" },
                        8.0,
                        scale,
                        with_alpha(palette.primary, if active { fade } else { 0.44 * fade }),
                    ),
                    text(title)
                        .font(crate::frontend::ui::UI_FONT)
                        .size(11.5 * legible_type_scale(scale)),
                    container(text("")).width(Length::Fill),
                ]
                .spacing(9.0 * scale)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding([7.0 * scale, 7.0 * scale])
            .on_press(Message::Settings(SettingsMsg::SelectSection(index)))
            .style(move |_theme, status| {
                line_button_style(active, keyboard_focused && active, palette, fade, status)
            }),
        );
    }
    let guide =
        container(text("")).width(Length::Fixed(1.0)).height(Length::Fill).style(move |_| {
            crate::frontend::ui::bg_style(Background::Color(with_alpha(
                palette.primary,
                0.42 * fade * reveal,
            )))
        });
    container(row![guide, children].spacing(9.0 * scale))
        .width(Length::Fill)
        .height(Length::Fixed(branch_height(sections.len(), scale, reveal)))
        .padding(Padding { top: 2.0 * scale, right: 0.0, bottom: 4.0 * scale, left: 17.0 * scale })
        .clip(true)
        .into()
}

fn branch_height(section_count: usize, scale: f32, reveal: f32) -> f32 {
    (section_count as f32 * 36.0 * scale.max(1.0) + 6.0 * scale) * reveal.clamp(0.0, 1.0)
}

struct ReadingInput<'a> {
    tab: &'a str,
    card: Card,
    settings: Vec<Row>,
    transition_preview: Option<iced::advanced::image::Handle>,
    values: &'a HashMap<String, String>,
    has_selected_preset: bool,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
}

fn take_transition_preview(settings: &mut Vec<Row>) -> bool {
    let present = settings.iter().any(|row| matches!(row.control, Control::Preview));
    settings.retain(|row| !matches!(row.control, Control::Preview));
    present
}

fn transition_preview_stage<'a>(
    handle: Option<iced::advanced::image::Handle>,
    width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    let height = (width * 9.0 / 16.0).clamp(250.0 * scale, 430.0 * scale);
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

fn reading_surface<'a>(input: ReadingInput<'a>, focus: FocusCtx<'_>) -> Element<'a, Message> {
    let ReadingInput {
        tab,
        card,
        mut settings,
        transition_preview,
        values,
        has_selected_preset,
        available_width,
        scale,
        palette,
        fade,
        motion,
    } = input;
    let (section_title, section_subtitle) = (card.title, card.subtitle);
    let header = column![
        row![
            text(section_title)
                .font(crate::frontend::ui::UI_FONT)
                .size(46.0 * scale.clamp(0.88, 1.08))
                .line_height(iced::widget::text::LineHeight::Relative(1.0))
                .color(with_alpha(palette.surface_text, fade)),
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
        .align_y(Alignment::Start),
        container(
            label(
                if section_subtitle.is_empty() { category_note(tab) } else { section_subtitle },
                TYPE_SMALL,
                scale,
                with_alpha(palette.surface_text, 0.56 * fade),
            )
            .line_height(iced::widget::text::LineHeight::Relative(1.4)),
        )
        .width(Length::Fill),
        folio_horizontal_rule(with_alpha(palette.outline, FOLIO_RULE_ALPHA * fade)),
    ]
    .spacing(12.0 * scale);
    let content_width = (available_width - 76.0 * scale).max(480.0 * scale);
    let column_width = ((content_width - 34.0 * scale) * 0.5).max(230.0 * scale);
    let has_transition_preview = take_transition_preview(&mut settings);
    let controls = field_grid(
        settings,
        tab == "sources",
        values,
        has_selected_preset,
        focus,
        column_width,
        scale,
        palette,
        fade,
        motion,
    );
    let mut page = column![header, controls].spacing(28.0 * scale);
    if has_transition_preview {
        page = page.push(transition_preview_stage(
            transition_preview,
            content_width,
            scale,
            palette,
            fade,
        ));
    }
    scrollable(container(page).width(Length::Fill).padding(Padding {
        top: 34.0 * scale,
        right: 38.0 * scale,
        bottom: 48.0 * scale,
        left: 38.0 * scale,
    }))
    .direction(iced::widget::scrollable::Direction::Vertical(
        iced::widget::scrollable::Scrollbar::hidden(),
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn field_grid<'a>(
    settings: Vec<Row>,
    full_width: bool,
    values: &'a HashMap<String, String>,
    has_selected_preset: bool,
    focus: FocusCtx<'_>,
    column_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'a, Message> {
    if settings.is_empty() {
        return label(
            tr("settings-section-empty"),
            12.0,
            scale,
            with_alpha(palette.surface_text, 0.56 * fade),
        )
        .into();
    }

    let mut remaining: Vec<_> = settings.into_iter().enumerate().collect();
    let mut page = column![].spacing(30.0 * scale);
    while !remaining.is_empty() {
        if full_width || remaining[0].1.control.is_wide_field() {
            let (index, setting) = remaining.remove(0);
            page = page.push(field(
                setting,
                index,
                values,
                has_selected_preset,
                focus,
                column_width * 2.0 + 34.0 * scale,
                scale,
                palette,
                fade,
                motion,
            ));
            continue;
        }
        let compact = remaining[0].1.control.is_compact_field();
        let count = remaining
            .iter()
            .take_while(|(_, row)| row.control.is_compact_field() == compact)
            .count();
        let group: Vec<_> = remaining.drain(..count).collect();
        page = page.push(if compact {
            compact_field_grid(
                group,
                values,
                has_selected_preset,
                focus,
                column_width,
                scale,
                palette,
                fade,
                motion,
            )
        } else {
            regular_field_grid(
                group,
                values,
                has_selected_preset,
                focus,
                column_width,
                scale,
                palette,
                fade,
                motion,
            )
        });
    }
    page.into()
}

#[allow(clippy::too_many_arguments)]
fn regular_field_grid<'a>(
    settings: Vec<(usize, Row)>,
    values: &'a HashMap<String, String>,
    has_selected_preset: bool,
    focus: FocusCtx<'_>,
    column_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'a, Message> {
    let mut grid = column![].spacing(30.0 * scale);
    let mut settings = settings.into_iter();
    while let Some((left_index, left)) = settings.next() {
        let left = field(
            left,
            left_index,
            values,
            has_selected_preset,
            focus,
            column_width,
            scale,
            palette,
            fade,
            motion,
        );
        let pair: Element<'a, Message> = if let Some((right_index, right)) = settings.next() {
            let right = field(
                right,
                right_index,
                values,
                has_selected_preset,
                focus,
                column_width,
                scale,
                palette,
                fade,
                motion,
            );
            row![
                container(left).width(Length::FillPortion(1)),
                container(text("")).width(Length::Fixed(1.0)).height(Length::Fill).style(
                    move |_| crate::frontend::ui::bg_style(Background::Color(with_alpha(
                        palette.outline,
                        0.34 * fade,
                    )))
                ),
                container(right).width(Length::FillPortion(1)),
            ]
            .spacing(17.0 * scale)
            .align_y(Alignment::Start)
            .into()
        } else {
            row![
                container(left).width(Length::FillPortion(1)),
                container(text("")).width(Length::FillPortion(1)),
            ]
            .spacing(35.0 * scale)
            .align_y(Alignment::Start)
            .into()
        };
        grid = grid.push(pair);
    }
    grid.into()
}

#[allow(clippy::too_many_arguments)]
fn compact_field_grid<'a>(
    mut settings: Vec<(usize, Row)>,
    values: &'a HashMap<String, String>,
    has_selected_preset: bool,
    focus: FocusCtx<'_>,
    column_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'a, Message> {
    let right = settings.split_off(settings.len().div_ceil(2));
    let mut left_column = column![].spacing(0.0);
    for (index, setting) in settings {
        left_column = left_column.push(field(
            setting,
            index,
            values,
            has_selected_preset,
            focus,
            column_width,
            scale,
            palette,
            fade,
            motion,
        ));
    }
    let mut right_column = column![].spacing(0.0);
    for (index, setting) in right {
        right_column = right_column.push(field(
            setting,
            index,
            values,
            has_selected_preset,
            focus,
            column_width,
            scale,
            palette,
            fade,
            motion,
        ));
    }
    row![
        container(left_column).width(Length::FillPortion(1)),
        container(text("")).width(Length::Fixed(1.0)).height(Length::Fill).style(move |_| {
            crate::frontend::ui::bg_style(Background::Color(with_alpha(
                palette.outline,
                0.34 * fade,
            )))
        }),
        container(right_column).width(Length::FillPortion(1)),
    ]
    .spacing(17.0 * scale)
    .align_y(Alignment::Start)
    .into()
}

fn compact_field_text(title: &str, desc: &str, copy_width: f32, scale: f32) -> (String, String) {
    (
        crate::frontend::ui::ellipsize_text(
            title.trim(),
            12.0 * legible_type_scale(scale),
            copy_width,
        ),
        crate::frontend::ui::ellipsize_text(
            desc.trim(),
            TYPE_SMALL * legible_type_scale(scale),
            copy_width * 3.0,
        ),
    )
}

fn compact_field_copy(
    title: &str,
    desc: &str,
    copy_width: f32,
    scale: f32,
    palette: &Palette,
    fade: f32,
) -> Element<'static, Message> {
    let (title, description) = compact_field_text(title, desc, copy_width, scale);
    if description.is_empty() {
        label(title, 12.0, scale, with_alpha(palette.surface_text, fade)).into()
    } else {
        column![
            label(title, 12.0, scale, with_alpha(palette.surface_text, fade)),
            label(description, TYPE_SMALL, scale, with_alpha(palette.surface_text, 0.52 * fade))
                .width(Length::Fixed(copy_width))
                .line_height(iced::widget::text::LineHeight::Relative(1.12))
                .wrapping(iced::widget::text::Wrapping::Word),
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
    has_selected_preset: bool,
    focused: bool,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'a, Message> {
    let Row { title, desc, control } = setting;
    let editor_width = match &control {
        Control::Number { .. } => (available_width * 0.34).clamp(118.0 * scale, 176.0 * scale),
        Control::TextField { .. } => (available_width * 0.42).clamp(142.0 * scale, 248.0 * scale),
        Control::KeyBinding { .. } => (available_width * 0.4).clamp(132.0 * scale, 210.0 * scale),
        _ => unreachable!("compact inline field requires one scalar editor"),
    };
    let copy_width = (available_width - editor_width - 44.0 * scale).max(72.0 * scale);
    let copy = compact_field_copy(&title, &desc, copy_width, scale, palette, fade);
    let editor = control::widget(
        control,
        values,
        has_selected_preset,
        focused,
        None,
        editor_width,
        scale * 0.78,
        palette,
        fade,
        None,
        motion,
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

#[allow(clippy::too_many_arguments)]
fn compact_action_field(
    setting: Row,
    index: usize,
    focused: bool,
    available_width: f32,
    scale: f32,
    palette: &Palette,
    fade: f32,
) -> Element<'_, Message> {
    let Row { title, desc, control } = setting;
    let Control::Toggle { path, value } = control else {
        unreachable!("compact action field requires a toggle");
    };
    let enabled = tr("settings-control-enabled");
    let disabled = tr("settings-control-disabled");
    let action_scale = scale * 0.82;
    let action_width = crate::frontend::ui::folio_action_width(enabled, action_scale)
        .max(crate::frontend::ui::folio_action_width(disabled, action_scale));
    let copy_width = (available_width - action_width - 44.0 * scale).max(72.0 * scale);
    let copy = compact_field_copy(&title, &desc, copy_width, scale, palette, fade);
    let action = crate::frontend::ui::folio_action(
        if value { enabled } else { disabled },
        value,
        Some(Message::Settings(SettingsMsg::Toggle(path, !value))),
        Length::Fixed(action_width),
        action_scale,
        palette,
    );
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

#[allow(clippy::too_many_arguments)]
fn compact_group_field<'a>(
    setting: Row,
    index: usize,
    values: &'a HashMap<String, String>,
    has_selected_preset: bool,
    focus: FocusCtx<'_>,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'a, Message> {
    let Row { title, desc, control } = setting;
    let title = if title.is_empty() {
        tr("settings-selector-saved-styles-label").to_string()
    } else {
        title
    };
    let id = control.compact_bar_id().expect("compact group requires a bar identity");
    let (expanded, reveal) = focus.bar(&id);
    let (focused, _) = focus.on(SettingsFocus::Controls, index);
    let is_stack_bar = matches!(&control, Control::StackBar { .. });
    let description_in_tray = !is_stack_bar;
    let summary = match &control {
        Control::MotionWeights { weights } => {
            weights
                .iter()
                .map(|(_, key, _)| values.get(key).map_or("0", String::as_str))
                .collect::<Vec<_>>()
                .join(" / ")
                + " ms"
        }
        Control::StackBar { summary, .. } => summary.clone(),
        _ => unreachable!("compact group requires multiple editors"),
    };
    let content: Element<'a, Message> = if let Control::StackBar { preview, .. } = &control {
        let preview_width = 86.0 * scale;
        let preview_height = 48.0 * scale;
        let identity_width = (available_width * 0.24).clamp(110.0 * scale, 185.0 * scale);
        let wallpaper_width =
            (available_width - preview_width - identity_width - 96.0 * scale).max(100.0 * scale);
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
        let title = crate::frontend::ui::ellipsize_text(
            title.trim(),
            12.5 * legible_type_scale(scale),
            identity_width,
        );
        let desc = crate::frontend::ui::ellipsize_text(
            desc.trim(),
            TYPE_SMALL * legible_type_scale(scale),
            identity_width,
        );
        let summary = crate::frontend::ui::ellipsize_text(
            summary.trim(),
            TYPE_SMALL * legible_type_scale(scale),
            wallpaper_width,
        );
        row![
            preview,
            column![
                label(title, 12.5, scale, with_alpha(palette.surface_text, fade)),
                label(desc, TYPE_SMALL, scale, with_alpha(palette.surface_text, 0.52 * fade)),
            ]
            .width(Length::Fixed(identity_width))
            .spacing(1.0 * scale),
            container(text("")).width(Length::Fill),
            label(summary, TYPE_SMALL, scale, with_alpha(palette.primary, fade))
                .width(Length::Fixed(wallpaper_width))
                .wrapping(iced::widget::text::Wrapping::None)
                .align_x(iced::alignment::Horizontal::Right),
        ]
        .spacing(12.0 * scale)
        .align_y(Alignment::Center)
        .into()
    } else {
        row![
            label(title, 12.5, scale, with_alpha(palette.surface_text, fade)),
            container(text("")).width(Length::Fill),
            label(summary, TYPE_SMALL, scale, with_alpha(palette.primary, fade),)
                .width(Length::Fixed((available_width * 0.42).max(90.0 * scale)))
                .wrapping(iced::widget::text::Wrapping::None)
                .align_x(iced::alignment::Horizontal::Right),
        ]
        .spacing(10.0 * scale)
        .align_y(Alignment::Center)
        .into()
    };
    let (editor, options_height) = match control {
        Control::StackBar { rows, .. } => (
            detail_rows(
                rows,
                index,
                values,
                has_selected_preset,
                available_width - 18.0 * scale,
                scale,
                palette,
                fade,
                focus,
                motion,
            ),
            176.0,
        ),
        control => (
            control::widget(
                control,
                values,
                has_selected_preset,
                focused,
                None,
                available_width - 42.0 * scale,
                scale,
                palette,
                fade,
                focus.armed,
                motion,
            ),
            110.0,
        ),
    };
    let options: Element<'a, Message> = if desc.trim().is_empty() || !description_in_tray {
        editor
    } else {
        column![
            label(desc, TYPE_SMALL, scale, with_alpha(palette.surface_text, 0.58 * fade),),
            editor,
        ]
        .spacing(6.0 * scale)
        .into()
    };
    crate::frontend::ui::folio_stack_bar(
        content,
        expanded,
        expanded,
        reveal,
        Message::Settings(SettingsMsg::ToggleBar(id, index)),
        None,
        options,
        if is_stack_bar { 68.0 } else { 42.0 },
        options_height,
        scale,
        palette,
    )
}

fn field<'a>(
    setting: Row,
    index: usize,
    values: &'a HashMap<String, String>,
    has_selected_preset: bool,
    focus: FocusCtx<'_>,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'a, Message> {
    if setting.control.is_inline_editor() {
        let (focused, _) = focus.on(SettingsFocus::Controls, index);
        return compact_inline_field(
            setting,
            index,
            values,
            has_selected_preset,
            focused,
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
            has_selected_preset,
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
        return compact_action_field(
            setting,
            index,
            focused,
            available_width,
            scale,
            palette,
            fade,
        );
    }
    let Row { title, desc, control } = setting;
    if let Control::Details { id, summary, rows } = control {
        let expanded = focus.expanded_details.contains(&id);
        let body = detail_rows(
            rows,
            index,
            values,
            has_selected_preset,
            available_width,
            scale,
            palette,
            fade,
            focus,
            motion,
        );
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
        Control::Number { key: control_key, .. } | Control::TextField { key: control_key, .. } => {
            control_key == key
        }
        Control::MotionWeights { weights } => {
            weights.iter().any(|(_, control_key, _)| control_key == key)
        }
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
        has_selected_preset,
        focused,
        focused_choice,
        available_width,
        scale,
        palette,
        fade,
        focus.armed,
        motion,
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
fn detail_rows<'a>(
    rows: Vec<Row>,
    parent_index: usize,
    values: &'a HashMap<String, String>,
    has_selected_preset: bool,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    focus: FocusCtx<'_>,
    motion: MotionProfile,
) -> Element<'a, Message> {
    let mut content = column![].spacing(15.0 * scale);
    for row in rows {
        if row.control.is_inline_editor() {
            content = content.push(compact_inline_field(
                row,
                parent_index,
                values,
                has_selected_preset,
                false,
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
                has_selected_preset,
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
            content = content.push(compact_action_field(
                row,
                parent_index,
                false,
                (available_width - 24.0 * scale).max(160.0 * scale),
                scale * 0.94,
                palette,
                fade,
            ));
            continue;
        }
        let Row { title, desc, control } = row;
        let widget = control::widget(
            control,
            values,
            has_selected_preset,
            false,
            None,
            (available_width - 24.0 * scale).max(160.0 * scale),
            scale * 0.94,
            palette,
            fade,
            focus.armed,
            motion,
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

fn line_button_style(
    active: bool,
    focused: bool,
    palette: &Palette,
    fade: f32,
    status: button::Status,
) -> button::Style {
    crate::frontend::ui::folio_line_button_style(active, focused, palette, fade, status)
}

fn tracker_position(active: usize, count: usize) -> f32 {
    if count <= 1 { 0.0 } else { active.min(count - 1) as f32 / (count - 1) as f32 }
}

struct IndexRail {
    active: usize,
    count: usize,
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
        for index in 0..self.count {
            let y = top + (bottom - top) * tracker_position(index, self.count);
            frame.stroke(
                &Path::line(Point::new(x - 2.5, y), Point::new(x + 2.5, y)),
                Stroke::default()
                    .with_color(with_alpha(self.palette.outline, 0.58 * self.fade))
                    .with_width(1.0),
            );
        }
        let y = top + (bottom - top) * tracker_position(self.active, self.count);
        let radius = 5.0;
        let diamond = Path::new(|builder| {
            builder.move_to(Point::new(x, y - radius));
            builder.line_to(Point::new(x + radius, y));
            builder.line_to(Point::new(x, y + radius));
            builder.line_to(Point::new(x - radius, y));
            builder.close();
        });
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
        let line = with_alpha(self.palette.outline, 0.075 * self.fade);
        let accent = with_alpha(self.palette.primary, 0.09 * self.fade);
        let centre = Point::new(bounds.width * 0.72, bounds.height * 0.56);
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
        let marker = Path::new(|builder| {
            builder.move_to(Point::new(centre.x, centre.y - 7.0));
            builder.line_to(Point::new(centre.x + 7.0, centre.y));
            builder.line_to(Point::new(centre.x, centre.y + 7.0));
            builder.line_to(Point::new(centre.x - 7.0, centre.y));
            builder.close();
        });
        frame.stroke(&marker, Stroke::default().with_color(accent).with_width(1.3));
        vec![frame.into_geometry()]
    }
}
