mod action;
mod canvas;
mod catalog;
pub mod edit_id;
mod interaction;
mod menu;
mod model;
mod reorder;
mod view;

pub use action::{BarAction, BarIntent};
pub use canvas::FilterBar;
#[cfg(test)]
pub use catalog::MENU_MAX_ROWS;
pub(crate) use catalog::{ICON_FAV, ICON_HIDDEN};
pub use catalog::{MENU_CHROME_H, MENU_ROW_H, SORTS, TYPES, cycle_key, sort_label_key};
pub use menu::MenuKind;
pub(in crate::frontend::ui) use menu::item_contains;
pub(crate) use menu::{folder_leaf, folder_short_label};
pub use model::{BarItem, BarShow, BarVisualStyle, build_bar_with_tasks, verticalize_bar};
pub(crate) use reorder::{EditBar, ReorderPreview};
pub(crate) use view::{
    control_background, control_border, control_text, draw_button, draw_edit_bar,
};

#[cfg(test)]
use super::theme_bar::{THEME_BACKENDS, ThemeBar};
#[cfg(test)]
use canvas::BarState;
#[cfg(test)]
use catalog::{
    DROP_ARROW, DROP_ARROW_UP, next_orient, next_resolution, orient_label, resolution_label,
};
#[cfg(test)]
use menu::folder_depth;
pub use model::BarModel;
#[cfg(test)]
use model::mirror_model;
#[cfg(test)]
use view::split_icon_label;

mod tests;
