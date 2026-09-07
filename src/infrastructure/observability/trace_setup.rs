use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

struct FileMaker(Arc<Mutex<skwd_log::RotatingWriter>>);
struct FileSink(Arc<Mutex<skwd_log::RotatingWriter>>);

impl Write for FileSink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.0.lock() {
            Ok(mut file) => file.write(buf),
            Err(_) => Ok(buf.len()),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match self.0.lock() {
            Ok(mut file) => file.flush(),
            Err(_) => Ok(()),
        }
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for FileMaker {
    type Writer = FileSink;
    fn make_writer(&'a self) -> Self::Writer {
        FileSink(self.0.clone())
    }
}

#[cfg(feature = "obs-chrome")]
static CHROME_GUARD: std::sync::OnceLock<Mutex<Option<tracing_chrome::FlushGuard>>> =
    std::sync::OnceLock::new();

fn level_str(debug: bool) -> &'static str {
    if debug {
        return "debug";
    }
    match std::env::var("SKWD_WALL_LOG").as_deref() {
        Ok("trace") => "trace",
        Ok("debug") => "debug",
        Ok("warn") => "warn",
        Ok("error") => "error",
        _ => "info",
    }
}

fn log_path() -> Option<std::path::PathBuf> {
    skwd_log::log_path("skwd-wall")
}

fn open_log_arc() -> Option<Arc<Mutex<skwd_log::RotatingWriter>>> {
    let path = log_path()?;
    skwd_log::RotatingWriter::new(path).ok().map(|file| Arc::new(Mutex::new(file)))
}

pub fn init_tracing(debug: bool) {
    let lvl = level_str(debug);
    let filter =
        EnvFilter::new(format!("warn,skwd_wall={lvl},skwd_wall_core={lvl},wall_proto={lvl}"));

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_target(false)
        .with_writer(io::stderr);

    let file_layer = open_log_arc().map(|file| {
        tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_target(false)
            .with_writer(FileMaker(file))
    });

    let registry = tracing_subscriber::registry().with(filter).with(stderr_layer).with(file_layer);

    #[cfg(feature = "obs-tracy")]
    let registry = registry.with(tracing_tracy::TracyLayer::default());

    #[cfg(feature = "obs-chrome")]
    let registry = {
        let layer = match std::env::var("SKWD_WALL_TRACE") {
            Ok(path) if !path.is_empty() => {
                let (cl, guard) =
                    tracing_chrome::ChromeLayerBuilder::new().file(path).include_args(true).build();
                let _ = CHROME_GUARD.set(Mutex::new(Some(guard)));
                Some(cl)
            }
            _ => None,
        };
        registry.with(layer)
    };

    registry.init();
}

pub fn flush_chrome() {
    #[cfg(feature = "obs-chrome")]
    if let Some(mutex) = CHROME_GUARD.get() {
        if let Ok(mut guard) = mutex.lock() {
            drop(guard.take());
        }
    }
}
