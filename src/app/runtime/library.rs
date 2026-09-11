use log::{info, warn};
use serde_json::json;

use crate::domain::library::filter::{Filters, filter_sort};
use crate::frontend::scene::layout::Mode;
use crate::rendering::scene::atlas::AtlasMap;

#[allow(clippy::wildcard_imports)]
use super::super::*;

impl App {
    pub(in crate::app) fn demo_showcase_index(&self, count: usize) -> usize {
        if self.runtime_state.demo.is_none() || count == 0 {
            return 0;
        }
        if self.scene.mode == Mode::Hex {
            let rows = self.config.hex_rows();
            let cols = self.config.hex_cols();
            return ((cols / 2) * rows + rows / 2).min(count / 2);
        }
        (count / 2).min(6)
    }

    pub(in crate::app) fn on_list(&mut self, catalog: crate::domain::library::catalog::Catalog) {
        self.library_session.library.replace(catalog);
        if (self.tags.editing || self.tags.card_drawer_open)
            && let Some(key) = self.tags.card_key.as_deref()
            && self.library_session.library.catalog().items.iter().any(|item| item.key == key)
        {
            self.library_session.library.replace_tags(key, self.tags.card_locked.clone());
        }
        self.theme.cache.clear();
        self.theme.preview_target = None;
        crate::infrastructure::observability::log_startup_checkpoint("list_received");
        self.rebuild_folder_options();
        self.refilter();
        let count = self.library_session.library.catalog().items.len();
        self.preview_resources.atlas = Some(AtlasMap::new(count.max(1)));
        if count == 0 {
            warn!(
                "list ready but EMPTY - scanner index not built yet; waiting for scan_done to refresh"
            );
        } else {
            info!("list ready: {} items, {} filtered", count, self.library_session.filtered.len());
        }
        crate::shell::trim_heap();
    }

    fn filtered_indices(&self) -> Vec<u32> {
        let catalog = self.library_session.library.catalog();
        let mut filtered = filter_sort(catalog, &self.library_session.filters);
        if let Some((_, _, keys)) = &self.library_session.playlist_filter {
            filtered.retain(|&index| keys.contains(&catalog.items[index as usize].key));
        }
        if self.tags.search_mode == SearchMode::Describe
            && !self.tags.semantic.search.trim().is_empty()
            && self.tags.semantic.resolved
            && !self.tags.semantic.exclusions.is_empty()
        {
            filtered.retain(|&index| {
                catalog.tags.get(&catalog.items[index as usize].key).is_none_or(|item_tags| {
                    !self
                        .tags
                        .semantic
                        .exclusions
                        .iter()
                        .any(|excluded| item_tags.iter().any(|tag| tag == excluded))
                })
            });
        }
        if self.tags.search_mode == SearchMode::Describe
            && !self.tags.semantic.search.trim().is_empty()
            && self.tags.semantic.resolved
        {
            let ranks: std::collections::HashMap<&str, usize> = self
                .tags
                .semantic
                .ranked
                .iter()
                .enumerate()
                .map(|(rank, key)| (key.as_str(), rank))
                .collect();
            filtered
                .retain(|index| ranks.contains_key(catalog.items[*index as usize].key.as_str()));
            if !self.tags.semantic.sort_override {
                filtered.sort_by_key(|index| {
                    ranks
                        .get(catalog.items[*index as usize].key.as_str())
                        .copied()
                        .unwrap_or(usize::MAX)
                });
            }
        }
        filtered
    }

    pub(in crate::app) fn refilter(&mut self) {
        self.library_session.filtered = self.filtered_indices();
        self.library_session.visible_count = self.library_session.filtered.len();
        self.scene.relayout(self.library_session.filtered.len());
    }

    pub(in crate::app) fn refilter_from_start(&mut self) {
        if self.scene.filter_swap_active() {
            self.library_session.filter_transition_pending = true;
            self.library_session.visible_count = self.filtered_indices().len();
            return;
        }
        let sandy_from = (self.scene.mode == Mode::Sandy)
            .then(|| {
                self.library_session
                    .filtered
                    .get(self.scene.sandy_displayed())
                    .map(|&store_index| store_index as usize)
            })
            .flatten();
        let list_before = std::mem::take(&mut self.library_session.filtered);
        self.library_session.filtered = self.filtered_indices();
        self.library_session.visible_count = self.library_session.filtered.len();
        let count = self.library_session.filtered.len();
        self.scene.reset_to_index(self.demo_showcase_index(count), count);
        if self.library_session.filtered != list_before && !self.library_session.filtered.is_empty()
        {
            self.scene.filter_storm(sandy_from);
        }
    }

    pub(in crate::app) fn refilter_semantic_from_start(&mut self) {
        let list_before = std::mem::take(&mut self.library_session.filtered);
        self.library_session.filtered = self.filtered_indices();
        self.library_session.visible_count = self.library_session.filtered.len();
        self.library_session.filter_transition_pending = false;
        if self.scene.mode == Mode::Sandy {
            self.scene.sandy_settle_now();
        }
        let count = self.library_session.filtered.len();
        self.scene.reset_to_index(self.demo_showcase_index(count), count);
        if self.library_session.filtered != list_before && !self.library_session.filtered.is_empty()
        {
            self.scene.filter_storm(None);
        }
    }

    pub(in crate::app) fn flush_pending_filter(&mut self) {
        if self.library_session.filter_transition_pending && !self.scene.filter_swap_active() {
            self.library_session.filter_transition_pending = false;
            self.refilter_from_start();
            self.retick();
        }
    }

    pub(in crate::app) fn close_flip_after_removal(&mut self) {
        if self.scene.flip_open() {
            self.tags.editing = false;
            self.tags.card_drawer_open = false;
            self.scene.set_tag_editing(false);
            self.scene.close_flip();
        }
    }

    pub(in crate::app) fn refilter_after_removal(&mut self) {
        self.close_flip_after_removal();
        self.refilter();
        if !self.library_session.filtered.is_empty() {
            self.scene.filter_storm(None);
        }
    }

    pub(in crate::app) fn change_filters(&mut self, edit: impl FnOnce(&mut Filters)) {
        let before = self.library_session.filters.clone();
        edit(&mut self.library_session.filters);
        if self.library_session.filters != before && self.config.filter_bar_sticky() {
            self.config.save_key(
                "filterBar.last",
                json!({
                    "color": self.library_session.filters.color,
                    "kind": self.library_session.filters.kind,
                    "folder": self.library_session.filters.folder,
                    "showHiddenFolders": self.library_session.filters.show_hidden_folders,
                    "sort": self.library_session.filters.sort,
                    "orient": self.library_session.filters.orient,
                    "resolution": self.library_session.filters.resolution,
                    "favouritesOnly": self.library_session.filters.favourites_only,
                }),
            );
        }
        self.refilter_from_start();
        self.chrome.bar.cache.clear();
        self.retick();
    }
}
