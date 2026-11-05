use std::time::{Duration, Instant};

use iced::widget::{Space, button, column, container, text, text_input};
use iced::{Alignment, Color, Element, Length, Padding, Size, mouse, window};

use super::tint::SELECTED_TINT;
use super::{
    KeyLook, OptionsSpec, Piece, Popover, ROW_TINT, RevealSpec, SAY_LEADING, SAY_SIZE, Scrub,
    SentenceSpec, Side, StaggerSpec, ToggleSpec, Token, options_button, reveal, sentence, split,
    stagger, tint, wipe_toggle,
};
use crate::frontend::theme::Palette;
use crate::frontend::ui::{UI_FONT, bg_style, box_style, folio_horizontal_rule, label, row, start};
use crate::i18n::{Script, active_script, set_language};

const OUT: &str = "SKWD_CALM_OUT";
const SCREEN: Size = Size::new(1366.0, 768.0);
const STRIP: Size = Size::new(760.0, 330.0);
pub(super) const STRIP_WIDTHS: [f32; 2] = [460.0, 260.0];
pub(super) const STRIP_PADDING: f32 = 8.0;
const FPS: u64 = 1;
const IDLE: u64 = 2;
const MODE: u64 = 3;
const NAME: u64 = 4;
const BIND: u64 = 5;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Msg {
    Open,
}

pub(crate) fn board() -> Palette {
    let rgb = Color::from_rgb8;
    let text = rgb(230, 232, 236);
    let dark = rgb(27, 16, 6);
    Palette {
        primary: rgb(240, 163, 94),
        primary_text: dark,
        surface: rgb(21, 24, 30),
        surface_text: text,
        surface_variant: rgb(42, 47, 56),
        surface_container: rgb(29, 33, 40),
        background: rgb(15, 17, 21),
        outline: rgb(138, 145, 156),
        tertiary: rgb(238, 128, 144),
        secondary: rgb(238, 128, 144),
        secondary_text: dark,
        tertiary_text: dark,
        error: rgb(238, 128, 144),
        error_text: dark,
        surface_variant_text: text,
        shadow: Color::BLACK,
        hover: rgb(42, 47, 56),
        hover_text: text,
    }
}

fn surface() -> Color {
    board().surface
}

fn open_row() -> Color {
    tint(surface(), board().primary, ROW_TINT)
}

fn ink(plane: Color, alpha: f32) -> Color {
    tint(plane, board().surface_text, alpha)
}

fn rule(plane: Color, alpha: f32) -> Element<'static, Msg> {
    folio_horizontal_rule(tint(plane, board().outline, alpha))
}

fn number(key: u64, label: &'static str, value: f64) -> Piece<'static, Msg> {
    let scrub = Scrub { value, min: 0.0, max: 240.0, step: 1.0 };
    let dial = Token::Number { scrub, change: Box::new(|_| Msg::Open), commit: Msg::Open };
    Piece::token(key, label, dial, Msg::Open)
}

pub(super) fn say_on(
    pieces: Vec<Piece<'static, Msg>>,
    size: f32,
    plane: Color,
    popover: Option<Popover<'static, Msg>>,
) -> SentenceSpec<'static, Msg> {
    SentenceSpec {
        pieces,
        size,
        leading: SAY_LEADING,
        scale: 1.0,
        palette: board(),
        plane,
        focused: None,
        ring: false,
        popover,
        centred: false,
    }
}

fn say(
    pieces: Vec<Piece<'static, Msg>>,
    size: f32,
    plane: Color,
    popover: Option<Popover<'static, Msg>>,
) -> Element<'static, Msg> {
    sentence(say_on(pieces, size, plane, popover))
}

fn resolved(tag: &str) -> &'static str {
    match tag {
        "sv-SE" => {
            "På batteri begränsar väljaren till \u{e000}, video stoppas efter \u{e001} utan inmatning och bakgrunder använder läget \u{e002}. Profilen sparas som \u{e003} och \u{e004} växlar till den."
        }
        "ar-SA" => {
            "على البطارية يقتصر المنتقي على \u{e000}، ويتوقف الفيديو بعد \u{e001} دون إدخال، وتستخدم الخلفيات وضع \u{e002}. يُحفظ الملف الشخصي باسم \u{e003} ويبدّل إليه \u{e004}."
        }
        "ja-JP" => {
            "バッテリー駆動中はピッカーを\u{e000}に制限し、操作がないと\u{e001}後に動画を停止し、壁紙は\u{e002}モードを使います。\u{e004}で切り替えるプロファイルは\u{e003}として保存されます。"
        }
        _ => {
            "On battery the picker caps at \u{e000}, video stops after \u{e001} without input, and wallpapers use \u{e002} mode. The profile is saved as \u{e003} and \u{e004} switches to it."
        }
    }
}

fn labels(tag: &str) -> [&'static str; 5] {
    match tag {
        "sv-SE" => ["30 bilder/s", "5 min", "Balanserat", "Bärbar", "Ctrl+B"],
        "ar-SA" => ["30 إطارًا/ث", "5 دقائق", "متوازن", "حاسوب محمول", "Ctrl+B"],
        "ja-JP" => ["30 fps", "5 分", "バランス", "ノートPC", "Ctrl+B"],
        _ => ["30 fps", "5 min", "Balanced", "Laptop", "Ctrl+B"],
    }
}

pub(super) fn locale(tag: &str) -> Vec<Piece<'static, Msg>> {
    let [fps, idle, mode, name, bind] = labels(tag);
    split(
        resolved(tag),
        vec![
            vec![number(FPS, fps, 30.0)],
            vec![number(IDLE, idle, 5.0)],
            vec![Piece::token(MODE, mode, Token::Menu, Msg::Open)],
            vec![Piece::token(NAME, name, Token::Text, Msg::Open)],
            vec![Piece::token(BIND, bind, Token::Key(KeyLook::Bound), Msg::Open)],
        ],
    )
}

fn menu() -> Element<'static, Msg> {
    let palette = board();
    let ground = surface();
    let item = move |word: &'static str, on: bool| {
        container(
            row![
                label(if on { "◆" } else { "◇" }, 8.0, 1.0, palette.primary),
                label(word, 12.0, 1.0, palette.surface_text)
            ]
            .spacing(9.0)
            .align_y(Alignment::Center),
        )
        .padding([8.0, 10.0])
        .width(Length::Fill)
        .style(move |_| {
            bg_style(if on { tint(ground, palette.primary, SELECTED_TINT) } else { ground })
        })
    };
    container(column![item("Quality", false), item("Balanced", true), item("Battery saver", false)])
        .padding(4.0)
        .width(Length::Fixed(180.0))
        .style(move |_| super::style::popover(ground, &palette))
        .into()
}

fn editor() -> Element<'static, Msg> {
    let palette = board();
    text_input("", "Laptop")
        .on_input(|_| Msg::Open)
        .font(UI_FONT)
        .size(12.5)
        .padding(Padding::from([0.0, 2.0]))
        .width(Length::Fixed(64.0))
        .style(move |_, _| super::style::editor(Color::TRANSPARENT, &palette))
        .into()
}

fn toggle(identity: u64, on: bool, plane: Color) -> Element<'static, Msg> {
    wipe_toggle(ToggleSpec {
        identity,
        on,
        focused: false,
        labels: ["Off", "On"],
        message: Msg::Open,
        instant: true,
        scale: 1.0,
        palette: board(),
        plane,
    })
}

fn options(identity: u64, open: bool, plane: Color) -> Element<'static, Msg> {
    options_button(OptionsSpec {
        identity,
        label: "Options",
        open,
        focused: false,
        message: Msg::Open,
        instant: true,
        scale: 1.0,
        palette: board(),
        plane,
    })
}

fn value_row(
    index: usize,
    title: &'static str,
    hint: &'static str,
    value: Element<'static, Msg>,
) -> Element<'static, Msg> {
    let plane = open_row();
    let cell = column![
        container(
            row![
                label(title, 12.5, 1.0, board().surface_text),
                label(hint, 10.5, 1.0, ink(plane, 0.5)),
                Space::new().width(Length::Fill),
                value,
            ]
            .spacing(8.0)
            .align_y(Alignment::Center),
        )
        .padding([9.0, 2.0]),
        rule(plane, 0.2),
    ];
    let spec = StaggerSpec {
        identity: index as u64,
        index,
        open: true,
        instant: true,
        planes: [surface(), plane],
    };
    stagger(cell, spec)
}

fn group(head: &'static str, rows: Vec<Element<'static, Msg>>) -> Element<'static, Msg> {
    let mut group =
        column![container(label(head, 10.0, 1.0, ink(open_row(), 0.5))).padding([2.0, 2.0])];
    for row in rows {
        group = group.push(row);
    }
    group.width(Length::Fill).into()
}

fn body(edit: bool) -> Element<'static, Msg> {
    let plane = open_row();
    let value = |key, text, amount| say(vec![number(key, text, amount)], 12.5, plane, None);
    let profile = say(
        vec![Piece::token(NAME, "Laptop", Token::Text, Msg::Open)],
        12.5,
        plane,
        edit.then(|| Popover { key: NAME, side: Side::Over, content: editor() }),
    );
    let left = group(
        "ALWAYS",
        vec![
            value_row(
                0,
                "Max FPS",
                "refresh rate, or 60 for a cooler laptop",
                value(FPS, "144 fps", 144.0),
            ),
            value_row(1, "Idle pause", "0 disables", value(IDLE, "5 min", 5.0)),
            value_row(2, "Performance mode", "also in Wallpapers", toggle(3, false, plane)),
        ],
    );
    let right = group(
        "ON BATTERY",
        vec![
            value_row(3, "On-battery picker FPS", "lower of this and Max FPS", {
                value(FPS, "30 fps", 30.0)
            }),
            value_row(4, "Profile name", "shown in the bar", profile),
            value_row(5, "Wallpaper performance mode", "direct video", toggle(4, true, plane)),
        ],
    );
    let divider = container(Space::new())
        .width(Length::Fixed(1.0))
        .height(Length::Fill)
        .style(move |_| bg_style(tint(plane, board().outline, 0.34)));
    let columns = row![left, divider, right].spacing(17.0).height(Length::Shrink);
    reveal(
        container(columns).padding(Padding { top: 2.0, right: 30.0, bottom: 18.0, left: 260.0 }),
        RevealSpec { identity: 1, open: true, instant: true, floor: 0.0 },
    )
}

fn heading(title: &'static str, note: Option<&'static str>, plane: Color) -> Element<'static, Msg> {
    let mut name = column![label(title, 15.0, 1.0, board().surface_text)].spacing(3.0);
    if let Some(note) = note {
        name = name.push(
            label(note, 11.0, 1.0, ink(plane, 0.56)).line_height(text::LineHeight::Relative(1.35)),
        );
    }
    container(name).width(Length::Fixed(240.0)).into()
}

fn section(menu_open: bool, edit: bool) -> Element<'static, Msg> {
    let plane = open_row();
    let pieces = vec![
        Piece::plain("On battery the picker caps at "),
        number(FPS, "30 fps", 30.0),
        Piece::plain(", wallpapers use "),
        Piece::token(MODE, "Balanced", Token::Menu, Msg::Open),
        Piece::plain(" mode and the profile is saved as "),
        Piece::token(NAME, "Laptop", Token::Text, Msg::Open),
        Piece::plain("."),
    ];
    let popover = menu_open.then(|| Popover { key: MODE, side: Side::Below, content: menu() });
    let head = row![
        heading(
            "Laptop & power",
            Some("Limits that apply on battery, with the saver on or off."),
            plane
        ),
        container(say(pieces, SAY_SIZE, plane, popover)).width(Length::Fill).align_x(start()),
        toggle(1, true, plane),
        options(2, true, plane),
    ]
    .spacing(20.0)
    .align_y(Alignment::Center);
    let sec = column![
        rule(surface(), 0.34),
        container(head).padding(Padding { top: 13.0, right: 4.0, bottom: 13.0, left: 0.0 }),
        body(edit),
    ];
    container(sec).style(move |_| bg_style(plane)).into()
}

fn closed(title: &'static str, pieces: Vec<Piece<'static, Msg>>, id: u64) -> Element<'static, Msg> {
    let plane = surface();
    let head = row![
        heading(title, None, plane),
        container(say(pieces, SAY_SIZE, plane, None)).width(Length::Fill),
        options(id, false, plane),
    ]
    .spacing(20.0)
    .align_y(Alignment::Center);
    column![
        rule(plane, 0.34),
        container(head).padding(Padding { top: 8.0, right: 4.0, bottom: 8.0, left: 0.0 })
    ]
    .into()
}

fn sheet(menu_open: bool, edit: bool) -> Element<'static, Msg> {
    let palette = board();
    let plane = surface();
    let page = column![
        column![
            label("System", 22.0, 1.0, palette.surface_text),
            label("Language, graphics, power and bug reports.", 11.0, 1.0, ink(plane, 0.56)),
        ]
        .spacing(9.0),
        column![
            section(menu_open, edit),
            closed(
                "Graphics",
                vec![
                    Piece::plain("The picker uses "),
                    Piece::token(MODE, "Automatic", Token::Menu, Msg::Open),
                    Piece::plain(" and previews decode on the GPU."),
                ],
                5,
            ),
            closed(
                "Language",
                vec![
                    Piece::plain("The interface is in "),
                    Piece::token(MODE, "English", Token::Menu, Msg::Open),
                    Piece::plain("."),
                ],
                6,
            ),
            rule(plane, 0.34),
        ],
    ]
    .spacing(22.0);
    let index =
        container(Space::new()).width(Length::Fixed(318.0)).height(Length::Fill).style(move |_| {
            box_style(palette.background, tint(palette.background, palette.outline, 0.46))
        });
    let sheet = container(row![
        index,
        container(page).padding(Padding { top: 34.0, right: 38.0, bottom: 0.0, left: 38.0 })
    ])
    .width(Length::Fixed(1344.0))
    .height(Length::Fixed(746.0))
    .style(move |_| box_style(plane, tint(palette.background, palette.outline, 0.58)));
    container(sheet)
        .padding(11.0)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| bg_style(palette.background))
        .into()
}

fn strip(tag: &str) -> Element<'static, Msg> {
    let plane = surface();
    let frame = |width: f32| {
        container(say(locale(tag), SAY_SIZE, plane, None))
            .width(Length::Fixed(width))
            .align_x(start())
            .padding(STRIP_PADDING)
            .style(move |_| box_style(plane, tint(plane, board().outline, 0.2)))
    };
    let [wide, narrow] = STRIP_WIDTHS;
    container(column![frame(wide), frame(narrow)].spacing(18.0).align_x(start()))
        .padding(20.0)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| bg_style(plane))
        .into()
}

pub(super) fn load(tag: &str) {
    let previous = active_script();
    set_language(tag);
    crate::shell::reload_ui_font(previous);
}

#[test]
#[ignore = "manual offscreen render of the calm settings kit"]
fn render_calm_kit() {
    let out = std::env::var(OUT).expect(OUT);
    set_language("en-US");
    iced_wgpu::graphics::text::font_system()
        .write()
        .expect("font system")
        .load_font(crate::shell::ui_font_bytes(Script::Latin).into());
    crate::frontend::headless::capture(
        SCREEN,
        UI_FONT,
        &format!("{out}/section-en-US.png"),
        || sheet(false, false),
    );
    crate::frontend::headless::capture(SCREEN, UI_FONT, &format!("{out}/section-open.png"), || {
        sheet(true, true)
    });
    for tag in ["en-US", "sv-SE", "ar-SA", "ja-JP"] {
        load(tag);
        crate::frontend::headless::capture(
            STRIP,
            UI_FONT,
            &format!("{out}/sentence-{tag}.png"),
            || strip(tag),
        );
    }
    set_language("en-US");
}

fn folding(open: bool) -> Element<'static, Msg> {
    let tall = button(Space::new().height(Length::Fixed(200.0)).width(Length::Fill))
        .on_press(Msg::Open)
        .padding(0.0)
        .width(Length::Fill)
        .style(|_, _| button::Style {
            background: Some(Color::from_rgb(0.0, 1.0, 0.0).into()),
            ..button::Style::default()
        });
    reveal(tall, RevealSpec { identity: 9, open, instant: false, floor: 0.0 })
}

#[test]
#[ignore = "needs a Vulkan adapter"]
fn reveal_in_interface() {
    use iced_runtime::user_interface::{Cache, State, UserInterface};

    let mut renderer = crate::frontend::headless::renderer(UI_FONT);
    let screen = Size::new(100.0, 300.0);
    let viewport = iced_wgpu::graphics::Viewport::with_physical_size(
        Size::new(screen.width as u32, screen.height as u32),
        1.0,
    );
    let start = Instant::now();
    let cursor = mouse::Cursor::Available(iced::Point::new(50.0, 20.0));
    let mut messages = Vec::new();
    let mut cache = Cache::default();
    let mut frame = |open: bool, events: &[iced::Event], messages: &mut Vec<Msg>| {
        let mut ui =
            UserInterface::build(folding(open), screen, std::mem::take(&mut cache), &mut renderer);
        let (state, _) = ui.update(
            events,
            cursor,
            &mut renderer,
            &mut iced::advanced::clipboard::Null,
            messages,
        );
        ui.draw(
            &mut renderer,
            &iced::Theme::Dark,
            &iced::advanced::renderer::Style::default(),
            cursor,
        );
        let pixels = renderer.screenshot(&viewport, Color::BLACK);
        let width = screen.width as usize;
        let painted = (0..screen.height as usize)
            .filter(|y| pixels[(y * width + width / 2) * 4 + 1] > 128)
            .count();
        cache = ui.into_cache();
        (
            matches!(state, State::Updated { redraw_request: window::RedrawRequest::Wait, .. }),
            painted,
        )
    };
    let redraw = |ms: u64| {
        iced::Event::Window(window::Event::RedrawRequested(start + Duration::from_millis(ms)))
    };
    let click = [
        iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
    ];
    assert_eq!(frame(false, &[redraw(0)], &mut messages), (true, 0));
    frame(true, &[redraw(10)], &mut messages);
    let (idle, painted) = frame(true, &[redraw(205)], &mut messages);
    assert!(!idle);
    assert!(painted.abs_diff(100) <= 4, "half open at {painted}");
    frame(true, &click, &mut messages);
    assert!(messages.is_empty(), "moving blocks input");
    frame(true, &[redraw(400)], &mut messages);
    assert_eq!(frame(true, &[redraw(500)], &mut messages), (true, 200));
    frame(true, &click, &mut messages);
    assert_eq!(messages, [Msg::Open]);
}

const HEAD: f32 = 60.0;
const HEAD_FILL: Color = Color::from_rgb(0.0, 0.0, 1.0);

fn opening(open: bool) -> Element<'static, Msg> {
    let plane = open_row();
    let rows = (0..3).map(|index| {
        let spec = StaggerSpec {
            identity: index as u64,
            index,
            open,
            instant: false,
            planes: [surface(), plane],
        };
        stagger(toggle(10 + index as u64, true, plane), spec)
    });
    let head = container(Space::new().width(Length::Fill).height(Length::Fixed(HEAD)))
        .style(|_| bg_style(HEAD_FILL));
    let body = iced::widget::Column::with_children(rows).width(Length::Fill);
    column![head, reveal(body, RevealSpec { identity: 9, open, instant: false, floor: 0.0 })].into()
}

#[test]
#[ignore = "needs a Vulkan adapter"]
fn rising_rows_stay_inside() {
    use iced_runtime::user_interface::{Cache, UserInterface};

    let mut renderer = crate::frontend::headless::renderer(UI_FONT);
    let screen = Size::new(200.0, 200.0);
    let viewport = iced_wgpu::graphics::Viewport::with_physical_size(
        Size::new(screen.width as u32, screen.height as u32),
        1.0,
    );
    let start = Instant::now();
    let mut cache = Cache::default();
    let mut pixels = Vec::new();
    for (open, ms) in [(false, 0), (true, 10), (true, 60), (true, 110), (true, 160), (true, 160)] {
        let redraw =
            iced::Event::Window(window::Event::RedrawRequested(start + Duration::from_millis(ms)));
        let mut ui =
            UserInterface::build(opening(open), screen, std::mem::take(&mut cache), &mut renderer);
        let _ = ui.update(
            &[redraw],
            mouse::Cursor::Unavailable,
            &mut renderer,
            &mut iced::advanced::clipboard::Null,
            &mut Vec::new(),
        );
        ui.draw(
            &mut renderer,
            &iced::Theme::Dark,
            &iced::advanced::renderer::Style::default(),
            mouse::Cursor::Unavailable,
        );
        pixels = renderer.screenshot(&viewport, Color::BLACK);
        cache = ui.into_cache();
    }
    if let Ok(out) = std::env::var(OUT) {
        ::image::RgbaImage::from_raw(screen.width as u32, screen.height as u32, pixels.clone())
            .expect("frame")
            .save(format!("{out}/reveal-mid-rise.png"))
            .expect("png");
    }
    let width = screen.width as usize;
    let head = HEAD as usize;
    let escaped: Vec<_> = (head - 8..head)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .filter(|(x, y)| pixels[(y * width + x) * 4..][..3] != [0, 0, 255])
        .collect();
    assert!(escaped.is_empty(), "{} pixels over the head", escaped.len());
}
