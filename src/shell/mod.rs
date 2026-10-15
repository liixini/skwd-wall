#[cfg(target_os = "linux")]
mod fontconfig_env;
#[cfg(target_os = "linux")]
mod fonts;
#[allow(clippy::module_inception)]
mod shell;
mod tests;

pub use shell::*;
