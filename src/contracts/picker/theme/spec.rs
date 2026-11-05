pub use wall_proto::STUDIO_ENGINE;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PaletteSpec {
    pub primary: Option<String>,
    pub primary_text: Option<String>,
    pub surface: Option<String>,
    pub surface_text: Option<String>,
    pub surface_variant: Option<String>,
    pub surface_container: Option<String>,
    pub background: Option<String>,
    pub outline: Option<String>,
    pub tertiary: Option<String>,
    pub secondary: Option<String>,
    pub secondary_text: Option<String>,
    pub tertiary_text: Option<String>,
    pub error: Option<String>,
    pub error_text: Option<String>,
    pub surface_variant_text: Option<String>,
    pub shadow: Option<String>,
    pub hover: Option<String>,
    pub hover_text: Option<String>,
}
