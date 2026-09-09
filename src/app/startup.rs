use std::sync::{Arc, Mutex, OnceLock};

use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};

use crate::contracts::preview::UploadQueue;
use crate::domain::library::filter::Filters;
use crate::frontend::animation::MotionProfile;
use crate::frontend::scene::layout::{
    ExtraParams, GridLayout, GridParams, HexCurve, HexParams, HexShape, MIN_HEX_R, Mode,
    SliceParams, StageParams,
};
use crate::frontend::theme::Palette;
use crate::infrastructure::config::Config;
use crate::infrastructure::ipc::DaemonClient;
use crate::infrastructure::preview::DecodePool;
use crate::infrastructure::runtime::Wake;

use super::{
    App, AppRuntimeState, ChromeState, DaemonState, EFFECT_NAMES, InputState, LibrarySession,
    PanelsState, PreviewResources, SearchMode, SourceBrowserState, TagState, ThemeState,
    browser_wall_params, scene::SceneCore,
};

static WAKE_RX: OnceLock<Mutex<Option<UnboundedReceiver<Wake>>>> = OnceLock::new();
static WAKE_TX: OnceLock<Mutex<Option<UnboundedSender<Wake>>>> = OnceLock::new();

pub(crate) fn wake_sender() -> Option<UnboundedSender<Wake>> {
    WAKE_TX.get()?.lock().ok()?.clone()
}

pub(super) fn take_wake_receiver() -> Option<UnboundedReceiver<Wake>> {
    WAKE_RX.get_or_init(|| Mutex::new(None)).lock().ok()?.take()
}

impl Default for App {
    fn default() -> Self {
        crate::infrastructure::observability::log_startup_checkpoint("app_construct_start");
        let app = Self::with_config(Config::load());
        crate::infrastructure::observability::log_startup_checkpoint("app_construct_done");
        app
    }
}

impl App {
    pub(super) fn with_config(config: Config) -> Self {
        Self::with_config_using(config, DaemonClient::start)
    }

    pub(super) fn with_config_using(
        config: Config,
        start_daemon: impl FnOnce(UnboundedSender<Wake>) -> DaemonClient,
    ) -> Self {
        crate::i18n::set_language(&config.str_path(skwd_config::keys::general::LANGUAGE));
        let (tx, rx) = unbounded::<Wake>();
        let stale_rx = WAKE_RX
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .replace(rx);
        if stale_rx.is_some() {
            log::warn!("wake channel rebuilt: control routes to the newest App instance");
        }
        let uploads: UploadQueue = Arc::new(Mutex::new(Vec::new()));
        let pool = DecodePool::start(uploads.clone(), tx.clone(), 6);
        spawn_selftest(&tx);
        spawn_hud_ticker(&tx);
        let daemon = DaemonState::new(start_daemon(tx.clone()));
        let palette =
            Palette::from_spec(&crate::infrastructure::theme::load_palette(&config.cache_dir()));
        let motion = MotionProfile::new(
            config.motion_fast_ms(),
            config.motion_standard_ms(),
            config.motion_slow_ms(),
        );
        WAKE_TX
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .replace(tx.clone());
        crate::infrastructure::observability::log_startup_checkpoint("pre_scene_init");
        let mut browser_layout = layout_params(&config);
        browser_layout.grid = browser_wall_params(&config);
        let scene = init_scene(&config, tx.clone());
        crate::infrastructure::observability::log_startup_checkpoint("scene_init_done");
        let wallpaper_dir = config.wallpaper_dir();
        let video_dir = config.video_dir();
        let filters = startup_filters(&config);
        #[cfg(not(test))]
        crate::infrastructure::observability::timing::init(&config.cache_dir());
        let filter_bar_visible =
            std::env::var_os("SKWD_WALL_BAR").is_some() || config.filter_bar_always_visible();
        let filter_bar_vertical =
            config.filter_bar_orientation() == "vertical" && scene.mode != Mode::Sandy;
        let search_mode = SearchMode::from_config(
            &config.str_path(skwd_config::keys::tagging::DEFAULT_SEARCH_MODE),
        );
        let mut app = Self {
            runtime_state: AppRuntimeState::new(
                crate::infrastructure::runtime::FrameClock::start(tx.clone()),
                tx.clone(),
            ),
            daemon,
            preview_resources: PreviewResources::new(pool, uploads),
            config,
            theme: ThemeState::new(palette, motion),
            library_session: LibrarySession::new(filters, wallpaper_dir, video_dir),
            scene,
            chrome: ChromeState::new(filter_bar_visible, filter_bar_vertical, motion),
            panels: PanelsState::default(),
            source_browser: SourceBrowserState::new(
                motion,
                (
                    browser_layout.slices,
                    browser_layout.grid,
                    browser_layout.hex,
                    browser_layout.extra,
                ),
                tx,
            ),
            input: InputState::default(),
            tags: TagState::with_search_mode(search_mode),
        };
        app.apply_motion_speeds();
        app.reload_bindings();
        if std::env::var_os("SKWD_WALL_SETTINGS").is_some() {
            app.panels.settings.open = true;
            app.init_settings_inputs();
        }
        if std::env::var_os("SKWD_WALL_BROWSER").is_some() {
            app.source_browser.browser = Some(crate::frontend::browser::Browser::new(
                crate::frontend::browser::Source::Wallhaven,
            ));
            app.source_browser.entrance.run(0.0, 1.0);
            app.source_browser.wall.begin_session();
        }
        let startup = crate::infrastructure::runtime::startup_panel();
        if startup.as_deref() == Some("mixer") {
            app.panels.audio = Some(crate::frontend::audio_panel::AudioPanel::new_with_motion(
                app.motion_profile(),
            ));
        }
        if startup.as_deref() == Some("theme-audition") {
            app.theme.audition_open = true;
            app.theme.audition_focused = true;
        }
        app
    }
}

fn spawn_selftest(tx: &UnboundedSender<Wake>) {
    if std::env::var_os("SKWD_WALL_SELFTEST").is_none() {
        return;
    }
    let test_tx = tx.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(2500));
        for step in 0..4u32 {
            let _ = test_tx.unbounded_send(Wake::SelfTest(step));
            std::thread::sleep(std::time::Duration::from_millis(1500));
        }
    });
}

fn spawn_hud_ticker(tx: &UnboundedSender<Wake>) {
    if !crate::infrastructure::observability::metrics::hud_active() {
        return;
    }
    let hud_tx = tx.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            if hud_tx.unbounded_send(Wake::HudTick).is_err() {
                break;
            }
        }
    });
}

fn init_scene(config: &Config, tx: UnboundedSender<Wake>) -> SceneCore {
    let layout = layout_params(config);
    let mut scene =
        SceneCore::new(layout.mode, layout.slices, layout.grid, layout.hex, layout.extra);
    scene.set_motion_profile(MotionProfile::new(
        config.motion_fast_ms(),
        config.motion_standard_ms(),
        config.motion_slow_ms(),
    ));
    scene.set_filter_swap_ms(config.filter_swap_ms());
    scene.set_open_fade_ms(config.open_fade_ms());
    scene.set_open_fade_from(config.open_fade_from());
    scene.set_launch_animation(&config.launch_animation());
    scene.set_wake(tx);
    scene.set_preview_config(
        config.video_preview_enabled(),
        config.video_preview_delay_ms(),
        config.video_preview_fps(),
    );
    scene.set_sandy_res_scale(config.sandy_res_scale());
    let graphics = crate::infrastructure::capabilities::graphics_card(
        &crate::rendering::capabilities::WgpuGraphicsProbe,
    );
    scene.set_sandy_lod(crate::rendering::capabilities::effective_lod(
        config.sandy_lod(),
        config.sandy_lod_auto(),
        graphics.tier,
    ));
    let flip_name = config.str_path(skwd_config::keys::selector::FLIP_EFFECT);
    if let Some(id) = EFFECT_NAMES.iter().position(|&name| name == flip_name) {
        scene.set_effect(id as u32);
    }
    scene.set_card_flip_options(
        config.card_flip_duration_ms(),
        config.card_flip_shader(),
        config.card_flip_back_reveal(),
    );
    scene
}

pub(crate) struct LayoutParams {
    pub(crate) mode: Mode,
    pub(crate) slices: SliceParams,
    pub(crate) grid: GridParams,
    pub(crate) hex: HexParams,
    pub(crate) extra: ExtraParams,
}

pub(crate) fn layout_params(config: &Config) -> LayoutParams {
    let mode_name = std::env::var("SKWD_WALL_MODE").unwrap_or_else(|_| config.display_mode());
    let mode = Mode::from_key(&mode_name);
    let (slice_x, slice_y) = config.slice_position();
    let sp = SliceParams {
        offset_x: slice_x,
        offset_y: slice_y,
        slice_w: config.slice_width(),
        expanded_w: config.expanded_width(),
        slice_h: config.slice_height(),
        spacing: config.slice_spacing(),
        skew: config.skew_offset(),
        edge_tilt: config.slice_edge_tilt(),
        visible_count: config.visible_count(),
        corners: config.slice_corners(),
        wobble: config.slice_wobble(),
        wobble_strength: config.slice_wobble_strength(),
    };
    let [
        grid_x,
        grid_y,
        grid_scale,
        grid_rotation,
        grid_perspective,
        grid_shear_x,
        grid_shear_y,
        grid_depth_angle,
    ] = config.grid_stage();
    let gp = GridParams {
        cols: config.grid_columns(),
        rows: config.grid_rows(),
        thumb_w: config.grid_thumb_width(),
        thumb_h: config.grid_thumb_height(),
        gap_x: config.grid_gap_x(),
        gap_y: config.grid_gap_y(),
        corner_radius: if config.grid_round_corners() { config.grid_corner_radius() } else { 0.0 },
        border_width: config.grid_border_width(),
        layout: GridLayout::from_key(&config.grid_layout()),
        stagger: config.grid_stagger(),
        selected_scale: config.grid_selected_scale(),
        flow_wave: config.grid_flow_wave(),
        flow_frequency: config.grid_flow_frequency(),
        scatter: config.grid_scatter(),
        scale_variance: config.grid_scale_variance(),
        cylinder_bend: config.grid_cylinder_bend(),
        cylinder_radius: config.grid_cylinder_radius(),
        stage: StageParams {
            offset_x: grid_x,
            offset_y: grid_y,
            scale: grid_scale,
            rotation: grid_rotation,
            perspective: grid_perspective,
            shear_x: grid_shear_x,
            shear_y: grid_shear_y,
            depth_angle: grid_depth_angle,
        },
    };
    let [
        hex_x,
        hex_y,
        hex_scale,
        hex_rotation,
        hex_perspective,
        hex_shear_x,
        hex_shear_y,
        hex_depth_angle,
    ] = config.hex_stage();
    let hp = HexParams {
        r: config.hex_radius().max(MIN_HEX_R),
        rows: config.hex_rows(),
        cols: config.hex_cols(),
        scroll_step: config.hex_scroll_step(),
        curve: HexCurve::from_key(&config.hex_curve()),
        shape: HexShape::from_key(&config.hex_shape()),
        curve_strength: config.hex_arc_intensity(),
        curve_frequency: config.hex_curve_frequency(),
        gap_x: config.hex_gap_x(),
        gap_y: config.hex_gap_y(),
        aspect: config.hex_aspect(),
        stagger: config.hex_stagger(),
        lens: config.hex_lens(),
        lens_radius: config.hex_lens_radius(),
        orbit: config.hex_orbit(),
        orbit_radius: config.hex_orbit_radius(),
        twist: config.hex_twist(),
        scatter: config.hex_scatter(),
        stage: StageParams {
            offset_x: hex_x,
            offset_y: hex_y,
            scale: hex_scale,
            rotation: hex_rotation,
            perspective: hex_perspective,
            shear_x: hex_shear_x,
            shear_y: hex_shear_y,
            depth_angle: hex_depth_angle,
        },
    };
    let (sandy_x, sandy_y) = config.sandy_position();
    let xp = ExtraParams {
        sandy: crate::frontend::scene::sandy::SandyParams {
            offset_x: sandy_x,
            offset_y: sandy_y,
            center_h: config.sandy_center(),
            slice_w: config.sandy_slice_width(),
            slice_h: config.sandy_slice_height(),
            spacing: config.sandy_spacing(),
            duration_ms: config.sandy_duration(),
            blend_ms: config.sandy_blend(),
            strands: config.sandy_strands(),
            twist: config.sandy_twist(),
            orbit: config.sandy_orbit(),
            turbulence: config.sandy_turbulence(),
            waist: config.sandy_waist(),
            front: config.sandy_front(),
            fan: config.sandy_fan(),
            arc: config.sandy_arc(),
            swap_loop: config.sandy_swap_loop(),
            swap_style: config.sandy_swap_style_index(),
            skew: config.sandy_skew(),
            corners: config.slice_corners(),
            edge_speed: config.sandy_edge_speed(),
            ring_size: config.sandy_ring_size(),
            ring_spin: config.sandy_ring_spin(),
            ring_wave: config.sandy_ring_wave(),
            ring_soft: config.sandy_ring_soft(),
            ring_blend: config.sandy_ring_blend(),
            ring_hold: config.sandy_ring_hold(),
            grain: config.sandy_grain(),
            video_out_live: config.sandy_video_out_live(),
        },
    };
    LayoutParams { mode, slices: sp, grid: gp, hex: hp, extra: xp }
}

pub(crate) fn startup_filters(config: &Config) -> Filters {
    let weather_active = config.weather_match() && !config.locale().is_empty();
    if config.filter_bar_sticky() {
        Filters {
            color: config.last_filter_color(),
            kind: config.last_filter_kind(),
            folder: config.last_filter_folder(),
            sort: config.last_filter_sort(),
            orient: config.last_filter_orient(),
            resolution: config.last_filter_resolution(),
            favourites_only: config.last_filter_favourites(),
            weather_active,
            ..Filters::default()
        }
    } else {
        Filters { folder: config.default_folder(), weather_active, ..Filters::default() }
    }
}
