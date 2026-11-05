use iced::Color;
use iced::widget::image::Handle;
use skwd_palette::studio::{Area, Role, Shape, StudioPalette, to_oklch};

use super::controls::{Hit, Knob, Plate, Row, View};
use super::layout::{self, Head, Plan};
use super::motion::{
    Axis, BASE, Crossfade, FAST, Glide, SLOW, SNAP, Swap, Tint, WHEEL_MOVE, WHEEL_TINT,
};
use super::text;
use super::tokens::{INTRO_END, KID_DOTS, KID_HEIGHT, POP_LINGER, REVEAL_MARGIN, SECTION_HEIGHT};
use crate::domain::studio::{
    Group, Mode, ModeChoice, Picked, Pixels, Saved, Source, Studio, has_terminal, is_area, shown,
};
use crate::frontend::theme::rgb_color;

pub const SWATCHES: [Role; 8] = [
    Role::Base,
    Role::Panel,
    Role::Raised,
    Role::Text,
    Role::Quiet,
    Role::Accent,
    Role::Second,
    Role::Third,
];
pub const WHEEL_BINS: usize = 36;
pub const ACCENTS: [Role; 3] = [Role::Accent, Role::Second, Role::Third];
const FALLBACK_ASPECT: f32 = 16.0 / 9.0;
const FULL_CHROMA: f32 = 0.2;

pub type Dots = [Tint; KID_DOTS];

#[derive(Debug, Clone)]
pub enum Load<T> {
    Loading,
    Ready(T),
    Failed,
}

#[derive(Debug, Clone)]
pub struct Decoded {
    pub pixels: Pixels,
    pub grey: Handle,
}

#[derive(Debug, Clone)]
pub struct Wallpaper {
    pub key: String,
    pub label: String,
    pub image: Handle,
    pub current: bool,
    pub aspect: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Schemes {
    Unread,
    Reading,
    Read,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub palette: StudioPalette,
    pub view: View,
    pub image: Handle,
    pub aspect: f32,
    pub terminal: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Picking {
    pub draft: Option<Area>,
    pub anchor: Option<(f32, f32)>,
    pub prior: Option<Area>,
    pub prior_source: Source,
    pub before: Source,
    pub dragging: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mounts {
    pub main: f32,
    pub naming: f32,
    pub picking: f32,
    pub tags: f32,
    pub draft: f32,
    pub wheel: f32,
    pub kid_mark: f32,
}

pub struct Swaps {
    pub pick: Swap,
    pub apply: Swap,
    pub values: [Swap; Row::ALL.len()],
}

impl Swaps {
    fn new() -> Self {
        Self {
            pick: Swap::new(Axis::Y),
            apply: Swap::new(Axis::Y),
            values: std::array::from_fn(|_| Swap::new(Axis::X)),
        }
    }

    fn settle(&mut self, now: f32) -> f32 {
        [&mut self.pick, &mut self.apply]
            .into_iter()
            .chain(self.values.iter_mut())
            .map(|swap| swap.settle(now))
            .fold(f32::NEG_INFINITY, f32::max)
    }
}

pub struct WheelState {
    pub ticks: Vec<Glide>,
    pub dots: [(Glide, Glide); ACCENTS.len()],
    pub tints: [Tint; ACCENTS.len()],
    pub shape: Shape,
    pub was: Shape,
    pub shape_at: f32,
}

#[derive(PartialEq)]
pub struct Applied {
    palette: StudioPalette,
    view: View,
    source: Source,
    mode: Mode,
    mode_choice: ModeChoice,
    mode_picked: bool,
    tuning: skwd_palette::studio::Tuning,
    area: Option<(Area, String)>,
}

pub struct ColourStudio {
    pub model: Studio,
    pub(super) time_mode: Mode,
    pub wallpaper: Wallpaper,
    pub decoded: Load<Decoded>,
    pub aspect: Option<f32>,
    pub now: f32,
    busy: f32,
    pub ready_at: Option<f32>,
    pub intro: bool,
    pub group: Group,
    pub group_at: f32,
    pub row: Row,
    pub row_at: f32,
    pub row_dirs: [f32; Row::ALL.len()],
    pub view: View,
    pub applied: Option<Applied>,
    pub dir: f32,
    pub naming: Option<String>,
    pub picking: Option<Picking>,
    pub areas: Vec<(String, Area)>,
    pub saved: Vec<Saved>,
    pub saved_picks: Vec<Option<StudioPalette>>,
    pub saved_at: f32,
    shown: Option<StudioPalette>,
    pub raw: Option<StudioPalette>,
    pub scenes: Crossfade<SceneKey, Scene>,
    pub compared: Crossfade<SceneKey, Scene>,
    pub heads: Vec<(Head, Glide)>,
    pub opened_top: Glide,
    pub sec_mark: Glide,
    pub kid_mark: Glide,
    pub kid_mark_on: bool,
    pub modes_top: Glide,
    pub wheel_top: Glide,
    pub wheel_size: Option<f32>,
    pub tabs: [iced::Rectangle; Row::ALL.len()],
    pub band: [Glide; 2],
    pub pop: Glide,
    pub pop_x: Glide,
    pub pop_at: f32,
    pub pop_hover: bool,
    pub pop_until: f32,
    pub trails: [Glide; Group::ALL.len()],
    pub tags: [Glide; 2],
    pub panes: [Glide; Group::ALL.len()],
    pub knobs: [Glide; Knob::ALL.len()],
    pub switch: Glide,
    pub split: Glide,
    pub veil: Glide,
    pub hidden: Glide,
    pub clip: [Glide; 4],
    pub swatches: [Tint; SWATCHES.len()],
    pub dots: Vec<(Source, Dots)>,
    pub wheel: WheelState,
    pub swaps: Swaps,
    pub modes_label: String,
    pub hover: Option<Hit>,
    pub hover_at: f32,
    pub unhover: Option<Hit>,
    pub unhover_at: f32,
    pub pressed: Option<Plate>,
    pub pressed_at: f32,
    pub knob_drag: Option<Knob>,
    pub sliding: bool,
    pub quiet_until: f32,
    pub mounts: Mounts,
    pub plan: Plan,
    pub analysed: Option<String>,
    pub area_image: Option<(String, String)>,
    pub schemes: Schemes,
    official: Option<Picked>,
    catalogue: Option<CatalogueKey>,
}

pub type SceneKey = (StudioPalette, View, String);

#[derive(Debug, Clone, PartialEq)]
struct CatalogueKey {
    wallpaper: String,
    image: String,
    seed: String,
    dark: bool,
    methods: usize,
    materials: usize,
    area: Option<[f32; 4]>,
    saved: usize,
    decoded: bool,
}

const SAME_AREA: f32 = 1e-4;

fn close(a: Area, b: Area) -> bool {
    (a.left - b.left).abs() < SAME_AREA
        && (a.top - b.top).abs() < SAME_AREA
        && (a.right - b.right).abs() < SAME_AREA
        && (a.bottom - b.bottom).abs() < SAME_AREA
}

fn tint(color: Color, duration: f32) -> Tint {
    Tint::new(color, duration)
}

impl ColourStudio {
    pub fn new(model: Studio, wallpaper: Wallpaper, saved: Vec<Saved>) -> Self {
        let group = model.group();
        let time_mode = model.mode;
        let aspect = wallpaper.aspect;
        let clear = Color::TRANSPARENT;
        let plan = layout::plan(0, group, |_| Vec::new(), group);
        let mut studio = Self {
            model,
            time_mode,
            wallpaper,
            decoded: Load::Loading,
            aspect,
            now: 0.0,
            busy: INTRO_END,
            ready_at: None,
            intro: true,
            group,
            group_at: 0.0,
            row: Row::View,
            row_at: f32::NEG_INFINITY,
            row_dirs: [1.0; Row::ALL.len()],
            view: View::Desktop,
            applied: None,
            dir: 1.0,
            naming: None,
            picking: None,
            areas: Vec::new(),
            saved,
            saved_picks: Vec::new(),
            saved_at: 0.0,
            shown: None,
            raw: None,
            scenes: Crossfade::default(),
            compared: Crossfade::default(),
            heads: Vec::new(),
            opened_top: Glide::new(0.0, BASE),
            sec_mark: Glide::new(0.0, BASE),
            kid_mark: Glide::new(0.0, BASE),
            kid_mark_on: false,
            modes_top: Glide::new(0.0, BASE),
            wheel_top: Glide::new(0.0, BASE),
            wheel_size: None,
            tabs: layout::tabs(super::draw::tab_needs(), super::draw::header_limit()),
            band: [Glide::new(0.0, BASE), Glide::new(0.0, BASE)],
            pop: Glide::new(0.0, BASE),
            pop_x: Glide::new(0.0, BASE),
            pop_at: f32::NEG_INFINITY,
            pop_hover: false,
            pop_until: f32::NEG_INFINITY,
            trails: std::array::from_fn(|_| Glide::new(0.0, SLOW)),
            tags: std::array::from_fn(|_| Glide::new(1.0, FAST)),
            panes: std::array::from_fn(|_| Glide::new(0.0, FAST)),
            knobs: std::array::from_fn(|_| Glide::new(0.0, BASE)),
            switch: Glide::new(1.0, BASE),
            split: Glide::new(0.5, BASE),
            veil: Glide::new(0.0, SLOW),
            hidden: Glide::new(0.0, BASE),
            clip: [
                Glide::new(0.0, BASE),
                Glide::new(0.0, BASE),
                Glide::new(1.0, BASE),
                Glide::new(1.0, BASE),
            ],
            swatches: std::array::from_fn(|_| tint(clear, BASE)),
            dots: Vec::new(),
            wheel: WheelState {
                ticks: (0..WHEEL_BINS)
                    .map(|_| Glide::new(0.0, WHEEL_TINT).with_curve(SNAP))
                    .collect(),
                dots: std::array::from_fn(|_| {
                    (
                        Glide::new(0.0, WHEEL_MOVE).with_curve(SNAP),
                        Glide::new(0.0, WHEEL_MOVE).with_curve(SNAP),
                    )
                }),
                tints: std::array::from_fn(|_| tint(clear, WHEEL_TINT)),
                shape: Shape::Plain,
                was: Shape::Plain,
                shape_at: f32::NEG_INFINITY,
            },
            swaps: Swaps::new(),
            modes_label: String::new(),
            hover: None,
            hover_at: f32::NEG_INFINITY,
            unhover: None,
            unhover_at: f32::NEG_INFINITY,
            pressed: None,
            pressed_at: f32::NEG_INFINITY,
            knob_drag: None,
            sliding: false,
            quiet_until: f32::NEG_INFINITY,
            mounts: Mounts {
                main: 0.0,
                naming: 0.0,
                picking: 0.0,
                tags: 0.0,
                draft: 0.0,
                wheel: 0.0,
                kid_mark: 0.0,
            },
            plan,
            analysed: None,
            area_image: None,
            schemes: Schemes::Unread,
            official: None,
            catalogue: None,
        };
        studio.switch.jump(if studio.model.compare { 1.0 } else { 0.0 });
        studio.split.jump(studio.model.split);
        let band = studio.tabs[studio.row.index()];
        studio.band[0].jump(band.x);
        studio.band[1].jump(band.width);
        studio.pop_x.jump(super::draw::pop(&studio, studio.row).rect.x);
        studio
    }

    pub fn hold(&mut self, until: f32) {
        self.busy = self.busy.max(until);
    }

    pub fn animating(&self) -> bool {
        self.now < self.busy
    }

    pub fn set_time_mode(&mut self, mode: Mode) {
        self.time_mode = mode;
        if self.model.mode_choice == ModeChoice::ByTime && self.model.mode != mode {
            self.model.choose_mode(ModeChoice::ByTime, mode);
            self.refresh_area();
            self.sync();
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.now += dt;
        let until = self.swaps.settle(self.now);
        self.hold(until);
        if !self.pop_hover && self.now >= self.pop_until && self.pop.target() > 0.0 {
            let until = self.pop.toward(0.0, self.now);
            self.hold(until);
        }
        let until = self.scenes.settle(self.now).max(self.compared.settle(self.now));
        self.hold(until);
    }

    pub fn quiet(&self) -> bool {
        self.knob_drag.is_some()
            || self.sliding
            || self.picking.as_ref().is_some_and(|pick| pick.dragging)
            || self.now < self.quiet_until
    }

    pub fn palette(&self) -> Option<StudioPalette> {
        self.shown
    }

    pub fn aspect(&self) -> f32 {
        self.aspect.unwrap_or(FALLBACK_ASPECT)
    }

    pub fn pixels(&self) -> Option<&Pixels> {
        match &self.decoded {
            Load::Ready(decoded) => Some(&decoded.pixels),
            Load::Loading | Load::Failed => None,
        }
    }

    pub fn saved_area(&self) -> Option<Area> {
        self.areas.iter().find(|(key, _)| *key == self.wallpaper.key).map(|(_, area)| *area)
    }

    pub fn set_saved_area(&mut self, area: Option<Area>) {
        self.areas.retain(|(key, _)| *key != self.wallpaper.key);
        if let Some(area) = area {
            self.areas.push((self.wallpaper.key.clone(), area));
        }
    }

    pub fn intro_age(&self) -> Option<f32> {
        let ready = self.ready_at?;
        self.intro.then_some(self.now - ready).filter(|age| *age < INTRO_END)
    }

    pub fn content_age(&self) -> f32 {
        self.ready_at.map_or(f32::NEG_INFINITY, |ready| self.now - ready)
    }

    pub fn shown_saved(&self) -> &[Saved] {
        shown(&self.saved)
    }

    pub fn dots_for(&self, source: &Source) -> Option<[Color; KID_DOTS]> {
        let (_, dots) = self.dots.iter().find(|(key, _)| key == source)?;
        Some(dots.map(|dot| dot.value(self.now)))
    }

    pub fn knob(&self, knob: Knob) -> f32 {
        let tuning = &self.model.tuning;
        match knob {
            Knob::Warmth => tuning.warmth,
            Knob::Colour => tuning.colour,
            Knob::Contrast => tuning.contrast,
        }
    }

    pub fn pane_scroll(&self, group: Group) -> f32 {
        self.panes[group.index()].value(self.now)
    }

    pub fn head_top(&self, head: Head) -> f32 {
        self.heads
            .iter()
            .find(|(item, _)| *item == head)
            .map_or(0.0, |(_, glide)| glide.value(self.now))
    }

    pub fn reveal(&mut self) {
        let Some(opened) = self.plan.opened.as_ref() else {
            return;
        };
        let Some(view) = opened.pane else {
            return;
        };
        let Some(index) = opened.kids.iter().position(|kid| *kid == self.model.source) else {
            return;
        };
        let max = opened.max_scroll();
        let group = opened.group;
        let top = layout::kid_top(index);
        let scroll = self.panes[group.index()].target();
        if top < scroll + REVEAL_MARGIN || top + KID_HEIGHT > scroll + view - REVEAL_MARGIN {
            let target = (top - view / 2.0 + KID_HEIGHT / 2.0).clamp(0.0, max);
            let until = self.panes[group.index()].toward(target, self.now);
            self.hold(until);
        }
    }

    fn scene(&self, palette: StudioPalette, terminal: bool) -> Scene {
        Scene {
            palette,
            view: self.view,
            image: self.wallpaper.image.clone(),
            aspect: self.aspect(),
            terminal,
        }
    }

    pub fn wants_schemes(&self) -> bool {
        let material = |source: &Source| matches!(source, Source::Material(_));
        material(&self.model.source)
            || self.group == Group::Matugen
            || shown(&self.saved).iter().any(|entry| material(&entry.source))
    }

    pub fn invalidate(&mut self) {
        self.catalogue = None;
    }

    fn catalogue_key(&self) -> CatalogueKey {
        let snapshot = self.model.snapshot.as_ref();
        CatalogueKey {
            wallpaper: self.wallpaper.key.clone(),
            image: snapshot.map_or_else(String::new, |snapshot| snapshot.image.clone()),
            seed: snapshot.map_or_else(String::new, |snapshot| snapshot.seed.clone()),
            dark: self.model.mode.dark(),
            methods: snapshot.map_or(0, |snapshot| snapshot.methods.len()),
            materials: snapshot.map_or(0, |snapshot| snapshot.materials.len()),
            area: self.model.area.as_ref().map(|picked| picked.area.edges()),
            saved: self.saved.len(),
            decoded: self.pixels().is_some(),
        }
    }

    pub fn mark_undone(&mut self) {
        self.applied = None;
        self.hold(self.now + BASE + FAST);
    }

    pub fn undoable(&self) -> bool {
        self.applied.is_some() && self.applied == self.applied_state()
    }

    fn applied_state(&self) -> Option<Applied> {
        let palette = self.shown?;
        let area = self
            .saved_area()
            .zip(self.model.snapshot.as_ref().map(|snapshot| snapshot.image.clone()))
            .filter(|_| is_area(&self.model.source));
        Some(Applied {
            palette,
            view: self.view,
            source: self.model.source.clone(),
            mode: self.model.mode,
            mode_choice: self.model.mode_choice,
            mode_picked: self.model.mode_picked,
            tuning: self.model.tuning,
            area,
        })
    }

    pub fn flash(&mut self) {
        self.pop_until = self.now + POP_LINGER;
        self.open_pop();
        self.hold(self.pop_until);
    }

    pub fn open_pop(&mut self) {
        if self.pop.target() <= 0.0 {
            self.pop_at = self.now;
            self.pop_x.jump(super::draw::pop(self, self.row).rect.x);
        }
        let until = self.pop.toward(1.0, self.now);
        self.hold(until);
    }

    pub fn pop_open(&self) -> bool {
        self.pop.target() > 0.0 && self.picking.is_none()
    }

    pub fn set_decoded(&mut self, decoded: Load<Decoded>) {
        if let Load::Ready(found) = &decoded {
            self.aspect = Some(found.pixels.aspect);
        }
        self.decoded = decoded;
        let aspect = self.aspect();
        let image = &self.wallpaper.image;
        for scene in self.scenes.items_mut().chain(self.compared.items_mut()) {
            if scene.image == *image {
                scene.aspect = aspect;
            }
        }
        self.fill_official();
        self.refresh_area();
        self.sync();
    }

    fn fill_official(&mut self) {
        let Some(area) =
            self.official.as_ref().filter(|found| found.patches.is_empty()).map(|found| found.area)
        else {
            return;
        };
        let local = self.pixels().and_then(|pixels| Picked::read(pixels, area));
        if let (Some(official), Some(local)) = (self.official.as_mut(), local) {
            official.patches = local.patches;
            self.catalogue = None;
        }
    }

    pub fn adopt_area(&mut self, asked: Area, snapshot: &crate::domain::studio::Snapshot) {
        let Some(variant) = snapshot.methods.first() else {
            return;
        };
        let Some(area) = self.saved_area().filter(|area| close(*area, asked)) else {
            return;
        };
        let local = self.pixels().and_then(|pixels| Picked::read(pixels, area));
        self.official = Some(Picked {
            area,
            patches: local.map(|picked| picked.patches).unwrap_or_default(),
            dark: variant.dark,
            light: variant.light,
            wheel: snapshot.wheel.clone(),
        });
        self.invalidate();
        self.refresh_area();
        self.sync();
    }

    pub fn mark_applied(&mut self) {
        self.applied = self.applied_state();
        self.hold(self.now + BASE + FAST);
    }

    pub fn refresh_area(&mut self) {
        let box_area = self
            .picking
            .as_ref()
            .and_then(|pick| pick.draft)
            .or_else(|| self.saved_area().map(|area| self.frame().to_preview(area)));
        let picked = box_area.and_then(|shown| {
            let image = self.frame().to_image(shown);
            if let Some(official) =
                self.official.as_ref().filter(|official| close(official.area, image))
            {
                return Some(official.clone());
            }
            let pixels = self.pixels()?;
            Picked::read(pixels, image)
        });
        self.model.area = picked;
    }

    pub fn frame(&self) -> crate::domain::studio::Frame {
        crate::domain::studio::Frame::cover(self.aspect())
    }

    pub fn refresh_saved(&mut self) {
        let picks: Vec<Option<StudioPalette>> = shown(&self.saved)
            .iter()
            .map(|entry| {
                if is_area(&entry.source) {
                    let pixels = self.pixels()?;
                    let area = entry.area?;
                    let picked = Picked::read(pixels, area)?;
                    return Some(picked.palette(self.model.mode.dark()));
                }
                self.model.swatch(&entry.source)
            })
            .collect();
        self.saved_picks = picks;
    }

    fn sync_dots(&mut self) {
        let now = self.now;
        let sources: Vec<Source> =
            Group::ALL.iter().flat_map(|group| self.model.catalogue(*group)).collect();
        let mut until = f32::NEG_INFINITY;
        for source in sources {
            let Some(palette) = self.model.swatch(&source) else {
                if let Some((_, dots)) = self.dots.iter_mut().find(|(key, _)| *key == source) {
                    for dot in dots.iter_mut() {
                        let clear = Color { a: 0.0, ..dot.value(now) };
                        until = until.max(dot.toward(clear, now));
                    }
                }
                continue;
            };
            let colors = ACCENTS.map(|role| rgb_color(palette.get(role)));
            if let Some((_, dots)) = self.dots.iter_mut().find(|(key, _)| *key == source) {
                for (dot, color) in dots.iter_mut().zip(colors) {
                    until = until.max(dot.toward(color, now));
                }
            } else {
                let dots = colors.map(|color| {
                    let mut dot = tint(Color { a: 0.0, ..color }, BASE);
                    until = until.max(dot.toward(color, now));
                    dot
                });
                self.dots.push((source, dots));
            }
        }
        self.hold(until);
    }

    fn sync_plan(&mut self) {
        let now = self.now;
        let catalogue = |group| self.model.catalogue(group);
        let plan =
            layout::plan(self.shown_saved().len(), self.group, catalogue, self.model.group());
        let mut until = f32::NEG_INFINITY;
        for section in &plan.sections {
            match self.heads.iter_mut().find(|(head, _)| *head == section.head) {
                Some((_, glide)) => until = until.max(glide.toward(section.top, now)),
                None => self.heads.push((section.head, Glide::new(section.top, BASE))),
            }
        }
        self.heads.retain(|(head, _)| plan.sections.iter().any(|section| section.head == *head));
        if let Some(section) = plan.sections.iter().find(|section| section.holds) {
            until = until.max(self.sec_mark.toward(section.top + SECTION_HEIGHT / 2.0, now));
        }
        for group in Group::ALL {
            let holds = plan
                .sections
                .iter()
                .any(|section| section.holds && section.head == Head::Group(group));
            until =
                until.max(self.trails[group.index()].toward(if holds { 1.0 } else { 0.0 }, now));
        }
        let selected =
            plan.opened.as_ref().filter(|opened| opened.pane.is_none()).and_then(|opened| {
                let index = opened.kids.iter().position(|kid| *kid == self.model.source)?;
                Some(opened.top + layout::kid_top(index) + KID_HEIGHT / 2.0)
            });
        match selected {
            Some(centre) if self.kid_mark_on => {
                until = until.max(self.kid_mark.toward(centre, now));
            }
            Some(centre) => {
                self.kid_mark.jump(centre);
                self.kid_mark_on = true;
                self.mounts.kid_mark = now;
                until = until.max(now + SLOW + super::tokens::MARK_DELAY);
            }
            None => self.kid_mark_on = false,
        }
        let modes = layout::modes_top(plan.bottom);
        until = until.max(self.modes_top.toward(modes, now));
        until = until.max(self.wheel_top.toward(layout::wheel_top(modes), now));
        let size = layout::wheel_size(modes);
        if size.is_some() && self.wheel_size.is_none() {
            self.mounts.wheel = now;
            until = until.max(now + SLOW + super::tokens::WHEEL_NOTE_DELAY);
        }
        self.wheel_size = size;
        if let Some(opened) = &plan.opened {
            let slot = opened.group.index();
            let max = opened.max_scroll();
            if self.panes[slot].target() > max {
                until = until.max(self.panes[slot].toward(max, now));
            }
            until = until.max(self.opened_top.toward(opened.top, now));
        }
        self.plan = plan;
        self.hold(until);
    }

    fn sync_preview(&mut self, palette: Option<StudioPalette>) {
        if self.picking.is_some() {
            return;
        }
        let now = self.now;
        let mut until = f32::NEG_INFINITY;
        if let Some(palette) = palette {
            let key = (palette, self.view, self.wallpaper.key.clone());
            let scene = self.scene(palette, has_terminal(&self.model.source));
            until = self.scenes.show(key, scene, now);
        }
        if let Some(live) = self.model.live {
            let key = (live, self.view, self.wallpaper.key.clone());
            let scene = self.scene(live, true);
            until = until.max(self.compared.show(key, scene, now));
        }
        self.hold(until);
    }

    fn sync_colours(&mut self, palette: Option<&StudioPalette>) {
        let now = self.now;
        let mut until = f32::NEG_INFINITY;
        if let Some(palette) = palette {
            for (slot, role) in self.swatches.iter_mut().zip(SWATCHES) {
                until = until.max(slot.toward(rgb_color(palette.get(role)), now));
            }
            let wheel = &mut self.wheel;
            for ((x, y), (tinted, role)) in
                wheel.dots.iter_mut().zip(wheel.tints.iter_mut().zip(ACCENTS))
            {
                let color = palette.get(role);
                let lch = to_oklch(color);
                let reach = (lch.chroma / FULL_CHROMA).min(1.0);
                let angle = lch.hue.to_radians();
                until = until.max(x.toward(reach * angle.cos(), now));
                until = until.max(y.toward(-reach * angle.sin(), now));
                until = until.max(tinted.toward(rgb_color(color), now));
            }
        }
        let strengths = self.model.wheel().to_vec();
        for (index, glide) in self.wheel.ticks.iter_mut().enumerate() {
            let strength = strengths.get(index).copied().unwrap_or_default();
            until = until.max(glide.toward(strength, now));
        }
        let shape = self.model.shape();
        if shape != self.wheel.shape {
            self.wheel.was = std::mem::replace(&mut self.wheel.shape, shape);
            self.wheel.shape_at = now;
            until = until.max(now + SLOW);
        }
        self.hold(until);
    }

    fn sync_texts(&mut self) {
        let now = self.now;
        let quiet = self.quiet();
        let pick = text::pick_tag(self);
        let apply = text::apply(self.undoable());
        let mut until = self.swaps.pick.set(pick, 1.0, now, false);
        until = until.max(self.swaps.apply.set(apply.to_string(), 1.0, now, false));
        for row in Row::ALL {
            let value = text::value(self, row);
            let row_dir = self.row_dirs[row.index()];
            until = until.max(self.swaps.values[row.index()].set(value, row_dir, now, quiet));
        }
        self.modes_label = text::modes_label(self);
        self.hold(until);
    }

    pub(super) fn sync_controls(&mut self) {
        let now = self.now;
        let mut until = f32::NEG_INFINITY;
        let band = self.tabs[self.row.index()];
        until = until.max(self.band[0].toward(band.x, now));
        until = until.max(self.band[1].toward(band.width, now));
        let left = super::draw::pop(self, self.row).rect.x;
        until = until.max(self.pop_x.toward(left, now));
        for knob in Knob::ALL {
            let share = super::draw::fraction(knob, self.knob(knob));
            let glide = &mut self.knobs[knob as usize];
            if self.knob_drag == Some(knob) {
                glide.jump(share);
            } else {
                until = until.max(glide.toward(share, now));
            }
        }
        until = until.max(self.switch.toward(if self.model.compare { 1.0 } else { 0.0 }, now));
        if self.sliding {
            self.split.jump(self.model.split);
        } else {
            until = until.max(self.split.toward(self.model.split, now));
        }
        let split = self.model.split;
        let shown = [split > super::tokens::NOW_TAG_FROM, split < super::tokens::NEW_TAG_UNTIL];
        for (glide, on) in self.tags.iter_mut().zip(shown) {
            until = until.max(glide.toward(if on { 1.0 } else { 0.0 }, now));
        }
        let picking = self.picking.as_ref();
        let veiled = if picking.is_some() { 1.0 } else { 0.0 };
        until = until.max(self.veil.toward(veiled, now));
        until = until.max(self.hidden.toward(veiled, now));
        let clip = picking
            .and_then(|pick| pick.draft)
            .map_or([0.0, 0.0, 1.0, 1.0], |area| [area.left, area.top, area.right, area.bottom]);
        let dragging = picking.is_some_and(|pick| pick.dragging);
        for (glide, edge) in self.clip.iter_mut().zip(clip) {
            if dragging {
                glide.jump(edge);
            } else {
                until = until.max(glide.toward(edge, now));
            }
        }
        self.hold(until);
    }

    pub fn sync(&mut self) {
        let palette = self.model.palette();
        self.shown = palette;
        self.raw = self.model.raw();
        let staged = palette.or(self.model.live.filter(|_| self.scenes.shown().is_none()));
        if self.ready_at.is_none() && staged.is_some() {
            self.ready_at = Some(self.now);
            self.hold(self.now + INTRO_END);
        }
        self.sync_preview(staged);
        let key = self.catalogue_key();
        if self.catalogue != Some(key.clone()) {
            self.sync_dots();
            self.refresh_saved();
            self.catalogue = Some(key);
        }
        self.sync_plan();
        self.sync_colours(palette.as_ref());
        self.sync_texts();
        self.sync_controls();
    }

    pub fn untuned(&self, role: Role) -> Option<StudioPalette> {
        self.model.untuned(role)
    }
}

#[cfg(test)]
pub(super) mod tests;
