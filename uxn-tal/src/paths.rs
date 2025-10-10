// src/paths.rs
use std::path::PathBuf;

pub fn uxntal_roms_get_path() -> Option<PathBuf> {
    let home_dir = dirs::home_dir();
    if let Some(home) = home_dir {
        let uxn_path = home.join(".uxntal").join("roms");
        return Some(uxn_path);
    }
    None
}