use iced::font::Weight;
use iced::{Color, Font, Point, Rectangle};
use skwd_palette::studio::StudioPalette;

use super::pen::{Pen, Words};
use crate::frontend::components::lerp_color;
use crate::frontend::theme::rgb_color;
use crate::frontend::ui::cached_line_width;
use crate::i18n::tr;

pub const UNIT: f32 = 100.0;
pub const LINE: f32 = 1.35;
const HAIRLINE: f32 = 1.0;

pub const SANS: Font = Font::DEFAULT;
pub const MEDIUM: Font = Font { weight: Weight::Medium, ..Font::DEFAULT };
pub const SEMIBOLD: Font = Font { weight: Weight::Semibold, ..Font::DEFAULT };
pub const MONO: Font = Font::MONOSPACE;
pub const MONO_BOLD: Font = Font { weight: Weight::Bold, ..Font::MONOSPACE };

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Desk {
    Browser,
    Chat,
    Terminal,
}

#[derive(Clone, Copy)]
pub struct Tones {
    pub bg: Color,
    pub surface: Color,
    pub raised: Color,
    pub fg: Color,
    pub muted: Color,
    pub primary: Color,
    pub on_primary: Color,
    pub second: Color,
    pub third: Color,
}

impl Tones {
    pub fn of(palette: &StudioPalette) -> Self {
        Self {
            bg: rgb_color(palette.base),
            surface: rgb_color(palette.panel),
            raised: rgb_color(palette.raised),
            fg: rgb_color(palette.text),
            muted: rgb_color(palette.quiet),
            primary: rgb_color(palette.accent),
            on_primary: rgb_color(palette.on_accent),
            second: rgb_color(palette.second),
            third: rgb_color(palette.third),
        }
    }
}

pub fn mix(from: Color, to: Color, share: f32) -> Color {
    lerp_color(from, to, share)
}

pub struct Space {
    rect: Rectangle,
    pub unit: f32,
}

impl Space {
    pub fn new(rect: Rectangle) -> Self {
        Self { rect, unit: rect.width / UNIT }
    }

    pub fn at(&self, x: f32, y: f32, width: f32, height: f32) -> Rectangle {
        Rectangle {
            x: self.rect.x + x * self.unit,
            y: self.rect.y + y * self.unit,
            width: width * self.unit,
            height: height * self.unit,
        }
    }

    pub fn point(&self, x: f32, y: f32) -> Point {
        Point::new(self.rect.x + x * self.unit, self.rect.y + y * self.unit)
    }

    pub fn height(&self) -> f32 {
        self.rect.height / self.unit
    }

    pub fn words<'a>(
        &self,
        content: &'a str,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
    ) -> Words<'a> {
        Words::new(content, self.point(x, y), size * self.unit, color)
            .font(SANS)
            .line(LINE)
            .middle()
    }

    pub fn width(&self, content: &str, size: f32, font: Font) -> f32 {
        cached_line_width(content, size * self.unit, font) / self.unit
    }

    pub fn hairline(&self) -> f32 {
        HAIRLINE / self.unit
    }
}

const BROWSER_TABS: [&str; 5] = [
    "colour-studio-browser-tab-notes",
    "colour-studio-browser-tab-picker",
    "colour-studio-browser-tab-wiki",
    "colour-studio-browser-tab-gallery",
    "colour-studio-browser-tab-docs",
];
const BROWSER_TILES: [&str; 5] = ["wallhaven", "forgejo", "youtrack", "niri", "docs.rs"];

struct BrowserSpec {
    pad: f32,
    side: f32,
    font: f32,
    url: f32,
    url_radius: f32,
    url_font: f32,
    url_pad: f32,
    gap: f32,
    tab: f32,
    tab_radius: f32,
    tab_pad: f32,
    icon: f32,
    icon_radius: f32,
    icon_gap: f32,
    page_radius: f32,
    search_share: f32,
    search: f32,
    search_radius: f32,
    search_pad: f32,
    stack_gap: f32,
    tile_column: f32,
    tile_gap: f32,
    tile: f32,
    tile_radius: f32,
    tile_label_gap: f32,
    tile_font: f32,
    edge_mix: f32,
    on_mix: f32,
}

const BROWSER: BrowserSpec = BrowserSpec {
    pad: 0.8,
    side: 19.0,
    font: 1.25,
    url: 3.2,
    url_radius: 0.8,
    url_font: 1.15,
    url_pad: 1.0,
    gap: 0.6,
    tab: 2.9,
    tab_radius: 0.7,
    tab_pad: 0.8,
    icon: 1.3,
    icon_radius: 0.3,
    icon_gap: 0.7,
    page_radius: 1.0,
    search_share: 0.58,
    search: 4.0,
    search_radius: 0.9,
    search_pad: 1.6,
    stack_gap: 2.4,
    tile_column: 6.6,
    tile_gap: 1.8,
    tile: 5.0,
    tile_radius: 1.0,
    tile_label_gap: 0.6,
    tile_font: 1.05,
    edge_mix: 0.55,
    on_mix: 0.65,
};

fn browser(pen: &mut Pen<'_>, rect: Rectangle, tones: &Tones, radius: f32) {
    let spec = &BROWSER;
    let space = Space::new(rect);
    let height = space.height();
    let ov = mix(tones.muted, tones.bg, spec.edge_mix);
    let low = mix(tones.bg, tones.surface, 0.5);
    let pc = mix(tones.primary, tones.bg, spec.on_mix);
    pen.rounded(rect, radius, tones.bg);
    let url = space.at(spec.pad, spec.pad, spec.side, spec.url);
    pen.rounded(url, spec.url_radius * space.unit, tones.surface);
    let label = tr("colour-studio-browser-address");
    pen.words(space.words(
        label,
        spec.pad + spec.url_pad,
        spec.pad + spec.url / 2.0,
        spec.url_font,
        tones.third,
    ));
    let mut top = spec.pad + spec.url + spec.gap + spec.gap;
    for (index, key) in BROWSER_TABS.iter().enumerate() {
        let on = index == 0;
        let tab = space.at(spec.pad, top, spec.side, spec.tab);
        pen.rounded(tab, spec.tab_radius * space.unit, if on { pc } else { low });
        let icon_top = top + (spec.tab - spec.icon) / 2.0;
        let icon = space.at(spec.pad + spec.tab_pad, icon_top, spec.icon, spec.icon);
        pen.rounded(icon, spec.icon_radius * space.unit, if on { tones.primary } else { ov });
        let x = spec.pad + spec.tab_pad + spec.icon + spec.icon_gap;
        let clip = space.at(spec.pad, top, spec.side - spec.tab_pad, spec.tab);
        pen.clipped(clip, |pen| {
            pen.words(space.words(
                tr(key),
                x,
                top + spec.tab / 2.0,
                spec.font,
                if on { tones.fg } else { tones.third },
            ));
        });
        top += spec.tab + spec.gap;
    }
    let left = spec.pad + spec.side + spec.pad;
    let width = UNIT - left - spec.pad;
    let page_height = height - 2.0 * spec.pad;
    let page = space.at(left, spec.pad, width, page_height);
    let line = space.hairline();
    pen.rounded(
        space.at(left - line, spec.pad - line, width + 2.0 * line, page_height + 2.0 * line),
        (spec.page_radius + line) * space.unit,
        ov,
    );
    pen.rounded(page, spec.page_radius * space.unit, tones.bg);
    let tile_block = spec.tile + spec.tile_label_gap + spec.tile_font * LINE;
    let block = spec.search + spec.stack_gap + tile_block;
    let first = spec.pad + (page_height - block) / 2.0;
    let search_width = width * spec.search_share;
    let search_left = left + (width - search_width) / 2.0;
    let search = space.at(search_left, first, search_width, spec.search);
    pen.rounded(search, spec.search_radius * space.unit, tones.surface);
    pen.ring(search, spec.search_radius * space.unit, HAIRLINE, ov);
    let hint = tr("colour-studio-browser-search");
    pen.words(space.words(
        hint,
        search_left + spec.search_pad,
        first + spec.search / 2.0,
        spec.font,
        tones.third,
    ));
    let count = BROWSER_TILES.len() as f32;
    let span = count * spec.tile_column + (count - 1.0) * spec.tile_gap;
    let tiles_left = left + (width - span) / 2.0;
    let tiles_top = first + spec.search + spec.stack_gap;
    for (index, label) in BROWSER_TILES.iter().enumerate() {
        let column = tiles_left + index as f32 * (spec.tile_column + spec.tile_gap);
        let centre = column + spec.tile_column / 2.0;
        let tile = space.at(centre - spec.tile / 2.0, tiles_top, spec.tile, spec.tile);
        pen.rounded(tile, spec.tile_radius * space.unit, tones.surface);
        pen.ring(tile, spec.tile_radius * space.unit, HAIRLINE, ov);
        let label_y = tiles_top + spec.tile + spec.tile_label_gap + spec.tile_font * LINE / 2.0;
        pen.words(space.words(label, centre, label_y, spec.tile_font, tones.third).centered());
    }
}

const CHAT_SERVER: &str = "skwd";
const CHAT_ME: &str = "liixini";
const CHAT_SERVERS: [&str; 4] = ["DM", "sk", "nr", "wh"];
const CHAT_OPEN_SERVER: usize = 1;
const CHAT_CHANNELS: [&str; 4] = ["# general", "# wallpapers", "# skwd-wall", "# showcase"];
const CHAT_VOICE: &str = "lounge";
const CHAT_OPEN: usize = 1;
const CHAT_MENTION: &str = "@liixini";
const CHAT_LINK: &str = "skwd.dev/colour-studio";
const CHAT_PEOPLE: [(&str, &str); 3] = [("mira", "19:52"), ("tove", "20:03"), (CHAT_ME, "20:11")];

struct ChatSpec {
    font: f32,
    rail: f32,
    column: f32,
    server: f32,
    server_gap: f32,
    server_top: f32,
    server_font: f32,
    server_on_radius: f32,
    pill_left: f32,
    pill_inset: f32,
    pill: f32,
    pill_radius: f32,
    head: f32,
    head_pad: f32,
    list_pad: (f32, f32),
    list_gap: f32,
    cat_font: f32,
    cat_pad: (f32, f32, f32),
    channel_pad: (f32, f32),
    channel_radius: f32,
    me: f32,
    me_pad: f32,
    avatar: f32,
    me_gap: f32,
    small: f32,
    top_pad: f32,
    top_gap: f32,
    topic_font: f32,
    messages_pad: (f32, f32),
    message_gap: f32,
    face: f32,
    face_gap: f32,
    time_font: f32,
    time_gap: f32,
    mention_pad: f32,
    mention_radius: f32,
    input: f32,
    input_margin: f32,
    input_radius: f32,
    input_pad: f32,
    underline: f32,
    edge_mix: f32,
    strong_mix: f32,
    faint_mix: f32,
    column_mix: f32,
    selected_mix: f32,
    mention_mix: f32,
    editor_mix: f32,
}

const CHAT: ChatSpec = ChatSpec {
    font: 1.5,
    rail: 7.5,
    column: 25.0,
    server: 5.0,
    server_gap: 1.2,
    server_top: 1.4,
    server_font: 1.4,
    server_on_radius: 1.5,
    pill_left: 1.25,
    pill_inset: 0.8,
    pill: 0.5,
    pill_radius: 0.4,
    head: 6.0,
    head_pad: 1.6,
    list_pad: (1.2, 0.8),
    list_gap: 0.3,
    cat_font: 1.25,
    cat_pad: (0.8, 0.8, 0.2),
    channel_pad: (0.5, 0.9),
    channel_radius: 0.6,
    me: 7.0,
    me_pad: 1.2,
    avatar: 3.8,
    me_gap: 1.0,
    small: 1.2,
    top_pad: 1.8,
    top_gap: 1.2,
    topic_font: 1.3,
    messages_pad: (1.6, 1.8),
    message_gap: 1.6,
    face: 4.4,
    face_gap: 1.2,
    time_font: 1.15,
    time_gap: 0.8,
    mention_pad: 0.3,
    mention_radius: 0.4,
    input: 5.4,
    input_margin: 1.8,
    input_radius: 1.0,
    input_pad: 1.8,
    underline: 0.55,
    edge_mix: 0.6,
    strong_mix: 0.64,
    faint_mix: 0.48,
    column_mix: 0.86,
    selected_mix: 0.08,
    mention_mix: 0.12,
    editor_mix: 0.92,
};

fn chat(pen: &mut Pen<'_>, rect: Rectangle, tones: &Tones, radius: f32) {
    let spec = &CHAT;
    let space = Space::new(rect);
    let height = space.height();
    let line = space.hairline();
    let ov = mix(tones.muted, tones.bg, spec.edge_mix);
    let on64 = mix(tones.bg, tones.fg, spec.strong_mix);
    let on48 = mix(tones.bg, tones.fg, spec.faint_mix);
    let column_bg = mix(tones.bg, tones.raised, spec.column_mix);
    let selected = mix(tones.raised, tones.fg, spec.selected_mix);
    let tinted = mix(tones.bg, tones.primary, spec.mention_mix);
    let editor = mix(tones.bg, tones.surface, spec.editor_mix);
    pen.rounded(rect, radius, tones.bg);
    pen.fill_rect(space.at(spec.rail - line, 0.0, line, height), ov);
    for (index, label) in CHAT_SERVERS.iter().enumerate() {
        let top = spec.server_top + index as f32 * (spec.server + spec.server_gap);
        let centre = spec.rail / 2.0;
        let on = index == CHAT_OPEN_SERVER;
        let tile = space.at(centre - spec.server / 2.0, top, spec.server, spec.server);
        let radius = if on { spec.server_on_radius } else { spec.server / 2.0 };
        pen.rounded(tile, radius * space.unit, tones.raised);
        if on {
            let pill = space.at(
                centre - spec.server / 2.0 - spec.pill_left,
                top + spec.pill_inset,
                spec.pill,
                spec.server - 2.0 * spec.pill_inset,
            );
            pen.rounded(pill, spec.pill_radius * space.unit, tones.fg);
        }
        pen.words(
            space
                .words(label, centre, top + spec.server / 2.0, spec.server_font, tones.fg)
                .font(MEDIUM)
                .centered(),
        );
    }
    let column_left = spec.rail;
    pen.fill_rect(space.at(column_left, 0.0, spec.column, height), column_bg);
    pen.fill_rect(space.at(column_left, spec.head - line, spec.column, line), ov);
    pen.words(
        space
            .words(CHAT_SERVER, column_left + spec.head_pad, spec.head / 2.0, spec.font, tones.fg)
            .font(SEMIBOLD),
    );
    let mut top = spec.head + spec.list_pad.0;
    let inner_left = column_left + spec.list_pad.1;
    let inner_width = spec.column - 2.0 * spec.list_pad.1;
    let category = |pen: &mut Pen<'_>, label: &str, top: f32| {
        let height = spec.cat_pad.0 + spec.cat_font * LINE + spec.cat_pad.2;
        let middle = top + spec.cat_pad.0 + spec.cat_font * LINE / 2.0;
        pen.words(
            space
                .words(label, inner_left + spec.cat_pad.1, middle, spec.cat_font, on48)
                .font(MEDIUM),
        );
        height
    };
    let channel_height = 2.0 * spec.channel_pad.0 + spec.font * LINE;
    top += category(pen, tr("colour-studio-chat-text-channels"), top) + spec.list_gap;
    for (index, label) in CHAT_CHANNELS.iter().enumerate() {
        let on = index == CHAT_OPEN;
        if on {
            pen.rounded(
                space.at(inner_left, top, inner_width, channel_height),
                spec.channel_radius * space.unit,
                selected,
            );
        }
        let color = if on { tones.fg } else { on64 };
        pen.words(space.words(
            label,
            inner_left + spec.channel_pad.1,
            top + channel_height / 2.0,
            spec.font,
            color,
        ));
        top += channel_height + spec.list_gap;
    }
    top += category(pen, tr("colour-studio-chat-voice-channels"), top) + spec.list_gap;
    pen.words(space.words(
        CHAT_VOICE,
        inner_left + spec.channel_pad.1,
        top + channel_height / 2.0,
        spec.font,
        on64,
    ));
    let me_top = height - spec.me;
    pen.fill_rect(space.at(column_left, me_top, spec.column, spec.me), tones.bg);
    let avatar_centre =
        space.point(column_left + spec.me_pad + spec.avatar / 2.0, me_top + spec.me / 2.0);
    pen.disc(avatar_centre, spec.avatar / 2.0 * space.unit, tones.raised);
    let name_x = column_left + spec.me_pad + spec.avatar + spec.me_gap;
    let name_middle = me_top + spec.me / 2.0 - spec.small * LINE / 2.0;
    pen.words(space.words(CHAT_ME, name_x, name_middle, spec.font, tones.fg));
    let status = tr("colour-studio-chat-status");
    pen.words(space.words(
        status,
        name_x,
        name_middle + (spec.font + spec.small) * LINE / 2.0,
        spec.small,
        on64,
    ));
    let main_left = column_left + spec.column;
    pen.fill_rect(space.at(main_left, spec.head - line, UNIT - main_left, line), ov);
    let channel = CHAT_CHANNELS[CHAT_OPEN];
    let title_x = main_left + spec.top_pad;
    pen.words(space.words(channel, title_x, spec.head / 2.0, spec.font, tones.fg).font(SEMIBOLD));
    let topic_x = title_x + space.width(channel, spec.font, SEMIBOLD) + spec.top_gap;
    pen.words(space.words(
        tr("colour-studio-chat-topic"),
        topic_x,
        spec.head / 2.0,
        spec.topic_font,
        on64,
    ));
    let input_top = height - spec.input_margin - spec.input;
    let input = space.at(
        main_left + spec.input_margin,
        input_top,
        UNIT - main_left - 2.0 * spec.input_margin,
        spec.input,
    );
    pen.rounded(input, spec.input_radius * space.unit, editor);
    let prompt = tr("colour-studio-chat-input");
    pen.words(space.words(
        prompt,
        main_left + spec.input_margin + spec.input_pad,
        input_top + spec.input / 2.0,
        spec.font,
        on64,
    ));
    let messages_left = main_left + spec.messages_pad.1;
    let mut bottom = input_top - spec.messages_pad.0;
    let lines = [
        vec![(tr("colour-studio-chat-first").to_string(), Span::Plain)],
        vec![
            (CHAT_MENTION.to_string(), Span::Mention),
            (format!(" {}", tr("colour-studio-chat-second")), Span::Plain),
        ],
        vec![
            (format!("{} ", tr("colour-studio-chat-third")), Span::Plain),
            (CHAT_LINK.to_string(), Span::Link),
        ],
    ];
    for ((name, time), spans) in CHAT_PEOPLE.iter().zip(lines.iter()).rev() {
        let top = bottom - spec.face;
        pen.disc(
            space.point(messages_left + spec.face / 2.0, top + spec.face / 2.0),
            spec.face / 2.0 * space.unit,
            tones.raised,
        );
        let text_left = messages_left + spec.face + spec.face_gap;
        let first = top + spec.font * LINE / 2.0;
        pen.words(space.words(name, text_left, first, spec.font, tones.fg).font(SEMIBOLD));
        let time_x = text_left + space.width(name, spec.font, SEMIBOLD) + spec.time_gap;
        pen.words(space.words(time, time_x, first, spec.time_font, on48));
        let second = first + spec.font * LINE;
        let mut x = text_left;
        for (content, span) in spans {
            let width = space.width(content, spec.font, SANS);
            match span {
                Span::Mention => {
                    let chip = space.at(
                        x,
                        second - spec.font * LINE / 2.0,
                        width + 2.0 * spec.mention_pad,
                        spec.font * LINE,
                    );
                    pen.rounded(chip, spec.mention_radius * space.unit, tinted);
                    pen.words(space.words(
                        content,
                        x + spec.mention_pad,
                        second,
                        spec.font,
                        tones.primary,
                    ));
                    x += 2.0 * spec.mention_pad;
                }
                Span::Link => {
                    pen.words(space.words(content, x, second, spec.font, tones.primary));
                    let under = second + spec.font * spec.underline;
                    pen.fill_rect(space.at(x, under, width, line), tones.primary);
                }
                Span::Plain => pen.words(space.words(content, x, second, spec.font, tones.fg)),
            }
            x += width;
        }
        bottom = top - spec.message_gap;
    }
}

enum Span {
    Plain,
    Mention,
    Link,
}

struct TerminalSpec {
    left: f32,
    top: f32,
    size: f32,
    line: f32,
}

const TERMINAL: TerminalSpec = TerminalSpec { left: 3.0, top: 2.4, size: 2.6, line: 1.55 };
const PLAIN: usize = usize::MAX;
const TERMINAL_LINES: [&[(&str, usize, bool)]; 8] = [
    &[("~/wallpaper", 6, false), (" ", PLAIN, false), ("❯", 5, false), (" ls", PLAIN, false)],
    &[
        ("anime/", 4, true),
        ("  ", PLAIN, false),
        ("nature/", 4, true),
        ("  ", PLAIN, false),
        ("dusk.webp", 5, false),
        ("  ", PLAIN, false),
        ("build.sh", 2, false),
    ],
    &[
        ("~/wallpaper", 6, false),
        (" ", PLAIN, false),
        ("❯", 5, false),
        (" git status -s", PLAIN, false),
    ],
    &[
        (" M", 1, false),
        (" theme.rs   ", PLAIN, false),
        ("A ", 2, false),
        (" oklch.rs", PLAIN, false),
    ],
    &[
        ("~/wallpaper", 6, false),
        (" ", PLAIN, false),
        ("❯", 5, false),
        (" cargo test", PLAIN, false),
    ],
    &[("test oklch_roundtrip ... ", PLAIN, false), ("ok", 2, false)],
    &[("test gamut_clip ... ", PLAIN, false), ("FAILED", 1, true)],
    &[("warning", 3, false), (": unused variable ", PLAIN, false), ("`seed`", 11, false)],
];

fn terminal(
    pen: &mut Pen<'_>,
    rect: Rectangle,
    palette: &StudioPalette,
    colours: bool,
    radius: f32,
) {
    let spec = &TERMINAL;
    let space = Space::new(rect);
    let tones = Tones::of(palette);
    pen.rounded(rect, radius, tones.bg);
    let step = spec.size * spec.line;
    let row = |index: usize| spec.top + index as f32 * step + step / 2.0;
    for (index, spans) in TERMINAL_LINES.iter().enumerate() {
        let mut x = spec.left;
        for (content, slot, bold) in *spans {
            let font = if *bold { MONO_BOLD } else { MONO };
            let color =
                if colours && *slot != PLAIN { rgb_color(palette.ansi[*slot]) } else { tones.fg };
            pen.words(
                space.words(content, x, row(index), spec.size, color).font(font).line(spec.line),
            );
            x += space.width(content, spec.size, font);
        }
    }
    if !colours {
        let note = tr("colour-studio-terminal-none");
        let italic = Font { style: iced::font::Style::Italic, ..MONO };
        pen.words(
            space
                .words(note, spec.left, row(TERMINAL_LINES.len()), spec.size, tones.muted)
                .font(italic)
                .line(spec.line),
        );
    }
}

pub fn app(
    pen: &mut Pen<'_>,
    which: Desk,
    rect: Rectangle,
    palette: &StudioPalette,
    colours: bool,
    radius: f32,
) {
    let tones = Tones::of(palette);
    match which {
        Desk::Browser => browser(pen, rect, &tones, radius),
        Desk::Chat => chat(pen, rect, &tones, radius),
        Desk::Terminal => terminal(pen, rect, palette, colours, radius),
    }
}
