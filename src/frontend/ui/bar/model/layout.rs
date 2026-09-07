use super::super::super::theme_bar::ThemeBar;
use super::super::catalog::{MENU_MAX_ROWS, MENU_ROW_H};
use super::theme::theme_items;
use super::types::BarItem;

pub(super) fn wrap_items(
    items: &mut [BarItem],
    max_width: f32,
    bar_height: f32,
    gap: f32,
    y0: f32,
) -> (usize, f32, f32) {
    let count = items.len();
    let mut row = 0usize;
    let mut current_x = 0.0f32;
    let mut widest = 0.0f32;
    for index in 0..count {
        let width = items[index].w;
        let advance = if index + 1 < count { items[index + 1].x - items[index].x } else { width };
        if max_width > 0.0 && current_x > 0.0 && current_x + width > max_width {
            row += 1;
            current_x = 0.0;
        }
        items[index].x = current_x;
        items[index].y = y0 + row as f32 * (bar_height + gap);
        widest = widest.max(current_x + width);
        current_x += advance;
    }
    (row, widest, current_x)
}

pub(super) fn theme_rows(
    items: &mut Vec<BarItem>,
    theme: Option<&ThemeBar>,
    scale: f32,
    menu_up: bool,
    max_width: f32,
    bar_height: f32,
    gap: f32,
    rows_height: &mut f32,
    width: &mut f32,
) -> Option<(f32, f32)> {
    let theme = theme?;
    let mut theme_items = theme_items(theme, scale, menu_up);
    let (mut row, mut widest, current_x) =
        wrap_items(&mut theme_items, max_width, bar_height, gap, *rows_height + gap);
    let cells_width = 6.0 * 14.0 * scale + 10.0 * scale;
    let last_y = *rows_height + gap + row as f32 * (bar_height + gap);
    let swatch = if max_width > 0.0 && current_x + cells_width > max_width {
        row += 1;
        (0.0, last_y + bar_height + gap + (bar_height - 14.0 * scale) / 2.0)
    } else {
        widest = widest.max(current_x + cells_width);
        (current_x + 10.0 * scale, last_y + (bar_height - 14.0 * scale) / 2.0)
    };
    *rows_height += gap + (row as f32 + 1.0) * bar_height + row as f32 * gap;
    *width = width.max(widest.min(max_width.max(1.0))).max(1.0);
    items.extend(theme_items);
    Some(swatch)
}

pub(super) fn menu_rows(
    folder_menu_open: bool,
    has_folders: bool,
    folders: usize,
    theme: Option<&ThemeBar>,
) -> usize {
    if folder_menu_open && has_folders {
        return folders;
    }
    theme.filter(|bar| bar.menu_open).map_or(0, |bar| bar.backend_count)
}

pub(super) fn menu_space(
    items: &mut [BarItem],
    swatch_at: &mut Option<(f32, f32)>,
    length: usize,
    scale: f32,
    menu_up: bool,
) -> f32 {
    if length == 0 {
        return 0.0;
    }
    let visible = length.min(MENU_MAX_ROWS) as f32;
    let menu_height = 6.0 + visible * MENU_ROW_H * scale + 8.0;
    if menu_up {
        shift_rows(items, swatch_at, menu_height);
    }
    menu_height
}

fn shift_rows(items: &mut [BarItem], swatch_at: &mut Option<(f32, f32)>, delta_y: f32) {
    for item in items {
        item.y += delta_y;
    }
    if let Some((_, swatch_y)) = swatch_at {
        *swatch_y += delta_y;
    }
}
