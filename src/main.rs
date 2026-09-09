mod app;
mod backend;
mod contracts;
mod domain;
mod frontend;
mod helm_compat;
mod i18n;
mod infrastructure;
mod rendering;
mod shell;

#[cfg(all(feature = "obs-dhat", not(feature = "obs-heap")))]
use std::sync::OnceLock;

use log::info;

use infrastructure::observability::{
    init_startup_clock, log_startup_checkpoint, logging, metrics, rss_label,
};

#[cfg(feature = "obs-heap")]
#[global_allocator]
static GLOBAL: skwd_log::alloc::Counting<std::alloc::System> = skwd_log::alloc::system();

#[cfg(all(feature = "obs-dhat", not(feature = "obs-heap")))]
#[global_allocator]
static GLOBAL_DHAT: dhat::Alloc = dhat::Alloc;

#[cfg(all(feature = "obs-dhat", not(feature = "obs-heap")))]
static DHAT: OnceLock<std::sync::Mutex<Option<dhat::Profiler>>> = OnceLock::new();

#[cfg(target_env = "gnu")]
const MALLOC_ARENA_MAX: libc::c_int = 2;
#[cfg(target_env = "gnu")]
const MALLOC_MMAP_THRESHOLD: libc::c_int = 512 * 1024;

struct Args {
    arguments: Vec<String>,
    debug: bool,
    version: bool,
    help: bool,
}

fn parse_args(arguments: Vec<String>) -> Args {
    Args {
        debug: arguments.iter().any(|argument| argument == "--debug"),
        version: arguments.iter().any(|argument| argument == "--version" || argument == "-V"),
        help: arguments.first().is_some_and(|argument| argument == "--help" || argument == "-h"),
        arguments,
    }
}

fn log_diag() {
    let cfg = infrastructure::config::Config::load();
    let socket = std::env::var("SKWD_WALL_V2_SOCK")
        .unwrap_or_else(|_| wall_proto::socket_path().display().to_string());
    info!("skwd-wall debug mode (metrics on, log=debug)");
    info!("  socket        = {socket}");
    info!("  wallpaper_dir = {}", cfg.wallpaper_dir());
    info!("  cache_dir     = {}", cfg.cache_dir());
    info!("  monitor       = {}", cfg.main_monitor());
    for key in ["WAYLAND_DISPLAY", "XDG_RUNTIME_DIR", "SKWD_WALL_V2_CONFIG", "SKWD_WALL_V2_SOCK"] {
        info!("  env {key} = {}", std::env::var(key).unwrap_or_else(|_| "(unset)".into()));
    }
}

fn main() {
    #[cfg(target_env = "gnu")]
    unsafe {
        libc::mallopt(libc::M_ARENA_MAX, MALLOC_ARENA_MAX);
        libc::mallopt(libc::M_MMAP_THRESHOLD, MALLOC_MMAP_THRESHOLD);
    }
    init_startup_clock();

    let args = parse_args(std::env::args().skip(1).collect());
    infrastructure::runtime::set_debug(args.debug);
    if args.help {
        println!("skwd-wall-v2 {}", env!("CARGO_PKG_VERSION"));
        println!(
            "usage: skwd-wall-v2 [--mixer] [--debug] [--version | -V]  (no arguments launches the picker)"
        );
        println!("  --mixer  Open the wallpaper mixer directly, including in a running instance.");
        println!("control commands delegate to skwd-helm:\n");
        let _ = std::io::Write::flush(&mut std::io::stdout());
    }
    if let Some(code) = helm_compat::try_delegate(&args.arguments) {
        std::process::exit(code);
    }

    #[cfg(all(feature = "obs-dhat", not(feature = "obs-heap")))]
    let _ = DHAT.set(std::sync::Mutex::new(Some(dhat::Profiler::new_heap())));
    #[cfg(feature = "obs-tracy")]
    let _tracy = tracy_client::Client::start();
    #[cfg(feature = "obs-trace")]
    logging::init_tracing(args.debug);
    #[cfg(not(feature = "obs-trace"))]
    logging::init(args.debug);

    if args.version {
        println!("skwd-wall-v2 {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if args.debug {
        metrics::force_on();
        log_diag();
    }

    shell::acquire_single_instance();

    if let Ok(ms) = std::env::var("SKWD_WALL_AUTO_EXIT_MS")
        && let Ok(ms) = ms.parse::<u64>()
    {
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            info!("auto exit after {ms} ms, rss = {}", rss_label());
            hard_exit(0);
        });
    }

    log_startup_checkpoint("exec");
    if let Err(err) = shell::run() {
        log::error!("{err}");
        std::process::exit(1);
    }
}

pub fn hard_exit(code: i32) -> ! {
    flush_obs_guards();
    unsafe { libc::_exit(code) }
}

fn flush_obs_guards() {
    #[cfg(all(feature = "obs-dhat", not(feature = "obs-heap")))]
    if let Some(mutex) = DHAT.get() {
        if let Ok(mut guard) = mutex.lock() {
            drop(guard.take());
        }
    }
    #[cfg(feature = "obs-trace")]
    logging::flush_chrome();
}

#[path = "main_tests.rs"]
mod tests;
