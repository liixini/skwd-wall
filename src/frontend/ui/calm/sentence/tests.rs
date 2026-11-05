use iced::advanced::graphics::text::Paragraph;
use iced::advanced::graphics::text::cosmic_text::LayoutGlyph;
use iced::advanced::text::{self, Paragraph as _, Span};
use iced::advanced::{Shell, layout};
use iced::keyboard::{self, Key, key::Named};
use iced::{Color, Event, Font, Pixels, Point, Rectangle, Size, Vector, alignment, mouse};

use super::super::render_tests::{STRIP_PADDING, STRIP_WIDTHS, board, locale, say_on};
use super::super::sentinel;
use super::super::tint::tint;
use super::{
    ADD_ALPHA, ADD_RULE, ASIDE_ALPHA, Inks, KeyLook, LINK_RULE, MUTED_ALPHA, MUTED_LINE_ALPHA,
    PLAIN_ALPHA, Piece, Popover, SAY_LEADING, SAY_SIZE, SELECTED_TINT, SOFT_ALPHA, Scrub, Sentence,
    SentenceSpec, Side, State, Stroke, Token, UNBOUND_ALPHA, UNBOUND_RULE, VALUE_CLOSE, VALUE_OPEN,
    dashes, key_width, shown, split, text_alpha, token_lines, unbroken,
};
use crate::frontend::ui::{UI_FONT, cached_line_width, legible_type_scale};
use crate::i18n::Script;

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Open(u64),
    Set(f64),
    Commit,
}

fn dial(scrub: Scrub) -> Token<'static, Msg> {
    Token::Number { scrub, change: Box::new(Msg::Set), commit: Msg::Commit }
}

const FPS: Scrub = Scrub { value: 30.0, min: 0.0, max: 240.0, step: 1.0 };
const RATE: u64 = 7;
const MODE: u64 = 9;

fn battery(focused: Option<u64>) -> Sentence<'static, Msg> {
    let palette = board();
    Sentence {
        spec: SentenceSpec {
            pieces: vec![
                Piece::plain("The picker caps at "),
                Piece::token(RATE, "30 fps", dial(FPS), Msg::Open(RATE)),
                Piece::plain(" in "),
                Piece::token(MODE, "Balanced", Token::Menu, Msg::Open(MODE)),
                Piece::plain(" mode."),
            ],
            size: SAY_SIZE,
            leading: SAY_LEADING,
            scale: 1.0,
            palette,
            plane: palette.surface,
            focused,
            ring: false,
            popover: None,
            centred: false,
        },
    }
}

fn run<T>(body: impl FnOnce(&mut Shell<'_, Msg>) -> T) -> (T, Vec<Msg>, bool) {
    let mut messages = Vec::new();
    let mut shell = Shell::new(&mut messages);
    let out = body(&mut shell);
    let captured = shell.is_event_captured();
    (out, messages, captured)
}

fn limits(width: f32) -> layout::Limits {
    layout::Limits::new(Size::ZERO, Size::new(width, f32::INFINITY))
}

fn press_key(named: Named) -> Event {
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: Key::Named(named),
        modified_key: Key::Named(named),
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::empty(),
        text: None,
        repeat: false,
    })
}

#[test]
fn nudge_rounds_and_clamps() {
    let half = Scrub { value: 1.2, min: 0.0, max: 2.0, step: 0.5 };
    let flat = Scrub { step: 0.0, ..FPS };
    for (scrub, steps, expected) in [
        (FPS, 1.0, 31.0),
        (FPS, -40.0, 0.0),
        (FPS, 400.0, 240.0),
        (half, 0.0, 1.0),
        (half, 3.0, 2.0),
        (flat, 3.0, 30.0),
    ] {
        assert_eq!(scrub.nudge(steps), expected, "{scrub:?} by {steps}");
    }
}

#[test]
fn glyphs_match_design() {
    let kinds: [(Token<'_, Msg>, &str, f32); 4] = [
        (Token::Menu, "▾", 8.0),
        (dial(FPS), "↕", 8.0),
        (Token::Text, "✎", 9.0),
        (Token::Key(KeyLook::Bound), "⌨", 9.0),
    ];
    for (kind, glyph, size) in kinds {
        assert_eq!(kind.glyph(), (glyph, size));
    }
}

#[test]
fn drag_waits_for_slop() {
    let sentence = battery(None);
    let mut state = State::default();
    run(|shell| sentence.press(&mut state, Some(RATE), 100.0, shell));
    let ((), messages, _) = run(|shell| sentence.drag(&mut state, 98.0, shell));
    assert!(messages.is_empty());
    let ((), messages, captured) = run(|shell| sentence.drag(&mut state, 92.0, shell));
    assert_eq!(messages, [Msg::Set(32.0)]);
    assert!(captured);
    let ((), messages, _) = run(|shell| sentence.drag(&mut state, 104.0, shell));
    assert_eq!(messages, [Msg::Set(29.0)]);
    let ((), messages, _) = run(|shell| sentence.release(&mut state, Some(RATE), shell));
    assert_eq!(messages, [Msg::Commit], "drag commits once");
}

#[test]
fn drag_is_absolute() {
    let sentence = battery(None);
    let mut state = State::default();
    run(|shell| sentence.press(&mut state, Some(RATE), 100.0, shell));
    let ((), first, _) = run(|shell| sentence.drag(&mut state, 80.0, shell));
    let ((), second, _) = run(|shell| sentence.drag(&mut state, 80.0, shell));
    assert_eq!(first, second, "same pointer, same value");
    let ((), high, _) = run(|shell| sentence.drag(&mut state, -10_000.0, shell));
    assert_eq!(high, [Msg::Set(240.0)], "clamped");
}

#[test]
fn scrolled_drag_uses_cursor() {
    let sentence = battery(None);
    let mut state = State::default();
    let node = sentence.shape(&mut state, &limits(600.0));
    let line = state.lines(0)[0];
    let bounds = Rectangle::new(Point::new(100.0, 50.0), node.size());
    let token = Point::new(bounds.x + line.left + 2.0, bounds.y + line.baseline - 2.0);
    let scroll = Vector::new(0.0, 500.0);
    let at = |point: Point| mouse::Cursor::Available(point);
    let down = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
    run(|shell| sentence.handle(&mut state, &down, bounds, at(token), shell));
    let lifted = token - Vector::new(0.0, 8.0);
    let moved = Event::Mouse(mouse::Event::CursorMoved { position: lifted + scroll });
    let ((), messages, _) =
        run(|shell| sentence.handle(&mut state, &moved, bounds, at(lifted), shell));
    assert_eq!(messages, [Msg::Set(32.0)]);
}

#[test]
fn click_presses_token() {
    let sentence = battery(None);
    let mut state = State::default();
    let ((), _, captured) = run(|shell| sentence.press(&mut state, Some(MODE), 40.0, shell));
    assert!(captured);
    let ((), messages, _) = run(|shell| sentence.release(&mut state, Some(MODE), shell));
    assert_eq!(messages, [Msg::Open(MODE)]);
    run(|shell| sentence.press(&mut state, Some(MODE), 40.0, shell));
    let ((), messages, _) = run(|shell| sentence.release(&mut state, None, shell));
    assert!(messages.is_empty(), "released elsewhere");
}

#[test]
fn wheel_needs_focus() {
    let up = mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 };
    let ((), messages, _) = run(|shell| battery(None).wheel(Some(RATE), up, shell));
    assert!(messages.is_empty());
    let focused = battery(Some(RATE));
    let ((), messages, _) = run(|shell| focused.wheel(Some(RATE), up, shell));
    assert_eq!(messages, [Msg::Set(31.0), Msg::Commit]);
    let ((), messages, _) = run(|shell| focused.wheel(None, up, shell));
    assert!(messages.is_empty(), "wheel off the token");
}

#[test]
fn keys_follow_focus() {
    let sentence = battery(Some(RATE));
    let ((), messages, captured) = run(|shell| sentence.key(&Key::Named(Named::Enter), shell));
    assert_eq!(messages, [Msg::Open(RATE)]);
    assert!(captured);
    for named in [Named::ArrowUp, Named::ArrowDown, Named::ArrowLeft, Named::Space, Named::Tab] {
        let ((), messages, captured) = run(|shell| sentence.key(&Key::Named(named), shell));
        assert!(messages.is_empty() && !captured, "{named:?}");
    }
    let menu = battery(Some(MODE));
    let ((), messages, _) = run(|shell| menu.key(&Key::Named(Named::Space), shell));
    assert_eq!(messages, [Msg::Open(MODE)]);
    let ((), messages, _) = run(|shell| battery(None).key(&Key::Named(Named::Enter), shell));
    assert!(messages.is_empty(), "nothing focused");
}

#[test]
fn engaged_number_steps() {
    let mut engaged = battery(None);
    engaged.spec.popover = Some(super::Popover {
        key: RATE,
        side: super::Side::Over,
        content: iced::widget::Space::new().into(),
    });
    let ((), messages, captured) = run(|shell| engaged.key(&Key::Named(Named::ArrowUp), shell));
    assert_eq!(messages, [Msg::Set(31.0)]);
    assert!(captured);
    let ((), messages, _) = run(|shell| engaged.key(&Key::Named(Named::ArrowDown), shell));
    assert_eq!(messages, [Msg::Set(29.0)]);
    let ((), messages, captured) = run(|shell| engaged.key(&Key::Named(Named::Enter), shell));
    assert!(messages.is_empty() && !captured, "editor submits");
}

#[test]
fn keys_yield_when_busy() {
    let mut open = battery(Some(MODE));
    open.spec.popover = Some(super::Popover {
        key: MODE,
        side: super::Side::Below,
        content: iced::widget::Space::new().into(),
    });
    for named in [Named::Enter, Named::ArrowDown] {
        let ((), messages, _) = run(|shell| open.key(&Key::Named(named), shell));
        assert!(messages.is_empty(), "menu open");
    }
    let sentence = battery(Some(RATE));
    let mut state = State::default();
    let ((), messages, _) = run(|shell| {
        shell.capture_event();
        sentence.handle(
            &mut state,
            &press_key(Named::ArrowUp),
            Rectangle::default(),
            mouse::Cursor::Unavailable,
            shell,
        );
    });
    assert!(messages.is_empty(), "already captured");
}

#[test]
fn split_follows_locale() {
    let tokens = || -> Vec<Vec<Piece<'static, Msg>>> {
        vec![
            vec![Piece::token(RATE, "30", Token::Text, Msg::Open(RATE))],
            vec![Piece::token(MODE, "Balanced", Token::Menu, Msg::Open(MODE))],
        ]
    };
    let shape = |pieces: Vec<Piece<'_, Msg>>| -> Vec<String> {
        pieces
            .into_iter()
            .map(|piece| match piece {
                Piece::Plain(text) | Piece::Inked { text, .. } => text.into_owned(),
                Piece::Aside(text) => format!("({text})"),
                Piece::Soft(text) | Piece::Faint(text) => format!("~{text}"),
                Piece::Value(text) => format!("<{text}>"),
                Piece::Token { key, .. } => format!("#{key}"),
            })
            .collect()
    };
    let mark = |slot| sentinel(slot).expect("slot");
    let swapped = format!("{} first, then {}.", mark(1), mark(0));
    assert_eq!(shape(split(&swapped, tokens())), ["#9", " first, then ", "#7", "."]);
    let arabic = "\u{e001} أولًا ثم \u{e000}";
    assert_eq!(shape(split(arabic, tokens())), ["#9", " أولًا ثم ", "#7"]);
    assert_eq!(shape(split(&format!("Only {}", mark(1)), tokens())), ["Only ", "#9", "#7"]);
}

#[test]
fn slots_hold_rows() {
    let shape = |pieces: Vec<Piece<'_, Msg>>| -> Vec<String> {
        pieces
            .into_iter()
            .map(|piece| match piece {
                Piece::Plain(text) | Piece::Inked { text, .. } => text.into_owned(),
                Piece::Aside(text) => format!("({text})"),
                Piece::Soft(text) | Piece::Faint(text) => format!("~{text}"),
                Piece::Value(text) => format!("<{text}>"),
                Piece::Token { key, .. } => format!("#{key}"),
            })
            .collect()
    };
    let keys = vec![
        Piece::token(RATE, "Left", Token::Key(KeyLook::Bound), Msg::Open(RATE)),
        Piece::plain(" "),
        Piece::token(MODE, "A", Token::Key(KeyLook::Bound), Msg::Open(MODE)),
    ];
    let text = format!("{} then {}", sentinel(0).expect("slot"), sentinel(1).expect("slot"));
    assert_eq!(shape(split(&text, vec![Vec::new(), keys])), [" then ", "#7", " ", "#9"]);
}

#[test]
fn values_split_out() {
    let shape = |pieces: Vec<Piece<'_, Msg>>| -> Vec<String> {
        pieces
            .into_iter()
            .map(|piece| match piece {
                Piece::Plain(text) | Piece::Inked { text, .. } => text.into_owned(),
                Piece::Aside(text) => format!("({text})"),
                Piece::Soft(text) | Piece::Faint(text) => format!("~{text}"),
                Piece::Value(text) => format!("<{text}>"),
                Piece::Token { key, .. } => format!("#{key}"),
            })
            .collect()
    };
    let token = || vec![vec![Piece::token(RATE, "30", Token::Text, Msg::Open(RATE))]];
    let text =
        format!("Every {}, from {VALUE_OPEN}images{VALUE_CLOSE}.", sentinel(0).expect("slot"));
    assert_eq!(shape(split(&text, token())), ["Every ", "#7", ", from ", "<images>", "."]);
    let open = format!("{VALUE_OPEN}{VALUE_CLOSE}On {VALUE_OPEN}startup");
    assert_eq!(shape(split(&open, Vec::new())), ["On ", "<startup>"]);
}

#[test]
fn sentinel_range_capped() {
    assert_eq!(sentinel(0), Some('\u{e000}'));
    assert_eq!(sentinel(0x18FD), Some('\u{f8fd}'));
    assert_eq!(sentinel(0x18FE), None);
    assert_eq!(sentinel(u32::MAX), None);
}

#[test]
fn arabic_tokens_one_line() {
    iced::advanced::graphics::text::font_system()
        .write()
        .expect("font system")
        .load_font(crate::shell::ui_font_bytes(Script::Arabic).into());
    let [_, narrow] = STRIP_WIDTHS;
    let sentence = Sentence { spec: say_on(locale("ar-SA"), SAY_SIZE, board().surface, None) };
    let mut state = State::default();
    sentence.shape(&mut state, &limits(narrow - STRIP_PADDING * 2.0));
    let tokens = sentence.tokens().count();
    assert!(tokens > 0);
    for token in 0..tokens {
        assert_eq!(state.lines(token).len(), 1, "token {token}");
    }
}

#[test]
fn arabic_fits_width() {
    iced::advanced::graphics::text::font_system()
        .write()
        .expect("font system")
        .load_font(crate::shell::ui_font_bytes(Script::Arabic).into());
    for frame in STRIP_WIDTHS {
        let limit = frame - STRIP_PADDING * 2.0;
        let sentence = Sentence { spec: say_on(locale("ar-SA"), SAY_SIZE, board().surface, None) };
        let mut state = State::default();
        let node = sentence.shape(&mut state, &limits(limit));
        let widest =
            state.paragraph.buffer().layout_runs().map(|run| run.line_w).fold(0.0, f32::max);
        assert!(node.size().width <= limit, "{} over {limit}", node.size().width);
        assert!(widest <= limit, "line {widest} over {limit}");
    }
}

#[test]
fn tokens_never_break() {
    assert_eq!(unbroken("5 min"), "5\u{2060}\u{a0}\u{2060}m\u{2060}i\u{2060}n\u{202f}");
    assert_eq!(unbroken("حاسوب محمول"), "حاسوب\u{a0}محمول\u{202f}", "no joiners in RTL");
    assert_eq!(unbroken("30 إطارًا/ث"), "30\u{a0}إطارًا/\u{2060}ث\u{202f}", "joined after slash");
}

#[test]
fn ring_needs_keyboard() {
    let mut sentence = battery(Some(RATE));
    assert!(!sentence.ringed(RATE), "pointer focus");
    sentence.spec.ring = true;
    assert!(sentence.ringed(RATE));
    assert!(!sentence.ringed(MODE));
}

#[test]
fn wrapped_token_lines() {
    let spans: [Span<'_, (), Font>; 2] =
        [Span::new("Saved as "), Span::new("a rather long laptop profile name")];
    let paragraph = Paragraph::with_spans(text::Text {
        content: spans.as_slice(),
        bounds: Size::new(120.0, f32::INFINITY),
        size: Pixels(SAY_SIZE),
        line_height: text::LineHeight::Relative(SAY_LEADING),
        font: UI_FONT,
        align_x: text::Alignment::Default,
        align_y: alignment::Vertical::Top,
        shaping: text::Shaping::Advanced,
        wrapping: text::Wrapping::WordOrGlyph,
    });
    let owners = [None, Some(0)];
    let lines = token_lines(&paragraph, &owners, 0);
    assert!(lines.len() >= 2, "token wraps");
    for pair in lines.windows(2) {
        assert!(pair[1].baseline > pair[0].baseline);
    }
    assert!(lines.iter().all(|line| line.right > line.left));
    assert!(token_lines(&paragraph, &owners, 1).is_empty());
    assert!(paragraph.min_bounds().width <= 120.0);
}

#[test]
fn overflow_breaks_before_token() {
    iced::advanced::graphics::text::font_system()
        .write()
        .expect("font system")
        .load_font(crate::shell::ui_font_bytes(Script::Arabic).into());
    let [_, narrow] = STRIP_WIDTHS;
    let limit = narrow - STRIP_PADDING * 2.0;
    let sentence = Sentence { spec: say_on(locale("ar-SA"), SAY_SIZE, board().surface, None) };
    let mut state = State::default();
    sentence.shape(&mut state, &limits(limit));
    assert_eq!(super::overflow(&state, limit), None);
    assert!(state.spans.iter().any(|span| span.text == super::LINE_BREAK), "forced break");
}

#[test]
fn rtl_plain_wraps() {
    iced::advanced::graphics::text::font_system()
        .write()
        .expect("font system")
        .load_font(crate::shell::ui_font_bytes(Script::Arabic).into());
    let palette = board();
    let said = "يجري البحث في Bing اليومي وYouTube. يحتاج Unsplash إلى مفتاح وصول. يحتاج Pexels إلى مفتاح API.";
    let sentence = Sentence {
        spec: SentenceSpec {
            pieces: vec![Piece::<Msg>::plain(said)],
            size: SAY_SIZE,
            leading: SAY_LEADING,
            scale: 1.0,
            palette,
            plane: palette.surface,
            focused: None,
            ring: false,
            popover: None,
            centred: false,
        },
    };
    let mut state = State::default();
    let _ = sentence.shape(&mut state, &limits(700.0));
    let node = sentence.shape(&mut state, &limits(554.0));
    let runs: Vec<_> = state.paragraph.buffer().layout_runs().map(|run| run.line_w).collect();
    assert!(runs.len() > 1, "{runs:?}");
    assert!(runs.iter().all(|width| *width <= 554.0), "{runs:?}");
    assert!(node.size().width <= 554.0);
}

#[test]
fn plain_keeps_leading_space() {
    let sentence = battery(None);
    let laid = sentence.spans(&[], &[]);
    let texts: Vec<_> = laid.spans.iter().map(|span| span.text.as_ref()).collect();
    assert!(texts.contains(&" in "), "{texts:?}");
}

#[test]
fn rtl_breaks_stay_rtl() {
    iced::advanced::graphics::text::font_system()
        .write()
        .expect("font system")
        .load_font(crate::shell::ui_font_bytes(Script::Arabic).into());
    let palette = board();
    let sentence = Sentence {
        spec: SentenceSpec {
            pieces: vec![
                Piece::plain("يجري البحث في المصادر اليومية "),
                Piece::token(RATE, "Steam Workshop API", Token::Menu, Msg::Open(RATE)),
                Piece::plain(" API."),
            ],
            size: SAY_SIZE,
            leading: SAY_LEADING,
            scale: 1.0,
            palette,
            plane: palette.surface,
            focused: None,
            ring: false,
            popover: None,
            centred: false,
        },
    };
    let mut state = State::default();
    crate::i18n::with_language("ar-SA", || sentence.shape(&mut state, &limits(200.0)));
    assert!(state.spans.iter().any(|span| span.text == super::LINE_BREAK), "forced break");
    let runs: Vec<_> = state.paragraph.buffer().layout_runs().map(|run| run.rtl).collect();
    assert!(runs.len() > 1 && runs.iter().all(|rtl| *rtl), "{runs:?}");
}

const LOOKS: [KeyLook; 5] =
    [KeyLook::Bound, KeyLook::Clash, KeyLook::Unbound, KeyLook::Add, KeyLook::Listening];

fn ink_pair(look: KeyLook) -> (Inks, Inks) {
    let palette = board();
    let plane = palette.surface;
    (
        Token::<Msg>::Key(look).inks(&palette, plane, false),
        Token::<Msg>::KeyLead(look).inks(&palette, plane, false),
    )
}

#[test]
fn lead_inks_match_key() {
    for look in LOOKS {
        let (key, lead) = ink_pair(look);
        assert!(key.text == lead.text && key.rule == lead.rule, "{look:?}");
        assert!(key.stroke == lead.stroke && key.fill == lead.fill, "{look:?}");
        assert_eq!(Token::<Msg>::KeyLead(look).glyph().0, "");
        assert_eq!(Token::<Msg>::KeyLead(look).cursor(), mouse::Interaction::Pointer);
    }
}

#[test]
fn key_inks_follow_look() {
    let palette = board();
    let plane = palette.surface;
    let faded = |alpha| tint(plane, palette.surface_text, alpha);
    let (bound, _) = ink_pair(KeyLook::Bound);
    assert!(
        bound.text == palette.primary && bound.stroke == Stroke::Dotted && bound.fill.is_none()
    );
    let (clash, _) = ink_pair(KeyLook::Clash);
    assert_eq!(clash.fill, Some(tint(plane, palette.primary, SELECTED_TINT)));
    let (none, _) = ink_pair(KeyLook::Unbound);
    assert!(none.text == faded(UNBOUND_ALPHA) && none.rule == faded(UNBOUND_RULE));
    let (add, _) = ink_pair(KeyLook::Add);
    assert!(add.text == faded(ADD_ALPHA) && add.rule == faded(ADD_RULE));
    assert!(add.stroke == Stroke::Dashed);
    let (listening, _) = ink_pair(KeyLook::Listening);
    assert!(listening.text == palette.surface_text && listening.stroke == Stroke::Solid);
}

#[test]
fn link_inks_solid() {
    let palette = board();
    let link = Token::<Msg>::Link;
    let inks = link.inks(&palette, palette.surface, false);
    assert!(inks.text == palette.surface_text && inks.stroke == Stroke::Solid);
    assert!(inks.rule == tint(palette.surface, palette.surface_text, LINK_RULE));
    assert_eq!(link.glyph().0, "");
    assert_eq!(link.cursor(), mouse::Interaction::Pointer);
}

#[test]
fn listening_ignores_press() {
    for look in LOOKS {
        let expected = look != KeyLook::Listening;
        assert_eq!(Token::<Msg>::Key(look).takes_press(), expected, "{look:?}");
        assert_eq!(Token::<Msg>::KeyLead(look).takes_press(), expected, "{look:?}");
    }
    assert!(Token::<Msg>::Menu.takes_press() && Token::<Msg>::Link.takes_press());
}

#[test]
fn listening_press_falls_through() {
    let palette = board();
    let sentence = Sentence {
        spec: SentenceSpec {
            pieces: vec![Piece::token(
                RATE,
                "Press a key",
                Token::Key(KeyLook::Listening),
                Msg::Open(RATE),
            )],
            size: SAY_SIZE,
            leading: SAY_LEADING,
            scale: 1.0,
            palette,
            plane: palette.surface,
            focused: None,
            ring: false,
            popover: None,
            centred: false,
        },
    };
    let mut state = State::default();
    let ((), messages, captured) = run(|shell| sentence.press(&mut state, Some(RATE), 40.0, shell));
    assert!(!captured && messages.is_empty());
    assert!(state.press.is_none());
    let ((), messages, _) = run(|shell| sentence.release(&mut state, Some(RATE), shell));
    assert!(messages.is_empty());
}

#[test]
fn popover_anchors_by_side() {
    let palette = board();
    let anchor = |side| {
        let pieces = vec![
            Piece::token(RATE, "Ctrl+", Token::KeyLead(KeyLook::Listening), Msg::Open(RATE)),
            Piece::plain("\n"),
            Piece::token(RATE, "X", Token::Key(KeyLook::Listening), Msg::Open(RATE)),
            Piece::token(MODE, "Other", Token::Menu, Msg::Open(MODE)),
        ];
        let sentence = Sentence {
            spec: SentenceSpec {
                pieces,
                size: SAY_SIZE,
                leading: SAY_LEADING,
                scale: 1.0,
                palette,
                plane: palette.surface,
                focused: None,
                ring: false,
                popover: None,
                centred: false,
            },
        };
        let popover = Popover { key: RATE, side, content: iced::widget::Space::new().into() };
        sentence.anchor_ordinal(&popover)
    };
    assert_eq!(anchor(Side::AboveEnd), Some(0));
    assert_eq!(anchor(Side::BelowEnd), Some(1));
    assert_eq!(anchor(Side::Below), Some(1));
}

#[test]
fn key_width_adds_glyph() {
    let size = 11.5;
    let label = "Ctrl+Left";
    let bare = cached_line_width(&unbroken(label), size, UI_FONT);
    let glyph = |look| {
        let (mark, mark_size) = Token::<Msg>::Key(look).glyph();
        cached_line_width(mark, mark_size * legible_type_scale(1.0), UI_FONT)
    };
    for look in LOOKS {
        assert_eq!(key_width(label, look, size), bare + glyph(look), "{look:?}");
    }
    assert!(glyph(KeyLook::Bound) > 0.0);
    assert_ne!(glyph(KeyLook::Add), glyph(KeyLook::Bound));
}

#[test]
fn soft_text_sits_between() {
    let plain = Piece::<Msg>::plain("a");
    let soft = Piece::<Msg>::Soft("a".into());
    let aside = Piece::<Msg>::Aside("a".into());
    assert_eq!(text_alpha(&soft), SOFT_ALPHA);
    assert_eq!(text_alpha(&plain), PLAIN_ALPHA);
    assert_eq!(text_alpha(&aside), ASIDE_ALPHA);
}

#[test]
fn muted_tokens_fade() {
    let palette = board();
    let plane = palette.surface;
    let inks = Token::<Msg>::Menu.inks(&palette, plane, true);
    assert_eq!(inks.text, tint(plane, palette.surface_text, MUTED_ALPHA));
    assert_eq!(inks.rule, tint(plane, palette.surface_text, MUTED_LINE_ALPHA));
}

#[test]
fn only_keys_clip_dashes() {
    let tail = dashes(0.0, 8.0, 3.0, 6.0, true);
    assert_eq!(tail, [(0.0, 3.0), (6.0, 2.0)]);
    let dots = dashes(0.0, 8.0, 3.0, 6.0, false);
    assert_eq!(dots, [(0.0, 3.0), (6.0, 3.0)]);
    let (bound, lead) = ink_pair(KeyLook::Bound);
    assert!(bound.clipped && lead.clipped);
    let palette = board();
    for kind in [Token::<Msg>::Menu, Token::Text, Token::Link, dial(FPS)] {
        assert!(!kind.inks(&palette, palette.surface, false).clipped);
    }
}

#[test]
fn text_wraps_at_spaces() {
    let wrapped = shown(&Token::<Msg>::Text, "two words", false);
    assert!(wrapped.contains(' '));
    for kind in [Token::<Msg>::Menu, Token::Key(KeyLook::Bound), Token::Link] {
        assert!(!shown(&kind, "two words", false).contains(' '));
    }
}

#[test]
fn mentions_skip_gap() {
    use super::GLYPH_GAP;
    let plain = shown::<Msg>(&Token::Mention { underlined: false }, "Colours", false);
    assert!(!plain.ends_with(GLYPH_GAP), "{plain:?}");
    let link = shown::<Msg>(&Token::Link, "Colours", false);
    assert!(link.ends_with(GLYPH_GAP), "{link:?}");
}

const FA_LABEL: [&str; 2] = ["مسیر پیکربندی\u{2002}", "برنامه\u{200c}ی Matugen بیرونی"];

fn fa_label(width: f32) -> State {
    iced::advanced::graphics::text::font_system()
        .write()
        .expect("font system")
        .load_font(crate::shell::ui_font_bytes(crate::i18n::Script::Arabic).into());
    let pieces = FA_LABEL.iter().map(|part| Piece::<Msg>::inked(*part, 12.0, Color::WHITE));
    let sentence = Sentence { spec: super::prose(pieces.collect(), super::PLAIN_LEADING) };
    let mut state = State::default();
    sentence.shape(&mut state, &limits(width));
    state
}

#[test]
fn mixed_runs_fit() {
    let mut broke = 0;
    for width in (60..240).step_by(3) {
        let state = fa_label(width as f32);
        assert!(state.paragraph.min_bounds().width <= width as f32, "{width}");
        broke += usize::from(state.spans.iter().any(|span| span.text == super::LINE_BREAK));
    }
    assert!(broke > 0, "no width broke");
}

#[test]
fn mixed_runs_keep_words() {
    let words = |text: &str| text.split_whitespace().map(str::to_string).collect::<Vec<_>>();
    for width in (60..240).step_by(3) {
        let spans = fa_label(width as f32).spans;
        let parts: Vec<_> =
            spans.iter().map(|span| span.text.replace(super::RTL_MARK, "")).collect();
        let laid = parts.join(" ");
        let whole: Vec<_> = FA_LABEL.iter().flat_map(|part| words(part)).collect();
        assert_eq!(words(&laid), whole, "{width}");
    }
}

#[test]
fn sliver_keeps_first_line() {
    let palette = board();
    let sentence = Sentence {
        spec: SentenceSpec {
            pieces: vec![Piece::<Msg>::plain("Wide words wrap")],
            size: SAY_SIZE,
            leading: SAY_LEADING,
            scale: 1.0,
            palette,
            plane: palette.surface,
            focused: None,
            ring: false,
            popover: None,
            centred: false,
        },
    };
    let mut state = State::default();
    let _ = sentence.shape(&mut state, &limits(1.0));
    let first = state.spans.first().expect("span");
    assert!(!first.text.is_empty() && first.text != super::LINE_BREAK, "{:?}", first.text);
}

const UNITS: [&str; 7] = ["924 px", "40 %", "300 ms", "1.5 x", "90°", "Desk", "dawn_01.png"];

fn units_said() -> State {
    iced::advanced::graphics::text::font_system()
        .write()
        .expect("font system")
        .load_font(crate::shell::ui_font_bytes(Script::Arabic).into());
    let mut pieces = vec![Piece::plain("گوشه\u{200c}ها ")];
    for (at, unit) in UNITS.into_iter().enumerate() {
        if at > 0 {
            pieces.push(Piece::plain("، "));
        }
        let key = at as u64;
        pieces.push(Piece::token(key, unit, Token::Menu, Msg::Open(key)));
    }
    pieces.push(Piece::plain(" از بالا."));
    let palette = board();
    let sentence = Sentence {
        spec: SentenceSpec {
            pieces,
            size: SAY_SIZE,
            leading: SAY_LEADING,
            scale: 1.0,
            palette,
            plane: palette.surface,
            focused: None,
            ring: false,
            popover: None,
            centred: false,
        },
    };
    let mut state = State::default();
    crate::i18n::with_language("fa-IR", || sentence.shape(&mut state, &limits(2000.0)));
    state
}

#[test]
fn rtl_units_read_ltr() {
    let state = units_said();
    let run = state.paragraph.buffer().layout_runs().next().expect("one line");
    assert!(run.rtl);
    let inked = |glyph: &&LayoutGlyph| {
        let text = run.text.get(glyph.start..glyph.end).unwrap_or_default();
        glyph.w > 0.0 && !text.trim().is_empty()
    };
    let mut spans = Vec::new();
    for (token, unit) in UNITS.iter().enumerate() {
        let label = state.owners.iter().position(|owner| *owner == Some(token)).expect("label");
        let mut glyphs: Vec<_> =
            run.glyphs.iter().filter(|glyph| glyph.metadata == label).filter(inked).collect();
        glyphs.sort_by_key(|glyph| glyph.start);
        let xs: Vec<f32> = glyphs.iter().map(|glyph| glyph.x).collect();
        assert!(xs.is_sorted(), "{unit} reads {xs:?}");
        spans.push((xs[0], xs[xs.len() - 1]));
    }
    for pair in spans.windows(2) {
        let ((first, _), (_, last)) = (pair[0], pair[1]);
        assert!(last < first, "{spans:?}");
        let comma = run.glyphs.iter().any(|glyph| {
            run.text.get(glyph.start..glyph.end) == Some("،") && glyph.x > last && glyph.x < first
        });
        assert!(comma, "{spans:?}");
    }
}
