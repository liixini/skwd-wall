use std::collections::HashSet;

use crate::domain::library::catalog::{Catalog, Wallpaper};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagUpdate {
    pub key: String,
    pub tags: Vec<String>,
}

#[derive(Default)]
pub struct LibraryState {
    catalog: Catalog,
    dedup_keys: HashSet<String>,
    pub(super) tag_revision: u64,
}

impl LibraryState {
    pub const fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    pub const fn tag_revision(&self) -> u64 {
        self.tag_revision
    }

    pub fn replace(&mut self, catalog: Catalog) {
        self.dedup_keys =
            catalog.items.iter().map(|wallpaper| wallpaper.dedup_key().to_string()).collect();
        self.catalog = catalog;
        self.mark_tags_changed();
    }

    pub fn insert(&mut self, wallpaper: Wallpaper) -> bool {
        if self.dedup_keys.contains(wallpaper.dedup_key()) {
            return false;
        }
        self.dedup_keys.insert(wallpaper.dedup_key().to_string());
        self.catalog.items.push(wallpaper);
        true
    }

    pub fn update_thumbnail(
        &mut self,
        key: &str,
        thumb: Option<&str>,
        generated: Option<bool>,
    ) -> Option<usize> {
        let index = self.catalog.items.iter().position(|item| item.key == key)?;
        if let Some(thumb) = thumb.filter(|path| !path.is_empty()) {
            self.catalog.items[index].thumb = thumb.to_string();
        }
        if let Some(generated) = generated {
            self.catalog.items[index].thumbnail_generated = generated;
        }
        Some(index)
    }

    pub fn remove_by_key(&mut self, key: &str) -> bool {
        let Some(pos) = self.catalog.items.iter().rposition(|wallpaper| wallpaper.key == key)
        else {
            return false;
        };
        let dedup_key = self.catalog.items[pos].dedup_key().to_string();
        self.catalog.items.remove(pos);
        self.dedup_keys.remove(&dedup_key);
        true
    }

    pub fn remove_file(&mut self, name: &str) {
        if let Some(pos) = self.catalog.items.iter().rposition(|wallpaper| wallpaper.name == name) {
            self.catalog.items.remove(pos);
        }
        self.dedup_keys.remove(name);
    }

    pub fn rename_file(&mut self, old: &str, new: &str, new_path: &str, metadata: RenameMetadata) {
        for wallpaper in &mut self.catalog.items {
            if wallpaper.name == old {
                wallpaper.key = format!("static:{new}");
                wallpaper.name = new.to_string();
                wallpaper.path = new_path.to_string();
                wallpaper.mtime = metadata.mtime;
                wallpaper.filesize = metadata.filesize;
                wallpaper.width = metadata.width;
                wallpaper.height = metadata.height;
                break;
            }
        }
        self.dedup_keys.remove(old);
        self.dedup_keys.insert(new.to_string());
    }

    pub fn remove_folder(&mut self, names: &[String]) {
        let names_set: HashSet<&str> = names.iter().map(String::as_str).collect();
        self.catalog.items.retain(|wallpaper| !names_set.contains(wallpaper.name.as_str()));
        for name in names {
            self.dedup_keys.remove(name);
        }
    }

    pub fn record_applied(&mut self, key: &str, we_id: &str, name: &str) -> Option<usize> {
        let match_key = if !key.is_empty() {
            key.to_string()
        } else if !we_id.is_empty() {
            we_id.to_string()
        } else {
            file_stem(name).to_string()
        };
        if match_key.is_empty() {
            return None;
        }
        let last_applied = self
            .catalog
            .items
            .iter()
            .map(|wallpaper| wallpaper.last_applied)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        for (idx, wallpaper) in self.catalog.items.iter_mut().enumerate() {
            if wallpaper.key == match_key || wallpaper.applied_key() == match_key {
                wallpaper.apply_count += 1;
                wallpaper.last_applied = last_applied;
                return Some(idx);
            }
        }
        None
    }

    pub fn remove_tag_at(&mut self, key: &str, index: usize) -> Option<TagUpdate> {
        let mut tags = self.catalog.tags.get(key).cloned().unwrap_or_default();
        if index >= tags.len() {
            return None;
        }
        tags.remove(index);
        self.catalog.tags.insert(key.to_string(), tags.clone());
        self.mark_tags_changed();
        Some(TagUpdate { key: key.to_string(), tags })
    }

    #[allow(dead_code)]
    pub fn add_tag(&mut self, key: &str, tag: String) -> Option<TagUpdate> {
        let mut tags = self.catalog.tags.get(key).cloned().unwrap_or_default();
        if tags.iter().any(|existing| existing == &tag) {
            return None;
        }
        tags.push(tag);
        self.catalog.tags.insert(key.to_string(), tags.clone());
        self.mark_tags_changed();
        Some(TagUpdate { key: key.to_string(), tags })
    }

    pub fn replace_tags(&mut self, key: &str, tags: Vec<String>) -> Option<TagUpdate> {
        let current = self.catalog.tags.get(key).cloned().unwrap_or_default();
        if current == tags {
            return None;
        }
        self.catalog.tags.insert(key.to_string(), tags.clone());
        self.mark_tags_changed();
        Some(TagUpdate { key: key.to_string(), tags })
    }

    pub fn add_tags_to_indices(&mut self, indices: &[u32], additions: &[String]) -> Vec<TagUpdate> {
        let mut updates = Vec::new();
        for &index in indices {
            let Some(key) =
                self.catalog.items.get(index as usize).map(|wallpaper| wallpaper.key.clone())
            else {
                continue;
            };
            let mut tags = self.catalog.tags.get(&key).cloned().unwrap_or_default();
            let mut changed = false;
            for tag in additions {
                if !tags.iter().any(|existing| existing == tag) {
                    tags.push(tag.clone());
                    changed = true;
                }
            }
            if changed {
                self.catalog.tags.insert(key.clone(), tags.clone());
                updates.push(TagUpdate { key, tags });
            }
        }
        if !updates.is_empty() {
            self.mark_tags_changed();
        }
        updates
    }

    pub fn toggle_favourite(&mut self, key: &str) -> bool {
        if self.catalog.favourites.remove(key) {
            false
        } else {
            self.catalog.favourites.insert(key.to_string());
            true
        }
    }

    fn mark_tags_changed(&mut self) {
        self.tag_revision = self.tag_revision.wrapping_add(1);
    }
}

fn file_stem(name: &str) -> &str {
    match name.rfind('.') {
        Some(dot) if dot > 0 => &name[..dot],
        _ => name,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenameMetadata {
    pub mtime: i64,
    pub filesize: i64,
    pub width: i64,
    pub height: i64,
}
