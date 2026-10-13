use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use crate::contracts::preview::UploadQueue;
use crate::domain::library::catalog::Catalog;
use crate::frontend::animation::{MotionProfile, MotionTier, Spring, Tween};
use crate::frontend::scene::hand::{Move, PoseTween};
use crate::frontend::scene::layout::{ExtraParams, GridParams, HexParams, Hit, Mode, SliceParams};
use crate::frontend::scene::{
    BackPanel, Chrome, InstanceRaw, LaunchAnim, RenderSnapshot, SandySnap, Transition,
};
use crate::frontend::theme::Palette;
use crate::infrastructure::preview::DecodePool;
use crate::rendering::scene::atlas::AtlasMap;

use super::preview_state::PreviewState;

pub struct SceneCore {
    pub mode: Mode,
    pub sp: SliceParams,
    pub gp: GridParams,
    pub hp: HexParams,
    pub xp: ExtraParams,
    pub(super) sp_target: SliceParams,
    pub(super) gp_target: GridParams,
    pub(super) hp_target: HexParams,
    pub(super) xp_target: ExtraParams,
    pub current: usize,
    pub hex_row: usize,
    pub hover: Option<usize>,
    pub kb_nav: bool,
    pub(super) user_engaged: bool,
    pub(super) camera: Spring,
    pub(super) collection_open: Spring,
    pub(super) collection_card: Option<usize>,
    pub(super) layout_camera_anchor: bool,
    pub(super) scroll_accum: f32,
    pub(super) scroll_vel: f32,
    pub(super) vis_lo: usize,
    pub(super) vis_hi: usize,
    pub(super) card: CardState,
    pub(super) sandy: SandyState,
    pub(super) hand: HandState,
    pub(super) motion: SceneMotionState,
    pub viewport: (f32, f32),
    pub render: Arc<RenderSnapshot>,
    pub last_mouse: Option<(f32, f32)>,
    pub input_idle: bool,
    pub(super) direct_thumbnails: bool,
    pub(super) filter_bar_footprint: Option<(bool, f32, f32)>,
    pub(super) reading_rtl: bool,
    pub(super) preview_state: PreviewState,
}

pub(super) struct SceneMotionState {
    pub(super) profile: MotionProfile,
    pub(super) scale: f32,
    pub(super) entrance: Spring,
    pub(super) visibility: Spring,
    pub(super) open_fade_ms: f32,
    pub(super) open_fade_from: f32,
    pub(super) launch_anim: LaunchAnim,
    pub(super) entrance_pending: bool,
    pub(super) center_inset: Spring,
    pub(super) last_tick: Option<Instant>,
    pub(super) last_anim_log: Option<Instant>,
    pub(super) needs_frame: bool,
    pub(super) transition: Option<Transition>,
    pub(super) first_frame_logged: bool,
    pub(super) ready_logged: bool,
}

impl SceneMotionState {
    fn new() -> Self {
        let profile = MotionProfile::default();
        let mut entrance = profile.spring(0.0, MotionTier::Fast);
        entrance.retarget(1.0);
        let mut visibility = profile.spring(1.0, MotionTier::Standard);
        visibility.retarget(1.0);
        Self {
            profile,
            scale: 1.0,
            entrance,
            visibility,
            open_fade_ms: profile.duration_ms(MotionTier::Standard),
            open_fade_from: 0.0,
            launch_anim: LaunchAnim::Fade,
            entrance_pending: false,
            center_inset: profile.spring(0.0, MotionTier::Standard),
            last_tick: None,
            last_anim_log: None,
            needs_frame: true,
            transition: None,
            first_frame_logged: false,
            ready_logged: false,
        }
    }
}

pub(super) struct CardState {
    pub(super) removal: crate::frontend::scene::RemovalAction,
    pub(super) flipped: Option<usize>,
    pub(super) flip_effect: u32,
    pub(super) flip: Spring,
    pub(super) flip_duration_ms: f32,
    pub(super) flip_shader_enabled: bool,
    pub(super) flip_back_enabled: bool,
    pub(super) det_p: f32,
    pub(super) det_target: f32,
    pub(super) det_src: [f32; 4],
    pub(super) fav_fill: f32,
    pub(super) fav_target: f32,
    pub(super) fav_snap: bool,
    pub(super) tag_open: Spring,
    pub(super) chip_pop: Spring,
    pub(super) pop_idx: i32,
    pub(super) pending_back: Option<BackPanel>,
    pub(super) widths: HashMap<usize, Spring>,
    pub(super) fades: HashMap<usize, Spring>,
    pub(super) hex_scales: HashMap<usize, Spring>,
    pub(super) selection: HashMap<usize, Spring>,
    pub(super) filter_cache: Vec<(InstanceRaw, u32, f32)>,
    pub(super) filter_old: Vec<(InstanceRaw, u32, f32)>,
    pub(super) filter_cell: HashMap<(i32, i32), usize>,
    pub(super) filter_in: HashMap<(i32, i32), f32>,
    pub(super) filter_wave: f32,
    pub(super) filter_ms: f32,
}

impl CardState {
    fn new() -> Self {
        let motion = MotionProfile::default();
        Self {
            removal: crate::frontend::scene::RemovalAction::Delete,
            flipped: None,
            flip_effect: 0,
            flip: motion.override_spring(0.0, 1500.0),
            flip_duration_ms: 1500.0,
            flip_shader_enabled: true,
            flip_back_enabled: true,
            det_p: 0.0,
            det_target: 0.0,
            det_src: [0.0; 4],
            fav_fill: 0.0,
            fav_target: 0.0,
            fav_snap: false,
            tag_open: motion.spring(0.0, MotionTier::Fast),
            chip_pop: motion.spring(1.0, MotionTier::Slow),
            pop_idx: -1,
            pending_back: None,
            widths: HashMap::new(),
            fades: HashMap::new(),
            hex_scales: HashMap::new(),
            selection: HashMap::new(),
            filter_cache: Vec::new(),
            filter_old: Vec::new(),
            filter_cell: HashMap::new(),
            filter_in: HashMap::new(),
            filter_wave: 10.0,
            filter_ms: motion.duration_ms(MotionTier::Slow),
        }
    }
}

pub(super) struct SandyState {
    pub(super) prog: Tween,
    pub(super) from: Option<usize>,
    pub(super) dir: f32,
    pub(super) carry: f32,
    pub(super) bfrom: Option<usize>,
    pub(super) bfrom2: Option<usize>,
    pub(super) bcut: f32,
    pub(super) bto: Option<usize>,
    pub(super) storm_to: Option<usize>,
    pub(super) ring_to: Option<usize>,
    pub(super) bmix: Spring,
    pub(super) swirl: Spring,
    pub(super) swirl_hold: f32,
    pub(super) pop: Spring,
    pub(super) hero_fade: Spring,
    pub(super) hero_restore_at: Option<Instant>,
    pub(super) filter_from: Option<usize>,
    pub(super) edge_pan: f32,
    pub(super) cam_free: bool,
    pub(super) sel_pos: Option<(f32, f32)>,
    pub(super) snap: Option<SandySnap>,
    pub(super) res_scale: f32,
    pub(super) lod_max: f32,
    pub(super) motion: f32,
}

impl SandyState {
    fn new() -> Self {
        let motion = MotionProfile::default();
        Self {
            prog: motion.override_tween(1.0, 1250.0),
            from: None,
            dir: 1.0,
            carry: 0.0,
            bfrom: None,
            bfrom2: None,
            bcut: 1.0,
            bto: None,
            storm_to: None,
            ring_to: None,
            bmix: motion.spring(1.0, MotionTier::Slow),
            swirl: motion.spring(0.0, MotionTier::Slow),
            swirl_hold: 0.0,
            pop: motion.spring(1.0, MotionTier::Slow),
            hero_fade: motion.spring(1.0, MotionTier::Standard),
            hero_restore_at: None,
            filter_from: None,
            edge_pan: 0.0,
            cam_free: false,
            sel_pos: None,
            snap: None,
            res_scale: 1.0,
            lod_max: 1.0,
            motion: 0.0,
        }
    }
}

pub(super) const BACKDROP_FADE_MS: f32 = 450.0;

pub(super) struct HandState {
    pub(super) offset: usize,
    pub(super) shown: Vec<usize>,
    pub(super) rig_x: Spring,
    pub(super) rig_y: Spring,
    pub(super) drag: Option<(f32, f32)>,
    pub(super) lift: HashMap<usize, Spring>,
    pub(super) deal: Option<HandDeal>,
    pub(super) flip_p: f32,
    pub(super) flip_closing: bool,
    pub(super) flip_seed: f32,
    pub(super) lseed: f32,
    pub(super) bob_t: f32,
    pub(super) cycle: Move,
    pub(super) deals: u32,
    pub(super) backdrop_store: Option<usize>,
    pub(super) backdrop_prev: Option<usize>,
    pub(super) backdrop_fade: Tween,
    pub(super) reveal: Option<Reveal>,
    pub(super) tall_ready: Option<usize>,
    pub(super) tall_failed: Option<usize>,
    pub(super) tall_requested: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Face {
    Card,
    Back { store: usize, column: bool },
    Slice { store: usize, column: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Layout {
    Fan,
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RevealPhase {
    Turning,
    Held,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Reveal {
    pub(super) open: bool,
    pub(super) phase: RevealPhase,
    pub(super) from: Layout,
    pub(super) to: Layout,
    pub(super) turn: f32,
    pub(super) turns_done: u32,
    pub(super) faces: [Face; 2],
    pub(super) len: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum DealPhase {
    Out,
    In,
}

pub(super) struct HandDeal {
    pub(super) phase: DealPhase,
    pub(super) t: f32,
    pub(super) mv: Move,
    pub(super) cards: Vec<CardAnim>,
    pub(super) end: f32,
}

pub(super) struct CardAnim {
    pub(super) store: usize,
    pub(super) slot: usize,
    pub(super) n: f32,
    pub(super) pose: PoseTween,
    pub(super) ribbons: Vec<PoseTween>,
    pub(super) ghosts: [PoseTween; 2],
}

impl HandState {
    fn new() -> Self {
        let motion = MotionProfile::default();
        let mut rig_x = motion.override_spring(0.0, 450.0);
        rig_x.set_zeta(0.75);
        let mut rig_y = motion.override_spring(0.0, 450.0);
        rig_y.set_zeta(0.75);
        Self {
            offset: 0,
            shown: Vec::new(),
            rig_x,
            rig_y,
            drag: None,
            lift: HashMap::new(),
            deal: None,
            flip_p: 0.0,
            flip_closing: false,
            flip_seed: 0.71,
            lseed: 0.31,
            bob_t: 0.0,
            cycle: Move::Corkscrew,
            deals: 0,
            backdrop_store: None,
            backdrop_prev: None,
            backdrop_fade: motion.override_tween(1.0, BACKDROP_FADE_MS),
            reveal: None,
            tall_ready: None,
            tall_failed: None,
            tall_requested: None,
        }
    }
}

pub struct RebuildCtx<'a> {
    pub catalog: &'a Catalog,
    pub filtered: &'a [u32],
    pub atlas: &'a mut Option<AtlasMap>,
    pub pool: &'a DecodePool,
    pub palette: &'a Palette,
    pub uploads: &'a UploadQueue,
    pub hover_fades: Option<&'a HashMap<usize, Spring>>,
}

pub(super) struct RebuildSinks<'a> {
    pub(super) instances: &'a mut Vec<InstanceRaw>,
    pub(super) hits: &'a mut Vec<Hit>,
    pub(super) chrome: &'a mut Vec<Chrome>,
    pub(super) wanted: &'a mut HashSet<usize>,
}

impl SceneCore {
    pub fn new(
        mode: Mode,
        sp: SliceParams,
        gp: GridParams,
        hp: HexParams,
        xp: ExtraParams,
    ) -> Self {
        let mut scene = Self {
            mode,
            sp,
            gp,
            hp,
            xp,
            sp_target: sp,
            gp_target: gp,
            hp_target: hp,
            xp_target: xp,
            current: 0,
            hex_row: 0,
            hover: None,
            kb_nav: false,
            user_engaged: false,
            camera: MotionProfile::default().spring(
                0.0,
                if mode == Mode::Grid { MotionTier::Slow } else { MotionTier::Standard },
            ),
            collection_open: MotionProfile::default().spring(0.0, MotionTier::Standard),
            collection_card: None,
            layout_camera_anchor: false,
            scroll_accum: 0.0,
            scroll_vel: 0.0,
            vis_lo: 0,
            vis_hi: 0,
            card: CardState::new(),
            sandy: SandyState::new(),
            hand: HandState::new(),
            motion: SceneMotionState::new(),
            viewport: (0.0, 0.0),
            render: Arc::new(RenderSnapshot::default()),
            last_mouse: None,
            input_idle: false,
            direct_thumbnails: false,
            filter_bar_footprint: None,
            reading_rtl: false,
            preview_state: PreviewState::new(),
        };
        scene.retime_shared_motion();
        scene
    }
}
