mod chrome;
mod composition;
mod panels;
mod settings;
mod tags;
mod transient;

pub(crate) use chrome::filter_bar_footprint;
#[cfg(test)]
pub(super) use chrome::overview_set;
#[cfg(test)]
pub(crate) use composition::tag_cloud_visible;
pub use composition::{view, view_single};
#[cfg(test)]
pub(crate) use tags::{cloud_fit_height, cloud_fit_width};
