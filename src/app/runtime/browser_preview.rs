use std::time::{Duration, Instant};

use crate::app::state::PreviewDecode;
use crate::app::{App, Pending};
use crate::contracts::browser::{PreviewFrames, PreviewIdentity, PreviewSize, Source};
use crate::frontend::browser::{BrowserMediaView, PreviewAnimation, frame_handles};
use crate::infrastructure::runtime::Wake;

const HOVER_DELAY: Duration = Duration::from_millis(180);
const PREVIEW_TIMEOUT: Duration = Duration::from_secs(30);
const DETAILS_METHOD: &str = "source.details";
const DECODE_THREAD: &str = "browser-preview";
const SOURCE_PARAM: &str = "source";
const ID_PARAM: &str = "id";
const MAX_PREVIEW_REQUESTS: usize = 2;
const MAX_SELECTED_REQUESTS: usize = 4;
const MAX_CACHED_THUMBNAILS: usize = 128;
const MAX_CARD_REQUESTS: usize = 3;
const MAX_ANIMATION_BYTES: usize = 96 << 20;

impl App {
    fn browser_media_target(&self) -> Option<PreviewIdentity> {
        if self.runtime_state.stashed || self.source_browser.preview_closing {
            return None;
        }
        let browser = self.source_browser.browser.as_ref()?;
        let index = browser.session.preview.or_else(|| {
            (browser.source == Source::Steam).then_some(browser.session.hover).flatten()
        })?;
        let item = browser.session.items.get(index)?;
        let asset = if browser.session.preview.is_some() { browser.view.asset } else { 0 };
        let url = if asset == 0 {
            item.full_url.clone()
        } else {
            item.workshop.as_ref()?.previews.get(asset - 1)?.poster_url().into_owned()
        };
        (!url.is_empty()).then(|| PreviewIdentity {
            source: browser.source,
            id: item.id.clone(),
            url,
        })
    }

    pub(in crate::app) fn sync_browser_media(&mut self, now: Instant) {
        self.sync_card_animations(now);
        let desired = self.browser_media_target();
        if self.source_browser.media.key != desired {
            self.reset_browser_media(desired.clone(), now);
        }
        let Some(key) = desired else { return };
        let browser = self.source_browser.browser.as_ref().unwrap();
        let modal = browser.session.preview.is_some();
        let elapsed = now.saturating_duration_since(self.source_browser.media.since.unwrap_or(now));
        if !self.source_browser.media.requested && (modal || elapsed >= HOVER_DELAY) {
            let cached = browser
                .session
                .items
                .iter()
                .find(|item| item.id == key.id)
                .and_then(|item| {
                    item.preview_paths.get(&key.url).cloned().or_else(|| {
                        (item.full_url == key.url).then(|| item.preview_path.clone()).flatten()
                    })
                })
                .filter(|path| std::path::Path::new(path).is_file());
            if let Some(path) = cached {
                self.source_browser.media.requested = true;
                self.source_browser.media.since = Some(now);
                self.queue_browser_decode(key.clone(), path.clone());
                self.queue_browser_thumbnail(&key, &path);
            } else if self.source_browser.media.requests.contains(&key)
                || self.source_browser.media.cards.requests.contains_key(&key.url)
            {
                self.source_browser.media.requested = true;
                self.source_browser.media.since = Some(now);
            } else if self.source_browser.media.requests.len() < MAX_SELECTED_REQUESTS {
                let Some(item) = browser.session.items.iter().find(|item| item.id == key.id) else {
                    return;
                };
                self.source_browser.media.requested = true;
                self.source_browser.media.since = Some(now);
                self.source_browser.media.failed.remove(&key);
                let mut request = browser.preview_request(item);
                request.full_url.clone_from(&key.url);
                request.refresh = self.source_browser.media.refresh;
                let call = crate::infrastructure::browser::encode_preview(&request);
                self.source_browser.media.requests.insert(key.clone());
                self.call_tracked(
                    call.method,
                    call.params,
                    Pending::BrowserPreview {
                        source: key.source,
                        id: key.id.clone(),
                        url: key.url.clone(),
                    },
                );
            }
        }
        if elapsed >= PREVIEW_TIMEOUT
            && self
                .source_browser
                .browser
                .as_ref()
                .is_some_and(|browser| browser.view.media.loading)
        {
            self.fail_browser_preview(&key, crate::i18n::tr("browser-preview-timeout").to_string());
        }
        let media = &self.source_browser.media;
        let frame = media.frame_index(now);
        if let (Some(handle), Some(browser)) =
            (media.handles.get(frame), self.source_browser.browser.as_mut())
            && browser.view.media.frame != frame
        {
            browser.view.media.frame = frame;
            browser.view.media.image = Some(handle.clone());
        }
    }

    pub(in crate::app) fn reset_browser_media(
        &mut self,
        desired: Option<PreviewIdentity>,
        now: Instant,
    ) {
        let media = &mut self.source_browser.media;
        media.generation = media.generation.wrapping_add(1);
        media.since = desired.as_ref().map(|_| now);
        media.requested = false;
        media.refresh = false;
        media.handles.clear();
        media.delays.clear();
        media.duration = Duration::ZERO;
        media.started = None;
        media.decode_pending = None;
        media.decode_path = None;
        let same_item = media
            .key
            .as_ref()
            .zip(desired.as_ref())
            .is_some_and(|(old, new)| old.source == new.source && old.id == new.id);
        if !same_item {
            media.thumbnail_generation = media.thumbnail_generation.wrapping_add(1);
            media.thumbnails.clear();
            media.thumbnail_queued.clear();
            media.thumbnail_done.clear();
            media.thumbnail_order.clear();
            media.failed.clear();
            for browser in
                self.source_browser.browser.iter_mut().chain(self.source_browser.tabs.values_mut())
            {
                for item in &mut browser.session.items {
                    item.preview_handles.clear();
                }
            }
        }
        for browser in
            self.source_browser.browser.iter_mut().chain(self.source_browser.tabs.values_mut())
        {
            browser.view.media = BrowserMediaView::default();
            if desired.is_none() {
                browser.view.details_loading = false;
            }
        }
        if let (Some(key), Some(browser)) = (desired.as_ref(), self.source_browser.browser.as_mut())
        {
            browser.view.media = BrowserMediaView {
                item_id: key.id.clone(),
                url: key.url.clone(),
                loading: true,
                ..BrowserMediaView::default()
            };
            if let Some(item) = browser.item_mut(&key.id) {
                item.preview_error = None;
            }
        }
        media.key = desired;
    }

    pub(in crate::app) fn retry_browser_preview(&mut self) {
        let key = self.source_browser.media.key.clone();
        if let Some(key) = key.as_ref()
            && let Some(browser) = self.source_browser.browser.as_mut()
            && let Some(item) = browser.item_mut(&key.id)
        {
            item.preview_paths.remove(&key.url);
            item.preview_handles.remove(&key.url);
            if item.full_url == key.url {
                item.preview_path = None;
            }
        }
        if let Some(key) = key.as_ref() {
            self.source_browser.media.failed.remove(key);
        }
        self.reset_browser_media(key, Instant::now());
        self.source_browser.media.refresh = true;
        self.sync_browser_media(Instant::now());
        self.retick();
    }

    pub(in crate::app) fn fail_browser_preview(&mut self, key: &PreviewIdentity, error: String) {
        self.finish_card_request(&key.url);
        self.source_browser.media.requests.remove(key);
        self.source_browser.media.failed.insert(key.clone());
        if self.source_browser.media.key.as_ref() != Some(key) {
            self.sync_browser_media(Instant::now());
            self.request_workshop_media();
            return;
        }
        if let Some(browser) = self.source_browser.browser.as_mut() {
            browser.view.media.loading = false;
            browser.view.media.error = Some(error.clone());
            if let Some(item) = browser.item_mut(&key.id) {
                item.preview_error = Some(error);
            }
        }
        self.source_browser.media.handles.clear();
        self.source_browser.media.delays.clear();
        self.sync_browser_media(Instant::now());
        self.request_workshop_media();
    }

    pub(in crate::app) fn adopt_browser_preview(
        &mut self,
        key: &PreviewIdentity,
        path: &str,
        error: Option<String>,
    ) {
        if let Some(error) = error.filter(|error| !error.is_empty()) {
            self.fail_browser_preview(key, error);
            return;
        }
        if path.is_empty() {
            self.finish_card_request(&key.url);
            return;
        }
        self.source_browser.media.requests.remove(key);
        self.source_browser.media.failed.remove(key);
        if let Some(browser) = self.source_browser.browser_for_source_mut(key.source)
            && let Some(item) = browser.item_mut(&key.id)
        {
            item.preview_paths.insert(key.url.clone(), path.to_owned());
            if item.full_url == key.url {
                item.preview_path = Some(path.to_owned());
            }
        }
        if self.source_browser.media.key.as_ref() == Some(key) {
            self.queue_browser_decode(key.clone(), path.to_owned());
        }
        if self.finish_card_request(&key.url) {
            self.queue_card_decode(key.clone(), path.to_owned());
        }
        self.queue_browser_thumbnail(key, path);
        self.sync_browser_media(Instant::now());
        self.request_workshop_media();
    }

    fn queue_browser_decode(&mut self, key: PreviewIdentity, path: String) {
        if self.source_browser.media.decode_path.as_deref() == Some(path.as_str()) {
            return;
        }
        self.source_browser.media.decode_path = Some(path.clone());
        self.source_browser.media.decode_pending = Some(PreviewDecode {
            key,
            path,
            generation: self.source_browser.media.generation,
            size: PreviewSize::Hero,
        });
        self.start_browser_decode();
    }

    fn queue_browser_thumbnail(&mut self, key: &PreviewIdentity, path: &str) {
        let Some(browser) = self.source_browser.browser.as_ref() else { return };
        let Some(item) = browser.session.preview.and_then(|index| browser.session.items.get(index))
        else {
            return;
        };
        let selected = self.source_browser.media.key.as_ref() == Some(key);
        if browser.source != key.source
            || item.id != key.id
            || !item.workshop.as_ref().is_some_and(|details| {
                details.previews.iter().any(|preview| preview.poster_url() == key.url)
            })
            || item.preview_handles.contains_key(&key.url)
            || self.source_browser.media.failed.contains(key)
            || self.source_browser.media.thumbnail_queued.contains(key)
            || (!selected && self.source_browser.media.thumbnail_done.contains(key))
        {
            return;
        }
        if item.preview_handles.len() + self.source_browser.media.thumbnail_queued.len()
            >= MAX_CACHED_THUMBNAILS
        {
            if !selected {
                return;
            }
            if !self.evict_browser_thumbnail() {
                let Some(skipped) = self.source_browser.media.thumbnails.pop_back() else { return };
                self.source_browser.media.thumbnail_queued.remove(&skipped.key);
                self.source_browser.media.thumbnail_done.insert(skipped.key);
            }
        }
        self.source_browser.media.thumbnail_queued.insert(key.clone());
        self.source_browser.media.thumbnails.push_back(PreviewDecode {
            key: key.clone(),
            path: path.to_owned(),
            generation: self.source_browser.media.thumbnail_generation,
            size: PreviewSize::Thumbnail,
        });
        self.start_browser_decode();
    }

    fn sync_card_animations(&mut self, now: Instant) {
        let enabled = !self.runtime_state.stashed && !self.config.reduced_motion();
        let state = &mut self.source_browser;
        let steam = state.browser.as_ref().filter(|browser| browser.source == Source::Steam);
        let Some(browser) = steam.filter(|_| enabled) else {
            if !state.media.cards.idle()
                || state
                    .browser
                    .iter()
                    .chain(state.tabs.values())
                    .any(|browser| !browser.view.animations.is_empty())
            {
                state.media.cards.reset();
                for browser in state.browser.iter_mut().chain(state.tabs.values_mut()) {
                    browser.view.animations.clear();
                }
            }
            state.media.cards.visible = false;
            return;
        };
        if state.media.cards.search != browser.session.search_generation {
            state.media.cards.reset();
            state.media.cards.search = browser.session.search_generation;
            if let Some(browser) = state.browser.as_mut() {
                browser.view.animations.clear();
            }
        }
        let cards = &mut state.media.cards;
        let expired: Vec<String> = cards
            .requests
            .iter()
            .filter(|(_, at)| now.saturating_duration_since(**at) >= PREVIEW_TIMEOUT)
            .map(|(url, _)| url.clone())
            .collect();
        for url in expired {
            cards.requests.remove(&url);
            cards.seen.insert(url);
        }
        let Some(browser) = state.browser.as_mut() else { return };
        let mut visible = false;
        let mut cached = Vec::new();
        let mut requests = Vec::new();
        if let Some(item) =
            browser.session.preview.and_then(|index| browser.session.items.get(index))
        {
            visible = std::iter::once(item.full_url.clone())
                .chain(item.workshop.iter().flat_map(|details| {
                    details.previews.iter().map(|preview| preview.poster_url().into_owned())
                }))
                .any(|url| browser.view.animations.contains_key(&url));
        } else {
            for hit in &state.wall.scene.render.hits {
                let Some(item) = browser.session.items.get(hit.index) else { continue };
                let url = item.full_url.as_str();
                if browser.view.animations.contains_key(url) {
                    visible = true;
                    continue;
                }
                if url.is_empty() || cards.seen.contains(url) || cards.requests.contains_key(url) {
                    continue;
                }
                let key = PreviewIdentity {
                    source: Source::Steam,
                    id: item.id.clone(),
                    url: url.to_owned(),
                };
                match item.preview_paths.get(url).or(item.preview_path.as_ref()) {
                    Some(path) => cached.push((key, path.clone())),
                    None if cards.requests.len() + requests.len() < MAX_CARD_REQUESTS
                        && !state.media.requests.contains(&key) =>
                    {
                        requests.push((browser.preview_request(item), key));
                    }
                    None => {}
                }
            }
        }
        cards.visible = visible;
        if visible {
            let epoch = *cards.epoch.get_or_insert(now);
            browser.view.clock = now.saturating_duration_since(epoch);
        }
        for (key, path) in cached {
            self.queue_card_decode(key, path);
        }
        for (request, key) in requests {
            let call = crate::infrastructure::browser::encode_preview(&request);
            self.source_browser.media.cards.requests.insert(key.url.clone(), now);
            self.call_tracked(
                call.method,
                call.params,
                Pending::BrowserPreview { source: key.source, id: key.id, url: key.url },
            );
        }
    }

    fn finish_card_request(&mut self, url: &str) -> bool {
        let cards = &mut self.source_browser.media.cards;
        let pending = cards.requests.remove(url).is_some();
        if pending {
            cards.seen.insert(url.to_owned());
        }
        pending
    }

    fn queue_card_decode(&mut self, key: PreviewIdentity, path: String) {
        let cards = &mut self.source_browser.media.cards;
        cards.seen.insert(key.url.clone());
        cards.queue.push_back(PreviewDecode {
            key,
            path,
            generation: cards.generation,
            size: PreviewSize::Card,
        });
        self.start_browser_decode();
    }

    fn store_animation(&mut self, key: &PreviewIdentity, animation: PreviewAnimation) {
        let state = &mut self.source_browser;
        let Some(browser) = state.browser.as_mut().filter(|browser| {
            browser.source == key.source
                && browser.session.items.iter().any(|item| item.id == key.id)
        }) else {
            return;
        };
        let cards = &mut state.media.cards;
        let url = key.url.clone();
        cards.order.retain(|known| known != &url);
        cards.order.push_back(url.clone());
        browser.view.animations.insert(url.clone(), animation);
        let shown: std::collections::HashSet<&str> = state
            .wall
            .scene
            .render
            .hits
            .iter()
            .filter_map(|hit| browser.session.items.get(hit.index))
            .map(|item| item.full_url.as_str())
            .collect();
        let mut total: usize = browser.view.animations.values().map(PreviewAnimation::bytes).sum();
        while total > MAX_ANIMATION_BYTES {
            let hidden = cards
                .order
                .iter()
                .position(|known| known != &url && !shown.contains(known.as_str()));
            let Some(index) = hidden.or_else(|| cards.order.iter().position(|known| known != &url))
            else {
                break;
            };
            let Some(evicted) = cards.order.remove(index) else { break };
            if hidden.is_some() {
                cards.seen.remove(&evicted);
            }
            if let Some(animation) = browser.view.animations.remove(&evicted) {
                total -= animation.bytes();
            }
        }
        self.retick();
    }

    fn evict_browser_thumbnail(&mut self) -> bool {
        while let Some(old) = self.source_browser.media.thumbnail_order.pop_front() {
            if let Some(browser) = self.source_browser.browser_for_source_mut(old.source)
                && let Some(item) = browser.item_mut(&old.id)
                && item.preview_handles.remove(&old.url).is_some()
            {
                return true;
            }
        }
        false
    }

    fn start_browser_decode(&mut self) {
        if self.source_browser.media.decode_busy {
            return;
        }
        let Some(job) = self
            .source_browser
            .media
            .decode_pending
            .take()
            .or_else(|| self.source_browser.media.thumbnails.pop_front())
            .or_else(|| self.source_browser.media.cards.queue.pop_front())
        else {
            return;
        };
        self.source_browser.media.decode_busy = true;
        let sender = self.runtime_state.wake_tx.clone();
        let key = job.key.clone();
        let size = job.size;
        let generation = job.generation;
        if let Err(error) =
            std::thread::Builder::new().name(DECODE_THREAD.to_owned()).spawn(move || {
                let result = crate::infrastructure::preview::decode_browser_preview(
                    std::path::Path::new(&job.path),
                    job.size,
                );
                let _ = sender.unbounded_send(Wake::BrowserPreview {
                    key: job.key,
                    generation: job.generation,
                    size: job.size,
                    result,
                });
            })
        {
            self.source_browser.media.decode_busy = false;
            if size == PreviewSize::Thumbnail {
                if self.source_browser.media.thumbnail_generation == generation {
                    self.source_browser.media.thumbnail_queued.remove(&key);
                    self.source_browser.media.failed.insert(key);
                }
            } else if self.source_browser.media.generation == generation {
                self.fail_browser_preview(&key, error.to_string());
            }
            self.start_browser_decode();
        }
    }

    pub(in crate::app) fn on_browser_preview_decoded(
        &mut self,
        key: &PreviewIdentity,
        generation: u64,
        size: PreviewSize,
        result: Result<PreviewFrames, String>,
    ) {
        self.source_browser.media.decode_busy = false;
        if size == PreviewSize::Card {
            if generation == self.source_browser.media.cards.generation
                && let Some(animation) = result.ok().and_then(PreviewAnimation::new)
            {
                self.store_animation(key, animation);
            }
            self.start_browser_decode();
            return;
        }
        if size == PreviewSize::Thumbnail {
            if self.source_browser.media.thumbnail_generation != generation {
                self.start_browser_decode();
                return;
            }
            self.source_browser.media.thumbnail_queued.remove(key);
            match result {
                Ok(decoded) => {
                    let active = self.source_browser.browser.as_ref().is_some_and(|browser| {
                        browser.source == key.source
                            && browser
                                .session
                                .preview
                                .and_then(|index| browser.session.items.get(index))
                                .is_some_and(|item| item.id == key.id)
                    });
                    let still = active.then(|| decoded.frames.first().cloned()).flatten();
                    let card = self
                        .source_browser
                        .browser
                        .as_ref()
                        .is_some_and(|browser| browser.view.animations.contains_key(&key.url));
                    if active
                        && !card
                        && let Some(animation) = PreviewAnimation::new(decoded)
                    {
                        self.store_animation(key, animation);
                    }
                    if let Some(frame) = still {
                        if self.source_browser.browser.as_ref().is_some_and(|browser| {
                            browser.session.items.iter().find(|item| item.id == key.id).is_some_and(
                                |item| item.preview_handles.len() >= MAX_CACHED_THUMBNAILS,
                            )
                        }) {
                            self.evict_browser_thumbnail();
                        }
                        if let Some(browser) = self.source_browser.browser.as_mut()
                            && let Some(item) = browser.item_mut(&key.id)
                        {
                            item.preview_handles.insert(
                                key.url.clone(),
                                iced::widget::image::Handle::from_rgba(
                                    frame.width,
                                    frame.height,
                                    iced::advanced::graphics::core::Bytes::from_owner(frame.rgba),
                                ),
                            );
                            self.source_browser.media.thumbnail_order.push_back(key.clone());
                            self.source_browser.media.thumbnail_done.insert(key.clone());
                        }
                    }
                }
                Err(_) => {
                    self.source_browser.media.failed.insert(key.clone());
                }
            }
            self.start_browser_decode();
            self.request_workshop_media();
            return;
        }
        if self.source_browser.media.key.as_ref() == Some(key)
            && self.source_browser.media.generation == generation
        {
            match result {
                Ok(decoded) => {
                    let media = &mut self.source_browser.media;
                    media.duration = decoded.duration;
                    media.started = Some(Instant::now());
                    media.delays = decoded.frames.iter().map(|frame| frame.delay).collect();
                    media.handles = frame_handles(decoded.frames);
                    if let Some(browser) = self.source_browser.browser.as_mut() {
                        browser.view.media.image = media.handles.first().cloned();
                        browser.view.media.frames = media.handles.len();
                        browser.view.media.frame = 0;
                        browser.view.media.loading = false;
                        browser.view.media.error = None;
                    }
                }
                Err(error) => self.fail_browser_preview(key, error),
            }
        }
        self.start_browser_decode();
    }

    pub(in crate::app) fn request_workshop_details(&mut self) {
        let Some(browser) = self.source_browser.browser.as_mut() else { return };
        if browser.source != Source::Steam || browser.view.details_loading {
            return;
        }
        let Some(item) = browser.session.preview.and_then(|index| browser.session.items.get(index))
        else {
            return;
        };
        if item.details_ready
            || item.workshop.as_ref().is_some_and(|details| !details.previews.is_empty())
        {
            return;
        }
        let id = item.id.clone();
        browser.view.details_loading = true;
        browser.view.details_error = None;
        self.call_tracked(
            DETAILS_METHOD,
            serde_json::json!({SOURCE_PARAM: Source::Steam.key(), ID_PARAM: id}),
            Pending::BrowserDetails { source: Source::Steam, id },
        );
    }

    pub(in crate::app) fn request_workshop_media(&mut self) {
        let (cached, requests) = {
            let Some(browser) = self.source_browser.browser.as_ref() else { return };
            if browser.source != Source::Steam {
                return;
            }
            let Some(item) =
                browser.session.preview.and_then(|index| browser.session.items.get(index))
            else {
                return;
            };
            let mut cached = Vec::new();
            let mut requests = Vec::new();
            let mut seen = std::collections::HashSet::new();
            if let Some(details) = item.workshop.as_ref() {
                for preview in &details.previews {
                    let url = preview.poster_url().into_owned();
                    if url.is_empty() || !seen.insert(url.clone()) {
                        continue;
                    }
                    if seen.len() > MAX_CACHED_THUMBNAILS {
                        break;
                    }
                    let key = PreviewIdentity {
                        source: Source::Steam,
                        id: item.id.clone(),
                        url: url.clone(),
                    };
                    if let Some(path) = item
                        .preview_paths
                        .get(&url)
                        .filter(|path| std::path::Path::new(path).is_file())
                    {
                        cached.push((key, path.clone()));
                    } else if !self.source_browser.media.failed.contains(&key)
                        && !self.source_browser.media.requests.contains(&key)
                    {
                        let mut request = browser.preview_request(item);
                        request.full_url = url;
                        requests.push((request, key));
                    }
                }
            }
            (cached, requests)
        };
        for (key, path) in cached {
            self.queue_browser_thumbnail(&key, &path);
        }
        if self
            .source_browser
            .browser
            .as_ref()
            .and_then(|browser| {
                browser.session.preview.and_then(|index| browser.session.items.get(index))
            })
            .is_some_and(|item| {
                item.preview_handles.len() + self.source_browser.media.thumbnail_queued.len()
                    >= MAX_CACHED_THUMBNAILS
            })
        {
            return;
        }
        for (request, key) in requests {
            if self.source_browser.media.requests.len() >= MAX_PREVIEW_REQUESTS {
                break;
            }
            let call = crate::infrastructure::browser::encode_preview(&request);
            self.source_browser.media.requests.insert(key.clone());
            self.call_tracked(
                call.method,
                call.params,
                Pending::BrowserPreview { source: Source::Steam, id: key.id, url: key.url },
            );
        }
    }
}
