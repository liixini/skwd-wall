mod arrange;
mod chrome;
mod composition;
pub(in crate::app) mod layout;
mod panels;
mod settings;
mod tags;
mod transient;

#[cfg(test)]
pub(crate) use arrange::ENTRY_HEIGHT;
#[cfg(test)]
pub(crate) use arrange::ENTRY_PAD;
pub(crate) use arrange::{
    FLOAT_CLEAR, arrange_bounds, clear_bottom, float_left, pinned, rest_bottom,
};
#[cfg(test)]
pub(super) use chrome::overview_set;
pub(crate) use chrome::{
    BarExtent, bar_bounds, bar_show, filter_bar_footprint, filter_bar_geometry, filter_bar_target,
};
pub(crate) use composition::view_count;
#[cfg(test)]
pub(crate) use composition::{entry_shown, scene_fade, tag_cloud_visible};
pub use composition::{view, view_single};
#[cfg(test)]
pub(crate) use tags::{cloud_fit_height, cloud_fit_width};
pub(crate) use tags::{tag_cloud_bounds, tag_cloud_geometry, tag_cloud_target};
