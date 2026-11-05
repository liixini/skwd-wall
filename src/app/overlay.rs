use super::App;
use super::helpers::commit_pending_tag;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Overlay {
    Help,
    CardPicker,
    AudioPanel,
    Playlists,
    TagMode,
    TagEditing,
    Detail,
    BrowserPreview,
    Browser,
    ThemeDesigner,
    ColourStudio,
    ScheduleEditor,
    SceneProperties,
    Effects,
    BackendsMenu,
    ThemeAudition,
    ThemeBar,
    TagCloud,
    FoldersMenu,
    Settings,
}

impl Overlay {
    pub(crate) const ALL: [Overlay; 20] = [
        Overlay::Help,
        Overlay::CardPicker,
        Overlay::AudioPanel,
        Overlay::Playlists,
        Overlay::TagMode,
        Overlay::TagEditing,
        Overlay::Detail,
        Overlay::BrowserPreview,
        Overlay::Browser,
        Overlay::ThemeDesigner,
        Overlay::ColourStudio,
        Overlay::ScheduleEditor,
        Overlay::SceneProperties,
        Overlay::Effects,
        Overlay::BackendsMenu,
        Overlay::ThemeAudition,
        Overlay::ThemeBar,
        Overlay::TagCloud,
        Overlay::FoldersMenu,
        Overlay::Settings,
    ];

    pub(crate) const ESC_ORDER: [Overlay; 20] = [
        Overlay::Help,
        Overlay::ColourStudio,
        Overlay::CardPicker,
        Overlay::AudioPanel,
        Overlay::Playlists,
        Overlay::TagMode,
        Overlay::TagEditing,
        Overlay::Detail,
        Overlay::BrowserPreview,
        Overlay::Browser,
        Overlay::ThemeDesigner,
        Overlay::ScheduleEditor,
        Overlay::SceneProperties,
        Overlay::Effects,
        Overlay::BackendsMenu,
        Overlay::ThemeAudition,
        Overlay::ThemeBar,
        Overlay::TagCloud,
        Overlay::FoldersMenu,
        Overlay::Settings,
    ];

    pub(crate) fn policy(self) -> OverlayPolicy {
        let (captures, obscures) = match self {
            Overlay::Help
            | Overlay::CardPicker
            | Overlay::AudioPanel
            | Overlay::Playlists
            | Overlay::Browser
            | Overlay::ThemeDesigner
            | Overlay::ColourStudio
            | Overlay::ScheduleEditor
            | Overlay::SceneProperties
            | Overlay::Effects
            | Overlay::ThemeAudition => (true, true),
            Overlay::TagEditing
            | Overlay::BackendsMenu
            | Overlay::FoldersMenu
            | Overlay::Settings => (true, false),
            Overlay::Detail
            | Overlay::BrowserPreview
            | Overlay::TagMode
            | Overlay::ThemeBar
            | Overlay::TagCloud => (false, false),
        };
        OverlayPolicy { captures, obscures }
    }
}

pub(crate) struct OverlayPolicy {
    pub(crate) captures: bool,
    pub(crate) obscures: bool,
}

impl App {
    pub(super) fn overlay_open(&self, ov: Overlay) -> bool {
        match ov {
            Overlay::Help => self.input.help_open,
            Overlay::CardPicker => self.panels.card_picker.is_some(),
            Overlay::AudioPanel => self.panels.audio.is_some(),
            Overlay::Playlists => self.panels.playlists.is_some(),
            Overlay::TagMode => self.tags.mode,
            Overlay::TagEditing => self.tags.editing || self.tags.card_drawer_open,
            Overlay::Detail => {
                self.detail_open() || self.scene.flip_open() || self.scene.hand_reveal_open()
            }
            Overlay::BrowserPreview => {
                self.source_browser.browser.as_ref().is_some_and(|br| br.session.preview.is_some())
            }
            Overlay::Browser => self.source_browser.browser.is_some(),
            Overlay::ThemeDesigner => self.panels.theme_designer.is_some(),
            Overlay::ColourStudio => self.panels.colour_studio.is_some(),
            Overlay::ScheduleEditor => self.panels.schedule.is_some(),
            Overlay::SceneProperties => self.panels.scene_properties.is_some(),
            Overlay::Effects => self.panels.effects.is_some(),
            Overlay::BackendsMenu => {
                self.chrome.bar.menu == Some(crate::frontend::ui::MenuKind::Backends)
            }
            Overlay::ThemeAudition => self.theme.audition_open,
            Overlay::ThemeBar => self.theme.bar_open,
            Overlay::TagCloud => self.tags.cloud_open,
            Overlay::FoldersMenu => {
                self.chrome.bar.menu == Some(crate::frontend::ui::MenuKind::Folders)
            }
            Overlay::Settings => self.panels.settings.open,
        }
    }

    pub(super) fn close_overlay(&mut self, ov: Overlay) {
        match ov {
            Overlay::Help => {
                self.input.help_open = false;
                self.retick();
            }
            Overlay::CardPicker => {
                self.panels.card_picker = None;
                self.retick();
            }
            Overlay::AudioPanel => {
                self.panels.audio = None;
                self.retick();
            }
            Overlay::Playlists => {
                if let Some(pl) = self.panels.playlists.as_mut()
                    && pl.filter_help
                {
                    pl.filter_help = false;
                    self.retick();
                    return;
                }
                let picker = self.panels.playlists.take().is_some_and(|pl| pl.picker);
                if !picker && self.library_session.playlist_filter.take().is_some() {
                    self.change_filters(|_| {});
                }
            }
            Overlay::TagMode => {
                self.tags.mode = false;
                self.tags.select.clear();
                self.tags.mass_tags.clear();
                self.tags.mass_input.clear();
                self.chrome.bar.cache.clear();
                self.retick();
            }
            Overlay::TagEditing => {
                if self.tags.editing {
                    commit_pending_tag(self);
                }
                self.tags.editing = false;
                self.tags.card_drawer_open = false;
                self.scene.set_tag_editing(false);
                self.retick();
            }
            Overlay::Detail => {
                self.tags.editing = false;
                self.tags.card_drawer_open = false;
                if self.scene.flip_open() || self.detail_open() {
                    self.scene.close_flip();
                } else {
                    self.scene.hand_reveal_close();
                }
                self.retick();
            }
            Overlay::BrowserPreview => {
                self.source_browser.preview_closing = true;
                self.retick();
            }
            Overlay::Browser => {
                self.clear_browser_previews();
                self.source_browser.close();
            }
            Overlay::ThemeDesigner => {
                self.panels.theme_designer = None;
                self.retick();
            }
            Overlay::ColourStudio => {
                self.panels.colour_studio = None;
                self.retick();
            }
            Overlay::ScheduleEditor => {
                super::helpers::sched_persist(self);
                self.panels.schedule = None;
                self.retick();
            }
            Overlay::SceneProperties => {
                self.panels.scene_properties = None;
                self.retick();
            }
            Overlay::Effects => {
                self.panels.effects = None;
                self.retick();
            }
            Overlay::BackendsMenu | Overlay::FoldersMenu => {
                self.chrome.bar.menu = None;
                self.chrome.bar.cache.clear();
                self.retick();
            }
            Overlay::ThemeAudition => {
                self.theme.audition_open = false;
                self.retick();
            }
            Overlay::ThemeBar => {
                self.theme.bar_open = false;
                self.chrome.bar.cache.clear();
                self.retick();
            }
            Overlay::TagCloud => {
                self.tags.cloud_open = false;
                self.tags.matching_tags_open = false;
                self.tags.semantic.search.clear();
                self.tags.tag_search.clear();
                self.clear_semantic_search();
                self.retick();
            }
            Overlay::Settings => {
                self.close_settings();
            }
        }
    }

    pub(super) fn topmost_overlay(&self) -> Option<Overlay> {
        Overlay::ESC_ORDER.into_iter().find(|&ov| self.overlay_open(ov))
    }

    pub(super) fn close_topmost_overlay(&mut self) -> bool {
        match self.topmost_overlay() {
            Some(ov) => {
                self.close_overlay(ov);
                true
            }
            None => false,
        }
    }

    pub(super) fn menu_capturing(&self) -> bool {
        Overlay::ALL.into_iter().any(|ov| ov.policy().captures && self.overlay_open(ov))
    }

    pub(super) fn input_scopes(&self) -> crate::domain::input::ActiveScopes {
        let search = self.tags.cloud_open;
        let downloads = self.overlay_open(Overlay::Browser);
        crate::domain::input::ActiveScopes {
            fields: search || downloads || self.tags.editing || self.menu_capturing(),
            search,
            downloads,
        }
    }

    pub(super) fn picker_obscured(&self) -> bool {
        Overlay::ALL.into_iter().any(|ov| ov.policy().obscures && self.overlay_open(ov))
    }
}

mod tests;
