#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphicsTier {
    Discrete,
    Integrated,
    Other,
}

impl GraphicsTier {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Discrete => "Discrete",
            Self::Integrated => "Integrated",
            Self::Other => "Other",
        }
    }
}

pub struct UnknownGraphicsTier;

impl std::str::FromStr for GraphicsTier {
    type Err = UnknownGraphicsTier;

    fn from_str(label: &str) -> Result<Self, Self::Err> {
        match label {
            "Discrete" => Ok(Self::Discrete),
            "Integrated" => Ok(Self::Integrated),
            "Other" => Ok(Self::Other),
            _ => Err(UnknownGraphicsTier),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphicsCard {
    pub name: String,
    pub tier: GraphicsTier,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphicsDevice {
    pub id: String,
    pub name: String,
}

pub trait GraphicsProbe {
    fn devices(&self) -> Vec<GraphicsDevice> {
        Vec::new()
    }
    fn probe(&self) -> GraphicsCard;
}
