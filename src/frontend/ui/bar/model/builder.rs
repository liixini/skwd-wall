use crate::domain::library::filter::Filters;
use crate::i18n::tr;

use super::super::super::misc::text_width;
use super::super::super::theme_bar::ThemeBar;
use super::super::action::BarAction;
use super::super::catalog::{
    ICON_FAV, ICON_FOLDER, ICON_RANDOM, SORTS, TYPES, next_orient, next_resolution, orient_label,
    resolution_label, type_label_key,
};
use super::layout::{menu_rows, menu_space, theme_rows, wrap_items};
use super::types::{BarItem, BarModel, BarNotice, BarShow, dropdown_item};

#[allow(clippy::fn_params_excessive_bools)]
pub fn build_bar_with_tasks(
    filters: &Filters,
    folder_options: &[String],
    folder_menu_open: bool,
    _filtered: usize,
    _total: usize,
    scale: f32,
    downloads_enabled: bool,
    audio_visible: bool,
    audio_muted: bool,
    show: &BarShow,
    max_width: f32,
    menu_up: bool,
    random_on: bool,
    theme: Option<&ThemeBar>,
    tasks: &[&crate::contracts::daemon::TaskStatus],
) -> BarModel {
    let bar_h = 24.0 * scale;
    let mut items = Vec::new();
    let mut x = 0.0f32;

    type_items(&mut items, &mut x, scale, &filters.kind, &show.types);

    if show.orient {
        let action = Some(BarAction::Orient(next_orient(&filters.orient).to_string()));
        push_item(
            &mut items,
            &mut x,
            scale,
            orient_label(&filters.orient),
            false,
            !filters.orient.is_empty(),
            action,
        );
    }

    let has_resolution =
        show.resolution_presets.iter().any(|preset| preset.matches_shape(&filters.orient));
    if show.resolution && (has_resolution || !filters.resolution.is_empty()) {
        let next = next_resolution(&filters.resolution, &show.resolution_presets, &filters.orient);
        push_item(
            &mut items,
            &mut x,
            scale,
            &resolution_label(&filters.resolution, &show.resolution_presets),
            false,
            !filters.resolution.is_empty(),
            Some(BarAction::Resolution(next)),
        );
    }

    let has_folders =
        show.folder && folder_options.iter().any(|folder| !matches!(folder.as_str(), "" | "*"));
    if has_folders {
        folder_item(&mut items, &mut x, scale, &filters.folder, menu_up, folder_menu_open);
    }

    sort_items(&mut items, &mut x, scale, &filters.sort, &show.sorts);

    if show.favourites {
        let action = Some(BarAction::Favs);
        push_item(&mut items, &mut x, scale, ICON_FAV, true, filters.favourites_only, action);
    }

    if show.random {
        let action = Some(BarAction::Random);
        push_item(&mut items, &mut x, scale, ICON_RANDOM, true, random_on, action);
    }

    if show.colors {
        color_items(&mut items, &mut x, scale, filters.color);
    }

    if show.theme {
        push_item(
            &mut items,
            &mut x,
            scale,
            "\u{f03d8}",
            true,
            theme.is_some(),
            Some(BarAction::ThemePanel),
        );
    }

    if show.tagcloud {
        let active = !filters.tags.is_empty() || !filters.numeric.is_empty();
        let action = Some(BarAction::TagCloud);
        push_item(&mut items, &mut x, scale, "\u{f0349}", true, active, action);
    }

    if downloads_enabled {
        push_item(&mut items, &mut x, scale, "\u{f01da}", true, false, Some(BarAction::Download));
    }

    if audio_visible {
        audio_items(&mut items, &mut x, scale, audio_muted);
    }

    push_item(&mut items, &mut x, scale, "\u{f0cb8}", true, false, Some(BarAction::Playlists));

    push_item(&mut items, &mut x, scale, "\u{f0493}", true, false, Some(BarAction::Settings));

    task_items(&mut items, &mut x, scale, tasks);

    let gap = 6.0 * scale;
    let content_w = items.iter().map(|item| item.x + item.w).fold(0.0, f32::max);
    let mut width;
    let mut rows_h = bar_h;
    if max_width > 0.0 && content_w > max_width && items.len() > 1 {
        let (row, widest, _) = wrap_items(&mut items, max_width, bar_h, gap, 0.0);
        rows_h = (row as f32 + 1.0) * bar_h + row as f32 * gap;
        width = widest.min(max_width).max(1.0);
    } else {
        width = content_w.max(1.0);
    }

    let mut swatch_at = theme_rows(
        &mut items,
        theme,
        scale,
        menu_up,
        max_width,
        bar_h,
        gap,
        &mut rows_h,
        &mut width,
    );

    let menu_len = menu_rows(folder_menu_open, has_folders, folder_options.len(), theme);
    let height = rows_h + menu_space(&mut items, &mut swatch_at, menu_len, scale, menu_up);
    BarModel { items, width, height: height.max(1.0), menu_up, swatch_at }
}

pub fn verticalize_bar(model: &mut BarModel, max_height: f32, scale: f32) {
    let gap = 3.0 * scale;
    let item_h = 24.0 * scale;
    let rail_w =
        model.items.iter().map(|item| (item.w - item.skew).max(1.0)).fold(84.0 * scale, f32::max);
    let max_height = max_height.max(item_h);
    let mut x = 0.0;
    let mut y = 0.0;
    let mut columns = 1usize;
    for item in &mut model.items {
        if y > 0.0 && y + item_h > max_height {
            columns += 1;
            x += rail_w + gap;
            y = 0.0;
        }
        item.x = x;
        item.y = y;
        item.w = rail_w + item.skew;
        item.h = item_h;
        y += item_h + gap;
    }
    if let Some((swatch_x, swatch_y)) = model.swatch_at.as_mut() {
        if y > 0.0 && y + item_h > max_height {
            columns += 1;
            x += rail_w + gap;
            y = 0.0;
        }
        *swatch_x = x + (rail_w - 84.0 * scale) * 0.5;
        *swatch_y = y + (item_h - 14.0 * scale) * 0.5;
        y += item_h;
    }
    model.width = columns as f32 * rail_w + columns.saturating_sub(1) as f32 * gap;
    model.height = if columns == 1 { y.max(item_h) } else { max_height };
    model.menu_up = false;
}

fn task_items(
    items: &mut Vec<BarItem>,
    x: &mut f32,
    scale: f32,
    tasks: &[&crate::contracts::daemon::TaskStatus],
) {
    for task in tasks {
        let name = match task.id.as_str() {
            "semantic-tags" | "analysis" => tr("filter-bar-task-tags"),
            "semantic-index" => tr("filter-bar-task-index"),
            "scan" => tr("filter-bar-task-scan"),
            id if id.starts_with("download:") => tr("filter-bar-task-download"),
            _ => task.label.as_str(),
        };
        let progress_label =
            if task.total > 0 && task.state == crate::contracts::daemon::TaskState::Completed {
                format!(" · {}", task.total)
            } else if task.total > 0 {
                format!(" · {}/{}", task.progress.min(task.total), task.total)
            } else {
                String::new()
            };
        let state = match &task.state {
            crate::contracts::daemon::TaskState::Running => tr("filter-bar-state-running"),
            crate::contracts::daemon::TaskState::Paused => tr("filter-bar-state-paused"),
            crate::contracts::daemon::TaskState::Completed => tr("filter-bar-state-done"),
            crate::contracts::daemon::TaskState::Failed => tr("filter-bar-state-failed"),
            crate::contracts::daemon::TaskState::Cancelled => tr("filter-bar-state-stopped"),
            crate::contracts::daemon::TaskState::Other(state) => state.as_str(),
        };
        let label = format!("{name} · {state}{progress_label}");
        let text_size = 9.5 * scale;
        let skew = 10.0 * scale;
        let width = text_width(&label, text_size, false) + 34.0 * scale + skew;
        *x += 4.0 * scale;
        items.push(BarItem {
            x: *x,
            y: 0.0,
            w: width,
            h: 24.0 * scale,
            skew,
            label,
            nerd: false,
            text_size,
            swatch: None,
            notice: Some(BarNotice {
                state: task.state.clone(),
                progress: (task.total > 0 && task.state.is_active())
                    .then(|| task.progress.min(task.total) as f32 / task.total as f32),
            }),
            active: false,
            action: None,
            z: 20,
        });
        *x += width - skew;

        let mut control = |label, action| {
            push_item(
                items,
                x,
                scale,
                label,
                false,
                false,
                Some(BarAction::TaskControl { id: task.id.clone(), action }),
            );
        };
        if task.capabilities.pause && task.state == crate::contracts::daemon::TaskState::Running {
            control(tr("filter-bar-pause"), crate::contracts::daemon::TaskControl::Pause);
        }
        if task.capabilities.resume && task.state == crate::contracts::daemon::TaskState::Paused {
            control(tr("filter-bar-resume"), crate::contracts::daemon::TaskControl::Resume);
        }
        if task.capabilities.stop {
            control(tr("filter-bar-stop"), crate::contracts::daemon::TaskControl::Stop);
        }
    }
}

fn push_item(
    items: &mut Vec<BarItem>,
    x: &mut f32,
    scale: f32,
    label: &str,
    nerd: bool,
    active: bool,
    action: Option<BarAction>,
) {
    let size = if nerd { 14.0 * scale } else { 10.0 * scale };
    let skew = 10.0 * scale;
    let w = text_width(label, size, nerd) + 24.0 * scale + skew;
    items.push(BarItem {
        x: *x,
        y: 0.0,
        w,
        h: 24.0 * scale,
        skew,
        label: label.to_string(),
        nerd,
        text_size: size,
        swatch: None,
        notice: None,
        active,
        action,
        z: if active { 10 } else { 1 },
    });
    *x += w - skew;
}

fn type_items(
    items: &mut Vec<BarItem>,
    x: &mut f32,
    scale: f32,
    kind: &str,
    show: &[&'static str],
) {
    for key in TYPES {
        if !show.contains(&key) {
            continue;
        }
        let active = kind == key;
        let next = if active { String::new() } else { key.to_string() };
        let action = Some(BarAction::SetType(next));
        push_item(items, x, scale, tr(type_label_key(key)), false, active, action);
    }
}

fn sort_items(
    items: &mut Vec<BarItem>,
    x: &mut f32,
    scale: f32,
    sort: &str,
    show: &[&'static str],
) {
    for (mode, icon) in SORTS {
        if !show.contains(&mode) {
            continue;
        }
        let action = Some(BarAction::Sort(mode.to_string()));
        push_item(items, x, scale, icon, true, sort == mode, action);
    }
}

fn folder_item(
    items: &mut Vec<BarItem>,
    x: &mut f32,
    scale: f32,
    folder: &str,
    menu_up: bool,
    open: bool,
) {
    let current = match folder {
        "*" => tr("filter-bar-folder-all"),
        "" => tr("filter-bar-folder-main"),
        name => name,
    }
    .to_uppercase();
    dropdown_item(
        items,
        x,
        scale,
        ICON_FOLDER,
        &current,
        menu_up,
        !folder.is_empty() || open,
        BarAction::FolderToggle,
    );
}

fn color_items(items: &mut Vec<BarItem>, x: &mut f32, scale: f32, color: i64) {
    *x += 6.0 * scale;
    let w = 28.0 * scale;
    let skew = 10.0 * scale;
    for idx in 0..13usize {
        let value = if idx == 12 { 99 } else { idx as i64 };
        let selected = color == value;
        items.push(BarItem {
            x: *x,
            y: 0.0,
            w,
            h: 24.0 * scale,
            skew,
            label: String::new(),
            nerd: false,
            text_size: 0.0,
            swatch: Some(idx),
            notice: None,
            active: selected,
            action: Some(BarAction::Color(if selected { -1 } else { value })),
            z: if selected { 10 } else { 1 },
        });
        *x += w - skew;
    }
    *x += 6.0 * scale;
}

fn audio_items(items: &mut Vec<BarItem>, x: &mut f32, scale: f32, muted: bool) {
    let icon = if muted { "\u{f075f}" } else { "\u{f057e}" };
    push_item(items, x, scale, icon, true, !muted, Some(BarAction::Audio));
    push_item(items, x, scale, "\u{f062e}", true, false, Some(BarAction::AudioPanel));
}
