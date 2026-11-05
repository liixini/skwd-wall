use iced::{Font, Size};
use paper_control::{ClockStyle, Depth, Face, Pin, Widget};

use super::raster::Stage;
use super::scenery_tests::scenery;
use super::state::Strand;
use crate::domain::strand::{Command, Item};
use crate::infrastructure::desktop::{Request, load};

const IMAGE: &str = "SKWD_STRAND_IMAGE";
const OUT: &str = "SKWD_STRAND_OUT";
const TICK: f32 = 1.0 / 60.0;
const SETTLE: usize = 180;

fn settle(strand: &mut Strand) {
    for _ in 0..SETTLE {
        strand.tick(TICK);
    }
}

fn editor(image: &str, output: [f32; 2]) -> Strand {
    let request = Request {
        wallpaper: Some(image.into()),
        fill: paper_geom::FillMode::Fill,
        size: output.map(|side| side as u32),
        background: paper_control::Background::default(),
        outlines: Vec::new(),
        font: None,
    };
    let stage = load(&request)
        .backdrop
        .map(|backdrop| Stage::new(backdrop.size, backdrop.rgba, backdrop.matte));
    let widgets = vec![
        Widget {
            face: Face::Clock(ClockStyle::Aperture),
            depth: Depth::Front,
            pin: Pin { anchor: [0.0, 0.0], offset: [376.0, 76.0] },
            height: Some(280.0 / 900.0),
            output: None,
        },
        Widget {
            face: Face::Clock(ClockStyle::Greeter),
            depth: Depth::Front,
            pin: Pin { anchor: [0.0, 1.0], offset: [600.0, -24.0] },
            height: Some(70.0 / 900.0),
            output: None,
        },
    ];
    let mut strand = Strand::new(output, 1.0, Some("DP-1".to_owned()));
    strand.adopt(widgets, scenery(), stage);
    settle(&mut strand);
    strand
}

fn render(strand: &Strand, out: &str) {
    let [width, height] = strand.output;
    crate::frontend::headless::capture(Size::new(width, height), Font::DEFAULT, out, || {
        strand.view()
    });
}

#[test]
#[ignore = "manual offscreen render of the widget editor"]
fn render_strand() {
    let image = std::env::var(IMAGE).expect("SKWD_STRAND_IMAGE");
    let out = std::env::var(OUT).expect("SKWD_STRAND_OUT");
    let mut strand = editor(&image, [1600.0, 900.0]);
    render(&strand, &format!("{out}/idle.png"));
    let id = strand.snapshot()[0].id;
    strand.select(id);
    settle(&mut strand);
    render(&strand, &format!("{out}/selected.png"));
    strand.command(Command::Remove);
    for _ in 0..30 {
        strand.tick(TICK);
    }
    render(&strand, &format!("{out}/toast.png"));
    strand.hover = Some((super::state::Hit::Item(Item::Bead(0, 1)), strand.now));
    settle(&mut strand);
    render(&strand, &format!("{out}/hover.png"));
    let small = editor(&image, [1366.0, 768.0]);
    render(&small, &format!("{out}/small.png"));
}
