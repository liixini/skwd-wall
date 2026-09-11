mod canvas;
mod model;

pub use canvas::BrowserBar;
pub use model::{BrowserAct, browser_bar_compact_size};

pub(crate) use model::{wallhaven_range_label_key, wallhaven_sort_label_key};

pub(crate) use model::{
    steam_category_label_key, steam_resolution_label, steam_sort_label_key,
    steam_trend_days_label_key, steam_type_label_key,
};
