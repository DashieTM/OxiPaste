use std::path::PathBuf;

use crate::SVG_PATH;

pub fn mk_svg(name: &'static str) -> PathBuf {
    SVG_PATH.join(name)
}

#[cfg(debug_assertions)]
pub fn svg_path() -> PathBuf {
    PathBuf::from("./assets")
}

#[cfg(not(debug_assertions))]
pub fn svg_path() -> PathBuf {
    use std::env;
    use std::path::Path;
    match env::current_exe() {
        Ok(exe_path) => exe_path
            .parent()
            .unwrap_or(&Path::new("/"))
            .join("../share/pixmaps/oxipaste"),
        Err(_) => PathBuf::from("./assets"),
    }
}
