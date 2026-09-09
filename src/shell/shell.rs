use std::time::Duration;

use crate::app;

pub(super) const UI_FONT_BYTES: &[u8] = include_bytes!("../../assets/RobotoCondensed-Bold.ttf");
pub(super) const NERD_FONT_BYTES: &[u8] =
    include_bytes!("../../assets/SymbolsNerdFont-Regular.ttf");
const CTRL_IO_TIMEOUT: Duration = Duration::from_secs(2);
const CTRL_READ_TIMEOUT: Duration = Duration::from_millis(300);

#[cfg(target_os = "linux")]
static DRIVER_RECLAIM_SCHEDULED: std::sync::Once = std::sync::Once::new();
#[cfg(target_os = "linux")]
pub(super) const NVIDIA_COMPILER_RECLAIM_ADVICE: libc::c_int = libc::MADV_COLD;

pub struct SmallExec(iced::futures::executor::ThreadPool);

impl iced::executor::Executor for SmallExec {
    fn new() -> Result<Self, iced::futures::io::Error> {
        iced::futures::executor::ThreadPool::builder()
            .pool_size(1)
            .name_prefix("iced-exec")
            .create()
            .map(Self)
    }

    fn spawn(&self, future: impl std::future::Future<Output = ()> + Send + 'static) {
        self.0.spawn_ok(future);
    }

    fn block_on<T>(&self, future: impl std::future::Future<Output = T>) -> T {
        iced::futures::executor::block_on(future)
    }
}

pub fn run() -> Result<(), String> {
    configure_picker_power_preference();
    configure_thumbnail_profile();
    #[cfg(target_os = "linux")]
    {
        unsafe { std::env::set_var("WGPU_BACKEND", "vulkan") };
        run_linux()
    }
    #[cfg(not(target_os = "linux"))]
    run_winit()
}

fn configure_thumbnail_profile() {
    let compressed =
        compressed_thumbnail_profile(std::env::var("SKWD_WALL_THUMBNAILS").ok().as_deref());
    crate::contracts::preview::set_compressed_thumbnails(compressed);
    iced_wgpu::set_texture_compression_bc_required(compressed);
    log::info!("thumbnail profile: {}", if compressed { "bc" } else { "rgba-poc" });
}

pub(super) fn compressed_thumbnail_profile(value: Option<&str>) -> bool {
    !value.is_some_and(|value| value.eq_ignore_ascii_case("rgba-poc"))
}

fn configure_picker_power_preference() {
    if let Some(preference) = std::env::var_os("WGPU_POWER_PREF") {
        log::info!(
            "picker GPU preference overridden by WGPU_POWER_PREF={}",
            preference.to_string_lossy()
        );
        return;
    }
    let root = std::fs::read_to_string(skwd_config::config_path())
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    let on_battery = skwd_config::on_battery_power();
    let preference = picker_power_preference(&root, on_battery);
    unsafe { std::env::set_var("WGPU_POWER_PREF", preference) };
    log::info!("picker GPU power preference: {preference} (on_battery={on_battery})");
}

pub(super) fn picker_power_preference(root: &serde_json::Value, on_battery: bool) -> &'static str {
    skwd_config::effective_gpu_preference(root, on_battery)
}

pub fn on_first_gpu_frame() {
    schedule_cold_start_reclaim();
}

#[cfg(target_os = "linux")]
fn schedule_cold_start_reclaim() {
    DRIVER_RECLAIM_SCHEDULED.call_once(|| {
        if std::env::var("SKWD_VK_RECLAIM_DRIVER").as_deref() == Ok("0") {
            return;
        }
        let _ = std::thread::Builder::new().name("rss-reclaim".into()).spawn(|| {
            std::thread::sleep(Duration::from_secs(3));
            reclaim_cold_start_pages();
        });
    });
}

#[cfg(not(target_os = "linux"))]
fn schedule_cold_start_reclaim() {}

#[cfg(target_os = "linux")]
fn reclaim_cold_start_pages() {
    let Ok(maps) = std::fs::read_to_string("/proc/self/maps") else { return };
    let mut cooled = 0usize;
    let mut mappings = 0usize;
    for line in maps.lines() {
        let Some((start, len)) = cold_nvidia_compiler_mapping(line) else { continue };
        if unsafe { libc::madvise(start.cast(), len, NVIDIA_COMPILER_RECLAIM_ADVICE) } == 0 {
            cooled = cooled.saturating_add(len);
            mappings += 1;
        }
    }
    trim_heap();
    if mappings > 0 {
        log::info!(
            "cooled NVIDIA compiler pages: {mappings} mappings, {} MiB virtual",
            cooled / (1024 * 1024)
        );
    }
}

#[cfg(target_os = "linux")]
pub(super) fn cold_nvidia_compiler_mapping(line: &str) -> Option<(*mut libc::c_void, usize)> {
    let mut fields = line.split_whitespace();
    let range = fields.next()?;
    let permissions = fields.next()?;
    if !permissions.starts_with('r') || permissions.as_bytes().get(1) == Some(&b'w') {
        return None;
    }
    if !line.contains("libnvidia-gpucomp.so") {
        return None;
    }
    let (start, end) = range.split_once('-')?;
    let start = usize::from_str_radix(start, 16).ok()?;
    let end = usize::from_str_radix(end, 16).ok()?;
    (end > start).then_some((start as *mut libc::c_void, end - start))
}

#[cfg(target_os = "linux")]
pub(super) fn control_socket_name() -> String {
    let uid = unsafe { libc::getuid() };
    match std::env::var("SKWD_WALL_V2_INSTANCE") {
        Ok(tag) if !tag.is_empty() => format!("skwd-wall-v2.{uid}.{tag}"),
        _ => format!("skwd-wall-v2.{uid}"),
    }
}

#[cfg(target_os = "linux")]
pub fn acquire_single_instance() {
    use std::io::ErrorKind;
    use std::os::linux::net::SocketAddrExt;
    use std::os::unix::net::{SocketAddr, UnixListener, UnixStream};

    let name = control_socket_name();
    let Ok(addr) = SocketAddr::from_abstract_name(name.as_bytes()) else {
        return;
    };
    match UnixListener::bind_addr(&addr) {
        Ok(listener) => {
            std::thread::Builder::new()
                .name("toggle-listener".into())
                .spawn(move || {
                    for stream in listener.incoming() {
                        let wake = match stream {
                            Ok(conn) => read_control_command(conn),
                            Err(_) => crate::infrastructure::runtime::Wake::Toggle,
                        };
                        log::info!("control: {wake:?}");
                        if matches!(wake, crate::infrastructure::runtime::Wake::Toggle)
                            && crate::app::wake_sender().is_none()
                        {
                            crate::hard_exit(0);
                        }
                        loop {
                            if let Some(tx) = crate::app::wake_sender() {
                                let _ = tx.unbounded_send(wake);
                                break;
                            }
                            std::thread::sleep(Duration::from_millis(5));
                        }
                    }
                })
                .ok();
        }
        Err(err) if err.kind() == ErrorKind::AddrInUse => {
            let startup = crate::infrastructure::runtime::startup_panel();
            if let Ok(mut stream) = UnixStream::connect_addr(&addr)
                && let Some(command) = startup_control_command(startup.as_deref())
            {
                use std::io::Write;
                let _ = stream.write_all(command.as_bytes());
            }
            log::info!("toggle: signaled running instance, exiting");
            crate::hard_exit(0);
        }
        Err(err) => log::warn!("single-instance guard unavailable: {err}"),
    }
}

#[cfg(target_os = "linux")]
fn read_control_command(
    stream: std::os::unix::net::UnixStream,
) -> crate::infrastructure::runtime::Wake {
    use std::io::Read;
    let _ = stream.set_read_timeout(Some(CTRL_READ_TIMEOUT));
    let mut buf = [0u8; 256];
    let mut got = 0usize;
    let mut conn = stream;
    while got < buf.len() {
        match conn.read(&mut buf[got..]) {
            Ok(count) if count > 0 => {
                got += count;
                if buf[..got].contains(&b'\n') {
                    break;
                }
            }
            _ => break,
        }
    }
    let wake = parse_control_command(&String::from_utf8_lossy(&buf[..got]));
    attach_reply(wake, conn)
}

pub(super) fn parse_control_command(raw: &str) -> crate::infrastructure::runtime::Wake {
    let line = raw.lines().next().unwrap_or("").trim();
    if line.is_empty() {
        crate::infrastructure::runtime::Wake::Toggle
    } else {
        crate::infrastructure::runtime::Wake::Command(line.into())
    }
}

pub(super) fn startup_control_command(start: Option<&str>) -> Option<&'static str> {
    match start {
        Some("mixer") => Some("open mixer\n"),
        Some("theme-audition") => Some("open theme-audition\n"),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
pub(super) fn attach_reply(
    wake: crate::infrastructure::runtime::Wake,
    stream: std::os::unix::net::UnixStream,
) -> crate::infrastructure::runtime::Wake {
    match wake {
        crate::infrastructure::runtime::Wake::Command(line)
            if line == "state" || line.starts_with("state ") =>
        {
            let reply =
                crate::infrastructure::runtime::Reply::new(Box::new(move |response: &str| {
                    use std::io::Write;
                    let mut conn = stream;
                    let _ = conn.set_write_timeout(Some(CTRL_IO_TIMEOUT));
                    let _ = conn.write_all(response.as_bytes());
                    let _ = conn.write_all(b"\n");
                }));
            crate::infrastructure::runtime::Wake::Query(line, reply)
        }
        other => other,
    }
}

#[cfg(not(target_os = "linux"))]
pub fn acquire_single_instance() {}

#[cfg(target_os = "linux")]
pub fn trim_heap() {
    #[cfg(target_env = "gnu")]
    unsafe {
        libc::malloc_trim(0)
    };
}

#[cfg(not(target_os = "linux"))]
pub fn trim_heap() {}

#[cfg(target_os = "linux")]
fn run_linux() -> Result<(), String> {
    if std::env::var_os("WAYLAND_DISPLAY").is_some()
        && std::env::var_os("SKWD_WALL_FORCE_WINIT").is_none()
    {
        run_layershell()
    } else {
        run_winit()
    }
}

#[cfg(target_os = "linux")]
fn boot() -> (app::App, iced::Task<app::Message>) {
    (app::App::default(), iced::Task::none())
}

#[cfg(target_os = "linux")]
fn run_layershell() -> Result<(), String> {
    use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
    use iced_layershell::settings::{LayerShellSettings, Settings};

    ensure_layershell_available()?;
    log::info!("shell mode: layershell (warm daemon)");
    iced_layershell::daemon(boot, namespace, app::update, app::view)
        .settings(Settings {
            layer_settings: LayerShellSettings {
                anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
                layer: Layer::Overlay,
                exclusive_zone: -1,
                size: Some((0, 0)),
                margin: (0, 0, 0, 0),
                keyboard_interactivity: KeyboardInteractivity::Exclusive,
                start_mode: start_mode(),
                events_transparent: false,
            },
            fonts: vec![UI_FONT_BYTES.into(), NERD_FONT_BYTES.into()],
            default_font: crate::frontend::ui::UI_FONT,
            antialiasing: true,
            ..Settings::default()
        })
        .style(app::style)
        .subscription(app::subscription)
        .executor::<SmallExec>()
        .run()
        .map_err(|error| {
            layershell_run_error_message(
                &std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
                &error,
            )
        })
}

#[cfg(target_os = "linux")]
fn ensure_layershell_available() -> Result<(), String> {
    use wayland_client::globals::{GlobalListContents, registry_queue_init};
    use wayland_client::protocol::wl_registry;
    use wayland_client::{Connection, Dispatch, QueueHandle};

    struct RegistryProbe;

    impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for RegistryProbe {
        fn event(
            _state: &mut Self,
            _registry: &wl_registry::WlRegistry,
            _event: wl_registry::Event,
            _globals: &GlobalListContents,
            _connection: &Connection,
            _queue: &QueueHandle<Self>,
        ) {
        }
    }

    let connection = Connection::connect_to_env()
        .map_err(|error| format!("Wayland connection failed: {error}"))?;
    let (globals, _queue) = registry_queue_init::<RegistryProbe>(&connection)
        .map_err(|error| format!("Wayland registry initialization failed: {error}"))?;
    let available = globals.contents().with_list(|list| {
        list.iter()
            .any(|global| layer_shell_global_usable(global.interface.as_str(), global.version))
    });
    if available {
        Ok(())
    } else {
        Err(layershell_error_message(
            &std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
            "zwlr_layer_shell_v1 version 3 or newer was not advertised",
        ))
    }
}

#[cfg(target_os = "linux")]
pub(super) fn layer_shell_global_usable(interface: &str, version: u32) -> bool {
    interface == "zwlr_layer_shell_v1" && version >= 3
}

#[cfg(target_os = "linux")]
pub(super) fn layershell_error_message(desktop: &str, underlying: &str) -> String {
    let lower = desktop.to_lowercase();
    let compositor = if lower.contains("gnome") || lower.contains("unity") {
        format!("{desktop} (Mutter) does not implement the wlr-layer-shell protocol")
    } else if lower.is_empty() {
        String::from("this compositor does not implement the wlr-layer-shell protocol")
    } else {
        format!("{desktop} does not appear to implement the wlr-layer-shell protocol")
    };
    format!(
        "skwd-wall could not create its layer-shell surface: {compositor}.\n\
         skwd-wall requires a wlroots-based compositor (niri, sway, Hyprland, river, wayfire, labwc), \
         KWin/KDE Plasma, or another compositor that supports wlr-layer-shell.\n\
         Set SKWD_WALL_FORCE_WINIT=1 to run in a plain window instead (limited), \
         or run `skwd-walld doctor` for details.\n\
         (underlying error: {underlying})"
    )
}

#[cfg(target_os = "linux")]
pub(super) fn layershell_run_error_message(
    desktop: &str,
    error: &iced_layershell::Error,
) -> String {
    use iced_layershell::Error;

    match error {
        Error::ExecutorCreationFailed(underlying) => format!(
            "skwd-wall could not start its async task executor.\n\
             This is an internal runtime initialization failure, not a missing layer-shell protocol.\n\
             Run `skwd-walld doctor` for environment details.\n\
             (underlying error: {underlying})"
        ),
        Error::GraphicsCreationFailed(underlying) => format!(
            "skwd-wall could not initialize GPU rendering for its layer-shell surface.\n\
             Check that a working Vulkan/wgpu adapter with BC texture compression is available.\n\
             Set SKWD_WALL_THUMBNAILS=rgba-poc to use the uncompressed compatibility profile, \
             then run `skwd-walld doctor` if initialization still fails.\n\
             (underlying error: {underlying:?})"
        ),
        Error::WindowCreationFailed(underlying) => {
            let compositor = if desktop.is_empty() { "the current compositor" } else { desktop };
            format!(
                "skwd-wall could not create its layer-shell surface under {compositor}.\n\
                 The compositor advertised wlr-layer-shell, but surface creation failed.\n\
                 Set SKWD_WALL_FORCE_WINIT=1 to run in a plain window instead (limited), \
                 or run `skwd-walld doctor` for details.\n\
                 (underlying error: {underlying})"
            )
        }
        Error::WaylandDispatchFailed(underlying) => format!(
            "skwd-wall's Wayland layer-shell event loop failed after startup.\n\
             Check the compositor log and run `skwd-walld doctor` for connection details.\n\
             (underlying error: {underlying:?})"
        ),
    }
}

#[cfg(target_os = "linux")]
fn start_mode() -> iced_layershell::settings::StartMode {
    use iced_layershell::settings::StartMode;
    if let Ok(name) = std::env::var("SKWD_WALL_OUTPUT")
        && !name.is_empty()
    {
        return StartMode::TargetScreen(name);
    }
    let monitor = crate::infrastructure::config::Config::load().main_monitor();
    if !monitor.is_empty() {
        log::info!("targeting configured monitor {monitor}");
        return StartMode::TargetScreen(monitor);
    }
    StartMode::Active
}

#[cfg(target_os = "linux")]
fn namespace() -> String {
    String::from("skwd-wall")
}

fn run_winit() -> Result<(), String> {
    log::info!("shell mode: winit");
    iced::application(app::App::default, app::update, app::view_single)
        .font(UI_FONT_BYTES)
        .font(NERD_FONT_BYTES)
        .default_font(crate::frontend::ui::UI_FONT)
        .antialiasing(true)
        .window(iced::window::Settings {
            fullscreen: true,
            decorations: false,
            transparent: true,
            level: iced::window::Level::AlwaysOnTop,
            ..iced::window::Settings::default()
        })
        .style(app::style)
        .subscription(app::subscription)
        .executor::<SmallExec>()
        .run()
        .map_err(|err| err.to_string())
}
