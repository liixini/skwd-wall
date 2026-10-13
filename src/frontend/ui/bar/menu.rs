use iced::{Point, Rectangle};

use super::super::misc::text_width;
use super::action::BarAction;
use super::canvas::FilterBar;
use super::catalog::{MENU_MAX_ROWS, MENU_ROW_H};
use super::model::{BarItem, BarModel, BarVisualStyle};
use super::view::item_visual_bounds;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuKind {
    Folders,
    Backends,
}

pub(super) const FOLDER_DEPTH_STEP: f32 = 16.0;
const FOLDER_LABEL_SIZE: f32 = 10.0;
const FOLDER_LABEL_PADDING: f32 = 65.0;

pub(super) fn folder_depth(folder: &str, roots: &[String]) -> usize {
    let Some(root) = roots
        .iter()
        .filter(|root| std::path::Path::new(folder).starts_with(root))
        .max_by_key(|root| root.len())
    else {
        return folder.matches('/').count();
    };
    folder.strip_prefix(root).unwrap_or("").trim_start_matches('/').matches('/').count()
        + usize::from(folder != root)
}

pub(crate) fn folder_leaf(folder: &str) -> &str {
    folder.rsplit('/').next().unwrap_or(folder)
}

pub(super) fn folder_label(folder: &str, roots: &[String]) -> String {
    if roots.iter().any(|root| root == folder) {
        return folder_short_label(folder, roots);
    }
    folder_leaf(folder).to_string()
}

pub(crate) fn folder_short_label(folder: &str, options: &[String]) -> String {
    let mut parts = folder.split('/').filter(|part| !part.is_empty()).rev();
    let mut label = parts.next().unwrap_or(folder).to_string();
    for part in parts {
        if options.iter().filter(|option| std::path::Path::new(option).ends_with(&label)).count()
            <= 1
        {
            break;
        }
        label = format!("{part}/{label}");
    }
    label
}

pub(in crate::frontend::ui) fn item_contains(item: &BarItem, x: f32, y: f32) -> bool {
    if y < item.y || y > item.y + item.h {
        return false;
    }
    let fraction = (y - item.y) / item.h;
    let lean = if item.skew >= 0.0 { 1.0 - fraction } else { fraction };
    let left = item.x + item.skew.abs() * lean;
    let right = item.x + item.w - item.skew.abs() * (1.0 - lean);
    x >= left && x <= right
}

impl FilterBar<'_> {
    fn folder_anchor(&self) -> Option<&BarItem> {
        self.model.items.iter().find(|item| matches!(item.action, Some(BarAction::FolderToggle)))
    }

    pub(super) fn active_menu(&self) -> Option<MenuKind> {
        if self.menu_open {
            Some(MenuKind::Folders)
        } else if self.backend_menu_open {
            Some(MenuKind::Backends)
        } else {
            None
        }
    }

    pub(super) fn menu_len(&self) -> usize {
        match self.active_menu() {
            Some(MenuKind::Folders) => self.folder_options.len(),
            Some(MenuKind::Backends) => self.backend_options.len(),
            None => 0,
        }
    }

    fn menu_anchor(&self) -> Option<&BarItem> {
        match self.active_menu()? {
            MenuKind::Folders => self.folder_anchor(),
            MenuKind::Backends => self
                .model
                .items
                .iter()
                .find(|item| matches!(item.action, Some(BarAction::ThemeBackendToggle))),
        }
    }

    pub(super) fn menu_row_h(&self) -> f32 {
        MENU_ROW_H * self.scale
    }

    pub(super) fn menu_max_scroll(&self) -> f32 {
        let count = self.menu_len();
        let visible = count.min(MENU_MAX_ROWS);
        (count.saturating_sub(visible)) as f32 * self.menu_row_h()
    }

    pub(super) fn menu_rect(&self) -> Option<Rectangle> {
        let kind = self.active_menu()?;
        let anchor = self.menu_anchor()?;
        let row_height = self.menu_row_h();
        let visible = self.menu_len().min(MENU_MAX_ROWS) as f32;
        let height = 6.0 + visible * row_height + 8.0;
        let width = match kind {
            MenuKind::Folders => self
                .folder_options
                .iter()
                .map(|folder| {
                    let indent = folder_depth(folder, self.folder_roots).min(6) as f32;
                    let label = if folder == crate::domain::library::filter::ALL_WALLPAPERS {
                        crate::i18n::tr("filter-bar-folder-all").to_string()
                    } else {
                        folder_label(folder, self.folder_roots)
                    };
                    text_width(&label, FOLDER_LABEL_SIZE * self.scale, false)
                        + (FOLDER_LABEL_PADDING + indent * FOLDER_DEPTH_STEP) * self.scale
                })
                .fold(170.0 * self.scale, f32::max),
            MenuKind::Backends => 150.0 * self.scale,
        }
        .min(self.model.width.max(120.0));
        let x = if self.model.rtl { anchor.x + anchor.w - width } else { anchor.x };
        Some(Rectangle {
            x: x.min(self.model.width - width).max(0.0),
            y: if self.model.menu_up { 0.0 } else { self.model.height - height },
            width,
            height,
        })
    }

    pub(super) fn menu_index_at(&self, x: f32, y: f32) -> Option<usize> {
        let rectangle = self.menu_rect()?;
        if x < rectangle.x
            || x > rectangle.x + rectangle.width
            || y < rectangle.y + 4.0
            || y > rectangle.y + rectangle.height
        {
            return None;
        }
        let index = ((y - rectangle.y - 4.0 + self.menu_scroll) / self.menu_row_h()) as usize;
        (index < self.menu_len()).then_some(index)
    }

    pub(super) fn hit(&self, x: f32, y: f32) -> Option<usize> {
        hit_item(&self.model, self.visual_style, x, y)
    }

    pub(super) fn over_menu(&self, position: Point) -> bool {
        let Some(rectangle) = self.menu_rect() else {
            return false;
        };
        position.x >= rectangle.x
            && position.x <= rectangle.x + rectangle.width
            && position.y >= rectangle.y
            && position.y <= rectangle.y + rectangle.height
    }
}

fn hit_item(model: &BarModel, style: BarVisualStyle, x: f32, y: f32) -> Option<usize> {
    let mut best: Option<(i32, usize)> = None;
    for (index, item) in model.items.iter().enumerate() {
        if item.action.is_some()
            && visual_item_contains(item, style, x, y)
            && best.is_none_or(|(max_depth, _)| item.z >= max_depth)
        {
            best = Some((item.z, index));
        }
    }
    best.map(|(_, index)| index)
}

fn visual_item_contains(item: &BarItem, style: BarVisualStyle, x: f32, y: f32) -> bool {
    if style == BarVisualStyle::Slices {
        item_contains(item, x, y)
    } else {
        let (visual_x, visual_w) = item_visual_bounds(item, style);
        y >= item.y && y <= item.y + item.h && x >= visual_x && x <= visual_x + visual_w
    }
}
