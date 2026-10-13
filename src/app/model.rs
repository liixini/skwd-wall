use std::collections::HashSet;

use skwd_config::LibraryItem;

use crate::app::SearchMode;
use crate::domain::library::catalog::{Wallpaper, WallpaperKind};
use crate::domain::library::filter::{ALL_WALLPAPERS, Filters};
use crate::frontend::theme::Palette;
use crate::infrastructure::config::Config;

use super::scene::SceneCore;
use super::{
    AppRuntimeState, ChromeState, DaemonState, InputState, LibrarySession, PanelsState,
    PreviewResources, SourceBrowserState, TagState, ThemeState,
};

pub struct App {
    pub(in crate::app) runtime_state: AppRuntimeState,
    pub(in crate::app) daemon: DaemonState,
    pub(in crate::app) preview_resources: PreviewResources,
    pub(in crate::app) config: Config,
    pub(in crate::app) theme: ThemeState,
    pub(in crate::app) library_session: LibrarySession,
    pub(in crate::app) scene: SceneCore,
    pub(in crate::app) chrome: ChromeState,
    pub(in crate::app) panels: PanelsState,
    pub(in crate::app) source_browser: SourceBrowserState,
    pub(in crate::app) input: InputState,
    pub(in crate::app) tags: TagState,
}

impl App {
    pub fn palette(&self) -> &Palette {
        &self.theme.palette
    }

    pub(in crate::app) fn tag_cloud_entries(
        &self,
    ) -> std::rc::Rc<Vec<crate::domain::library::search::TagEntry>> {
        use std::hash::{Hash, Hasher};

        let filter_query = if self.tags.search_mode == SearchMode::Tags {
            crate::app::tag_search_partial(&self.tags.tag_search)
        } else {
            ""
        };
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.library_session.library.tag_revision().hash(&mut hasher);
        self.library_session.filters.tags.hash(&mut hasher);
        filter_query.hash(&mut hasher);
        let key = hasher.finish();
        if let Some((cached_key, cached)) = self.tags.cloud_cache.borrow().as_ref()
            && *cached_key == key
        {
            return cached.clone();
        }
        let (entries, _) = crate::domain::library::search::recompute_tag_cloud(
            &self.library_session.library.catalog().tags,
            &self.library_session.filters.tags,
            filter_query,
            crate::frontend::tagcloud::MAX_VISIBLE,
        );
        let entries = std::rc::Rc::new(entries);
        *self.tags.cloud_cache.borrow_mut() = Some((key, entries.clone()));
        entries
    }

    pub(in crate::app) fn apply_library_profile(&mut self) {
        let root = self.config.root();
        let profile = match &self.library_session.profile_override {
            Some(id) => skwd_config::library_profile(root, id),
            None => skwd_config::active_library_profile(root),
        };
        let mut hidden = HashSet::new();
        let mut excluded = HashSet::new();
        for item in &self.library_session.library.catalog().items {
            match skwd_config::profile_state(profile.as_ref(), library_item(item)) {
                skwd_config::FolderState::Hide => {
                    hidden.insert(item.key.clone());
                }
                skwd_config::FolderState::Exclude => {
                    excluded.insert(item.key.clone());
                }
                skwd_config::FolderState::Show => {}
            }
        }
        self.library_session.library.set_profile_sets(hidden, excluded);
    }

    pub(in crate::app) fn rebuild_folder_options(&mut self) {
        self.library_session.folder_roots = skwd_config::library_folders(self.config.root());
        let catalog = self.library_session.library.catalog();
        let roots = &self.library_session.folder_roots;
        self.library_session.all_folders = catalog.available_folders(roots, None);
        let shown = catalog.available_folders(roots, Some(false));
        let mut options = vec![String::from("*")];
        options.extend(catalog.available_folders(roots, Some(self.library_session.filters.reveal)));
        self.library_session.folder_hidden = options
            .iter()
            .map(|folder| folder != ALL_WALLPAPERS && !shown.contains(folder))
            .collect();
        let selected = &self.library_session.filters.folder;
        let outside_roots = std::path::Path::new(selected).is_absolute()
            && !self
                .library_session
                .folder_roots
                .iter()
                .any(|root| std::path::Path::new(selected).starts_with(root));
        if !options.contains(selected)
            && (!self.library_session.library.catalog().items.is_empty() || outside_roots)
        {
            self.library_session.filters.folder = Filters::default().folder;
        }
        self.library_session.folder_options = options;
    }
}

pub(in crate::app) fn library_item(item: &Wallpaper) -> LibraryItem<'_> {
    match (LibraryItem::new(&item.key, &item.path), item.kind) {
        (LibraryItem::Other, WallpaperKind::We) => LibraryItem::Workshop,
        (LibraryItem::Other, _) if !item.path.is_empty() => {
            LibraryItem::Folder(std::path::Path::new(&item.path))
        }
        (classified, _) => classified,
    }
}
