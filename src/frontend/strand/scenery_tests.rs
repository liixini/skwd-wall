use paper_control::{Labels, MONTHS, Palette, WEEKDAYS};
use paper_widgets::{Lettering, Moment};

use super::raster::Scenery;

pub fn scenery() -> Scenery {
    let days: [String; WEEKDAYS] = std::array::from_fn(|index| format!("d{index}"));
    let months: [String; MONTHS] = std::array::from_fn(|index| format!("m{index}"));
    Scenery {
        palette: Palette { text: [236, 235, 231], accent: [157, 184, 255], shade: [18, 19, 22] },
        labels: Labels {
            days: days.clone(),
            day_names: days,
            months: months.clone(),
            month_names: months,
        },
        lettering: Lettering::open(paper_widgets::FONT).expect("font"),
        moment: Moment::civil(21, 47, 2, 27, 10),
    }
}
