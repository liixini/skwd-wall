#[allow(clippy::module_inception)]
mod filter;
mod tests;

pub use filter::{
    ALL_WALLPAPERS, Filters, ResolutionPreset, filter_sort, insert_index, parse_resolution,
    resolution_bounds_valid,
};
