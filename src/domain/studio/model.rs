use skwd_palette::studio::{
    AREA_KEY, MATUGEN_SCHEMES, Method, Reading, Role, Shape, StudioPalette, Tuning, from_theme,
    resolve,
};

use super::area::Picked;
use super::source::{Group, Source, area, is_area};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Dark,
    Light,
}

impl Mode {
    pub fn dark(self) -> bool {
        self == Self::Dark
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeChoice {
    Dark,
    Light,
    ByTime,
}

impl From<Mode> for ModeChoice {
    fn from(mode: Mode) -> Self {
        if mode.dark() { Self::Dark } else { Self::Light }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub key: String,
    pub dark: StudioPalette,
    pub light: StudioPalette,
    pub shape: Shape,
}

impl Variant {
    fn pick(&self, dark: bool) -> StudioPalette {
        if dark { self.dark } else { self.light }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Snapshot {
    pub image: String,
    pub seed: String,
    pub wheel: Vec<f32>,
    pub methods: Vec<Variant>,
    pub materials: Vec<Variant>,
    pub dark: Option<bool>,
}

struct Inputs<'a> {
    snapshot: Option<&'a Snapshot>,
    area: Option<&'a Picked>,
}

impl Reading for Inputs<'_> {
    fn method(&self, key: &str, dark: bool) -> Option<StudioPalette> {
        if key == AREA_KEY {
            return self.area.map(|picked| picked.palette(dark));
        }
        let snapshot = self.snapshot?;
        snapshot.methods.iter().find(|method| method.key == key).map(|method| method.pick(dark))
    }

    fn material(&self, scheme: &str, dark: bool) -> Option<StudioPalette> {
        let snapshot = self.snapshot?;
        snapshot.materials.iter().find(|item| item.key == scheme).map(|item| item.pick(dark))
    }
}

pub fn listed_theme(key: &str) -> bool {
    !skwd_palette::is_light_twin(key)
}

fn theme_dark(key: &str) -> Option<bool> {
    skwd_palette::preset(key).map(|preset| from_theme(&preset).is_dark())
}

pub fn fixed_theme(key: &str) -> Option<bool> {
    if skwd_palette::light_twin(key).is_some() {
        return None;
    }
    theme_dark(key)
}

pub fn has_terminal(source: &Source) -> bool {
    !matches!(source, Source::Material(_))
}

#[derive(Debug, Clone, PartialEq)]
pub struct Studio {
    pub snapshot: Option<Snapshot>,
    pub area: Option<Picked>,
    pub source: Source,
    pub mode: Mode,
    pub mode_choice: ModeChoice,
    pub mode_picked: bool,
    pub tuning: Tuning,
    pub role: Role,
    pub compare: bool,
    pub split: f32,
    pub live: Option<StudioPalette>,
    remembered: Vec<(String, bool, String)>,
}

impl Studio {
    pub fn new(source: Source, mode: Mode, tuning: Tuning) -> Self {
        Self {
            snapshot: None,
            area: None,
            source,
            mode,
            mode_choice: mode.into(),
            mode_picked: false,
            tuning,
            role: Role::Accent,
            compare: true,
            split: 0.5,
            live: None,
            remembered: Vec::new(),
        }
    }

    fn inputs(&self) -> Inputs<'_> {
        Inputs { snapshot: self.snapshot.as_ref(), area: self.area.as_ref() }
    }

    pub fn resolve(&self, source: &Source, tuning: &Tuning) -> Option<StudioPalette> {
        let (palette, _) = resolve(source, Some(&self.inputs()), tuning, || self.mode.dark())?;
        Some(palette)
    }

    pub fn palette(&self) -> Option<StudioPalette> {
        self.resolve(&self.source, &self.tuning)
    }

    pub fn raw(&self) -> Option<StudioPalette> {
        self.resolve(&self.source, &Tuning::default())
    }

    pub fn untuned(&self, role: Role) -> Option<StudioPalette> {
        let mut tuning = self.tuning;
        tuning.shades[role.index()] = 0;
        self.resolve(&self.source, &tuning)
    }

    pub fn swatch(&self, source: &Source) -> Option<StudioPalette> {
        self.resolve(source, &Tuning::default())
    }

    pub fn shape(&self) -> Shape {
        let Source::Method(key) = &self.source else {
            return Shape::Plain;
        };
        self.snapshot
            .iter()
            .flat_map(|snapshot| &snapshot.methods)
            .find(|method| method.key == *key)
            .map_or(Shape::Plain, |method| method.shape.clone())
    }

    pub fn wheel(&self) -> &[f32] {
        if is_area(&self.source)
            && let Some(picked) = &self.area
        {
            return &picked.wheel;
        }
        self.snapshot.as_ref().map_or(&[], |snapshot| snapshot.wheel.as_slice())
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.mode_choice = mode.into();
        self.mode_picked = true;
        let Source::Theme(name) = &self.source else {
            return;
        };
        let Some(current) = fixed_theme(name) else {
            return;
        };
        let want = mode.dark();
        if current == want {
            return;
        }
        let Some(family) = skwd_palette::family(name) else {
            return;
        };
        let remembered = self
            .remembered
            .iter()
            .find(|(kin, dark, _)| kin == family && *dark == want)
            .map(|(_, _, key)| key.clone());
        let twin = remembered.or_else(|| {
            skwd_palette::PRESETS
                .iter()
                .map(|(key, _)| *key)
                .find(|key| {
                    listed_theme(key)
                        && skwd_palette::family(key) == Some(family)
                        && fixed_theme(key) == Some(want)
                })
                .map(str::to_string)
        });
        let Some(twin) = twin.filter(|twin| twin != name) else {
            return;
        };
        self.remembered.retain(|(kin, dark, _)| !(kin == family && *dark == current));
        self.remembered.push((family.to_string(), current, name.clone()));
        self.source = Source::Theme(twin);
    }

    pub fn choose_mode(&mut self, choice: ModeChoice, effective: Mode) {
        self.set_mode(effective);
        self.mode_choice = choice;
    }

    pub fn group(&self) -> Group {
        Group::of(&self.source)
    }

    pub fn catalogue(&self, group: Group) -> Vec<Source> {
        match group {
            Group::Image | Group::Theory => {
                let family = group.family();
                let mut found: Vec<Source> = Method::ALL
                    .into_iter()
                    .filter(|method| Some(method.family()) == family)
                    .filter(|method| self.method_ready(method.key()))
                    .map(|method| Source::Method(method.key().to_string()))
                    .collect();
                if group == Group::Image {
                    found.push(area());
                }
                found
            }
            Group::Themes => skwd_palette::PRESETS
                .iter()
                .filter(|(key, _)| listed_theme(key))
                .map(|(key, _)| Source::Theme((*key).to_string()))
                .collect(),
            Group::Matugen => MATUGEN_SCHEMES
                .iter()
                .map(|scheme| Source::Material((*scheme).to_string()))
                .collect(),
        }
    }

    fn method_ready(&self, key: &str) -> bool {
        self.snapshot.as_ref().is_some_and(|snapshot| snapshot.methods.iter().any(|m| m.key == key))
    }

    pub fn order(&self) -> Vec<Source> {
        Group::ALL
            .into_iter()
            .flat_map(|group| self.catalogue(group))
            .filter(|source| !is_area(source) || self.area.is_some())
            .collect()
    }
}
