// src/fetch/resolver.rs
use std::path::{PathBuf};
use crate::{paths, util::hash_url};
use super::downloader;

pub fn resolve_entry_from_url(raw: &str)
    -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>>
{
    // Accept plain URLs and uxntal://… forms
    let url = if raw.starts_with("uxntal:") {
        crate::urlutil::extract_target_from_uxntal(raw).ok_or("bad uxntal URL")?
    } else {
        raw.to_string()
    };
    println!("raw URL: {}", raw);
    println!("Fetching from URL: {}", url);
    let roms_dir = paths::uxntal_roms_get_path()
        .ok_or("Failed to get uxntal roms directory")?;
    let rom_dir = roms_dir.join(format!("{}", hash_url(&url)));
    std::fs::create_dir_all(&rom_dir)?;

    // repo-aware path or single file
    if super::parse_repo(&url).is_some() {
        println!("Fetching repo tree for URL: {}", url);
        let res = super::fetch_repo_tree(&url, &rom_dir)?;
                //    let status = std::process::Command::new("e_window")
                // .arg(&format!("--title={}", url))
                // .arg(&url)
                // .status();
        Ok((res.entry_local, rom_dir))
    } else {
        println!("Fetching single file for URL: {}", url);
        let name = url
            .rsplit('/')
            .find(|s| !s.is_empty())
            .unwrap_or("downloaded.tal");
        let out = rom_dir.join(name);
        if !out.exists() {
            let bytes = downloader::http_get(&url)?;
            downloader::write_bytes(&out, &bytes)?;
            let _ = std::fs::write(out.with_extension("url"),
                                   format!("[InternetShortcut]\nURL={}\n", url));
        }
                //    let status = std::process::Command::new("e_window")
                // .arg(&format!("--title={}", url))
                // .arg(&url)
                // .status();
        Ok((out, rom_dir))
    }
}
