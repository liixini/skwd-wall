use super::syntax::{self, Syntax};
use iced::{Color, Font, Point, Rectangle};
use skwd_palette::studio::StudioPalette;

use super::mock::{LINE, MEDIUM, MONO, SANS, SEMIBOLD, Space, Tones, UNIT, mix};
use super::pen::Pen;
use crate::i18n::tr;

struct MusicSpec {
    font: f32,
    pad: f32,
    side: f32,
    player: f32,
    radius: f32,
    side_pad: (f32, f32),
    side_gap: f32,
    heading: f32,
    heading_gap: f32,
    art: f32,
    art_radius: f32,
    art_gap: f32,
    small: f32,
    main_pad: (f32, f32),
    main_gap: f32,
    greeting: f32,
    card_gap: f32,
    card_radius: f32,
    card_art: f32,
    card_text_gap: f32,
    row_gap: f32,
    row_pad: (f32, f32),
    row_radius: f32,
    row_number: f32,
    row_time: f32,
    row_column_gap: f32,
    player_pad: f32,
    player_gap: f32,
    now_width: f32,
    volume_width: f32,
    now_art: f32,
    now_gap: f32,
    controls_gap: f32,
    button_gap: f32,
    play: f32,
    play_font: f32,
    bar_font: f32,
    bar_label: f32,
    bar_gap: f32,
    track: f32,
    track_radius: f32,
    played: f32,
    volume: f32,
    soft_mix: f32,
    track_mix: f32,
    playing_mix: f32,
}

const MUSIC: MusicSpec = MusicSpec {
    font: 1.4,
    pad: 0.8,
    side: 24.0,
    player: 9.0,
    radius: 1.0,
    side_pad: (1.6, 1.4),
    side_gap: 1.1,
    heading: 1.5,
    heading_gap: 0.4,
    art: 4.4,
    art_radius: 0.5,
    art_gap: 1.0,
    small: 1.15,
    main_pad: (2.0, 2.2),
    main_gap: 1.6,
    greeting: 2.6,
    card_gap: 1.2,
    card_radius: 0.8,
    card_art: 5.0,
    card_text_gap: 1.0,
    row_gap: 0.2,
    row_pad: (0.6, 0.8),
    row_radius: 0.6,
    row_number: 2.4,
    row_time: 5.0,
    row_column_gap: 1.0,
    player_pad: 1.6,
    player_gap: 2.0,
    now_width: 26.0,
    volume_width: 16.0,
    now_art: 5.4,
    now_gap: 1.2,
    controls_gap: 0.8,
    button_gap: 2.6,
    play: 4.2,
    play_font: 1.6,
    bar_font: 1.1,
    bar_label: 4.0,
    bar_gap: 1.0,
    track: 0.5,
    track_radius: 0.3,
    played: 0.42,
    volume: 0.7,
    soft_mix: 0.3,
    track_mix: 0.08,
    playing_mix: 0.18,
};

const LIBRARY: [(&str, &str, Art); 4] = [
    ("colour-studio-music-liked", "colour-studio-music-songs", Art::AccentSecond),
    ("colour-studio-music-commits", "colour-studio-music-playlist", Art::ThirdRaised),
    ("colour-studio-music-rain", "colour-studio-music-playlist", Art::SecondSurface),
    ("colour-studio-music-dusk", "colour-studio-music-for-you", Art::RaisedAccent),
];
const CARDS: [(&str, Art); 3] = [
    ("colour-studio-music-liked", Art::AccentSecond),
    ("colour-studio-music-commits", Art::ThirdSurface),
    ("colour-studio-music-rain", Art::SecondRaised),
];
const SONGS: [(&str, &str, &str); 4] = [
    ("1", "Low tide", "3:41"),
    ("2", "Red ribbon", "4:12"),
    ("3", "Static bloom", "2:58"),
    ("4", "Night bus", "5:03"),
];
const PLAYING: usize = 1;
const NOW_TITLE: &str = "Red ribbon";
const NOW_ARTIST: &str = "The Quiet Hours";
const BUTTONS: [&str; 5] = ["⇄", "⏮\u{fe0e}", "❚❚", "⏭\u{fe0e}", "↻"];
const PLAY_SLOT: usize = 2;
const ELAPSED: &str = "1:44";
const LENGTH: &str = "4:12";
const SPEAKER: &str = "🔈\u{fe0e}";

#[derive(Clone, Copy)]
enum Art {
    AccentSecond,
    ThirdRaised,
    SecondSurface,
    RaisedAccent,
    ThirdSurface,
    SecondRaised,
}

impl Art {
    fn colors(self, tones: &Tones) -> (Color, Color) {
        match self {
            Self::AccentSecond => (tones.primary, tones.second),
            Self::ThirdRaised => (tones.third, tones.raised),
            Self::SecondSurface => (tones.second, tones.surface),
            Self::RaisedAccent => (tones.raised, tones.primary),
            Self::ThirdSurface => (tones.third, tones.surface),
            Self::SecondRaised => (tones.second, tones.raised),
        }
    }
}

fn art(pen: &mut Pen<'_>, rect: Rectangle, radius: f32, colors: (Color, Color)) {
    pen.rounded_gradient(
        rect,
        radius,
        rect.position(),
        Point::new(rect.x + rect.width, rect.y + rect.height),
        &[(0.0, colors.0), (1.0, colors.1)],
    );
}

pub fn music(pen: &mut Pen<'_>, rect: Rectangle, palette: &StudioPalette) {
    let spec = &MUSIC;
    let tones = Tones::of(palette);
    let space = Space::new(rect);
    let height = space.height();
    let low = mix(tones.bg, tones.surface, 0.5);
    let soft = mix(tones.fg, tones.bg, spec.soft_mix);
    let track_bg = mix(tones.raised, tones.fg, spec.track_mix);
    let tinted = mix(tones.bg, tones.primary, spec.playing_mix);
    let unit = space.unit;
    pen.fill_rect(rect, tones.bg);
    let body = height - 2.0 * spec.pad - spec.player - spec.pad;
    pen.rounded(space.at(spec.pad, spec.pad, spec.side, body), spec.radius * unit, tones.surface);
    let left = spec.pad + spec.side_pad.1;
    let mut top = spec.pad + spec.side_pad.0;
    pen.words(
        space
            .words(
                tr("colour-studio-music-library"),
                left,
                top + spec.heading * LINE / 2.0,
                spec.heading,
                tones.fg,
            )
            .font(SEMIBOLD),
    );
    top += spec.heading * LINE + spec.heading_gap + spec.side_gap;
    for (title, detail, colors) in LIBRARY {
        art(
            pen,
            space.at(left, top, spec.art, spec.art),
            spec.art_radius * unit,
            colors.colors(&tones),
        );
        let text_left = left + spec.art + spec.art_gap;
        let block = spec.font * LINE + spec.small * LINE;
        let first = top + (spec.art - block) / 2.0 + spec.font * LINE / 2.0;
        pen.words(space.words(tr(title), text_left, first, spec.font, tones.fg));
        pen.words(space.words(
            tr(detail),
            text_left,
            first + (spec.font + spec.small) * LINE / 2.0,
            spec.small,
            soft,
        ));
        top += spec.art + spec.side_gap;
    }
    let main_left = spec.pad + spec.side + spec.pad;
    let main_width = UNIT - main_left - spec.pad;
    pen.rounded(space.at(main_left, spec.pad, main_width, body), spec.radius * unit, tones.bg);
    let inner = main_left + spec.main_pad.1;
    let inner_width = main_width - 2.0 * spec.main_pad.1;
    let mut top = spec.pad + spec.main_pad.0;
    pen.words(
        space
            .words(
                tr("colour-studio-music-greeting"),
                inner,
                top + spec.greeting * LINE / 2.0,
                spec.greeting,
                tones.fg,
            )
            .font(SEMIBOLD),
    );
    top += spec.greeting * LINE + spec.main_gap;
    let card_width = (inner_width - 2.0 * spec.card_gap) / CARDS.len() as f32;
    for (index, (title, colors)) in CARDS.iter().enumerate() {
        let x = inner + index as f32 * (card_width + spec.card_gap);
        let card = space.at(x, top, card_width, spec.card_art);
        pen.rounded(card, spec.card_radius * unit, tones.raised);
        pen.clipped(card, |pen| {
            art(pen, space.at(x, top, spec.card_art, spec.card_art), 0.0, colors.colors(&tones));
            let label_x = x + spec.card_art + spec.card_text_gap;
            pen.words(
                space
                    .words(tr(title), label_x, top + spec.card_art / 2.0, spec.font, tones.fg)
                    .font(MEDIUM),
            );
        });
    }
    top += spec.card_art + spec.main_gap;
    let row_height = 2.0 * spec.row_pad.0 + spec.font * LINE;
    for (index, (number, title, time)) in SONGS.iter().enumerate() {
        let on = index == PLAYING;
        let row = space.at(inner, top, inner_width, row_height);
        if on {
            pen.rounded(row, spec.row_radius * unit, tinted);
        }
        let middle = top + row_height / 2.0;
        let start = inner + spec.row_pad.1;
        pen.words(space.words(number, start, middle, spec.font, soft));
        let title_x = start + spec.row_number + spec.row_column_gap;
        let (font, color) = if on { (MEDIUM, tones.primary) } else { (SANS, tones.fg) };
        pen.words(space.words(title, title_x, middle, spec.font, color).font(font));
        let time_x = inner + inner_width - spec.row_pad.1 - spec.row_time;
        pen.words(space.words(time, time_x, middle, spec.font, soft));
        top += row_height + spec.row_gap;
    }
    let player_top = height - spec.pad - spec.player;
    let player_width = UNIT - 2.0 * spec.pad;
    pen.rounded(space.at(spec.pad, player_top, player_width, spec.player), spec.radius * unit, low);
    let middle = player_top + spec.player / 2.0;
    let now_left = spec.pad + spec.player_pad;
    art(
        pen,
        space.at(now_left, middle - spec.now_art / 2.0, spec.now_art, spec.now_art),
        spec.art_radius * unit,
        (tones.primary, tones.second),
    );
    let text_left = now_left + spec.now_art + spec.now_gap;
    let block = spec.font * LINE + spec.small * LINE;
    let first = middle - block / 2.0 + spec.font * LINE / 2.0;
    pen.words(space.words(NOW_TITLE, text_left, first, spec.font, tones.fg));
    pen.words(space.words(
        NOW_ARTIST,
        text_left,
        first + (spec.font + spec.small) * LINE / 2.0,
        spec.small,
        soft,
    ));
    let controls_left = now_left + spec.now_width + spec.player_gap;
    let controls_right =
        spec.pad + player_width - spec.player_pad - spec.volume_width - spec.player_gap;
    let controls_middle = f32::midpoint(controls_left, controls_right);
    let controls_height = spec.play + spec.controls_gap + spec.bar_font * LINE;
    let buttons_middle = middle - controls_height / 2.0 + spec.play / 2.0;
    let widths: Vec<f32> = BUTTONS
        .iter()
        .enumerate()
        .map(
            |(index, label)| {
                if index == PLAY_SLOT { spec.play } else { space.width(label, spec.font, SANS) }
            },
        )
        .collect();
    let total: f32 = widths.iter().sum::<f32>() + spec.button_gap * (BUTTONS.len() - 1) as f32;
    let mut x = controls_middle - total / 2.0;
    for (index, (label, width)) in BUTTONS.iter().zip(&widths).enumerate() {
        let centre = x + width / 2.0;
        if index == PLAY_SLOT {
            pen.disc(space.point(centre, buttons_middle), spec.play / 2.0 * unit, tones.third);
            pen.words(
                space.words(label, centre, buttons_middle, spec.play_font, tones.bg).centered(),
            );
        } else {
            pen.words(space.words(label, centre, buttons_middle, spec.font, soft).centered());
        }
        x += width + spec.button_gap;
    }
    let bar_middle =
        buttons_middle + spec.play / 2.0 + spec.controls_gap + spec.bar_font * LINE / 2.0;
    pen.words(space.words(ELAPSED, controls_left, bar_middle, spec.bar_font, soft));
    let track_left = controls_left + spec.bar_label + spec.bar_gap;
    let track_right = controls_right - spec.bar_label - spec.bar_gap;
    let track =
        space.at(track_left, bar_middle - spec.track / 2.0, track_right - track_left, spec.track);
    pen.rounded(track, spec.track_radius * unit, track_bg);
    let played = Rectangle { width: track.width * spec.played, ..track };
    pen.rounded(played, spec.track_radius * unit, tones.third);
    pen.words(space.words(
        LENGTH,
        controls_right - spec.bar_label,
        bar_middle,
        spec.bar_font,
        soft,
    ));
    let volume_left = spec.pad + player_width - spec.player_pad - spec.volume_width;
    pen.words(space.words(SPEAKER, volume_left, middle, spec.font, soft));
    let speaker = space.width(SPEAKER, spec.font, SANS);
    let level_left = volume_left + speaker + spec.bar_gap;
    let level = space.at(
        level_left,
        middle - spec.track / 2.0,
        volume_left + spec.volume_width - level_left,
        spec.track,
    );
    pen.rounded(level, spec.track_radius * unit, track_bg);
    pen.rounded(
        Rectangle { width: level.width * spec.volume, ..level },
        spec.track_radius * unit,
        tones.fg,
    );
}

const CODE: [&str; 9] = [
    "// accent from the most salient cluster",
    "pub fn derive(dominant: &[Rgb], dark: bool) -> ThemePalette {",
    "    let (hue, saturation) = pick_accent(dominant);",
    "    let tertiary_hue = (hue + 60.0) % 360.0;",
    "    let tint = (saturation * 0.4).clamp(0.08, 0.22);",
    "    if dark {",
    "        log(\"dark palette\");",
    "    }",
    "}",
];
const FIRST_LINE: usize = 28;
const SELECTED: &str = "(hue, saturation)";
const CURRENT_LINE: usize = 2;
const TITLE: &str = "theme.rs · skwd-palette";
const PROJECT: &str = "skwd-palette";
const FILES: [(&str, bool); 7] = [
    ("▾ src", true),
    ("color.rs", false),
    ("quantize.rs", false),
    ("theme.rs", false),
    ("presets.rs", false),
    ("▸ semantic", true),
    ("▸ seed", true),
];
const OPEN_FILE: usize = 3;
const TABS: [&str; 2] = ["theme.rs", "presets.rs"];
const BRANCH: &str = "⌁ main";
const WARNINGS: &str = "⚠\u{fe0e} 1";
const LANGUAGE: &str = "Rust";
const BADGE: &str = "3";
const ACTIVITY: usize = 4;

struct CodeSpec {
    font: f32,
    title: f32,
    title_font: f32,
    status: f32,
    activity: f32,
    side: f32,
    icon: f32,
    icon_radius: f32,
    icon_ring: f32,
    icon_gap: f32,
    icon_top: f32,
    marker: f32,
    marker_left: f32,
    marker_reach: f32,
    badge: f32,
    badge_font: f32,
    badge_lift: f32,
    badge_shift: f32,
    head_pad: (f32, f32),
    head_font: f32,
    section_pad: (f32, f32),
    section_font: f32,
    file_pad: (f32, f32, f32),
    tabs: f32,
    tab_pad: f32,
    tab_line: f32,
    code_font: f32,
    code_line: f32,
    code_top: f32,
    gutter: f32,
    gutter_pad: f32,
    status_font: f32,
    status_gap: f32,
    remote_pad: f32,
    soft_mix: f32,
    container_mix: f32,
}

const CODE_SPEC: CodeSpec = CodeSpec {
    font: 1.3,
    title: 3.6,
    title_font: 1.2,
    status: 2.8,
    activity: 4.6,
    side: 21.0,
    icon: 2.2,
    icon_radius: 0.4,
    icon_ring: 0.22,
    icon_gap: 1.6,
    icon_top: 1.4,
    marker: 0.25,
    marker_left: 1.2,
    marker_reach: 0.3,
    badge: 1.6,
    badge_font: 1.0,
    badge_lift: 1.2,
    badge_shift: 1.6,
    head_pad: (1.0, 1.4),
    head_font: 1.1,
    section_pad: (0.5, 1.4),
    section_font: 1.15,
    file_pad: (0.35, 1.4, 2.6),
    tabs: 3.4,
    tab_pad: 1.6,
    tab_line: 0.22,
    code_font: 1.28,
    code_line: 1.75,
    code_top: 0.8,
    gutter: 4.8,
    gutter_pad: 1.6,
    status_font: 1.15,
    status_gap: 1.6,
    remote_pad: 1.4,
    soft_mix: 0.3,
    container_mix: 0.68,
};

struct CodeTones {
    low: Color,
    soft: Color,
    container: Color,
    on_container: Color,
}

fn ink(tones: &Tones, code: &CodeTones, kind: Syntax) -> Color {
    match kind {
        Syntax::Plain => tones.fg,
        Syntax::Keyword => tones.second,
        Syntax::Function | Syntax::Text => tones.primary,
        Syntax::Number | Syntax::Kind => tones.third,
        Syntax::Comment => tones.muted,
        Syntax::Punctuation => code.soft,
        Syntax::Selection => code.on_container,
    }
}

pub fn code(pen: &mut Pen<'_>, rect: Rectangle, palette: &StudioPalette) {
    let spec = &CODE_SPEC;
    let tones = Tones::of(palette);
    let space = Space::new(rect);
    let height = space.height();
    let unit = space.unit;
    let line = space.hairline();
    let shades = CodeTones {
        low: mix(tones.bg, tones.surface, 0.5),
        soft: mix(tones.fg, tones.bg, spec.soft_mix),
        container: mix(tones.primary, tones.bg, spec.container_mix),
        on_container: mix(tones.primary, tones.fg, 0.5),
    };
    pen.fill_rect(rect, tones.bg);
    pen.fill_rect(space.at(0.0, 0.0, UNIT, spec.title), shades.low);
    pen.fill_rect(space.at(0.0, spec.title - line, UNIT, line), tones.raised);
    pen.words(
        space.words(TITLE, UNIT / 2.0, spec.title / 2.0, spec.title_font, tones.fg).centered(),
    );
    let body_top = spec.title;
    let body_height = height - spec.title - spec.status;
    pen.fill_rect(space.at(0.0, body_top, spec.activity, body_height), shades.low);
    pen.fill_rect(space.at(spec.activity - line, body_top, line, body_height), tones.raised);
    let mut top = body_top + spec.icon_top;
    let centre = spec.activity / 2.0;
    for slot in 0..=ACTIVITY {
        if slot == 2 {
            let badge_top = top - spec.badge_lift;
            let badge_centre = space
                .point(centre - spec.badge / 2.0 + spec.badge_shift, badge_top + spec.badge / 2.0);
            pen.disc(badge_centre, spec.badge / 2.0 * unit, tones.primary);
            pen.words(
                space
                    .words(
                        BADGE,
                        centre - spec.badge / 2.0 + spec.badge_shift,
                        badge_top + spec.badge / 2.0,
                        spec.badge_font,
                        tones.on_primary,
                    )
                    .centered(),
            );
            top = badge_top + spec.badge + spec.icon_gap;
            continue;
        }
        let icon = space.at(centre - spec.icon / 2.0, top, spec.icon, spec.icon);
        let on = slot == 0;
        pen.ring(
            icon,
            spec.icon_radius * unit,
            spec.icon_ring * unit,
            if on { tones.fg } else { tones.muted },
        );
        if on {
            let marker = space.at(
                centre - spec.icon / 2.0 - spec.marker_left,
                top - spec.marker_reach,
                spec.marker,
                spec.icon + 2.0 * spec.marker_reach,
            );
            pen.fill_rect(marker, tones.primary);
        }
        top += spec.icon + spec.icon_gap;
    }
    let side_left = spec.activity;
    pen.fill_rect(space.at(side_left, body_top, spec.side, body_height), shades.low);
    pen.fill_rect(
        space.at(side_left + spec.side - line, body_top, line, body_height),
        tones.raised,
    );
    let mut top = body_top;
    let head_height = 2.0 * spec.head_pad.0 + spec.head_font * LINE;
    pen.words(
        space
            .words(
                tr("colour-studio-code-explorer"),
                side_left + spec.head_pad.1,
                top + head_height / 2.0,
                spec.head_font,
                tones.fg,
            )
            .font(SEMIBOLD),
    );
    top += head_height;
    let section_height = 2.0 * spec.section_pad.0 + spec.section_font * LINE;
    pen.fill_rect(space.at(side_left, top, spec.side - line, section_height), tones.surface);
    pen.words(
        space
            .words(
                PROJECT,
                side_left + spec.section_pad.1,
                top + section_height / 2.0,
                spec.section_font,
                tones.fg,
            )
            .font(SEMIBOLD),
    );
    top += section_height;
    let file_height = 2.0 * spec.file_pad.0 + spec.font * LINE;
    for (index, (name, folder)) in FILES.iter().enumerate() {
        if index == OPEN_FILE {
            pen.fill_rect(space.at(side_left, top, spec.side - line, file_height), tones.raised);
        }
        let left = side_left + if *folder { spec.file_pad.1 } else { spec.file_pad.2 };
        let color = if *folder { shades.soft } else { tones.fg };
        pen.words(space.words(name, left, top + file_height / 2.0, spec.font, color));
        top += file_height;
    }
    let editor_left = side_left + spec.side;
    let editor_width = UNIT - editor_left;
    pen.fill_rect(space.at(editor_left, body_top, editor_width, spec.tabs), shades.low);
    pen.fill_rect(
        space.at(editor_left, body_top + spec.tabs - line, editor_width, line),
        tones.raised,
    );
    let mut x = editor_left;
    for (index, tab) in TABS.iter().enumerate() {
        let width = space.width(tab, spec.font, SANS) + 2.0 * spec.tab_pad;
        let on = index == 0;
        if on {
            pen.fill_rect(space.at(x, body_top, width, spec.tabs), tones.bg);
            pen.fill_rect(
                space.at(x, body_top + spec.tabs - spec.tab_line, width, spec.tab_line),
                tones.primary,
            );
        }
        pen.fill_rect(space.at(x + width - line, body_top, line, spec.tabs), tones.raised);
        let color = if on { tones.fg } else { tones.muted };
        pen.words(space.words(tab, x + spec.tab_pad, body_top + spec.tabs / 2.0, spec.font, color));
        x += width;
    }
    let step = spec.code_font * spec.code_line;
    let mut top = body_top + spec.tabs + spec.code_top;
    for (index, line) in CODE.iter().enumerate() {
        let current = index == CURRENT_LINE;
        let number = (FIRST_LINE + index).to_string();
        let spans = syntax::marked(line, current.then_some(SELECTED));
        if current {
            pen.fill_rect(space.at(editor_left, top, editor_width, step), tones.surface);
        }
        let middle = top + step / 2.0;
        let number_right = editor_left + spec.gutter - spec.gutter_pad;
        let color = if current { tones.primary } else { tones.muted };
        pen.words(
            space.words(&number, number_right, middle, spec.code_font, color).font(MONO).right(),
        );
        let mut x = editor_left + spec.gutter;
        for (content, kind) in spans {
            let font: Font = if matches!(kind, Syntax::Comment) {
                Font { style: iced::font::Style::Italic, ..MONO }
            } else {
                MONO
            };
            let width = space.width(content, spec.code_font, font);
            if matches!(kind, Syntax::Selection) {
                pen.fill_rect(space.at(x, top, width, step), shades.container);
            }
            pen.words(
                space
                    .words(content, x, middle, spec.code_font, ink(&tones, &shades, kind))
                    .font(font)
                    .line(spec.code_line),
            );
            x += width;
        }
        top += step;
    }
    let status_top = height - spec.status;
    pen.fill_rect(space.at(0.0, status_top, UNIT, spec.status), tones.surface);
    pen.fill_rect(space.at(0.0, status_top, UNIT, line), tones.raised);
    let middle = status_top + spec.status / 2.0;
    let remote_width = space.width(BRANCH, spec.status_font, SANS) + 2.0 * spec.remote_pad;
    pen.fill_rect(space.at(0.0, status_top, remote_width, spec.status), tones.primary);
    pen.words(space.words(BRANCH, spec.remote_pad, middle, spec.status_font, tones.on_primary));
    let mut x = remote_width + spec.status_gap;
    let position = tr("colour-studio-code-position");
    for (content, color) in
        [(position, shades.soft), (WARNINGS, tones.third), (LANGUAGE, shades.soft)]
    {
        pen.words(space.words(content, x, middle, spec.status_font, color));
        x += space.width(content, spec.status_font, SANS) + spec.status_gap;
    }
}
