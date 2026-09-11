use iced::widget::canvas::{Frame, Path, Stroke};
use iced::widget::{
    button, canvas, column, container, image, mouse_area, row, scrollable, shader, stack, text,
    text_input,
};
use iced::{
    Alignment, Background, Color, Element, Font, Length, Padding, Point, Rectangle, Size, mouse,
};

use crate::frontend::animation::ease_out_cubic;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{UI_FONT, folio_horizontal_rule, label, mid_text, with_alpha};
use crate::i18n::{
    browser_downloading_count, browser_downloading_queued, browser_masthead, browser_queued_count,
    browser_results_page, tr, tr_args,
};

use super::model::{
    Browser, BrowserIntent as Message, BrowserItem, BrowserMsg, ON_MEDIA, PURITY_NSFW, PURITY_SFW,
    PURITY_SKETCHY, Source, SourceAvailability, SourceUnavailableReason, download_phase, fmt_clock,
    fmt_size, progress_label,
};

fn sheet_style(pal: &Palette, fade: f32) -> container::Style {
    let mut style = crate::frontend::ui::box_style(
        with_alpha(pal.surface, 0.985 * fade),
        with_alpha(pal.outline, 0.58 * fade),
    );
    style.shadow = iced::Shadow {
        color: with_alpha(Color::BLACK, 0.58 * fade),
        offset: iced::Vector::new(0.0, 16.0),
        blur_radius: 48.0,
    };
    style
}

fn catalogue_hero<'a>(
    br: &'a Browser,
    scale: f32,
    ease: f32,
    pal: &'a Palette,
    current_wallpaper_art: Option<&'a str>,
) -> Element<'a, Message> {
    let artwork_path = current_wallpaper_art
        .or_else(|| br.session.items.iter().find_map(preview_image_path).map(|(path, _)| path));
    let artwork: Element<'a, Message> = artwork_path.map_or_else(
        || container(text("")).into(),
        |path| {
            image(image::Handle::from_path(path))
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(iced::ContentFit::Cover)
                .opacity(ease)
                .into()
        },
    );
    let identity = column![
        label(tr("browser-hero-kicker"), 9.0, scale, with_alpha(pal.primary, 0.94 * ease)),
        text(br.source.label())
            .font(UI_FONT)
            .size(34.0 * scale.clamp(0.9, 1.08))
            .line_height(iced::widget::text::LineHeight::Relative(0.94))
            .color(with_alpha(pal.surface_text, 0.99 * ease)),
        label(
            tr("browser-hero-tagline"),
            crate::frontend::ui::TYPE_SMALL,
            scale,
            with_alpha(pal.surface_text, 0.64 * ease)
        ),
    ]
    .spacing(6.0 * scale);
    let counter = column![
        text(format!("{:02}", br.session.items.len()))
            .font(UI_FONT)
            .size(34.0 * scale.clamp(0.9, 1.08))
            .line_height(iced::widget::text::LineHeight::Relative(0.86))
            .color(with_alpha(pal.surface_text, 0.98 * ease)),
        label(
            browser_results_page(br.session.page),
            9.0,
            scale,
            with_alpha(pal.primary, 0.9 * ease)
        ),
    ]
    .spacing(4.0 * scale)
    .align_x(Alignment::End);
    let identity_plane = container(identity)
        .width(Length::Fixed(430.0 * scale))
        .padding(Padding {
            top: 15.0 * scale,
            right: 24.0 * scale,
            bottom: 17.0 * scale,
            left: 24.0 * scale,
        })
        .style(move |_| crate::frontend::ui::bg_style(Background::Color(pal.background)));
    let counter_plane = container(counter)
        .width(Length::Fixed(144.0 * scale))
        .height(Length::Fixed(76.0 * scale))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(move |_| crate::frontend::ui::bg_style(Background::Color(pal.surface_variant)));
    let cover_title = container(identity_plane)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Start)
        .align_y(Alignment::End)
        .padding(Padding { top: 0.0, right: 0.0, bottom: 18.0 * scale, left: 18.0 * scale });
    let issue = container(counter_plane)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::End)
        .align_y(Alignment::Start)
        .padding(Padding { top: 18.0 * scale, right: 18.0 * scale, bottom: 0.0, left: 0.0 });
    container(stack![artwork, cover_title, issue,])
        .width(Length::Fill)
        .height(Length::Fixed(132.0 * scale))
        .clip(true)
        .style(move |_| {
            crate::frontend::ui::box_style(
                with_alpha(pal.background, 0.72),
                with_alpha(pal.outline, 0.42),
            )
        })
        .into()
}

fn pill<'a>(label: String, bg: Color, fg: Color, size: f32) -> Element<'a, Message> {
    container(text(label).size(size).color(fg))
        .padding(Padding::from([1.0, 5.0]))
        .style(move |_t| crate::frontend::ui::bg_style(bg))
        .into()
}

fn action_btn(label: &str, msg: Message, accent: Color, fg: Color) -> Element<'_, Message> {
    button(text(label).size(12))
        .padding(Padding::from([4.0, 12.0]))
        .on_press(msg)
        .style(move |_t, _s| button::Style {
            background: Some(accent.into()),
            text_color: fg,
            border: iced::Border { radius: 0.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

fn source_tabs<'a>(
    active: Source,
    availability: &[SourceAvailability],
    scale: f32,
    pal: &'a Palette,
) -> Element<'a, Message> {
    let mut tabs = row![].spacing(1.0 * scale);
    for entry in availability.iter().copied() {
        let selected = entry.source == active;
        let enabled = entry.unavailable.is_none();
        let status = match entry.unavailable {
            None => tr("browser-source-available"),
            Some(SourceUnavailableReason::MissingSteamHelper) => tr("browser-source-steam-helper"),
            Some(SourceUnavailableReason::Disabled) => tr("browser-source-disabled"),
            Some(SourceUnavailableReason::MissingCredentials) => tr("browser-source-credentials"),
        };
        let tab = button(
            column![
                label(
                    entry.source.label(),
                    11.0,
                    scale,
                    with_alpha(
                        if selected { pal.primary_text } else { pal.surface_text },
                        if enabled { 0.96 } else { 0.34 },
                    ),
                ),
                label(
                    status,
                    8.0,
                    scale,
                    with_alpha(
                        if selected { pal.primary_text } else { pal.surface_text },
                        if enabled { 0.58 } else { 0.28 },
                    ),
                ),
            ]
            .spacing(2.0 * scale)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fixed(48.0 * scale))
        .padding(Padding::from([7.0 * scale, 5.0 * scale]))
        .on_press_maybe(
            (enabled && !selected)
                .then_some(Message::Update(BrowserMsg::SwitchSource(entry.source))),
        )
        .style(move |_theme, state| {
            crate::frontend::ui::folio_button_style(selected, false, pal, state)
        });
        tabs = tabs.push(tab);
    }
    container(tabs).width(Length::Fill).padding(Padding::from([0.0, 24.0 * scale])).into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BrowserCardAction {
    Save,
    Apply,
}

#[derive(Debug, Clone, Copy)]
struct BrowserCardActionRects {
    save: Rectangle,
    apply: Rectangle,
}

fn browser_card_action_rects(
    hit: &crate::frontend::scene::layout::Hit,
    scale: f32,
) -> BrowserCardActionRects {
    let card_w = hit.hw * 2.0;
    let card_h = hit.hh * 2.0;
    let height = (30.0 * scale).min(card_h);
    let total_w = (158.0 * scale).min(card_w);
    let save_w = total_w * 0.56;
    let x = hit.cx + hit.hw - total_w;
    let y = hit.cy + hit.hh - height;
    BrowserCardActionRects {
        save: Rectangle::new(Point::new(x, y), Size::new(save_w, height)),
        apply: Rectangle::new(Point::new(x + save_w, y), Size::new(total_w - save_w, height)),
    }
}

fn point_in_rect(rect: Rectangle, x: f32, y: f32) -> bool {
    x >= rect.x && x <= rect.x + rect.width && y >= rect.y && y <= rect.y + rect.height
}

pub(crate) fn browser_card_action_at(
    hit: &crate::frontend::scene::layout::Hit,
    x: f32,
    y: f32,
    scale: f32,
) -> Option<BrowserCardAction> {
    let actions = browser_card_action_rects(hit, scale);
    if point_in_rect(actions.apply, x, y) {
        Some(BrowserCardAction::Apply)
    } else if point_in_rect(actions.save, x, y) {
        Some(BrowserCardAction::Save)
    } else {
        None
    }
}

struct BrowserCardOverlay<'a> {
    browser: &'a Browser,
    render: std::sync::Arc<crate::frontend::scene::RenderSnapshot>,
    pal: &'a Palette,
    scale: f32,
    fade: f32,
}

impl canvas::Program<Message> for BrowserCardOverlay<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let fade = self.fade.clamp(0.0, 1.0);
        for hit in &self.render.hits {
            let Some(item) = self.browser.session.items.get(hit.index) else {
                continue;
            };
            if hit.cx + hit.hw < 0.0
                || hit.cx - hit.hw > bounds.width
                || hit.cy + hit.hh < 0.0
                || hit.cy - hit.hh > bounds.height
            {
                continue;
            }
            if let Some(clip) = self.render.clip
                && (hit.cy + hit.hh > clip[3] + 0.5 || hit.cy - hit.hh < clip[1] - 0.5)
            {
                continue;
            }
            if item.downloaded {
                draw_downloaded_badge(&mut frame, hit, self.pal, self.scale, fade);
            }
            if self.browser.session.hover == Some(hit.index) {
                draw_card_actions(
                    &mut frame,
                    hit,
                    item,
                    self.browser.session.pending_apply.as_deref() == Some(item.id.as_str()),
                    self.pal,
                    self.scale,
                    fade,
                );
            }
        }
        vec![frame.into_geometry()]
    }
}

fn draw_downloaded_badge(
    frame: &mut Frame,
    hit: &crate::frontend::scene::layout::Hit,
    pal: &Palette,
    scale: f32,
    fade: f32,
) {
    let height = (22.0 * scale).min(hit.hh * 2.0);
    let width = (70.0 * scale).min(hit.hw * 2.0);
    let rect = Rectangle::new(
        Point::new(hit.cx + hit.hw - width, hit.cy - hit.hh),
        Size::new(width, height),
    );
    let path = Path::rectangle(rect.position(), rect.size());
    frame.fill(&path, with_alpha(pal.primary, 0.94 * fade));
    frame.stroke(
        &path,
        Stroke::default().with_color(with_alpha(pal.primary_text, 0.42 * fade)).with_width(1.0),
    );
    frame.fill_text(mid_text(
        tr("browser-saved"),
        rect.center(),
        with_alpha(pal.primary_text, fade),
        10.0 * scale,
        Font { weight: iced::font::Weight::Bold, ..UI_FONT },
        Alignment::Center,
    ));
}

fn draw_card_actions(
    frame: &mut Frame,
    hit: &crate::frontend::scene::layout::Hit,
    item: &BrowserItem,
    applying: bool,
    pal: &Palette,
    scale: f32,
    fade: f32,
) {
    let actions = browser_card_action_rects(hit, scale);
    let save = Path::rectangle(actions.save.position(), actions.save.size());
    let apply = Path::rectangle(actions.apply.position(), actions.apply.size());
    frame.fill(&save, with_alpha(pal.surface_container, 0.94 * fade));
    frame.fill(&apply, with_alpha(pal.primary, 0.97 * fade));
    frame.stroke(
        &save,
        Stroke::default().with_color(with_alpha(pal.outline, 0.72 * fade)).with_width(1.0),
    );
    frame.stroke(
        &apply,
        Stroke::default().with_color(with_alpha(pal.primary_text, 0.38 * fade)).with_width(1.0),
    );
    let save_label = if item.downloaded {
        tr("browser-saved").to_string()
    } else if item.downloading {
        progress_label(tr("browser-saving"), item)
    } else {
        tr("browser-save").to_string()
    };
    frame.fill_text(mid_text(
        save_label,
        actions.save.center(),
        with_alpha(
            pal.surface_text,
            if item.downloaded || item.downloading { 0.56 * fade } else { 0.96 * fade },
        ),
        10.0 * scale,
        UI_FONT,
        Alignment::Center,
    ));
    frame.fill_text(mid_text(
        if applying { tr("browser-applying") } else { tr("browser-apply") },
        actions.apply.center(),
        with_alpha(pal.primary_text, if applying { 0.62 * fade } else { fade }),
        10.0 * scale,
        Font { weight: iced::font::Weight::Bold, ..UI_FONT },
        Alignment::Center,
    ));
}

pub(super) fn preview_image_path(item: &BrowserItem) -> Option<(&str, bool)> {
    item.preview_path.as_deref().filter(|path| !path.is_empty()).map(|path| (path, true)).or_else(
        || {
            (item.thumb_ready && !item.thumb_path.is_empty())
                .then_some((item.thumb_path.as_str(), false))
        },
    )
}

fn preview_modal<'a>(
    item: &'a BrowserItem,
    anim: f32,
    spin: f32,
    pal: &Palette,
    clip: Option<(&'a str, &'a str)>,
) -> Element<'a, Message> {
    let preview = preview_image_path(item);
    let full_quality = preview.is_some_and(|(_, ready)| ready);
    let img: Element<'a, Message> = match preview {
        Some((path, _)) => image(image::Handle::from_path(path))
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(iced::ContentFit::Contain)
            .opacity(anim)
            .into(),
        None => container(
            canvas(crate::frontend::ui::Spinner { angle: spin, color: pal.primary, size: 64.0 })
                .width(Length::Fixed(84.0))
                .height(Length::Fixed(84.0)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into(),
    };

    let close = button(
        text("\u{f0156}").size(20.0).font(crate::frontend::ui::NERD_FONT).color(Color::WHITE),
    )
    .padding(8)
    .on_press(Message::Update(BrowserMsg::ClosePreview))
    .style(|_t, _s| button::Style {
        background: Some(Color { r: 1.0, g: 1.0, b: 1.0, a: 0.1 }.into()),
        border: iced::Border { radius: 0.0.into(), ..Default::default() },
        ..Default::default()
    });

    let purity_bg = match item.purity.as_str() {
        "sketchy" => PURITY_SKETCHY,
        "nsfw" => PURITY_NSFW,
        _ => PURITY_SFW,
    };
    let white = ON_MEDIA;
    let mut info = row![
        text(&item.resolution).size(13).color(white),
        text(fmt_size(item.file_size)).size(12).color(Color { r: 1.0, g: 1.0, b: 1.0, a: 0.65 }),
        pill(item.category.clone(), Color { r: 1.0, g: 1.0, b: 1.0, a: 0.1 }, white, 11.0),
        pill(item.purity.clone(), purity_bg, white, 11.0),
    ]
    .spacing(20)
    .align_y(Alignment::Center);
    if !item.attribution.is_empty() {
        let credit = if item.attribution_url.is_empty() {
            item.attribution.clone()
        } else {
            format!("{}  \u{2022}  {}", item.attribution, item.attribution_url)
        };
        info = info.push(text(credit).size(12).color(Color { r: 1.0, g: 1.0, b: 1.0, a: 0.7 }));
    }

    if let Some((start, len)) = clip.filter(|_| item.duration_secs > 0) {
        let sel = crate::frontend::ui::with_alpha(pal.primary, 0.4);
        let field = move |value: &'a str,
                          placeholder: &'static str,
                          on_input: fn(String) -> Message|
              -> Element<'a, Message> {
            text_input(placeholder, value)
                .on_input(on_input)
                .size(12)
                .width(Length::Fixed(62.0))
                .padding(Padding::from([4.0, 6.0]))
                .style(move |_t, _s| iced::widget::text_input::Style {
                    background: crate::frontend::ui::with_alpha(Color::WHITE, 0.10).into(),
                    border: iced::Border {
                        color: crate::frontend::ui::with_alpha(Color::WHITE, 0.22),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    icon: Color::WHITE,
                    placeholder: crate::frontend::ui::with_alpha(Color::WHITE, 0.4),
                    value: Color::WHITE,
                    selection: sel,
                })
                .into()
        };
        let dim = Color { r: 1.0, g: 1.0, b: 1.0, a: 0.55 };
        info = info.push(text(tr("browser-clip")).size(12).color(white));
        info =
            info.push(field(start, "0:00", |value| Message::Update(BrowserMsg::ClipStart(value))));
        info = info.push(text("+").size(12).color(dim));
        info = info.push(field(len, "3:00", |value| Message::Update(BrowserMsg::ClipLen(value))));
        info = info.push(
            text(tr_args!("browser-duration-total", duration => fmt_clock(item.duration_secs)))
                .size(11)
                .color(dim),
        );
    }

    let save_label = if item.downloaded {
        tr("browser-saved").to_string()
    } else if item.downloading {
        progress_label(tr("browser-saving"), item)
    } else {
        tr("browser-save").to_string()
    };
    let can_save = !item.downloaded && !item.downloading;
    let save_btn = button(text(save_label).size(12))
        .padding(Padding::from([4.0, 14.0]))
        .on_press_maybe(can_save.then(|| Message::Update(BrowserMsg::Download(item.id.clone()))))
        .style(move |_t, _s| button::Style {
            background: Some(
                crate::frontend::ui::with_alpha(Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, 0.12)
                    .into(),
            ),
            text_color: white,
            border: iced::Border { radius: 0.0.into(), ..Default::default() },
            ..Default::default()
        });
    let apply_btn = action_btn(
        tr("browser-apply"),
        Message::Update(BrowserMsg::Apply(item.id.clone())),
        pal.primary,
        pal.primary_text,
    );
    let actions = row![save_btn, apply_btn].spacing(8).align_y(Alignment::Center);

    let bar = row![info, iced::widget::Space::new().width(Length::Fill), actions,]
        .align_y(Alignment::Center)
        .padding(Padding::from([0.0, 24.0]));

    let body = stack![
        container(img).width(Length::Fill).height(Length::Fill).padding(40),
        container(if full_quality {
            container(text("")).into()
        } else {
            strip_row(tr("browser-loading-preview").to_string(), false, Some(spin), pal)
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Start)
        .align_y(Alignment::Start)
        .padding(20),
        container(close)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::End)
            .align_y(Alignment::Start)
            .padding(20),
        container(
            container(bar)
                .width(Length::Fill)
                .height(Length::Fixed(56.0))
                .center_y(Length::Fixed(56.0))
                .style(move |_t| crate::frontend::ui::bg_style(Color {
                    a: 0.6 * anim,
                    ..Color::BLACK
                })),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(Alignment::End),
    ];
    crate::frontend::ui::inert_backdrop(
        container(body).width(Length::Fill).height(Length::Fill).style(move |_t| {
            crate::frontend::ui::bg_style(Color { a: 0.97 * anim, ..Color::BLACK })
        }),
        Message::Capture,
    )
}

fn strip_row<'a>(
    msg: String,
    is_error: bool,
    spin: Option<f32>,
    pal: &Palette,
) -> Element<'a, Message> {
    let bg = if is_error {
        crate::frontend::ui::with_alpha(pal.primary, 0.9)
    } else {
        crate::frontend::ui::with_alpha(pal.surface, 0.85)
    };
    let fg = if is_error { pal.primary_text } else { pal.surface_text };
    let mut content = row![].spacing(8).align_y(Alignment::Center);
    if let Some(angle) = spin {
        content = content.push(
            canvas(crate::frontend::ui::Spinner { angle, color: pal.primary, size: 12.0 })
                .width(Length::Fixed(14.0))
                .height(Length::Fixed(14.0)),
        );
    }
    content = content.push(text(msg).size(12).color(fg));
    container(content)
        .width(Length::Fill)
        .padding(Padding::from([6.0, 10.0]))
        .style(move |_t| crate::frontend::ui::bg_style(bg))
        .into()
}

pub(super) fn flight_label(
    pending_apply: bool,
    active: usize,
    queued: usize,
    searching: bool,
    source_name: &str,
    flight: Option<&BrowserItem>,
) -> Option<String> {
    if pending_apply {
        return Some(flight.map_or_else(
            || tr("browser-applying-status").to_string(),
            |it| progress_label(tr("browser-applying"), it),
        ));
    }
    if active + queued > 1 {
        return Some(match (active, queued) {
            (_, 0) => browser_downloading_count(active),
            (0, _) => browser_queued_count(queued),
            _ => browser_downloading_queued(active, queued),
        });
    }
    if active == 1 {
        return Some(flight.map_or_else(
            || tr("browser-downloading-status").to_string(),
            |it| progress_label(tr("browser-downloading"), it),
        ));
    }
    if queued == 1 {
        return Some(flight.filter(|it| !it.phase.is_empty()).map_or_else(
            || tr("browser-queued-status").to_string(),
            |it| format!("{}\u{2026}", download_phase(&it.phase)),
        ));
    }
    if searching {
        return Some(tr_args!("browser-searching-status", source => source_name));
    }
    None
}

fn status_strip<'a>(br: &Browser, spin: f32, pal: &Palette) -> Option<Element<'a, Message>> {
    if let Some(err) = br.session.error.as_ref().filter(|_| !br.session.items.is_empty()) {
        return Some(strip_row(err.clone(), true, None, pal));
    }
    let active = br.session.items.iter().filter(|it| it.downloading && !it.queued).count();
    let queued = br.session.items.iter().filter(|it| it.queued).count();
    let flight = br
        .session
        .pending_apply
        .as_ref()
        .and_then(|id| br.session.items.iter().find(|it| &it.id == id))
        .or_else(|| br.session.items.iter().find(|it| it.downloading && !it.queued))
        .or_else(|| br.session.items.iter().find(|it| it.queued));
    let msg = flight_label(
        br.session.pending_apply.is_some(),
        active,
        queued,
        br.session.loading && !br.session.items.is_empty(),
        br.source.label(),
        flight,
    )?;
    Some(strip_row(msg, false, Some(spin), pal))
}

pub fn view<'a>(
    br: &'a Browser,
    availability: &[SourceAvailability],
    show_apply_button: bool,
    viewport: (f32, f32),
    grid: crate::contracts::picker::BrowserGrid,
    wall_render: std::sync::Arc<crate::frontend::scene::RenderSnapshot>,
    wall_uploads: crate::contracts::preview::UploadQueue,
    wall_pool: std::sync::Arc<crate::contracts::preview::BufPool>,
    wall_chrome_cache: &'a canvas::Cache,
    entrance: f32,
    spin: f32,
    _shimmer: f32,
    preview_anim: f32,
    scale: f32,
    pal: &'a Palette,
    current_wallpaper_art: Option<&'a str>,
) -> Element<'a, Message> {
    let filter_w = (282.0 * scale).clamp(238.0, 310.0);
    let filter_inner_w = filter_w - 28.0 * scale;
    let workspace_gap = 14.0 * scale;
    let compact_bar_size = crate::frontend::ui::browser_bar_compact_size(br, scale, filter_inner_w);
    let (panel_w, panel_h, sheet_width, grid_h) = wall_aperture_dims(viewport, grid, scale);
    let ease = ease_out_cubic(entrance.clamp(0.0, 1.0));
    let _ = sheet_width;

    let bar = canvas(crate::frontend::ui::BrowserBar {
        browser: br,
        pal,
        scale,
        compact_width: filter_inner_w,
    })
    .width(Length::Fixed(filter_inner_w))
    .height(Length::Fixed(compact_bar_size.1));
    let filters = scrollable(
        column![
            label(
                tr("browser-search-label"),
                8.5,
                scale,
                with_alpha(pal.surface_text, 0.42 * ease)
            ),
            container(search_box(br, scale, pal))
                .width(Length::Fill)
                .padding(Padding::from([6.0 * scale, 8.0 * scale]))
                .style(move |_| {
                    crate::frontend::ui::box_style(
                        with_alpha(pal.background, 0.44),
                        with_alpha(pal.outline, 0.34),
                    )
                }),
            folio_horizontal_rule(with_alpha(pal.outline, 0.34)),
            label(tr("browser-filter-index"), 9.0, scale, with_alpha(pal.primary, 0.9 * ease)),
            label(
                tr("browser-filter-title"),
                19.0,
                scale,
                with_alpha(pal.surface_text, 0.96 * ease)
            ),
            label(
                tr(if show_apply_button {
                    "browser-filter-apply-desc"
                } else {
                    "browser-filter-desc"
                }),
                crate::frontend::ui::TYPE_SMALL,
                scale,
                with_alpha(pal.surface_text, 0.48 * ease)
            ),
            bar,
        ]
        .spacing(10.0 * scale)
        .padding(Padding { right: 10.0 * scale, ..Padding::ZERO }),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .direction(crate::frontend::ui::thin_vbar())
    .style(crate::frontend::ui::scroll_style(with_alpha(pal.outline, 0.72)));
    let mut parameters = column![filters].spacing(10.0 * scale);
    if show_apply_button && br.source.searchable() {
        parameters = parameters.push(action_btn(
            tr("browser-apply-filters"),
            Message::Update(BrowserMsg::SearchSubmit),
            pal.primary,
            pal.primary_text,
        ));
    }
    let parameters = container(parameters)
        .width(Length::Fixed(filter_w))
        .height(Length::Fill)
        .padding(Padding {
            top: 15.0 * scale,
            right: 4.0 * scale,
            bottom: 15.0 * scale,
            left: 14.0 * scale,
        })
        .style(move |_| {
            crate::frontend::ui::box_style(
                with_alpha(pal.surface_variant, 0.3),
                with_alpha(pal.outline, 0.34),
            )
        });

    let wall_shader: Element<'a, Message> = shader(crate::app::scene::BrowserSceneProgram {
        render: wall_render.clone(),
        uploads: wall_uploads,
        pool: wall_pool,
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into();
    let wall_chrome: Element<'a, Message> = canvas(crate::frontend::ui::ChromeCanvas {
        render: wall_render.clone(),
        pal,
        cache: wall_chrome_cache,
        overview_set: false,
        show_type_badges: false,
        show_video_indicators: true,
        fade: ease,
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into();
    let card_overlay: Element<'a, Message> =
        canvas(BrowserCardOverlay { browser: br, render: wall_render, pal, scale, fade: ease })
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    let wall = stack![
        wall_shader,
        wall_chrome,
        card_overlay,
        canvas(crate::frontend::ui::BrowserWallInputLayer).width(Length::Fill).height(Length::Fill),
    ];
    let body: Element<'a, Message> = if br.session.loading && br.session.items.is_empty() {
        container(
            canvas(crate::frontend::ui::Spinner { angle: spin, color: pal.primary, size: 90.0 })
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(120.0)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else if br.session.items.is_empty() {
        container(
            text(br.session.error.as_deref().unwrap_or(tr("browser-no-results")))
                .size(13)
                .color(with_alpha(pal.surface_text, 0.4)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else {
        wall.into()
    };
    let discovery = catalogue_hero(br, scale, ease, pal, current_wallpaper_art);
    let sheet_body: Element<'a, Message> = container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| {
            crate::frontend::ui::box_style(
                with_alpha(pal.background, 0.34),
                with_alpha(pal.outline, 0.24),
            )
        })
        .into();
    let contact_sheet: Element<'a, Message> = match status_strip(br, spin, pal) {
        Some(strip) => stack![
            sheet_body,
            container(strip).width(Length::Fill).height(Length::Fill).align_y(Alignment::Start),
        ]
        .into(),
        None => sheet_body,
    };
    let workspace = row![
        parameters,
        container(contact_sheet).width(Length::Fill).height(Length::Fixed(grid_h)),
    ]
    .height(Length::Fixed(grid_h))
    .spacing(workspace_gap);
    let content = column![discovery, workspace].spacing(14.0 * scale);
    let header = crate::frontend::ui::folio_masthead(
        browser_masthead(br.source.label(), br.session.items.len(), br.session.page),
        Message::Close,
        scale,
        pal,
    );
    let tabs = source_tabs(br.source, availability, scale, pal);
    let reading = container(content).width(Length::Fill).height(Length::Fill).padding(Padding {
        top: 16.0 * scale,
        right: 24.0 * scale,
        bottom: 22.0 * scale,
        left: 24.0 * scale,
    });
    let furniture = column![header, tabs, reading].spacing(0.0).height(Length::Fill);
    let atmosphere: Element<'a, Message> =
        br.session.items.iter().find_map(preview_image_path).map_or_else(
            || container(text("")).into(),
            |(path, _)| {
                image(image::Handle::from_path(path))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(iced::ContentFit::Cover)
                    .opacity(0.055 * ease)
                    .into()
            },
        );
    let wash = container(text("")).width(Length::Fill).height(Length::Fill).style(move |_| {
        crate::frontend::ui::bg_style(Background::Color(with_alpha(pal.surface, 0.92 * ease)))
    });
    let stage = stack![atmosphere, wash, furniture]
        .width(Length::Fixed(panel_w))
        .height(Length::Fixed(panel_h));

    let rise = (1.0 - ease) * 38.0;
    let panel = mouse_area(
        container(stage)
            .width(Length::Fixed(panel_w))
            .height(Length::Fixed(panel_h))
            .clip(true)
            .style(move |_| sheet_style(pal, ease)),
    )
    .on_press(Message::Capture);
    let main = crate::frontend::ui::inert_backdrop(
        container(container(panel).padding(Padding { top: rise, ..Padding::ZERO }))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_| crate::frontend::ui::folio_scrim_style(ease)),
        Message::Capture,
    );

    let clip = (br.source == Source::Youtube)
        .then_some((br.request.catalog.clip_start.as_str(), br.request.catalog.clip_len.as_str()));
    let overlay: Element<'a, Message> =
        match br.session.preview.and_then(|idx| br.session.items.get(idx)) {
            Some(item) => preview_modal(item, preview_anim, spin, pal, clip),
            None => container(text("")).into(),
        };
    stack![main, overlay].into()
}

pub fn wall_aperture_dims(
    viewport: (f32, f32),
    grid: crate::contracts::picker::BrowserGrid,
    scale: f32,
) -> (f32, f32, f32, f32) {
    let (vw, vh) = viewport;
    let cols = grid.cols.max(1);
    let grid_w = cols as f32 * grid.thumb_w + cols.saturating_sub(1) as f32 * grid.gap_x;
    let filter_w = (282.0 * scale).clamp(238.0, 310.0);
    let reading_padding_x = 48.0 * scale;
    let workspace_gap = 14.0 * scale;
    let desired_width = grid_w + filter_w + workspace_gap + reading_padding_x;
    let panel_w = desired_width.max(620.0 * scale).min((vw - 24.0).max(220.0));
    let desired_grid_h =
        grid.rows.max(1) as f32 * grid.thumb_h + grid.rows.saturating_sub(1) as f32 * grid.gap_y;
    let desired_height = desired_grid_h + 298.0 * scale;
    let panel_h = desired_height.max(500.0 * scale).min((vh - 24.0).max(160.0));
    let reading_width = (panel_w - reading_padding_x).max(220.0);
    let sheet_width = (reading_width - filter_w - workspace_gap).max(220.0);
    let grid_h = (panel_h - 292.0 * scale).max(100.0);
    (panel_w, panel_h, sheet_width, grid_h)
}

fn search_box<'a>(br: &'a Browser, scale: f32, pal: &'a Palette) -> Element<'a, Message> {
    if !br.source.searchable() {
        return container(text(br.source.label()).size(14.0 * scale).color(pal.surface_text))
            .padding(6)
            .into();
    }
    let placeholder = match br.source {
        Source::Steam => tr("browser-search-steam"),
        Source::Wallhaven => tr("browser-search-wallhaven"),
        Source::Unsplash => tr("browser-search-unsplash"),
        Source::Pexels => tr("browser-search-pexels"),
        Source::Youtube => tr("browser-search-youtube"),
        Source::Bing => tr("browser-search-placeholder"),
    };
    text_input(placeholder, &br.request.query)
        .font(UI_FONT)
        .on_input(|value| Message::Update(BrowserMsg::SearchInput(value)))
        .on_submit(Message::Update(BrowserMsg::SearchSubmit))
        .padding(6)
        .size(13.0 * scale)
        .width(Length::Fill)
        .style(move |_t, _s| crate::frontend::ui::ghost_input_style(pal))
        .into()
}
