use skwd_config::corners::{self, CornerKeys};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Slices,
    Depth,
    Grid,
    Hex,
    Sandy,
    Hand,
    Collection,
}

impl Mode {
    pub const ALL: [Self; 7] = [
        Self::Slices,
        Self::Depth,
        Self::Hex,
        Self::Grid,
        Self::Sandy,
        Self::Hand,
        Self::Collection,
    ];

    pub fn try_from_key(value: &str) -> Option<Self> {
        match value {
            "slices" => Some(Self::Slices),
            "depth" => Some(Self::Depth),
            "wall" | "grid" => Some(Self::Grid),
            "hex" => Some(Self::Hex),
            "sandy" | "nova" => Some(Self::Sandy),
            "hand" => Some(Self::Hand),
            "collection" => Some(Self::Collection),
            _ => None,
        }
    }

    pub fn from_key(value: &str) -> Self {
        Self::try_from_key(value).unwrap_or(Self::Slices)
    }

    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Slices => "slices",
            Self::Depth => "depth",
            Self::Grid => "wall",
            Self::Hex => "hex",
            Self::Sandy => "sandy",
            Self::Hand => "hand",
            Self::Collection => "collection",
        }
    }

    pub const fn corners(self) -> Option<&'static CornerKeys> {
        match self {
            Self::Slices | Self::Sandy => Some(&corners::SLICES),
            Self::Depth => Some(&corners::DEPTH),
            Self::Grid => Some(&corners::GRID),
            Self::Hand => Some(&corners::HAND),
            Self::Collection => Some(&corners::COLLECTION),
            Self::Hex => None,
        }
    }
}

mod tests;
