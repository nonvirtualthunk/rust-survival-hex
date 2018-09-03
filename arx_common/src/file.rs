use prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::env;
use dirs;


#[cfg(target_os = "macos")]
pub fn default_path() ->  Option<PathBuf> { Some(dirs::home_dir().expect("No home directory on mac os x?").join("Library/Application Support/")) }
#[cfg(target_os = "linux")]
pub fn default_path() -> Option<PathBuf> { Some(dirs::home_dir().expect("No home directory on linux?")) }
#[cfg(target_os = "windows")]
pub fn default_path() -> Option<PathBuf> { None }

pub fn save_game_path(game_name : Str) -> Option<PathBuf> {
    if let Ok(app_data_path) = env::var("APPDATA") {
        Some([&app_data_path, game_name, "saves"].iter().collect::<PathBuf>())
    } else if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
        Some([&xdg_data_home, game_name, "saves"].iter().collect::<PathBuf>())
    } else if let Some(default_path) = default_path() {
        Some(default_path.join([game_name, "saves"].iter().collect::<PathBuf>()))
    } else {
        None
    }
}