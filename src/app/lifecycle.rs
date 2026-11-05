use super::App;

impl App {
    pub(in crate::app) fn on_hidden(&mut self) {
        crate::app::update::strand::dismiss(self);
        self.save_browse_position();
        self.panels.transition_preview.stop();
        self.release_settings_preview();
        self.theme.suspended = true;
        self.theme.shell_preview_sent = None;
        self.theme.preview_target = None;
        self.daemon.client.call("wall.shell_preview_end", serde_json::json!({}));
        self.clear_browser_previews();
        self.scene.release_while_hidden();
        self.theme.palette = self.theme.base_palette;
        crate::shell::trim_heap();
    }
}
