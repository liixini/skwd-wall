use std::ffi::OsString;
use std::path::Path;

const FONTCONFIG_FILE: &str = "FONTCONFIG_FILE";
const FONTDB_CONFIG: &str = "<fontconfig><dir prefix=\"relative\">empty</dir></fontconfig>";
const FONTDB_DIR: &str = "empty";
const FONTDB_FILE: &str = "fonts.conf";
const FONTDB_PREFIX: &str = "fontdb-";
const FONTDB_NONUTF8: &str = "font cache path is not UTF-8";

struct FontconfigRestore(Option<OsString>);

impl Drop for FontconfigRestore {
    fn drop(&mut self) {
        match self.0.take() {
            Some(value) => unsafe { std::env::set_var(FONTCONFIG_FILE, value) },
            None => unsafe { std::env::remove_var(FONTCONFIG_FILE) },
        }
    }
}

pub(super) fn with_empty_config<T>(cache: &Path, build: impl FnOnce() -> T) -> Result<T, String> {
    std::fs::create_dir_all(cache).map_err(|err| err.to_string())?;
    let root = tempfile::Builder::new()
        .prefix(FONTDB_PREFIX)
        .tempdir_in(cache)
        .map_err(|err| err.to_string())?;
    std::fs::create_dir(root.path().join(FONTDB_DIR)).map_err(|err| err.to_string())?;
    let config = root.path().join(FONTDB_FILE);
    if config.to_str().is_none() {
        return Err(String::from(FONTDB_NONUTF8));
    }
    std::fs::write(&config, FONTDB_CONFIG).map_err(|err| err.to_string())?;
    let restore = FontconfigRestore(std::env::var_os(FONTCONFIG_FILE));
    unsafe { std::env::set_var(FONTCONFIG_FILE, config) };
    let result = build();
    drop(restore);
    Ok(result)
}
