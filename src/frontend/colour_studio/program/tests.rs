use super::*;
use crate::domain::studio::{Mode, Source, Studio};
use crate::frontend::colour_studio::state::Wallpaper;
use crate::frontend::theme::Palette;
use skwd_palette::studio::Tuning;

#[test]
fn moving_popover_hits_options() {
    let wallpaper = Wallpaper {
        key: "red".into(),
        label: "Red".into(),
        image: iced::widget::image::Handle::from_path("/w/red.webp"),
        current: true,
        aspect: None,
    };
    let model = Studio::new(Source::Theme("nord".into()), Mode::Dark, Tuning::default());
    let mut studio = ColourStudio::new(model, wallpaper, Vec::new());
    studio.sync();
    studio.tick(5.0);
    studio.update(&StudioMsg::Hover(Some(Hit::Tab(Row::View))));
    studio.tick(1.0);
    studio.update(&StudioMsg::Hover(Some(Hit::Tab(Row::FineTune))));
    studio.tick(0.01);
    let offset = draw::pop_offset(&studio, Row::FineTune);
    assert!(offset.x.abs() > 1.0);
    let plate = draw::options(&studio, Row::FineTune)[0].1.bounds();
    let at =
        Point::new(plate.x + plate.width / 2.0 + offset.x, plate.y + plate.height / 2.0 + offset.y);
    let palette = Palette::default();
    let canvas = StudioCanvas {
        studio: &studio,
        chrome: Chrome::new(&palette),
        ui_scale: 1.0,
        part: draw::PARTS[0],
        top: true,
    };
    assert_eq!(canvas.target(at), Target::Option(Row::FineTune, 0));
}
