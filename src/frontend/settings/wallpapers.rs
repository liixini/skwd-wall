#[cfg(test)]
#[path = "wallpapers_tests.rs"]
mod tests;

use std::collections::HashSet;

use crate::contracts::settings::drawing::{self, Phase};
use crate::contracts::settings::transitions::preset_duration_ms;
use crate::contracts::settings::{SettingsSource, keys};
use crate::i18n::{text, tr, tr_args};

use super::category::{APP, DISPLAYS, INTEGRATIONS, PLAYBACK, TRANSITIONS};
use super::places::{
    Asleep, Bind, Choice, LIST_KEY, PlaceSource, SUMMARY_JOINER, bind, checks, column, listed,
    looked, note, noted, park, rows, spoken, take,
};
use super::tables::{DRAWING_LIBRARY_SCOPE, DRAWING_SCOPE_KEY, SHADER_FAMILY_KEY};
use super::tabs::{
    RuntimeStatus, asleep_rows, niri_rows, plasma_lock_rows, take_card, take_control_paths,
};
use super::{ActionId, Cell, Control, Look, Note, Row, Say, Section, SectionId, Trim};

use keys::drawing as draw;
use keys::paper;
use keys::playback;
use keys::transition;
use keys::we_render as scenes;

pub(super) const BORROWED: [&str; 2] = [keys::performance::GPU_DEVICE, paper::VIDEO_MULTI_PROCESS];

pub(super) const DISPLAY_LIST: &str = "settings-tab-displays";
pub(super) const PLAY: &str = "settings-wallpapers-play";
pub(super) const MOTION: &str = "settings-section-wallpaper-transitions";
pub(super) const AWWW: &str = "settings-section-awww-transition";
pub(crate) const DRAWING: &str = "settings-wallpapers-drawing";
pub(super) const SOUND: &str = "settings-wallpapers-sound";
pub(super) const PAUSE: &str = "settings-playback-pause-title";
pub(super) const PLASMA: &str = "settings-wallpapers-plasma";
pub(super) const NIRI: &str = "settings-wallpapers-niri";

const DISPLAYS_CARD: &str = "settings-displays-card";
const ENGINE_CARD: &str = "settings-section-display";
const AUDIO_CARD: &str = "settings-section-audio";
const SCENES_CARD: &str = "settings-section-workshop-rendering";
const MOTION_CARD: &str = "settings-section-wallpaper-transitions";
const SAND_CARD: &str = "settings-section-transition-performance";
const DRAWING_CARD: &str = "settings-drawing-card";
const MOTION_TITLE: &str = "settings-transitions-transitions-card";

const DISPLAYS_DESC: &str = "settings-wallpapers-displays-desc";
const DISPLAYS_SAY: &str = "settings-wallpapers-displays-say";
const LOCK_TITLE: &str = "settings-wallpapers-lock";
const LOCK_HINT: &str = "settings-wallpapers-lock-hint";
const PLAY_DESC: &str = "settings-wallpapers-play-desc";
const PLAY_SAY: &str = "settings-wallpapers-play-say";
const LAYER_BOTTOM: &str = "settings-wallpapers-layer-bottom";
const GPU_HINT: &str = "settings-wallpapers-gpu-hint";
const TIMEOUT_HINT: &str = "settings-wallpapers-timeout-hint";
const ANIMATED: &str = "settings-wallpapers-animated";
const SCENES_TITLE: &str = "settings-wallpapers-scenes";
const SCENES_HINT: &str = "settings-wallpapers-scenes-hint";
const PARTICLES_HINT: &str = "settings-wallpapers-particles-hint";
const MOTION_DESC: &str = "settings-wallpapers-transitions-desc";
const MOTION_SAY: &str = "settings-wallpapers-transitions-say";
const MOTION_OFF: &str = "settings-wallpapers-transitions-off";
const MOTION_OFF_FOOT: &str = "settings-wallpapers-transitions-off-foot";
const PREVIEW: &str = "settings-transitions-preview-card";
const PREVIEW_TITLE: &str = "settings-transitions-show-preview-label";
const PREVIEW_HINT: &str = "settings-wallpapers-preview-hint";
const TIMING: &str = "settings-automation-timing";
const SAND: &str = "settings-transitions-family-sand";
const QUALITY_SHORT: &str = "settings-wallpapers-quality-short";
const QUALITY_HINT: &str = "settings-wallpapers-quality-hint";
const SHARPEN_HINT: &str = "settings-wallpapers-sharpen-hint";
const MONITORS: &str = "settings-wallpapers-monitors";
const PRIMARY_HINT: &str = "settings-wallpapers-primary-hint";
const PRIMARY_BLANK: &str = "settings-wallpapers-primary-blank";
const PRIMARY_ONLY: &str = "settings-transitions-scope-primary";
const PRIMARY_ONLY_HINT: &str = "settings-wallpapers-primary-only-hint";
const PRIMARY_COUNT: &str = "settings-wallpapers-primary-count";
const AWWW_DESC: &str = "settings-wallpapers-awww-desc";
const AWWW_SAY: &str = "settings-wallpapers-awww-say";
const AWWW_NONE: &str = "settings-wallpapers-awww-none";
const AWWW_NO_TYPE: &str = "settings-wallpapers-awww-no-type";
const WAVE_TITLE: &str = "settings-wallpapers-wave";
const WAVE_HINT: &str = "settings-wallpapers-wave-hint";
const POSITION_HINT: &str = "settings-wallpapers-position-hint";
const BEZIER_HINT: &str = "settings-wallpapers-bezier-hint";
const BEZIER_BLANK: &str = "settings-wallpapers-bezier-blank";
const INVERT_HINT: &str = "settings-wallpapers-invert-hint";
const DRAWING_DESC: &str = "settings-wallpapers-drawing-desc";
const DRAWING_SAY: &str = "settings-wallpapers-drawing-say";
const DRAWING_BLOCKED: &str = "settings-wallpapers-drawing-blocked";
const COLOURS_FILES: &str = "settings-wallpapers-colours-files";
const PREPARING: &str = "settings-wallpapers-preparing";
const PAPER_INK: &str = "settings-wallpapers-paper-ink";
const PREPARE: &str = "settings-wallpapers-prepare";
const SCOPE_VIEW: &str = "settings-wallpapers-scope-view";
const SCOPE_LIBRARY: &str = "settings-wallpapers-prepare-all";
const PREPARE_ROW: &str = "settings-drawing-scope-label";
const PACK: &str = "settings-drawing-pack-label";
const PACK_READY: &str = "settings-wallpapers-pack-ready";
const PACK_NONE: &str = "settings-wallpapers-pack-none";
const PACK_BLANK: &str = "settings-wallpapers-pack-blank";
const CACHE: &str = "settings-drawing-cache-label";
const UNAVAILABLE: &str = "settings-drawing-unavailable-label";
const PROGRESS: &str = "settings-drawing-progress-label";
const DEVICE_HINT: &str = "settings-wallpapers-device-hint";
const MEMORY_HINT: &str = "settings-wallpapers-memory-hint";
const SOUND_DESC: &str = "settings-wallpapers-sound-desc";
const SOUND_SAY: &str = "settings-wallpapers-sound-say";
const SOUND_MUTED: &str = "settings-wallpapers-sound-muted";
const VOLUME_HEADING: &str = "settings-wallpapers-volume";
const OTHER_APPS: &str = "settings-wallpapers-other-apps";
const PAUSE_DESC: &str = "settings-wallpapers-pause-desc";
const PAUSE_SAY: &str = "settings-wallpapers-pause-say";
const PAUSE_APPS: &str = "settings-wallpapers-pause-apps";
const PAUSE_NONE: &str = "settings-wallpapers-pause-none";
const PAUSE_RULES: &str = "settings-wallpapers-pause-rules";
const PROCESS_HINT: &str = "settings-wallpapers-process-hint";
const PROCESS_ACTION: &str = "settings-wallpapers-process-action";
const PROCESS_BLANK: &str = "settings-wallpapers-process-blank";
const RESUME_HINT: &str = "settings-wallpapers-resume-hint";
const SCOPE_ALL: &str = "settings-wallpapers-scope-all";
const SCOPE_DISPLAY: &str = "settings-wallpapers-scope-display";
const PLASMA_DESC: &str = "settings-wallpapers-plasma-desc";
const PLASMA_SAY: &str = "settings-wallpapers-plasma-say";
const PLASMA_LEFT: &str = "settings-wallpapers-plasma-left";
const PLASMA_STATIC: &str = "settings-wallpapers-plasma-static";
const PLASMA_FOLLOW: &str = "settings-wallpapers-plasma-follow";
const PLASMA_OFF_ROW: &str = "settings-wallpapers-plasma-off-row";
const IMAGE_HINT: &str = "settings-wallpapers-image-hint";
const IMAGE_BLANK: &str = "settings-wallpapers-image-blank";
const DYNAMIC_HINT: &str = "settings-wallpapers-dynamic-hint";
const NIRI_DESC: &str = "settings-wallpapers-niri-desc";
const NIRI_SAY: &str = "settings-wallpapers-niri-say";
const NIRI_OFF: &str = "settings-wallpapers-niri-off";
const SOURCE: &str = "settings-wallpapers-source";
const LOOK: &str = "settings-wallpapers-look";
const SOURCE_BLANK: &str = "settings-wallpapers-source-blank";
const BLUR: &str = "settings-wallpapers-blur";
const RADII: &str = "settings-wallpapers-radii";
const RADII_HINT: &str = "settings-wallpapers-radii-hint";
const DIM_HINT: &str = "settings-wallpapers-dim-hint";
const RULE: &str = "settings-niri-rule-label";

const LOCKS: [&str; 3] = ["none", "one", "some"];
const DETECTING: &str = "detecting";
const COUNTS: [&str; 3] = ["none", "all", "some"];
const STEP_HINT: &str = "settings-paper-awww-step-desc";
const LAYER_SLOT: Bind<'_> = Bind::new("layer", paper::WALLPAPER_LAYER);
const SCOPE_SLOT: Bind<'_> = Bind::new("scope", playback::FULLSCREEN_SCOPE);
const MODE_SLOT: Bind<'_> = Bind::new("mode", keys::plasma::LOCK_SCREEN_MODE);
const BACKEND_SLOT: Bind<'_> = Bind::new("backend", paper::ENGINE);
const FILL_SLOT: Bind<'_> = Bind::new("fill", keys::display::FILL_MODE);
const FAMILY_SLOT: Bind<'_> = Bind::new("family", SHADER_FAMILY_KEY);
const SHADER_SLOT: Bind<'_> = Bind::new("shader", transition::SHADER);
pub(super) const DURATION_SLOT: Bind<'_> = Bind::new("duration", transition::DURATION_MS);
const RATE_SLOT: Bind<'_> = Bind::new("rate", transition::FPS);
const DRAW_SLOT: Bind<'_> = Bind::new("duration", draw::DURATION_MS);
const PREPARE_ROWS: [(&str, Option<&str>); 3] = [
    (draw::AUTO_PREPARE, None),
    (draw::DEVICE, Some(DEVICE_HINT)),
    (draw::MEMORY_LIMIT_MIB, Some(MEMORY_HINT)),
];
pub(super) const VOLUME_SLOT: Bind<'_> = Bind::new("volume", keys::wallpaper::VOLUME);
const DIM_SLOT: Bind<'_> = Bind::new("dim", keys::niri::BACKDROP_DIM);
const IMAGE_SLOT: Bind<'_> =
    Bind { blank: Some(IMAGE_BLANK), ..Bind::new("image", keys::plasma::LOCK_SCREEN_IMAGE) };
const AWWW_SLOTS: [Bind<'_>; 3] = [
    Bind::new("type", paper::AWWW_TRANSITION_TYPE),
    Bind::new("filter", paper::AWWW_FILTER),
    Bind::new("duration", paper::AWWW_TRANSITION_DURATION_MS),
];

const DRAWN: Choice = Choice { arg: "state", values: ["off", "on"] };
const DUCKS: Choice = Choice { arg: "duck", values: ["no", "yes"] };
const LISTED: Choice = Choice { arg: "apps", values: ["no", "yes"] };
const SCENES_AS: Choice = Choice { arg: "scenes", values: ["poster", "live"] };
const BLURRED: Choice = Choice { arg: "blur", values: ["no", "yes"] };
const RECOLOURED: Choice = Choice { arg: "auto", values: ["no", "yes"] };
const DIM_CELL: usize = 2;
const STATE_ARG: &str = "state";
const NAMES_ARG: &str = "names";
const KIND_ARG: &str = "kind";
const MS_ARG: &str = "ms";
const RULES_ARG: &str = "rules";
const SHOWN_ARG: &str = "source";
const BOTTOM: &str = "bottom";
const RANDOM: &str = "random";
const NONE: &str = "none";
const SANDY_MS: u32 = 4000;
const TIMED: &str = "preset";
const SHADED: &str = "shader";
const LIVE: &str = "live";
const STATIC: &str = "static";
const FOLLOW: &str = "follow";
const OFF: &str = "off";
const STATIONARY: &str = "stationary";
const SCOPE_DISPLAY_VALUE: &str = "display";
const FOLLOWED: &str = "current";
const OWN: &str = "own";
const NIRI_PREFIX: &str = "niri.";
const AWWW_PREFIX: &str = "paper.awww";

const PAUSE_WINDOWS: [(&str, &str); 4] = [
    (playback::FULLSCREEN, "settings-wallpapers-rule-fullscreen"),
    (playback::MAXIMIZED, "settings-wallpapers-rule-maximised"),
    (playback::FOCUSED, "settings-wallpapers-rule-focused"),
    (keys::niri::FULL_WIDTH_PAUSE, "settings-wallpapers-rule-full-width"),
];
const NIRI_BLURS: [&str; 3] = [
    keys::niri::BACKDROP_BLUR_STATIC,
    keys::niri::BACKDROP_BLUR_VIDEO,
    keys::niri::BACKDROP_BLUR_WE,
];
const NIRI_RADII: [&str; 3] = [
    keys::niri::BACKDROP_BLUR_STATIC_RADIUS,
    keys::niri::BACKDROP_BLUR_VIDEO_RADIUS,
    keys::niri::BACKDROP_BLUR_WE_RADIUS,
];

pub(super) fn compose(source: &PlaceSource<'_>) -> Vec<Section> {
    let cfg = source.cfg;
    let mut playback = source.cards(PLAYBACK);
    let mut motion = source.cards(TRANSITIONS);
    let mut sections =
        vec![displays(source), play(source, &mut playback), transitions(cfg, &mut motion)];
    let awww_rows = take_card(&mut motion, text(AWWW));
    if !awww_rows.is_empty() {
        sections.push(awww(cfg, awww_rows));
    }
    sections.push(drawing(source, take_card(&mut motion, text(DRAWING_CARD))));
    sections.push(sound(cfg, take_card(&mut playback, text(AUDIO_CARD))));
    sections.push(pause(cfg, take_card(&mut playback, text(PAUSE))));
    sections.push(plasma(cfg));
    if cfg.is_niri() {
        sections.push(niri(source));
    }
    sections
}

pub(super) fn asleep(sections: &[Section], source: &PlaceSource<'_>) -> Vec<Asleep> {
    let mut seen: HashSet<String> = sections
        .iter()
        .flat_map(Section::rows)
        .filter_map(|row| row.control.path().map(str::to_string))
        .collect();
    asleep_rows(source.cfg, source.themes)
        .into_iter()
        .filter(|row| row.control.path().is_some_and(|path| seen.insert(path.to_string())))
        .filter_map(|row| {
            let wakes = wakes(row.control.path()?)?;
            Some(Asleep { row, wakes })
        })
        .collect()
}

fn wakes(path: &str) -> Option<&'static str> {
    match path {
        transition::PREVIEW_FPS => Some(transition::PREVIEW),
        transition::PRESET_TIMING => Some(SHADER_FAMILY_KEY),
        transition::DURATION_MS => Some(transition::PRESET_TIMING),
        transition::SHADER | transition::SAND_QUALITY | transition::SAND_SHARP => {
            Some(SHADER_FAMILY_KEY)
        }
        scenes::FPS | scenes::SCALING | scenes::DISABLE_PARTICLES => Some(keys::features::STEAM),
        keys::niri::OVERVIEW_ONLY_PLAYBACK => Some(keys::niri::OVERVIEW_MODE),
        keys::niri::FULL_WIDTH_PAUSE => Some(playback::FULLSCREEN),
        keys::plasma::LOCK_SCREEN_IMAGE | keys::plasma::LOCK_SCREEN_DYNAMIC => {
            Some(keys::plasma::LOCK_SCREEN_MODE)
        }
        _ if path.starts_with(transition::SHADER_SCOPES) => Some(SHADER_FAMILY_KEY),
        _ if path.starts_with(AWWW_PREFIX) => Some(paper::ENGINE),
        _ if path.starts_with(NIRI_PREFIX) => Some(keys::niri::OVERVIEW_BACKDROP),
        _ => None,
    }
}

fn headed(rows: Vec<Row>, title: &str, hint: Option<&str>) -> Vec<Row> {
    rows.into_iter()
        .map(|row| {
            let desc = [row.title.as_str(), row.desc.as_str()]
                .into_iter()
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            let note = hint.map_or(Note::Quiet, |hint| Note::Own(hint.to_string()));
            let own_title = Some(row.title).filter(|own| !own.is_empty());
            Row {
                title: title.to_string(),
                desc,
                look: Look { note, own_title, ..row.look },
                control: row.control,
            }
        })
        .collect()
}

pub(super) fn blanked(rows: Vec<Row>, blank: &'static str) -> Vec<Row> {
    rows.into_iter()
        .map(|row| Row { look: Look { blank: Some(tr(blank).to_string()), ..row.look }, ..row })
        .collect()
}

fn segment(title: &'static str) -> Row {
    Row::of(tr(title).to_string(), String::new(), Control::Segment)
}

fn phrased(slot: Bind<'static>, key: Option<&'static str>) -> Bind<'static> {
    Bind { blank: key, ..slot }
}

fn notes(section: &mut Section, lines: &[String]) {
    if !lines.is_empty() {
        section.say.notes.push(lines.join(SUMMARY_JOINER));
    }
}

fn displays(source: &PlaceSource<'_>) -> Section {
    let mut cards = source.cards(DISPLAYS);
    let mut pool = take_card(&mut cards, text(DISPLAYS_CARD));
    let head = take_control_paths(&mut pool, &[keys::display::FILL_MODE]);
    let (mut placements, mut locks, mut locked) = (Vec::new(), Vec::new(), Vec::new());
    for Row { title, desc, control, look } in pool {
        let Control::StackBar { summary, rows: mut inner, .. } = control else {
            placements.push(Row { title, desc, control, look });
            continue;
        };
        let hint = if summary.is_empty() { desc } else { [desc, summary].join(SUMMARY_JOINER) };
        let lock = take(&mut inner, |control| matches!(control, Control::Toggle { .. }));
        if lock.iter().any(|row| row.control.switched_on() == Some(true)) {
            locked.push(title.clone());
        }
        let shut = tr_args!(LOCK_TITLE, name => title.as_str());
        let lock_hint = locks.is_empty().then(|| tr(LOCK_HINT));
        locks.extend(headed(lock, &shut, lock_hint));
        let placement = take(&mut inner, |control| matches!(control, Control::Chips { .. }));
        placements.extend(headed(placement, &title, Some(&hint)));
        placements.extend(looked(noted(inner, None), Cell::Beside));
    }
    let detecting = !source.runtime.connected && source.runtime.statuses.is_empty();
    let state = match locked.len() {
        _ if detecting => DETECTING,
        0 => LOCKS[0],
        1 => LOCKS[1],
        _ => LOCKS[2],
    };
    let names: Vec<&str> = locked.iter().map(String::as_str).collect();
    let names = if names.is_empty() { String::new() } else { listed(LIST_KEY, &names) };
    let section = Section {
        head,
        groups: vec![column(None, placements), column(None, locks)],
        ..Section::new(SectionId::new(DISPLAYS, DISPLAY_LIST), tr(DISPLAY_LIST), tr(DISPLAYS_DESC))
    };
    spoken(section, |section| {
        bind(section, DISPLAYS_SAY, &[FILL_SLOT], &[(STATE_ARG, state), (NAMES_ARG, &names)])
    })
}

fn play(source: &PlaceSource<'_>, playback: &mut Vec<(super::Card, Vec<Row>)>) -> Section {
    let cfg = source.cfg;
    let mut pool = take_card(playback, text(ENGINE_CARD));
    let mut scene_rows = take_card(playback, text(SCENES_CARD));
    let mut borrowed: Vec<_> = source.cards(APP).into_iter().flat_map(|(_, rows)| rows).collect();
    let mut engine = rows(&mut borrowed, &[(keys::performance::GPU_DEVICE, Some(GPU_HINT))]);
    engine.extend(rows(&mut pool, &[(paper::LOAD_TIMEOUT_SECONDS, Some(TIMEOUT_HINT))]));
    engine.extend(rows(&mut borrowed, &[(paper::VIDEO_MULTI_PROCESS, None)]));
    let head = take_control_paths(&mut pool, &[paper::ENGINE, paper::WALLPAPER_LAYER]);
    let mut animated = headed(
        take_control_paths(&mut scene_rows, &[scenes::FPS]),
        tr(SCENES_TITLE),
        Some(tr(SCENES_HINT)),
    );
    animated.extend(looked(rows(&mut scene_rows, &[(scenes::SCALING, None)]), Cell::Beside));
    animated.extend(rows(&mut scene_rows, &[(scenes::DISABLE_PARTICLES, Some(PARTICLES_HINT))]));
    animated.extend(noted(pool, None));
    let section = Section {
        head,
        groups: vec![column(None, engine), column(Some(ANIMATED), animated)],
        trim: Trim::Bottom,
        ..Section::new(SectionId::new(PLAYBACK, PLAY), tr(PLAY), tr(PLAY_DESC))
    };
    let layer = cfg.text(paper::WALLPAPER_LAYER);
    let bottom = (layer.is_empty() || layer == BOTTOM).then_some(LAYER_BOTTOM);
    let layer = phrased(LAYER_SLOT, bottom);
    spoken(section, |section| bind(section, PLAY_SAY, &[BACKEND_SLOT, layer], &[]))
}

fn is_scope(control: &Control) -> bool {
    control.path().is_some_and(|path| path.starts_with(transition::SHADER_SCOPES))
        && matches!(control, Control::Dropdown { .. })
}

fn primary_only(pool: &mut Vec<Row>) -> Vec<Row> {
    let members = take(pool, is_scope);
    if members.is_empty() {
        return Vec::new();
    }
    let all = members.len();
    let picked: Vec<_> = members.iter().filter(|row| row.control.ticked() == Some(true)).collect();
    let state = match picked.len() {
        0 => COUNTS[0],
        count if count == all => COUNTS[1],
        _ => COUNTS[2],
    };
    let caption = if let [one] = picked.as_slice() {
        one.title.clone()
    } else {
        tr_args!(PRIMARY_COUNT, state => state, shown => picked.len(), all => all)
    };
    let mut lead = checks(tr(PRIMARY_ONLY), caption, members);
    if let Some(first) = lead.first_mut() {
        first.look.note = Note::Own(tr(PRIMARY_ONLY_HINT).to_string());
    }
    lead
}

fn transitions(cfg: &dyn SettingsSource, cards: &mut Vec<(super::Card, Vec<Row>)>) -> Section {
    let mut pool = take_card(cards, text(MOTION_CARD));
    pool.extend(take_card(cards, text(SAND_CARD)));
    let master = take_control_paths(&mut pool, &[transition::ENABLED]).pop();
    let on = master.as_ref().and_then(|row| row.control.switched_on()).unwrap_or(true);
    let shader = take_control_paths(&mut pool, &[transition::SHADER]);
    let head_paths = [SHADER_FAMILY_KEY, transition::DURATION_MS, transition::FPS];
    let mut head = take_control_paths(&mut pool, &head_paths[..1]);
    head.extend(shader);
    head.extend(take_control_paths(&mut pool, &head_paths[1..]));
    let preset = rows(&mut pool, &[(transition::PRESET_TIMING, None)]);
    let mut preview = headed(
        take_control_paths(&mut pool, &[transition::PREVIEW]),
        tr(PREVIEW_TITLE),
        Some(tr(PREVIEW_HINT)),
    );
    let rate = take_control_paths(&mut pool, &[transition::PREVIEW_FPS]);
    preview.extend(looked(noted(rate, None), Cell::Joined));
    let stage = take(&mut pool, |control| matches!(control, Control::Preview));
    let timed = !preset.is_empty();
    let timed_ms = preset_duration_ms(&cfg.text(transition::SHADER));
    let quality_hint = if cfg.number(transition::DURATION_MS) < f64::from(SANDY_MS) {
        tr_args!(QUALITY_SHORT, ms => SANDY_MS.to_string())
    } else {
        tr(QUALITY_HINT).to_string()
    };
    let mut sand_rows = noted(take_control_paths(&mut pool, &[transition::SAND_QUALITY]), None);
    for row in &mut sand_rows {
        row.look.note = Note::Own(quality_hint.clone());
    }
    sand_rows.extend(rows(&mut pool, &[(transition::SAND_SHARP, Some(SHARPEN_HINT))]));
    let primary_field =
        blanked(rows(&mut pool, &[(transition::SAND_PRIMARY, Some(PRIMARY_HINT))]), PRIMARY_BLANK);
    let family = cfg.text(transition::SHADER);
    let random = family.is_empty() || family == RANDOM;
    let scoped = if random { Vec::new() } else { primary_only(&mut pool) };
    let unscoped = take(&mut pool, is_scope);
    let mut left = preview;
    if timed {
        left.push(segment(TIMING));
        left.extend(preset);
    }
    if !sand_rows.is_empty() {
        left.push(segment(SAND));
        left.extend(sand_rows);
    }
    let mut right = primary_field;
    right.extend(scoped);
    right.extend(stage);
    let has_duration = head.iter().any(|row| row.control.path() == Some(transition::DURATION_MS));
    let kind = if random {
        RANDOM
    } else if has_duration {
        SHADED
    } else {
        TIMED
    };
    let mut slots = vec![FAMILY_SLOT];
    if !random {
        slots.push(SHADER_SLOT);
    }
    if has_duration {
        slots.push(DURATION_SLOT);
    }
    slots.push(RATE_SLOT);
    let id = SectionId::new(TRANSITIONS, MOTION);
    head.extend(unscoped);
    let mut section = Section {
        master,
        head,
        groups: vec![column(Some(PREVIEW), left), column(Some(MONITORS), right)],
        ..Section::new(id, tr(MOTION_TITLE), tr(MOTION_DESC))
    };
    if !on {
        let parked: Vec<_> = std::mem::take(&mut section.groups)
            .into_iter()
            .flat_map(|group| group.rows)
            .filter(|row| {
                !matches!(row.control, Control::Segment | Control::Preview | Control::Static)
            })
            .collect();
        section.head.extend(parked);
        section.foot = Some(tr(MOTION_OFF_FOOT).to_string());
        let mut section =
            spoken(section, |_| Some(Say { text: tr(MOTION_OFF).to_string(), ..Say::default() }));
        park(&mut section, transition::ENABLED, |_| true);
        return section;
    }
    let ms = timed_ms.unwrap_or_default().to_string();
    let mut section = spoken(section, |section| {
        bind(section, MOTION_SAY, &slots, &[(KIND_ARG, kind), (MS_ARG, &ms)])
    });
    park(&mut section, SHADER_FAMILY_KEY, |row| is_scope(&row.control));
    section
}

fn awww(cfg: &dyn SettingsSource, mut pool: Vec<Row>) -> Section {
    let head = take_control_paths(&mut pool, &AWWW_SLOTS.map(|slot| slot.path));
    let left = rows(
        &mut pool,
        &[
            (paper::AWWW_TRANSITION_FPS, None),
            (paper::AWWW_TRANSITION_STEP, Some(STEP_HINT)),
            (paper::AWWW_TRANSITION_ANGLE, None),
        ],
    );
    let mut right = headed(
        take_control_paths(&mut pool, &[paper::AWWW_TRANSITION_WAVE_WIDTH]),
        tr(WAVE_TITLE),
        Some(tr(WAVE_HINT)),
    );
    right.extend(looked(
        rows(&mut pool, &[(paper::AWWW_TRANSITION_WAVE_HEIGHT, None)]),
        Cell::Joined,
    ));
    right.extend(rows(&mut pool, &[(paper::AWWW_TRANSITION_POS, Some(POSITION_HINT))]));
    right.extend(blanked(
        rows(&mut pool, &[(paper::AWWW_TRANSITION_BEZIER, Some(BEZIER_HINT))]),
        BEZIER_BLANK,
    ));
    right.extend(rows(&mut pool, &[(paper::AWWW_INVERT_Y, Some(INVERT_HINT))]));
    let section = Section {
        head,
        groups: vec![column(None, left), column(None, right)],
        trim: Trim::Bottom,
        ..Section::new(SectionId::new(PLAYBACK, AWWW), tr(AWWW), tr(AWWW_DESC))
    };
    let current = cfg.text(paper::AWWW_TRANSITION_TYPE);
    if current == NONE {
        let slot = phrased(AWWW_SLOTS[0], Some(AWWW_NO_TYPE));
        let mut section = spoken(section, |section| bind(section, AWWW_NONE, &[slot], &[]));
        park(&mut section, paper::AWWW_TRANSITION_TYPE, |_| true);
        return section;
    }
    spoken(section, |section| bind(section, AWWW_SAY, &AWWW_SLOTS, &[]))
}

struct Prepared {
    files: Vec<Row>,
    hidden: Vec<Row>,
    prepare: Vec<Row>,
    running: Vec<Row>,
    preview: Vec<Row>,
}

pub(super) fn acted(pool: &mut Vec<Row>, wanted: impl Fn(&ActionId) -> bool) -> Vec<Row> {
    take(pool, |control| matches!(control, Control::ActionBtn { id, .. } if wanted(id)))
}

struct PrepareSpec<'a> {
    missing: bool,
    install: bool,
    path: &'a str,
    size: Option<&'a str>,
    library: bool,
    busy: bool,
}

fn prepared(pool: &mut Vec<Row>, spec: &PrepareSpec<'_>) -> Prepared {
    let choose = acted(pool, |id| *id == ActionId::ChooseDrawingPack);
    let detach = acted(pool, |id| *id == ActionId::DetachDrawingPack);
    let (shown, mut hidden) =
        if spec.missing || detach.is_empty() { (choose, detach) } else { (detach, choose) };
    let hint = match spec.size {
        Some(size) if !spec.path.is_empty() => [spec.path, size].join(SUMMARY_JOINER),
        Some(size) => size.to_string(),
        None if spec.install => tr(super::DRAWING_PACK_INSTALL).to_string(),
        None => tr(PACK_BLANK).to_string(),
    };
    let files = headed(shown, tr(PACK), Some(&hint));
    let mut single = acted(pool, |id| *id == ActionId::PrepareDrawing);
    for row in &mut single {
        if let Control::ActionBtn { label, .. } = &mut row.control {
            *label = tr(PREPARE).to_string();
        }
    }
    let every = acted(pool, |id| *id == ActionId::PrepareAllDrawings);
    let resume = acted(pool, |id| *id == ActionId::ResumeDrawing);
    let pause = acted(pool, |id| *id == ActionId::PauseDrawing);
    let cancel = acted(pool, |id| *id == ActionId::CancelDrawing);
    let mut running = resume;
    running.extend(pause);
    running.extend(cancel);
    let mut running = running.into_iter();
    let lead = running.next().map(|row| headed(vec![row], tr(PROGRESS), None));
    let mut rest = lead.unwrap_or_default();
    rest.extend(looked(noted(running.collect(), None), Cell::Joined));
    let (prepare, dormant) =
        if spec.library || spec.busy { (every, single) } else { (single, every) };
    hidden.extend(dormant);
    let preview =
        looked(noted(acted(pool, |id| *id == ActionId::PreviewDrawing), None), Cell::Value);
    Prepared {
        files,
        hidden,
        prepare: looked(noted(prepare, None), Cell::Joined),
        running: rest,
        preview,
    }
}

fn scoped(rows: Vec<Row>, runtime: RuntimeStatus<'_>) -> Vec<Row> {
    rows.into_iter()
        .map(|mut row| {
            if let Control::Dropdown { path, options, current, .. } = &mut row.control {
                *path = DRAWING_SCOPE_KEY.to_string();
                for (value, label) in options.iter_mut() {
                    if value == draw::SCOPE_VIEW {
                        *label = tr_args!(SCOPE_VIEW, count => runtime.drawing_view_count);
                    }
                }
                options.push((DRAWING_LIBRARY_SCOPE.to_string(), tr(SCOPE_LIBRARY).to_string()));
                if runtime.drawing_job.library {
                    *current = DRAWING_LIBRARY_SCOPE.to_string();
                }
            }
            row
        })
        .collect()
}

fn drawing(source: &PlaceSource<'_>, mut pool: Vec<Row>) -> Section {
    let cfg = source.cfg;
    let status = source.runtime.drawing;
    let master = take_control_paths(&mut pool, &[draw::ENABLED]).pop();
    let duration = take_control_paths(&mut pool, &[draw::DURATION_MS]);
    let battery = rows(&mut pool, &[(draw::ON_BATTERY, None)]);
    let auto = rows(&mut pool, &PREPARE_ROWS);
    let mut colours =
        headed(take_control_paths(&mut pool, &[draw::PAPER_COLOR]), tr(PAPER_INK), None);
    colours.extend(looked(
        noted(take_control_paths(&mut pool, &[draw::INK_COLOR]), None),
        Cell::Beside,
    ));
    colours.extend(rows(&mut pool, &[(draw::INK, None)]));
    let scope = scoped(noted(take_control_paths(&mut pool, &[draw::SCOPE]), None), source.runtime);
    let selection = take(&mut pool, |control| matches!(control, Control::StackBar { .. }));
    let statics = take(&mut pool, |control| matches!(control, Control::Static));
    let pack_state = status.map_or(drawing::PackState::Missing, |status| status.pack);
    let missing = pack_state != drawing::PackState::Available;
    let is_path = |row: &Row| status.is_some_and(|status| status.pack_path == row.title);
    let path_row = statics.iter().find(|row| is_path(row));
    let busy = source.runtime.drawing_job.running
        || status.is_some_and(|status| matches!(status.phase, Phase::Preparing | Phase::Paused));
    let spec = PrepareSpec {
        missing,
        install: status.is_some_and(drawing::Status::pack_missing),
        path: path_row.map_or("", |row| row.title.as_str()),
        size: path_row.map(|row| row.desc.as_str()),
        library: source.runtime.drawing_job.library,
        busy,
    };
    let plan = prepared(&mut pool, &spec);
    let clear = acted(&mut pool, |id| *id == ActionId::ClearDrawingCache);
    let cache_row = statics.iter().find(|row| row.title == tr(CACHE));
    let cache_size = cache_row.map(|row| row.desc.as_str());
    let cache = if clear.is_empty() {
        cache_row.cloned().into_iter().collect()
    } else {
        headed(clear, tr(CACHE), cache_size)
    };
    let blocked = statics.iter().any(|row| row.title == tr(UNAVAILABLE));
    let quiet = status.is_none_or(|status| matches!(status.phase, Phase::Idle | Phase::Ready));
    let mut lines = Vec::new();
    if !blocked && (quiet || pack_state == drawing::PackState::Invalid) {
        let pack = statics.iter().find(|row| row.title == tr(PACK));
        lines.push(match pack_state {
            drawing::PackState::Available => tr(PACK_READY).to_string(),
            drawing::PackState::Missing => tr(PACK_NONE).to_string(),
            drawing::PackState::Invalid => pack.map_or_else(String::new, |row| row.desc.clone()),
        });
    }
    for row in &statics {
        let known = row.title == tr(PACK)
            || row.title == tr(CACHE)
            || row.title == tr(UNAVAILABLE)
            || is_path(row);
        if !known {
            lines.push(note(row));
        }
    }
    if source.runtime.drawing_selection.is_some() {
        for row in &selection {
            if let Control::StackBar { summary, .. } = &row.control {
                lines.push([summary.as_str(), row.desc.as_str()].join(SUMMARY_JOINER));
            }
        }
    }
    let lines: Vec<_> = lines.into_iter().filter(|line| !line.is_empty()).collect();
    let mut prepare_row = headed(scope, tr(PREPARE_ROW), None);
    prepare_row.extend(looked(noted(plan.prepare, None), Cell::Joined));
    let mut left = colours;
    left.extend(plan.files);
    left.extend(cache);
    left.extend(plan.preview);
    let mut right = prepare_row;
    right.extend(plan.running);
    right.extend(battery);
    right.extend(auto);
    let mut head = duration;
    head.extend(plan.hidden);
    let mut section = Section {
        master,
        head,
        groups: vec![column(Some(COLOURS_FILES), left), column(Some(PREPARING), right)],
        trim: Trim::Bottom,
        ..Section::new(SectionId::new(TRANSITIONS, DRAWING), tr(DRAWING), tr(DRAWING_DESC))
    };
    section.head.extend(selection);
    let on = cfg.flag(draw::ENABLED);
    let state = DRAWN.pick(on);
    let mut section = spoken(section, |section| {
        if on && blocked {
            return Some(Say { text: tr(DRAWING_BLOCKED).to_string(), ..Say::default() });
        }
        bind(section, DRAWING_SAY, &[DRAW_SLOT], &[state])
    });
    park(&mut section, draw::ENABLED, |row| {
        row.control.path() != Some(draw::DURATION_MS) || (on && blocked)
    });
    notes(&mut section, &lines);
    section
}

fn sound(cfg: &dyn SettingsSource, mut pool: Vec<Row>) -> Section {
    let head = take_control_paths(&mut pool, &[keys::wallpaper::VOLUME]);
    let statuses = take(&mut pool, |control| matches!(control, Control::Static));
    let mute = rows(&mut pool, &[(keys::wallpaper::MUTE, None)]);
    let duck = rows(&mut pool, &[(playback::MUTE_ON_OTHER_AUDIO, None)]);
    let echo = noted(head.clone(), None).into_iter().next();
    let mut volume = column(Some(VOLUME_HEADING), mute);
    volume.echoes = echo.map(|row| (0, row)).into_iter().collect();
    let section = Section {
        head,
        groups: vec![volume, column(Some(OTHER_APPS), duck)],
        ..Section::new(SectionId::new(PLAYBACK, SOUND), tr(SOUND), tr(SOUND_DESC))
    };
    let muted = cfg.flag(keys::wallpaper::MUTE);
    let duck = DUCKS.pick(cfg.flag(playback::MUTE_ON_OTHER_AUDIO));
    let mut section = spoken(section, |section| {
        if muted {
            return Some(Say { text: tr(SOUND_MUTED).to_string(), ..Say::default() });
        }
        bind(section, SOUND_SAY, &[VOLUME_SLOT], &[duck])
    });
    if muted {
        park(&mut section, keys::wallpaper::MUTE, |_| true);
    }
    notes(&mut section, &statuses.iter().map(note).collect::<Vec<_>>());
    section
}

fn pause(cfg: &dyn SettingsSource, mut pool: Vec<Row>) -> Section {
    let head = take_control_paths(&mut pool, &[playback::FULLSCREEN_SCOPE]);
    let statuses = take(&mut pool, |control| matches!(control, Control::Static));
    let windows = PAUSE_WINDOWS.map(|(path, _)| (path, None));
    let left = rows(&mut pool, &windows);
    let mut right = rows(&mut pool, &[(playback::PROCESS_ENABLED, None)]);
    let chooser = acted(&mut pool, |id| *id == ActionId::ChooseRunningProcess);
    let mut chooser = blanked(noted(chooser, Some(PROCESS_HINT)), PROCESS_BLANK);
    for row in &mut chooser {
        row.title = tr("settings-playback-processes").to_string();
        if let Control::ActionBtn { label, .. } = &mut row.control {
            *label = tr(PROCESS_ACTION).to_string();
        }
    }
    right.extend(chooser);
    right.extend(rows(&mut pool, &[(playback::RESUME_DELAY, Some(RESUME_HINT))]));
    let section = Section {
        head,
        groups: vec![column(None, left), column(None, right)],
        ..Section::new(SectionId::new(PLAYBACK, PAUSE), tr(PAUSE), tr(PAUSE_DESC))
    };
    let picked: Vec<_> = PAUSE_WINDOWS
        .iter()
        .filter(|(path, _)| {
            cfg.flag(path) && (*path != keys::niri::FULL_WIDTH_PAUSE || cfg.is_niri())
        })
        .map(|(_, rule)| tr(rule))
        .collect();
    let apps = cfg.flag(playback::PROCESS_ENABLED);
    let scope_value = cfg.text(playback::FULLSCREEN_SCOPE);
    let scope_key = if scope_value == SCOPE_DISPLAY_VALUE { SCOPE_DISPLAY } else { SCOPE_ALL };
    let slot = phrased(SCOPE_SLOT, Some(scope_key));
    let rules = match picked.as_slice() {
        [one] => (*one).to_string(),
        many => tr_args!(PAUSE_RULES, count => many.len()),
    };
    let mut section = spoken(section, |section| {
        if !picked.is_empty() {
            let args = [(RULES_ARG, rules.as_str()), LISTED.pick(apps)];
            return bind(section, PAUSE_SAY, &[slot], &args);
        }
        let text = tr(if apps { PAUSE_APPS } else { PAUSE_NONE }).to_string();
        Some(Say { text, ..Say::default() })
    });
    if picked.is_empty() {
        park(&mut section, playback::FULLSCREEN, |_| true);
    }
    notes(&mut section, &statuses.iter().map(note).collect::<Vec<_>>());
    section
}

fn plasma(cfg: &dyn SettingsSource) -> Section {
    let mut pool = plasma_lock_rows(cfg);
    let mode = cfg.text(keys::plasma::LOCK_SCREEN_MODE);
    let mut head = take_control_paths(&mut pool, &[keys::plasma::LOCK_SCREEN_MODE]);
    let image = take_control_paths(&mut pool, &[keys::plasma::LOCK_SCREEN_IMAGE]);
    let image = blanked(image, IMAGE_BLANK);
    let echo = headed(
        image.clone(),
        &image.first().map_or_else(String::new, |row| row.title.clone()),
        Some(tr(IMAGE_HINT)),
    );
    let dynamic = rows(&mut pool, &[(keys::plasma::LOCK_SCREEN_DYNAMIC, Some(DYNAMIC_HINT))]);
    head.extend(image);
    head.append(&mut pool);
    let phrase = match mode.as_str() {
        OFF => Some(PLASMA_LEFT),
        STATIC => Some(PLASMA_STATIC),
        FOLLOW => Some(PLASMA_FOLLOW),
        _ => None,
    };
    let mode_slot = phrased(MODE_SLOT, phrase);
    let mut body = column(None, dynamic);
    if mode == OFF {
        body.rows.push(Row::of(tr(PLASMA_OFF_ROW).to_string(), String::new(), Control::Static));
    }
    body.echoes = echo.into_iter().next().map(|row| (0, row)).into_iter().collect();
    let body_empty = body.is_empty();
    let section = Section {
        head,
        groups: if body_empty { Vec::new() } else { vec![body] },
        ..Section::new(SectionId::new(INTEGRATIONS, PLASMA), tr(PLASMA), tr(PLASMA_DESC))
    };
    let live = SCENES_AS.pick(cfg.text(keys::plasma::LOCK_SCREEN_DYNAMIC) == LIVE);
    let mut slots = vec![mode_slot];
    if mode == STATIC {
        slots.push(IMAGE_SLOT);
    }
    let mut section = spoken(section, |section| {
        bind(section, PLASMA_SAY, &slots, &[live, (STATE_ARG, mode.as_str())])
    });
    park(&mut section, keys::plasma::LOCK_SCREEN_MODE, |row| {
        matches!(row.control, Control::ActionBtn { .. })
    });
    section
}

fn niri(source: &PlaceSource<'_>) -> Section {
    let cfg = source.cfg;
    let mut pool = niri_rows(cfg, source.themes);
    let master = take_control_paths(&mut pool, &[keys::niri::OVERVIEW_BACKDROP]).pop();
    let stationary = cfg.text(keys::niri::OVERVIEW_MODE) == STATIONARY;
    let dim = take_control_paths(&mut pool, &[keys::niri::BACKDROP_DIM]);
    let echo = noted(dim.clone(), Some(DIM_HINT)).into_iter().next();
    let code = take(&mut pool, |control| matches!(control, Control::Code { .. }));
    let statics = take(&mut pool, |control| matches!(control, Control::Static));
    let mut left = rows(&mut pool, &[(keys::niri::OVERVIEW_MODE, None)]);
    left.extend(rows(&mut pool, &[(keys::niri::BACKDROP_FOLLOW_WALLPAPER, None)]));
    let source_row = blanked(rows(&mut pool, &[(keys::niri::BACKDROP, None)]), SOURCE_BLANK);
    let refresh = noted(acted(&mut pool, |id| *id == ActionId::RefreshBackdrop), None);
    left.extend(if source_row.is_empty() {
        refresh
    } else {
        source_row.into_iter().chain(looked(refresh, Cell::Joined)).collect()
    });
    left.extend(headed(acted(&mut pool, |id| *id == ActionId::CopyLayerRule), tr(RULE), None));
    let blurs = take_control_paths(&mut pool, &NIRI_BLURS);
    let mut radii = take_control_paths(&mut pool, &NIRI_RADII);
    let mut radius_row = Vec::new();
    if !radii.is_empty() {
        let first = radii.remove(0);
        radius_row.extend(headed(vec![first], tr(RADII), Some(tr(RADII_HINT))));
        radius_row.extend(looked(noted(radii, None), Cell::Beside));
    }
    let mut right = checks(tr(BLUR), String::new(), blurs);
    right.extend(radius_row);
    let auto = rows(&mut pool, &[(keys::niri::BACKDROP_AUTO_THEME, None)]);
    let theme = noted(take_control_paths(&mut pool, &[keys::niri::BACKDROP_THEME]), None);
    let recolours = cfg.flag(keys::niri::BACKDROP_AUTO_THEME);
    let (theme, sleeping) = if recolours { (theme, Vec::new()) } else { (Vec::new(), theme) };
    right.extend(auto);
    right.extend(looked(theme, Cell::Joined));
    let mut look = column(Some(LOOK), right);
    look.echoes = echo.map(|row| (DIM_CELL, row)).into_iter().collect();
    let mut head = dim;
    head.extend(code);
    head.extend(sleeping);
    head.extend(noted(pool, None));
    let section = Section {
        master,
        head,
        groups: vec![column(Some(SOURCE), left), look],
        trim: Trim::Bottom,
        ..Section::new(SectionId::new(INTEGRATIONS, NIRI), tr(NIRI), tr(NIRI_DESC))
    };
    let shown = if stationary {
        STATIONARY
    } else if cfg.flag(keys::niri::BACKDROP_FOLLOW_WALLPAPER) {
        FOLLOWED
    } else {
        OWN
    };
    let blurred = BLURRED.pick(NIRI_BLURS.iter().any(|path| cfg.flag(path)));
    let recoloured = RECOLOURED.pick(!stationary && cfg.flag(keys::niri::BACKDROP_AUTO_THEME));
    let args = [(SHOWN_ARG, shown), blurred, recoloured];
    let off = section.switched_off();
    let mut section = spoken(section, |section| {
        if section.switched_off() {
            return Some(Say { text: tr(NIRI_OFF).to_string(), ..Say::default() });
        }
        bind(section, NIRI_SAY, &[DIM_SLOT], &args)
    });
    park(&mut section, keys::niri::OVERVIEW_BACKDROP, |row| {
        off || row.control.path() != Some(keys::niri::BACKDROP_DIM)
    });
    notes(&mut section, &statics.iter().map(|row| row.desc.clone()).collect::<Vec<_>>());
    section
}
