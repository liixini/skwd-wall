use std::collections::HashMap;

use iced::advanced::{text as iced_text, widget as iced_widget};

use super::*;

type InputState =
    iced::widget::text_input::State<<iced::Renderer as iced_text::Renderer>::Paragraph>;

fn field<'a>(
    key: &str,
    values: &'a HashMap<String, String>,
    palette: &'a Palette,
) -> Element<'a, Message> {
    super::widget(
        Control::TextField {
            key: key.into(),
            path: key.into(),
            placeholder: "Value",
            secret: false,
        },
        values,
        true,
        None,
        210.0,
        1.0,
        palette,
        1.0,
        None,
        MotionProfile::new(120.0, 300.0, 600.0),
        None,
    )
}

#[test]
fn field_focus_follows_identity() {
    let values = HashMap::new();
    let palette = Palette::default();
    let mut tree = iced_widget::Tree::empty();

    let first = field("library.path", &values, &palette);
    tree.diff(first.as_widget());
    tree.children[0].state.downcast_mut::<InputState>().focus();

    let same = field("library.path", &values, &palette);
    tree.diff(same.as_widget());
    assert!(tree.children[0].state.downcast_ref::<InputState>().is_focused());

    let other = field("weather.city", &values, &palette);
    tree.diff(other.as_widget());
    assert!(!tree.children[0].state.downcast_ref::<InputState>().is_focused());
}

const BOARD_WIDTH: f32 = 132.0;
const BOARD_SLACK: f32 = 4.0;

#[test]
fn buttons_measure_text() {
    let min = crate::frontend::ui::FOLIO_BUTTON_MIN_WIDTH;
    let long = button_width("Composition and crops", 1.0);
    assert!((long - BOARD_WIDTH).abs() < BOARD_SLACK, "long label {long}");
    assert!(long > button_width("Full image", 1.0));
    assert_eq!(button_width("Go", 1.0), min);
    assert_eq!(button_width("Go", 2.0), min * 2.0);
}

#[test]
fn buttons_scale_text() {
    let one = button_width("Composition and crops", 1.0);
    let large = button_width("Composition and crops", 1.5);
    assert!((large / one - 1.5).abs() < 0.1, "scaled {large} from {one}");
}

const FIT_MAX: f32 = 170.0;
const FIT_SAMPLE: &str = "Delete SigLIP2 ViT-B/16 large multiview index, quantized";

#[test]
fn long_label_wraps() {
    let (width, height) = fitted_button(FIT_SAMPLE, FIT_MAX, 1.0);
    assert_eq!(width, FIT_MAX);
    let ButtonHeight::Lines(lines) = height else { panic!("wrapped") };
    assert!(lines >= 2, "{lines}");
}

#[test]
fn short_label_stays() {
    let (width, height) = fitted_button("Delete model", FIT_MAX, 1.0);
    assert_eq!(width, button_width("Delete model", 1.0));
    assert!(matches!(height, ButtonHeight::Compact));
}

#[test]
fn glued_word_breaks() {
    let word = "x".repeat(60);
    let (_, height) = fitted_button(&format!("Delete {word}"), FIT_MAX, 1.0);
    assert!(matches!(height, ButtonHeight::Lines(lines) if lines >= 3));
}

const CAPTION_SIZE: iced::Size = iced::Size::new(160.0, 30.0);
const CAPTION_LABELS: [&str; 2] = ["استيراد حزمة", "Composition and crops"];

fn pre_wrap_caption(label: &str) -> Element<'static, Message> {
    container(
        text(label.to_owned())
            .font(crate::frontend::ui::UI_FONT)
            .size(BUTTON_TEXT_SIZE * legible_type_scale(1.0)),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

#[test]
#[ignore = "needs a Vulkan adapter"]
fn compact_caption_unchanged() {
    let dir = std::env::temp_dir().join(format!("caption-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let font = crate::frontend::ui::UI_FONT;
    let shot = |name: &str, view: &dyn Fn() -> Element<'static, Message>| {
        let path = dir.join(name);
        crate::frontend::headless::capture(CAPTION_SIZE, font, path.to_str().unwrap(), view);
        std::fs::read(path).unwrap()
    };
    for label in CAPTION_LABELS {
        let now = shot("now.png", &|| caption(label.to_owned(), false, 1.0));
        let was = shot("was.png", &|| pre_wrap_caption(label));
        assert_eq!(now, was, "{label}");
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn wrapped_height_fits_text() {
    let scale = 1.5;
    let (_, height) = fitted_button(FIT_SAMPLE, FIT_MAX * scale, scale);
    let ButtonHeight::Lines(lines) = height else { panic!("wrapped") };
    let line = BUTTON_TEXT_SIZE * legible_type_scale(scale) * BUTTON_LINE_LEADING;
    assert!(height.pixels(scale) >= lines as f32 * line, "{lines}");
}
