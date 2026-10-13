use iced::widget::{column, container, image, mouse_area, scrollable, stack, text, text_input};
use iced::{Alignment, Background, ContentFit, Element, Length};

use crate::app::Message;
use crate::frontend::components::with_alpha;
use crate::frontend::settings::{SettingsMsg, WeatherReviewTab};
use crate::frontend::theme::Palette;
use crate::frontend::ui::{TYPE_SMALL, label, row};
use crate::i18n::{tr, tr_args};

const REVIEW_WIDTH: f32 = 620.0;
const REVIEW_HEIGHT: f32 = 560.0;
const THUMB_WIDTH: f32 = 88.0;
const THUMB_HEIGHT: f32 = 60.0;
const ITEM_PADDING: f32 = 8.0;
const ITEM_GAP: f32 = 10.0;
const PANEL_PADDING: f32 = 20.0;
const SEARCH_FONT: f32 = 12.0;
const TITLE_FONT: f32 = 20.0;
const BUTTON_SCALE: f32 = 0.78;
const TAB_WIDTH: f32 = 116.0;
const TOGGLE_WIDTH: f32 = 96.0;
const CLOSE_WIDTH: f32 = 76.0;
const ITEM_NAME_FONT: f32 = 13.0;
const ITEM_TEXT_GAP: f32 = 3.0;
const PANEL_MIN_SCALE: f32 = 0.9;
const THUMB_TEXT_ALPHA: f32 = 0.66;
const THUMB_SURFACE_ALPHA: f32 = 0.7;
const BORDER_ALPHA: f32 = 0.4;
const KEY_ALPHA: f32 = 0.56;
const FOCUSED_SURFACE_ALPHA: f32 = 0.86;
const IDLE_SURFACE_ALPHA: f32 = 0.55;
const LIMIT_TEXT_ALPHA: f32 = 0.7;
const HELP_TEXT_ALPHA: f32 = 0.72;
const ERROR_TEXT_ALPHA: f32 = 0.9;
const SCRIM_ALPHA: f32 = 0.6;
const SUGGESTED_LABEL: &str = "settings-weather-review-suggested";
const APPROVED_LABEL: &str = "settings-weather-review-approved";
const LIBRARY_LABEL: &str = "settings-weather-review-library";
const ERROR_LABEL: &str = "settings-weather-review-error";
const TITLE_LABEL: &str = "settings-weather-review-title";

pub struct WeatherReviewItem {
    pub key: String,
    pub name: String,
    pub thumbnail: String,
    pub approved: bool,
    pub missing: bool,
}

pub struct WeatherReviewView {
    pub category: String,
    pub tab: WeatherReviewTab,
    pub query: String,
    pub items: Vec<WeatherReviewItem>,
    pub loading: bool,
    pub error: Option<String>,
    pub focus: usize,
    pub library_limited: bool,
}

fn tab_button<'a>(
    tab: WeatherReviewTab,
    label_key: &'static str,
    active: bool,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    crate::frontend::ui::folio_action(
        tr(label_key),
        active,
        Some(Message::Settings(SettingsMsg::WeatherReviewTab(tab))),
        Length::Fixed(TAB_WIDTH * scale),
        scale * BUTTON_SCALE,
        palette,
    )
}

#[expect(
    clippy::too_many_lines,
    reason = "one weather review panel composes its tabs, entries, and controls"
)]
pub fn layer<'a>(
    review: WeatherReviewView,
    scale: f32,
    palette: &'a Palette,
    reveal: f32,
) -> Element<'a, Message> {
    let WeatherReviewView { category, tab, query, items, loading, error, focus, library_limited } =
        review;
    let tabs = row![
        tab_button(
            WeatherReviewTab::Suggested,
            SUGGESTED_LABEL,
            tab == WeatherReviewTab::Suggested || focus == 0,
            scale,
            palette
        ),
        tab_button(
            WeatherReviewTab::Approved,
            APPROVED_LABEL,
            tab == WeatherReviewTab::Approved || focus == 1,
            scale,
            palette
        ),
        tab_button(
            WeatherReviewTab::Library,
            LIBRARY_LABEL,
            tab == WeatherReviewTab::Library || focus == 2,
            scale,
            palette
        ),
    ]
    .spacing(ITEM_GAP * scale);
    let empty = items.is_empty();
    let mut rows = column![].spacing(ITEM_GAP * scale);
    for (index, item) in items.into_iter().enumerate() {
        let art: Element<'a, Message> = if item.thumbnail.is_empty() || item.missing {
            container(label(
                if item.missing {
                    tr("settings-weather-review-missing")
                } else {
                    tr("settings-weather-review-no-preview")
                },
                TYPE_SMALL,
                scale,
                with_alpha(palette.surface_text, THUMB_TEXT_ALPHA),
            ))
            .width(Length::Fixed(THUMB_WIDTH * scale))
            .height(Length::Fixed(THUMB_HEIGHT * scale))
            .center(Length::Fill)
            .style(move |_| {
                crate::frontend::ui::box_style(
                    with_alpha(palette.surface_container, THUMB_SURFACE_ALPHA),
                    with_alpha(palette.outline, BORDER_ALPHA),
                )
            })
            .into()
        } else {
            image(image::Handle::from_path(item.thumbnail))
                .content_fit(ContentFit::Cover)
                .width(Length::Fixed(THUMB_WIDTH * scale))
                .height(Length::Fixed(THUMB_HEIGHT * scale))
                .into()
        };
        let action = crate::frontend::ui::folio_action(
            if item.approved {
                tr("settings-weather-review-remove")
            } else {
                tr("settings-weather-review-include")
            },
            item.approved || focus == index + crate::frontend::settings::WEATHER_REVIEW_TAB_COUNT,
            Some(Message::Settings(SettingsMsg::WeatherReviewToggle(item.key.clone()))),
            Length::Fixed(TOGGLE_WIDTH * scale),
            scale * BUTTON_SCALE,
            palette,
        );
        rows = rows.push(
            container(
                row![
                    art,
                    column![
                        label(item.name, ITEM_NAME_FONT, scale, palette.surface_text),
                        label(
                            item.key,
                            TYPE_SMALL,
                            scale,
                            with_alpha(palette.surface_text, KEY_ALPHA)
                        ),
                    ]
                    .spacing(ITEM_TEXT_GAP * scale)
                    .width(Length::Fill),
                    action,
                ]
                .spacing(ITEM_GAP * scale)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding(ITEM_PADDING * scale)
            .style(move |_| {
                crate::frontend::ui::box_style(
                    with_alpha(
                        palette.surface_container,
                        if focus == index + crate::frontend::settings::WEATHER_REVIEW_TAB_COUNT {
                            FOCUSED_SURFACE_ALPHA
                        } else {
                            IDLE_SURFACE_ALPHA
                        },
                    ),
                    with_alpha(
                        if focus == index + crate::frontend::settings::WEATHER_REVIEW_TAB_COUNT {
                            palette.primary
                        } else {
                            palette.outline
                        },
                        BORDER_ALPHA,
                    ),
                )
            }),
        );
    }
    if loading && tab == WeatherReviewTab::Suggested {
        rows = rows.push(label(
            tr("settings-weather-review-loading"),
            TYPE_SMALL,
            scale,
            palette.surface_text,
        ));
    } else if empty {
        rows = rows.push(label(
            tr("settings-weather-review-empty"),
            TYPE_SMALL,
            scale,
            palette.surface_text,
        ));
    }
    if library_limited && tab == WeatherReviewTab::Library {
        rows = rows.push(label(
            tr("settings-weather-review-library-limit"),
            TYPE_SMALL,
            scale,
            with_alpha(palette.surface_text, LIMIT_TEXT_ALPHA),
        ));
    }
    let error_text =
        error.map_or_else(String::new, |detail| tr_args!(ERROR_LABEL, detail => detail));
    let close = crate::frontend::ui::folio_action(
        tr("settings-weather-review-close"),
        true,
        Some(Message::Settings(SettingsMsg::WeatherReviewClose)),
        Length::Fixed(CLOSE_WIDTH * scale),
        scale * BUTTON_SCALE,
        palette,
    );
    let body = column![
        row![
            label(
                tr_args!(TITLE_LABEL, category => super::super::tabs::weather_category_label(&category)),
                TITLE_FONT,
                scale,
                palette.surface_text
            ),
            container(text("")).width(Length::Fill),
            close,
        ]
        .align_y(Alignment::Center),
        label(
            tr("settings-weather-review-help"),
            TYPE_SMALL,
            scale,
            with_alpha(palette.surface_text, HELP_TEXT_ALPHA)
        ),
        tabs,
        text_input(tr("settings-weather-review-search"), &query)
            .id(crate::frontend::settings::weather_review_input_id())
            .font(crate::frontend::ui::UI_FONT)
            .on_input(|value| Message::Settings(SettingsMsg::WeatherReviewSearch(value)))
            .size(SEARCH_FONT * crate::frontend::ui::legible_type_scale(scale))
            .padding([ITEM_PADDING * scale, ITEM_GAP * scale])
            .style(move |_theme, _status| crate::frontend::ui::workbench_input_style(
                palette.surface_text,
                palette.primary
            )),
        label(error_text, TYPE_SMALL, scale, with_alpha(palette.destructive(), ERROR_TEXT_ALPHA)),
        scrollable(rows)
            .height(Length::Fill)
            .direction(crate::frontend::ui::thin_vbar())
            .style(crate::frontend::ui::scroll_style(palette.primary)),
    ]
    .spacing(ITEM_GAP * scale);
    let panel = container(body)
        .width(Length::Fixed(REVIEW_WIDTH * scale.max(PANEL_MIN_SCALE)))
        .height(Length::Fixed(REVIEW_HEIGHT * scale.max(PANEL_MIN_SCALE)))
        .padding(PANEL_PADDING * scale)
        .style(move |_| crate::frontend::ui::folio_sheet_panel_style(palette, reveal));
    let centered = container(iced::widget::opaque(panel))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);
    stack![
        mouse_area(container(text("")).width(Length::Fill).height(Length::Fill).style(move |_| {
            crate::frontend::ui::bg_style(Background::Color(with_alpha(
                palette.background,
                SCRIM_ALPHA * reveal,
            )))
        }))
        .on_press(Message::Settings(SettingsMsg::WeatherReviewClose)),
        centered,
    ]
    .into()
}
