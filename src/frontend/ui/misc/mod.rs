mod dir;
mod fade_frame;
mod feedback;
mod focus_action;
mod folio;
mod layout;
mod measure;
mod motion_slot;
mod overlay;
mod style;
mod typography;

pub(crate) use dir::row;
pub use dir::{Flow, end, logical_padding, mirror_x, mirror_x_for, rtl, start, start_for};
#[cfg(test)]
pub use dir::{end_for, logical_padding_for, ordered};
pub use fade_frame::FadeFrame;
pub use feedback::Spinner;
pub use folio::{
    BUTTON_HEIGHT as FOLIO_BUTTON_HEIGHT, EYE_CLOSED, FOLIO_BUTTON_INSET, FOLIO_BUTTON_MIN_WIDTH,
    FOLIO_INDEX_WIDTH, TWISTY_CLOSED, TWISTY_OPEN, diamond, folio_action, folio_action_bar,
    folio_action_width, folio_action_width_in, folio_action_wrap, folio_destructive_action,
    folio_details, folio_field, folio_ghost_field, folio_icon_action, folio_index_shell,
    folio_index_shell_tinted, folio_inline_bar, folio_masthead, folio_scrim_style, folio_sheet,
    folio_sheet_panel_style, folio_stack_bar,
};
pub(crate) use folio::{
    FOLIO_ACCENT_ALPHA, FOLIO_FILL_ALPHA, FOLIO_OUTLINE_ALPHA, WIPE_EDGE_ALPHA, WIPE_FILL_ALPHA,
    folio_diagonal_edges, folio_wipe,
};
#[cfg(test)]
pub use layout::MIN_VIEWPORT;
pub use layout::{
    FOLIO_RULE_ALPHA, folio_horizontal_rule, folio_rule, folio_scroll_padding, folio_sheet_dims,
    hud, wrap_rows,
};
pub(crate) use measure::cached_line_width;
pub use motion_slot::motion_slot;
pub use overlay::{field_input, inert_backdrop};
pub use style::{
    bg_style, box_style, flat_button_style, folio_button_style, folio_line_button_style,
    folio_rail_style, ghost_input_style, panel_style, scrim, scrim_style, workbench_input_style,
};
pub use typography::{
    NERD_FONT, TYPE_SMALL, UI_FONT, UI_FONT_FAMILY, label, legible_type_scale, mid_text,
    sentence_case,
};
pub(crate) use typography::{
    anchor_rtl, ellipsize_text, glyph_width, has_rtl, shapes_across, text_width,
};

mod tests;
#[cfg(test)]
pub(crate) use typography::measured_line_width;
