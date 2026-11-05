pub use skwd_palette::studio::Source;
use skwd_palette::studio::{AREA_KEY, Family, Method};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Image,
    Theory,
    Themes,
    Matugen,
}

impl Group {
    pub const ALL: [Self; 4] = [Self::Image, Self::Theory, Self::Themes, Self::Matugen];

    pub fn of(source: &Source) -> Self {
        match source {
            Source::Method(key) if key == AREA_KEY => Self::Image,
            Source::Method(key) => match Method::from_key(key).map(Method::family) {
                Some(Family::Theory) => Self::Theory,
                Some(Family::Image) | None => Self::Image,
            },
            Source::Theme(_) => Self::Themes,
            Source::Material(_) => Self::Matugen,
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn family(self) -> Option<Family> {
        match self {
            Self::Image => Some(Family::Image),
            Self::Theory => Some(Family::Theory),
            Self::Themes | Self::Matugen => None,
        }
    }
}

pub fn is_area(source: &Source) -> bool {
    matches!(source, Source::Method(key) if key == AREA_KEY)
}

pub fn area() -> Source {
    Source::Method(AREA_KEY.to_string())
}
