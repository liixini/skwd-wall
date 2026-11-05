use iced::widget::{button, container, text};
use iced::{Element, Length, Padding, Rectangle, Size};

use crate::app::update::arrange::placement;
use crate::app::{App, Message};
use crate::domain::strand::zone_top;
use crate::frontend::arrange::{ArrangeMsg, Target};
use crate::frontend::strand::tokens::FRAME;
use crate::frontend::ui::{self, UI_FONT};
use crate::infrastructure::config::Config;

const ENTRY: &str = "arrange-entry";
pub(crate) const ENTRY_PAD: f32 = 18.0;
const ENTRY_TYPE: f32 = 12.0;
const ENTRY_LINE: f32 = 1.3;
const ENTRY_BUTTON_PAD: f32 = 10.0;
const THREAD_CLEAR: f32 = 8.0;
const ENTRY_INSET: f32 = ENTRY_BUTTON_PAD + ENTRY_BUTTON_PAD;
pub(crate) const ENTRY_HEIGHT: f32 = ENTRY_TYPE * ENTRY_LINE + ENTRY_INSET;
pub(crate) const FLOAT_CLEAR: f32 =
    2.0 * (FRAME.widget_pad + FRAME.gem * FRAME.gem_grow) + FRAME.stem_min;

pub(crate) const fn rest_bottom(height: f32) -> f32 {
    let edge = height - ENTRY_PAD;
    let thread = zone_top(height) - THREAD_CLEAR;
    if edge < thread { edge } else { thread }
}

pub(crate) fn clear_bottom(
    app: &App,
    config: &Config,
    viewport: (f32, f32),
    corner: (f32, f32),
    size: Size,
) -> f32 {
    let (left, bottom) = corner;
    let context = super::layout::LayoutContext::projected(config, viewport);
    let panel = super::tag_cloud_bounds(app, &context);
    let near = Rectangle {
        x: panel.x - FLOAT_CLEAR,
        y: panel.y - FLOAT_CLEAR,
        width: panel.width + 2.0 * FLOAT_CLEAR,
        height: panel.height + 2.0 * FLOAT_CLEAR,
    };
    let rest =
        Rectangle { x: left, y: bottom - size.height, width: size.width, height: size.height };
    if rest.intersects(&near) { bottom.min(panel.y - FLOAT_CLEAR) } else { bottom }
}

pub(crate) fn float_left(app: &App, config: &Config, viewport: (f32, f32)) -> f32 {
    if config.filter_bar_orientation() != super::layout::VERTICAL {
        return ENTRY_PAD;
    }
    let context = super::layout::LayoutContext::projected(config, viewport);
    let bar = super::chrome::bar_bounds(app, &context);
    let right = bar.x + bar.width;
    if bar.x < viewport.0 - right { right + ENTRY_PAD } else { ENTRY_PAD }
}

pub(crate) fn arrange_bounds(app: &App, config: &Config, viewport: (f32, f32)) -> Rectangle {
    let width = ui::cached_line_width(crate::i18n::tr(ENTRY), ENTRY_TYPE, UI_FONT);
    let size = Size::new(width + ENTRY_INSET, ENTRY_HEIGHT);
    let left = float_left(app, config, viewport);
    let bottom = clear_bottom(app, config, viewport, (left, rest_bottom(viewport.1)), size);
    placement::floated(config.arrange_position(), (left, bottom), size, viewport)
}

pub(crate) fn pinned<'a>(
    app: &App,
    target: Target,
    plate: Element<'a, Message>,
) -> Element<'a, Message> {
    let rect = placement::drawn(app, target);
    let bottom = (app.scene.viewport.1 - rect.y - rect.height).max(0.0);
    container(plate)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(iced::alignment::Vertical::Bottom)
        .padding(Padding { left: rect.x, bottom, ..Padding::ZERO })
        .into()
}

pub(super) fn entry(app: &App) -> Element<'_, Message> {
    let label = text(crate::i18n::tr(ENTRY))
        .font(UI_FONT)
        .size(ENTRY_TYPE)
        .line_height(iced::widget::text::LineHeight::Relative(ENTRY_LINE));
    let plate = button(label)
        .padding(ENTRY_BUTTON_PAD)
        .on_press(Message::Arrange(ArrangeMsg::Begin))
        .style(move |_, status| ui::folio_button_style(false, false, &app.theme.palette, status));
    pinned(app, Target::Arrange, plate.into())
}

pub(super) fn editor(app: &App) -> Element<'_, Message> {
    let session = app.panels.arrange.as_ref().unwrap();
    let style = ui::BarVisualStyle::from_key(&app.config.filter_bar_visual_style());
    let edit_bar =
        app.chrome.bar.reorder.as_ref().map(|reorder| reorder.edit_bar(style, &app.theme.swatch));
    session.editor.view(edit_bar, &app.theme.palette).map(Message::Arrange)
}
