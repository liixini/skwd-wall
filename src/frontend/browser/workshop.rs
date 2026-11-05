use iced::alignment::Horizontal;
use iced::widget::{Space, button, column, container, image, scrollable, stack, text};
use iced::{Alignment, Element, Length, Padding, Size};

use crate::contracts::browser::WorkshopPreview;
use crate::frontend::animation::ease_out_cubic;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{
    Flow, NERD_FONT, TYPE_SMALL, end, folio_action, folio_action_width, folio_horizontal_rule,
    folio_icon_action, folio_masthead, label, row, start, thin_hbar, with_alpha,
};
use crate::i18n::{tr, tr_args};

use super::model::{BrowserIntent, BrowserItem, BrowserMsg, BrowserViewState, Source};

const MARGIN: f32 = 20.0;
const GAP: f32 = 16.0;
const SHEET_BORDER: f32 = 1.0;
const RAIL_WIDTH: f32 = 320.0;
const RAIL_PADDING: f32 = 18.0;
const STAGE_INSET: f32 = 18.0;
const AMBIENT_ALPHA: f32 = 0.34;
const AMBIENT_WASH_ALPHA: f32 = 0.72;
const STAGE_OUTLINE_ALPHA: f32 = 0.34;
const RULE_ALPHA: f32 = 0.42;
const TITLE_SIZE: f32 = 22.0;
const FIGURE_SIZE: f32 = 19.0;
const BODY_SIZE: f32 = 12.5;
const KICKER_SIZE: f32 = 9.0;
const PLAY_SIZE: f32 = 28.0;
const META_ALPHA: f32 = 0.56;
const BODY_ALPHA: f32 = 0.82;
const BODY_LINE_HEIGHT: f32 = 1.4;
const TAG_FILL_ALPHA: f32 = 0.9;
const TAG_OUTLINE_ALPHA: f32 = 0.46;
const TAG_PADDING: [f32; 2] = [3.0, 8.0];
const PLAY_SHADE_ALPHA: f32 = 0.34;
const SMALL_GAP: f32 = 6.0;
const COLUMN_GAP: f32 = 14.0;
const GALLERY_GAP: f32 = 6.0;
const GALLERY_THUMB_WIDTH: f32 = 118.0;
const GALLERY_THUMB_HEIGHT: f32 = 66.0;
const GALLERY_PADDING: f32 = 2.0;
const SCROLLBAR_ROOM: f32 = 8.0;
const COLLAPSED_CHARS: usize = 360;
const DIGIT_GROUP: usize = 3;
const DIGIT_SEPARATOR: char = '\u{202f}';
const TAG_SEPARATOR: char = ',';
const PREVIOUS_GLYPH: &str = "\u{f0141}";
const NEXT_GLYPH: &str = "\u{f0142}";
const PLAY_GLYPH: &str = "\u{f040a}";
const SECONDS_PER_DAY: u64 = 86_400;
const MAX_DATE_TIMESTAMP: u64 = 253_402_300_799;
const CIVIL_EPOCH: i64 = 719_468;
const CIVIL_DAYS_PER_400: i64 = 146_097;
const CIVIL_LAST_DAY: i64 = 146_096;
const CIVIL_DAYS_PER_4: i64 = 1_460;
const CIVIL_DAYS_PER_100: i64 = 36_524;
const CIVIL_DAYS_PER_YEAR: i64 = 365;
const CIVIL_YEARS_PER_ERA: i64 = 400;
const CIVIL_LEAP_PERIOD: i64 = 4;
const CIVIL_CENTURY: i64 = 100;
const CIVIL_MONTH_SCALE: i64 = 5;
const CIVIL_MONTH_DAYS: i64 = 153;
const CIVIL_MARCH_LIMIT: i64 = 10;
const CIVIL_MARCH_OFFSET: i64 = 3;
const CIVIL_JANUARY_OFFSET: i64 = 9;
const CREATED_KEY: &str = "browser-workshop-created";
const UPDATED_KEY: &str = "browser-workshop-updated";
const WORKSHOP_ID_KEY: &str = "browser-workshop-id";
const POSITION_KEY: &str = "browser-workshop-position";
const GALLERY_POSITION_KEY: &str = "browser-workshop-gallery-position";
const SUBSCRIBERS_KEY: &str = "browser-workshop-subscribers";
const VOTES_UP_KEY: &str = "browser-workshop-votes-up";
const VOTES_DOWN_KEY: &str = "browser-workshop-votes-down";

pub(super) struct Placement {
    pub size: Size,
    pub index: usize,
    pub count: usize,
}

pub(super) fn view<'a>(
    item: &'a BrowserItem,
    place: &Placement,
    state: &'a BrowserViewState,
    scale: f32,
    anim: f32,
    pal: &'a Palette,
) -> Element<'a, BrowserIntent> {
    let ease = ease_out_cubic(anim.clamp(0.0, 1.0));
    let header = folio_masthead(
        tr_args!(POSITION_KEY,
            source => Source::Steam.label(),
            index => place.index + 1,
            count => place.count),
        BrowserIntent::Update(BrowserMsg::ClosePreview),
        scale,
        pal,
    );
    let body =
        row![media_column(item, place, state, scale, pal), details_rail(item, state, scale, pal)]
            .spacing(GAP * scale)
            .height(Length::Fill);
    let plate = container(column![
        header,
        container(body).width(Length::Fill).height(Length::Fill).padding(MARGIN * scale),
    ])
    .style(move |_| crate::frontend::ui::bg_style(with_alpha(pal.surface, ease)));
    let sheet = container(plate)
        .width(Length::Fixed(place.size.width))
        .height(Length::Fixed(place.size.height))
        .padding(SHEET_BORDER)
        .style(move |_| super::view::sheet_style(pal, ease));
    super::view::sheet_frame(sheet, ease, BrowserIntent::Update(BrowserMsg::ClosePreview))
}

fn media_column<'a>(
    item: &'a BrowserItem,
    place: &Placement,
    state: &'a BrowserViewState,
    scale: f32,
    pal: &'a Palette,
) -> Element<'a, BrowserIntent> {
    let mut media = column![stage(item, place, state, scale, pal)]
        .spacing(GALLERY_GAP * scale)
        .width(Length::Fill)
        .height(Length::Fill);
    let count = media_count(item);
    if count > 1 {
        let mut strip = row![].spacing(GALLERY_GAP * scale);
        for asset in 0..count {
            strip = strip.push(media_thumbnail(item, state, asset, scale, pal));
        }
        media = media
            .push(row![
                kicker(tr("browser-workshop-gallery"), scale, pal),
                Space::new().width(Length::Fill),
                kicker(
                    tr_args!(GALLERY_POSITION_KEY, index => state.asset + 1, count => count),
                    scale,
                    pal
                ),
            ])
            .push(scrollable(strip).direction(thin_hbar()).width(Length::Fill).height(
                Length::Fixed(
                    (GALLERY_THUMB_HEIGHT + GALLERY_PADDING * 2.0 + SCROLLBAR_ROOM) * scale,
                ),
            ));
    }
    media.into()
}

fn stage<'a>(
    item: &'a BrowserItem,
    place: &Placement,
    state: &'a BrowserViewState,
    scale: f32,
    pal: &'a Palette,
) -> Element<'a, BrowserIntent> {
    let artwork: Element<'a, BrowserIntent> = match stage_art(item, state) {
        Some(handle) => stack![
            image(handle.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(iced::ContentFit::Cover)
                .opacity(AMBIENT_ALPHA),
            container(Space::new()).width(Length::Fill).height(Length::Fill).style(move |_| {
                crate::frontend::ui::bg_style(with_alpha(pal.background, AMBIENT_WASH_ALPHA))
            }),
            container(
                image(handle)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(iced::ContentFit::Contain)
            )
            .padding(STAGE_INSET * scale),
        ]
        .into(),
        None => container(label(
            tr("browser-loading-preview"),
            BODY_SIZE,
            scale,
            with_alpha(pal.surface_text, META_ALPHA),
        ))
        .center(Length::Fill)
        .into(),
    };
    let mut status = row![].spacing(SMALL_GAP * scale).align_y(Alignment::Center);
    if let Some(error) = state.media.error.as_deref().or(item.preview_error.as_deref()) {
        status = status.push(label(error, BODY_SIZE, scale, pal.surface_text)).push(action(
            tr("browser-retry"),
            false,
            BrowserMsg::RetryPreview,
            scale,
            pal,
        ));
    } else if state.media.loading && stage_art(item, state).is_none() {
        status =
            status.push(label(tr("browser-loading-preview"), BODY_SIZE, scale, pal.surface_text));
    }
    if let Some(index) = video_index(item, state.asset) {
        status = status.push(action(
            tr("browser-workshop-video"),
            true,
            BrowserMsg::OpenVideo(index),
            scale,
            pal,
        ));
    }
    let step = |show: bool, glyph: &'a str, target: usize, align: Horizontal| {
        let control: Element<'a, BrowserIntent> = if show {
            folio_icon_action(
                glyph,
                BrowserIntent::Update(BrowserMsg::OpenPreview(target)),
                scale,
                pal,
            )
        } else {
            Space::new().into()
        };
        container(control)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(align)
            .align_y(Alignment::Center)
            .padding(SMALL_GAP * scale)
    };
    let last = place.count.saturating_sub(1);
    let (previous_glyph, next_glyph) = if crate::frontend::ui::rtl() {
        (NEXT_GLYPH, PREVIOUS_GLYPH)
    } else {
        (PREVIOUS_GLYPH, NEXT_GLYPH)
    };
    container(stack![
        artwork,
        step(place.index > 0, previous_glyph, place.index.saturating_sub(1), start()),
        step(place.index < last, next_glyph, (place.index + 1).min(last), end()),
        container(status)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(start())
            .align_y(Alignment::End)
            .padding(STAGE_INSET * scale),
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .clip(true)
    .style(move |_| {
        crate::frontend::ui::box_style(pal.background, with_alpha(pal.outline, STAGE_OUTLINE_ALPHA))
    })
    .into()
}

fn details_rail<'a>(
    item: &'a BrowserItem,
    state: &'a BrowserViewState,
    scale: f32,
    pal: &'a Palette,
) -> Element<'a, BrowserIntent> {
    let title = if item.title.trim().is_empty() { &item.id } else { &item.title };
    let mut head = column![label(title, TITLE_SIZE, scale, pal.surface_text)]
        .spacing(SMALL_GAP * scale)
        .align_x(start());
    if let Some(details) = item.workshop.as_ref().filter(|details| !details.creator_id.is_empty()) {
        let name = if details.creator_name.is_empty() {
            &details.creator_id
        } else {
            &details.creator_name
        };
        head = head.push(action(name.as_str(), false, BrowserMsg::OpenCreator, scale, pal));
    }
    let footer = column![
        folio_action(
            tr("browser-apply"),
            true,
            Some(BrowserIntent::Update(BrowserMsg::Apply(item.id.clone()))),
            Length::Fill,
            scale,
            pal,
        ),
        row![
            folio_action(
                save_label(item),
                false,
                (!item.downloaded && !item.downloading)
                    .then(|| BrowserIntent::Update(BrowserMsg::Download(item.id.clone()))),
                Length::Fill,
                scale,
                pal,
            ),
            folio_action(
                tr("browser-workshop-open"),
                false,
                Some(BrowserIntent::Update(BrowserMsg::OpenWorkshop)),
                Length::Fill,
                scale,
                pal,
            ),
        ]
        .spacing(SMALL_GAP * scale),
        row![
            meta(tr_args!(WORKSHOP_ID_KEY, id => &item.id), scale, pal),
            Space::new().width(Length::Fill),
            action(
                tr("browser-copy-id"),
                false,
                BrowserMsg::CopyWorkshopId(item.id.clone()),
                scale,
                pal
            ),
        ]
        .align_y(Alignment::Center),
    ]
    .spacing(SMALL_GAP * scale);
    container(
        column![
            head,
            rule(pal),
            scrollable(record(item, state, scale, pal))
                .direction(crate::frontend::ui::thin_vbar())
                .height(Length::Fill),
            rule(pal),
            footer,
        ]
        .spacing(COLUMN_GAP * scale),
    )
    .width(Length::Fixed(RAIL_WIDTH * scale))
    .height(Length::Fill)
    .padding(RAIL_PADDING * scale)
    .style(move |_| crate::frontend::ui::folio_rail_style(pal))
    .into()
}

fn record<'a>(
    item: &'a BrowserItem,
    state: &'a BrowserViewState,
    scale: f32,
    pal: &'a Palette,
) -> Element<'a, BrowserIntent> {
    let mut body = column![]
        .spacing(COLUMN_GAP * scale)
        .padding(Padding { right: SCROLLBAR_ROOM * scale, ..Padding::ZERO });
    if state.details_loading {
        body = body.push(meta(tr("browser-workshop-loading").to_owned(), scale, pal));
    } else if let Some(error) = &state.details_error {
        body = body.push(
            row![
                label(error, BODY_SIZE, scale, pal.surface_text).width(Length::Fill),
                action(tr("browser-retry"), false, BrowserMsg::RetryDetails, scale, pal),
            ]
            .spacing(SMALL_GAP * scale)
            .align_y(Alignment::Center),
        );
    }
    let details = item.workshop.as_ref();
    let mut figures = row![].spacing(GAP * scale);
    for (key, value) in [
        (SUBSCRIBERS_KEY, details.and_then(|details| details.subscriptions)),
        (VOTES_UP_KEY, details.and_then(|details| details.votes_up.map(u64::from))),
        (VOTES_DOWN_KEY, details.and_then(|details| details.votes_down.map(u64::from))),
    ] {
        if let Some(value) = value {
            figures = figures.push(
                column![
                    label(grouped(value), FIGURE_SIZE, scale, pal.surface_text),
                    kicker(tr(key), scale, pal),
                ]
                .spacing(GALLERY_PADDING * scale),
            );
        }
    }
    if !figures.is_empty() {
        body = body.push(figures);
    }
    let tags: Vec<Element<'a, BrowserIntent>> = item
        .category
        .split(TAG_SEPARATOR)
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(|tag| tag_plate(tag, scale, pal))
        .collect();
    if !tags.is_empty() {
        body = body.push(
            Flow::new(tags).spacing(SMALL_GAP * scale).wrap().vertical_spacing(SMALL_GAP * scale),
        );
    }
    let mut facts = column![].spacing(GALLERY_PADDING * scale);
    if !item.resolution.is_empty() {
        facts = facts.push(meta(item.resolution.clone(), scale, pal));
    }
    if let Some(details) = details {
        if details.time_created > 0 {
            facts = facts.push(meta(
                tr_args!(CREATED_KEY, date => utc_date(details.time_created)),
                scale,
                pal,
            ));
        }
        if details.time_updated > 0 {
            facts = facts.push(meta(
                tr_args!(UPDATED_KEY, date => utc_date(details.time_updated)),
                scale,
                pal,
            ));
        }
    }
    body = body.push(facts);
    if let Some(raw) =
        details.map(|details| details.description.as_str()).filter(|raw| !raw.trim().is_empty())
    {
        let (copy, clipped) = description(raw, state.description_expanded);
        body = body.push(
            label(copy, BODY_SIZE, scale, with_alpha(pal.surface_text, BODY_ALPHA))
                .line_height(iced::widget::text::LineHeight::Relative(BODY_LINE_HEIGHT)),
        );
        if clipped || state.description_expanded {
            body = body.push(action(
                tr(if state.description_expanded {
                    "browser-workshop-less"
                } else {
                    "browser-workshop-more"
                }),
                false,
                BrowserMsg::ToggleDescription,
                scale,
                pal,
            ));
        }
    }
    body.into()
}

fn action<'a>(
    title: &str,
    active: bool,
    msg: BrowserMsg,
    scale: f32,
    pal: &'a Palette,
) -> Element<'a, BrowserIntent> {
    folio_action(
        title.to_owned(),
        active,
        Some(BrowserIntent::Update(msg)),
        Length::Fixed(folio_action_width(title, scale)),
        scale,
        pal,
    )
}

fn save_label(item: &BrowserItem) -> String {
    if item.downloaded {
        tr("browser-saved").to_owned()
    } else if item.downloading {
        super::model::progress_label(tr("browser-saving"), item)
    } else {
        tr("browser-save").to_owned()
    }
}

fn rule<'a>(pal: &Palette) -> Element<'a, BrowserIntent> {
    folio_horizontal_rule(with_alpha(pal.outline, RULE_ALPHA))
}

fn kicker<'a>(
    value: impl text::IntoFragment<'a>,
    scale: f32,
    pal: &Palette,
) -> Element<'a, BrowserIntent> {
    label(value, KICKER_SIZE, scale, pal.primary).into()
}

fn meta(value: String, scale: f32, pal: &Palette) -> Element<'_, BrowserIntent> {
    label(value, TYPE_SMALL, scale, with_alpha(pal.surface_text, META_ALPHA)).into()
}

fn tag_plate<'a>(tag: &str, scale: f32, pal: &'a Palette) -> Element<'a, BrowserIntent> {
    container(label(tag.to_owned(), TYPE_SMALL, scale, pal.surface_text))
        .padding(Padding::from(TAG_PADDING.map(|side| side * scale)))
        .style(move |_| {
            crate::frontend::ui::box_style(
                with_alpha(pal.surface_container, TAG_FILL_ALPHA),
                with_alpha(pal.outline, TAG_OUTLINE_ALPHA),
            )
        })
        .into()
}

fn grouped(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() * 2);
    for (idx, digit) in digits.chars().enumerate() {
        if idx > 0 && (digits.len() - idx).is_multiple_of(DIGIT_GROUP) {
            out.push(DIGIT_SEPARATOR);
        }
        out.push(digit);
    }
    out
}

fn stage_art(item: &BrowserItem, state: &BrowserViewState) -> Option<image::Handle> {
    let selected = media_url(item, state.asset);
    let ready = selected.as_deref().is_some_and(|url| {
        state.media.item_id == item.id && state.media.url == url && state.media.image.is_some()
    });
    if ready {
        return state.media.image.clone();
    }
    selected.as_deref().and_then(|url| decoded_art(item, state, url)).or_else(|| {
        (state.asset == 0 && item.thumb_ready && !item.thumb_path.is_empty())
            .then(|| image::Handle::from_path(&item.thumb_path))
    })
}

fn decoded_art(item: &BrowserItem, state: &BrowserViewState, url: &str) -> Option<image::Handle> {
    state
        .animations
        .get(url)
        .map(|animation| animation.frame(state.clock).clone())
        .or_else(|| item.preview_handles.get(url).cloned())
}

fn media_thumbnail<'a>(
    item: &'a BrowserItem,
    state: &'a BrowserViewState,
    asset: usize,
    scale: f32,
    pal: &'a Palette,
) -> Element<'a, BrowserIntent> {
    let url = media_url(item, asset);
    let selected = asset == state.asset;
    let current = url.as_deref().is_some_and(|url| {
        selected
            && state.media.item_id == item.id
            && state.media.url == url
            && state.media.image.is_some()
    });
    let handle = if current {
        state.media.image.clone()
    } else {
        url.as_deref().and_then(|url| decoded_art(item, state, url)).or_else(|| {
            (asset == 0 && item.thumb_ready && !item.thumb_path.is_empty())
                .then(|| image::Handle::from_path(&item.thumb_path))
        })
    };
    let artwork: Element<'a, BrowserIntent> = match handle {
        Some(handle) => image(handle)
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(iced::ContentFit::Cover)
            .into(),
        None => container(Space::new()).width(Length::Fill).height(Length::Fill).into(),
    };
    let content = if video_index(item, asset).is_some() {
        stack![
            artwork,
            container(
                text(PLAY_GLYPH).font(NERD_FONT).size(PLAY_SIZE * scale).color(pal.surface_text)
            )
            .center(Length::Fill)
            .style(move |_| {
                crate::frontend::ui::bg_style(with_alpha(pal.background, PLAY_SHADE_ALPHA))
            }),
        ]
        .into()
    } else {
        artwork
    };
    button(content)
        .width(Length::Fixed(GALLERY_THUMB_WIDTH * scale))
        .height(Length::Fixed(GALLERY_THUMB_HEIGHT * scale))
        .padding(GALLERY_PADDING * scale)
        .on_press(BrowserIntent::Update(BrowserMsg::SelectPreview(asset)))
        .style(move |_, status| {
            crate::frontend::ui::folio_button_style(selected, false, pal, status)
        })
        .into()
}

fn media_url(item: &BrowserItem, asset: usize) -> Option<std::borrow::Cow<'_, str>> {
    if asset == 0 {
        return Some(std::borrow::Cow::Borrowed(&item.full_url));
    }
    preview_at(&item.workshop.as_ref()?.previews, asset).map(|(_, preview)| preview.poster_url())
}

fn video_index(item: &BrowserItem, asset: usize) -> Option<usize> {
    let (index, preview) = preview_at(&item.workshop.as_ref()?.previews, asset)?;
    preview.is_video().then_some(index)
}

fn preview_at(previews: &[WorkshopPreview], asset: usize) -> Option<(usize, &WorkshopPreview)> {
    let index = asset.checked_sub(1)?;
    previews.get(index).map(|preview| (index, preview))
}

fn media_count(item: &BrowserItem) -> usize {
    1 + item.workshop.as_ref().map_or(0, |details| details.previews.len())
}

fn description(raw: &str, expanded: bool) -> (String, bool) {
    if expanded {
        return (raw.to_owned(), false);
    }
    let Some((index, _)) = raw.char_indices().nth(COLLAPSED_CHARS) else {
        return (raw.to_owned(), false);
    };
    (format!("{}…", raw[..index].trim_end()), true)
}

fn utc_date(timestamp: u64) -> String {
    let days = (timestamp.min(MAX_DATE_TIMESTAMP) / SECONDS_PER_DAY) as i64 + CIVIL_EPOCH;
    let era = if days >= 0 { days } else { days - CIVIL_LAST_DAY } / CIVIL_DAYS_PER_400;
    let day_of_era = days - era * CIVIL_DAYS_PER_400;
    let year_of_era = (day_of_era - day_of_era / CIVIL_DAYS_PER_4
        + day_of_era / CIVIL_DAYS_PER_100
        - day_of_era / CIVIL_LAST_DAY)
        / CIVIL_DAYS_PER_YEAR;
    let mut year = year_of_era + era * CIVIL_YEARS_PER_ERA;
    let day_of_year = day_of_era
        - (CIVIL_DAYS_PER_YEAR * year_of_era + year_of_era / CIVIL_LEAP_PERIOD
            - year_of_era / CIVIL_CENTURY);
    let month_prime = (CIVIL_MONTH_SCALE * day_of_year + 2) / CIVIL_MONTH_DAYS;
    let day = day_of_year - (CIVIL_MONTH_DAYS * month_prime + 2) / CIVIL_MONTH_SCALE + 1;
    let month = month_prime
        + if month_prime < CIVIL_MARCH_LIMIT { CIVIL_MARCH_OFFSET } else { -CIVIL_JANUARY_OFFSET };
    year += i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests;
