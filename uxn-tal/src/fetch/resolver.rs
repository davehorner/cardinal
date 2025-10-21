// src/fetch/resolver.rs
use super::downloader;
use super::downloader::DownloaderRedirect;
use crate::fetch::html_redirect::extract_linkedin_redirect;
use crate::{
    paths,
    util::{hash_url, pause_on_error},
};
use std::path::PathBuf;

pub fn resolve_entry_from_url(raw: &str) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    // Accept plain URLs and uxntal://… forms
    let url = if raw.starts_with("uxntal:") {
        crate::urlutil::extract_target_from_uxntal(raw).unwrap_or_else(|| raw.to_string())
    } else {
        raw.to_string()
    };
    println!("raw URL: {}", raw);
    println!("Fetching from URL: {}", url);
    let roms_dir = paths::uxntal_roms_get_path().ok_or("Failed to get uxntal roms directory")?;
    let rom_dir = roms_dir.join(format!("{}", hash_url(&url)));
    std::fs::create_dir_all(&rom_dir)?;

    if super::parse_repo(&url).is_some() {
        println!("Fetching repo tree for URL: {}", url);
        let res = super::fetch_repo_tree(&url, &rom_dir)?;
        Ok((res.entry_local, rom_dir))
    } else {
        println!("Fetching single file for URL: {}", url);
        let name = url
            .rsplit('/')
            .find(|s| !s.is_empty())
            .unwrap_or("downloaded.tal");
        let out = rom_dir.join(name);
        if !out.exists() {
            match downloader::http_get(&url) {
                Ok(bytes) => {
                    downloader::write_bytes(&out, &bytes)?;
                    let _ = std::fs::write(
                        out.with_extension("url"),
                        format!("[InternetShortcut]\nURL={}\n", url),
                    );
                }
                Err(e) => {
                    if let Some(redirect) = e.downcast_ref::<DownloaderRedirect>() {
                        // If it's a redirect, try to get the HTML body from the error (if available)
                        if let Some(html_body) = e
                            .downcast_ref::<DownloaderRedirect>()
                            .map(|_| e.to_string())
                        {
                            // Write the HTML body to the cache file
                            let _ = std::fs::write(&out, html_body);
                        }
                        let _ = std::fs::write(
                            out.with_extension("url"),
                            format!("[InternetShortcut]\nURL={}\n", url),
                        );
                        // Re-exec self with the new URL and exit
                        let exe = std::env::current_exe()?;
                        let mut new_arg = redirect.url.to_string();
                        if let Some(orig_arg) = std::env::args().nth(1) {
                            if orig_arg.starts_with("uxntal:") {
                                if let Some(idx) = orig_arg.find("://") {
                                    let prefix = &orig_arg[..idx + 3];
                                    new_arg = format!("{}{}", prefix, redirect.url);
                                } else {
                                    new_arg = format!("uxntal://{}", redirect.url);
                                }
                            }
                        }
                        eprintln!(
                            "[uxntal] Attempting to re-exec self: {:?} with arg: {}",
                            exe, new_arg
                        );
                        match std::process::Command::new(&exe).arg(new_arg).status() {
                            Ok(status) => {
                                eprintln!(
                                    "[uxntal] Child process exited with status: {:?}",
                                    status
                                );
                                pause_on_error();
                                std::process::exit(status.code().unwrap_or(0));
                            }
                            Err(e) => {
                                eprintln!("[uxntal] Failed to spawn new process: {}", e);
                                pause_on_error();
                                std::process::exit(1);
                            }
                        }
                    } else {
                        return Err(e);
                    }
                }
            }
        } else {
            // If cached file is HTML, check for LinkedIn-style redirect and re-exec if found
            let content = std::fs::read_to_string(&out).unwrap_or_default();
            if let Some(real_url) = extract_linkedin_redirect(&content) {
                let _ = std::fs::write(
                    out.with_extension("url"),
                    format!("[InternetShortcut]\nURL={}\n", real_url),
                );
                // Re-exec self with the new URL and exit
                let exe = std::env::current_exe()?;
                // Try to reconstruct the original uxntal:... prefix if present
                let mut new_arg = real_url.to_string();
                if let Some(orig_arg) = std::env::args().nth(1) {
                    if orig_arg.starts_with("uxntal:") {
                        // Replace the URL part in the original arg with the new URL
                        if let Some(idx) = orig_arg.find("://") {
                            let prefix = &orig_arg[..idx + 3];
                            new_arg = format!("{}{}", prefix, real_url);
                        } else {
                            new_arg = format!("uxntal://{}", real_url);
                        }
                    }
                }
                eprintln!(
                    "[uxntal] Attempting to re-exec self: {:?} with arg: {}",
                    exe, new_arg
                );
                match std::process::Command::new(&exe).arg(new_arg).status() {
                    Ok(status) => {
                        eprintln!("[uxntal] Child process exited with status: {:?}", status);
                        pause_on_error();
                        std::process::exit(status.code().unwrap_or(0));
                    }
                    Err(e) => {
                        eprintln!("[uxntal] Failed to spawn new process: {}", e);
                        pause_on_error();
                        std::process::exit(1);
                    }
                }
            }
        }
        Ok((out, rom_dir))
    }
}
