use crate::app::SearchMode;
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

    pub(in crate::app) fn rebuild_folder_options(&mut self) {
        let mut options = vec![String::new(), String::from("*")];
        options.extend(self.library_session.library.catalog().available_folders());
        options.retain(|folder| self.library_session.filters.folder_visible(folder));
        self.library_session.folder_options = options;
    }
}
