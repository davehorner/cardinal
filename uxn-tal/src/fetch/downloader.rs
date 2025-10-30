use crate::fetch::html_redirect::extract_linkedin_redirect;
use crate::{fetch, resolve_entry_from_url};
use std::fmt;
use std::{fs, io::Write, path::Path};
use uxn_tal_defined::ProtocolParser;

#[derive(Debug)]
pub struct DownloaderRedirect {
    pub url: String,
}

impl fmt::Display for DownloaderRedirect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTML redirect detected: {}", self.url)
    }
}

impl std::error::Error for DownloaderRedirect {}

/// Resolves a URL (including uxntal: protocol) and fetches the main file and all includes into the cache directory.
/// Returns (entry_path, cache_dir). This is a unified interface for protocol parsing, downloading, and include resolution.
pub fn resolve_and_fetch_entry(
    raw: &str,
) -> Result<(std::path::PathBuf, std::path::PathBuf), Box<dyn std::error::Error>> {
    // 1. Parse protocol, extract real URL if needed (prefer uxntal_protocol)
    let parsed = if raw.starts_with("uxntal:") {
        ProtocolParser::parse(raw)
    } else {
        ProtocolParser::parse(&format!("uxntal://{}", raw))
    };
    let url = if !parsed.url.is_empty() {
        parsed.url.clone()
    } else {
        raw.to_string()
    };
    // 2. Use fetch_repo_tree if it's a repo, else download single file
    let (entry_path, cache_dir) = if fetch::parse_repo(&url).is_some() {
        let roms_dir =
            crate::paths::uxntal_roms_get_path().ok_or("Failed to get uxntal roms directory")?;
        use uxn_tal_common::hash_url;
        let rom_dir = roms_dir.join(format!("{}", hash_url(&url)));
        std::fs::create_dir_all(&rom_dir)?;
        let res = fetch::fetch_repo_tree(&url, &rom_dir)?;
        (res.entry_local, rom_dir)
    } else {
        // fallback to resolve_entry_from_url for single files
        resolve_entry_from_url(raw)?
    };

    // ORCA MODE: If orca mode is set, always copy canonical orca.rom into the cache dir and return that as the ROM path
    if let Some(orca_mode) = parsed.get("orca") {
        if orca_mode.as_bool() == Some(true) {
            // Find canonical orca.rom (try workspace roms/orca.rom)
            let workspace_orca_rom = std::path::Path::new("roms/orca.rom");
            let target_orca_rom = cache_dir.join("orca.rom");
            if workspace_orca_rom.exists() {
                std::fs::copy(workspace_orca_rom, &target_orca_rom)?;
                return Ok((target_orca_rom, cache_dir));
            }
        }
    }
    Ok((entry_path, cache_dir))
}

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
pub fn http_get(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Always use curl for git.sr.ht URLs
    if url.contains("git.sr.ht") {
        if let Ok(out) = std::process::Command::new("curl")
            .args(["-L", "-sS", url])
            .output()
        {
            if out.status.success() {
                return Ok(out.stdout);
            } else {
                return Err(format!(
                    "curl failed with exit code {} for {}",
                    out.status.code().unwrap_or(-1),
                    url
                )
                .into());
            }
        } else {
            return Err("Failed to spawn curl process".into());
        }
    }

    // Default: use reqwest for other URLs
    let resp = reqwest::blocking::get(url)?;
    let final_url = resp.url().to_string();
    let status = resp.status();
    eprintln!("[DEBUG] HTTP status: {}", status);
    eprintln!("[DEBUG] Final URL: {}", final_url);
    eprintln!("[DEBUG] Response headers:");
    for (k, v) in resp.headers().iter() {
        eprintln!("  {}: {:?}", k, v);
    }
    let is_html_content = resp
        .headers()
        .get("content-type")
        .is_some_and(|v| v.to_str().unwrap_or("").contains("text/html"));
    let body_bytes = resp.bytes()?;
    let body_str = String::from_utf8_lossy(&body_bytes);
    eprintln!(
        "[DEBUG] Response body (first 512 bytes):\n{}",
        &body_str[..body_str.len().min(512)]
    );

    // Try to detect LinkedIn-style HTML redirect pages
    if status.is_success() && is_html_content {
        if let Some(href) = extract_linkedin_redirect(&body_str) {
            eprintln!("[DEBUG] Found HTML redirect link: {}", href);
            // Return a custom error so resolver can handle it (do NOT fetch the real file here)
            return Err(Box::new(DownloaderRedirect { url: href }));
        }
    }

    if final_url != url {
        eprintln!(
            "[DEBUG] Redirect detected: {} -> {} (status: {})",
            url, final_url, status
        );
    }
    if status.as_u16() == 403 {
        if let Ok(out) = std::process::Command::new("curl")
            .args(["-L", "-sS", url])
            .output()
        {
            if out.status.success() {
                return Ok(out.stdout);
            } else {
                return Err(format!(
                    "curl failed with exit code {} for {}",
                    out.status.code().unwrap_or(-1),
                    url
                )
                .into());
            }
        }
    }
    if status.as_u16() == 404 {
        return Err(format!("HTTP 404 Not Found for {}", final_url).into());
    }
    if !status.is_success() {
        return Err(format!("HTTP {} for {}", status, final_url).into());
    }
    Ok(body_bytes.to_vec())
}

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub fn http_get(_url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    Err("http_get is not available in browser WASM".into())
}

pub fn write_bytes(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p)?;
    }
    let mut f = fs::File::create(path)?;
    f.write_all(bytes)?;
    Ok(())
}
