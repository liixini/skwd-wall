#[allow(clippy::wildcard_imports)]
use super::super::*;

impl App {
    pub(in crate::app) fn clear_browser_previews(&mut self) {
        self.reset_browser_media(None, std::time::Instant::now());
    }

    pub(in crate::app) fn run_browser_search(&mut self, append: bool) {
        if !append {
            self.source_browser.wall.scene.reset_to_start(0);
        }
        let Some((source, generation, mut call)) = (|| {
            let browser = self.source_browser.browser.as_mut()?;
            let manual = self.config.browser_apply_button(browser.source);
            if browser.session.search_generation == 0 {
                apply_browser_defaults(browser, &self.config);
            }
            if !append {
                self.source_browser.search_generation =
                    self.source_browser.search_generation.saturating_add(1);
                browser.session.search_generation = self.source_browser.search_generation;
            }
            let page = if append { browser.session.page.saturating_add(1) } else { 1 };
            browser.session.loading = true;
            if !append {
                browser.session.error = None;
                browser.session.page_failed = false;
            }
            let after = if append { browser.session.next_cursor.clone() } else { String::new() };
            let mut request = if append || (manual && browser.session.submitted_search.is_some()) {
                browser
                    .session
                    .submitted_search
                    .clone()
                    .unwrap_or_else(|| browser.search_request(page, &after))
            } else {
                let request = browser.search_request(1, "");
                browser.session.submitted_search = Some(request.clone());
                request
            };
            match &mut request {
                crate::contracts::browser::SearchRequest::Wallhaven(search) => search.page = page,
                crate::contracts::browser::SearchRequest::Steam(search) => search.page = page,
                crate::contracts::browser::SearchRequest::Catalog(search) => search.page = page,
            }
            Some((
                browser.source,
                browser.session.search_generation,
                crate::infrastructure::browser::encode_search(&request),
            ))
        })() else {
            return;
        };
        call.params["generation"] = serde_json::json!(generation);
        self.call_tracked(
            call.method,
            call.params,
            Pending::BrowserSearch { source, append, generation },
        );
    }
}
