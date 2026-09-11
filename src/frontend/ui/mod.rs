mod bar;
pub(crate) mod browser_bar;
mod browser_wall;
mod chips;
mod chrome;
mod color;
pub(crate) mod help;
mod misc;
mod pane;
mod slider;
mod theme_bar;

pub use crate::frontend::components::{parallelogram, with_alpha};

pub use bar::{
    BarAction, BarIntent, BarItem, BarShow, BarVisualStyle, FilterBar, MENU_MAX_ROWS, MENU_ROW_H,
    MenuKind, SORTS, TYPES, build_bar_with_tasks, sort_label_key, verticalize_bar,
};
pub(crate) use bar::{control_background, control_border, control_text};
pub use browser_bar::{BrowserAct, BrowserBar, browser_bar_compact_size};
pub use browser_wall::{BrowserWallInput, BrowserWallInputLayer};
pub use chips::{
    TagChips, chip_width, pl_chip, skwd_chip, tag_cloud_body_height, tag_cloud_row_count,
};
pub use chrome::{
    BackLayout, ChromeCanvas, PANEL_SKEW, SelectionMarks, back_bounds, back_contains, back_layout,
    chrome_signature,
};
pub use color::{
    color_bucket_name, cycle_color_left, cycle_color_right, parse_color_bucket, strip_bucket,
    swatch_color,
};
pub use help::{HelpIntent, help_overlay};
pub use misc::{
    FOLIO_INDEX_WIDTH, FOLIO_RULE_ALPHA, FadeFrame, NERD_FONT, Spinner, TYPE_SMALL, UI_FONT,
    bg_style, box_style, field_input, flat_button_style, folio_action, folio_action_bar,
    folio_action_width, folio_action_width_in, folio_action_wrap, folio_button_style,
    folio_destructive_action, folio_details, folio_field, folio_ghost_field, folio_horizontal_rule,
    folio_index_shell, folio_index_shell_tinted, folio_inline_bar, folio_line_button_style,
    folio_masthead, folio_rule, folio_scrim_style, folio_scroll_padding, folio_sheet,
    folio_sheet_dims, folio_sheet_panel_style, folio_stack_bar, ghost_input_style, hud,
    inert_backdrop, label, legible_type_scale, mid_text, panel_style, scrim_style, sentence_case,
    workbench_input_style, wrap_rows,
};
pub(crate) use misc::{ellipsize_text, folio_diagonal_edges, folio_diagonal_wipe};
pub use pane::{PANE_WHEEL_STEP, pane_id, scroll_style, smooth_pane, thin_hbar, thin_vbar};
pub use slider::folio_slider;
pub use theme_bar::{THEME_BACKENDS, ThemeBar, backend_menu_options};
