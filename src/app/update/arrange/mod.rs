mod board;
mod controls;
mod core;
pub(in crate::app) mod displays;
mod effects;
pub(in crate::app) mod inventory;
pub(in crate::app) mod placement;
pub(in crate::app) mod preview;

use core::{
    COLOR_GROUP, COLOR_PREFIX, CSV_SEPARATOR, SAVE_ERROR, STEP_BUTTON, STEP_DENSITY, STEP_MOVE,
    STEP_ORDER, STEP_RESET, STEP_SIZE, STEP_STYLE, apply_rect, changed, document, restore,
    search_rect, set_frame, visibility_key,
};
pub(in crate::app) use core::{
    animating, area_rect, key, picker_rect, record, snapshot, spacing_key, tick,
};

pub(in crate::app) fn update(
    app: &mut crate::app::App,
    message: crate::frontend::arrange::ArrangeMsg,
) -> iced::Task<crate::app::Message> {
    if inventory::defer(app, &message) { iced::Task::none() } else { core::dispatch(app, message) }
}
