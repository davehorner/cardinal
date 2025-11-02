// src/fetch/resolver.rs
use super::downloader;
use super::downloader::DownloaderRedirect;
use crate::fetch::html_redirect::extract_linkedin_redirect;
use crate::paths;
use crate::util::pause_on_error;
use std::path::PathBuf;
use uxn_tal_common::hash_url;

pub fn resolve_entry_from_url(raw: &str) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    // Accept plain URLs and uxntal://… forms with automatic git parsing enhancement
    let parsed = crate::parse_uxntal_url(raw);
    let url = if !parsed.url.is_empty() {
        parsed.url.clone()
    } else {
        raw.to_string()
    };

    // Check if this is a git@ URL that we can handle with git operations
    if url.starts_with("git@") {
        // Try to parse with git provider integration
        let repo_ref = if let Some(repo_ref) = &parsed.repo_ref {
            repo_ref.clone()
        } else {
            // If parsed.repo_ref is None, try parsing the git URL directly
            println!(
                "[GIT] No repo_ref in parsed result, trying direct git parsing for: {}",
                url
            );
            match super::parse_git_url_direct(&url) {
                Some(provider_repo_ref) => {
                    println!(
                        "[GIT] Direct git parsing succeeded: {:?}",
                        provider_repo_ref
                    );
                    // Convert provider::RepoRef to uxn_tal_defined::RepoRef
                    // Preserve original protocol (http/https) for git@http(s):// URLs
                    let url_git = if url.starts_with("git@https://") {
                        // For git@https:// URLs, use HTTPS format
                        format!(
                            "https://{}/{}/{}",
                            provider_repo_ref.host, provider_repo_ref.owner, provider_repo_ref.repo
                        )
                    } else if url.starts_with("git@http://") {
                        // For git@http:// URLs, use HTTP format
                        format!(
                            "http://{}/{}/{}",
                            provider_repo_ref.host, provider_repo_ref.owner, provider_repo_ref.repo
                        )
                    } else {
                        // For regular git@ URLs, use SSH format
                        format!(
                            "git@{}:{}/{}",
                            provider_repo_ref.host, provider_repo_ref.owner, provider_repo_ref.repo
                        )
                    };

                    uxn_tal_defined::RepoRef {
                        provider: provider_repo_ref.host.clone(),
                        owner: provider_repo_ref.owner.clone(),
                        repo: provider_repo_ref.repo.clone(),
                        branch: provider_repo_ref.branch.clone(),
                        path: provider_repo_ref.path.clone().unwrap_or_default(),
                        url_git,
                    }
                }
                None => {
                    println!("[GIT] Direct git parsing failed, falling back to regular handling");
                    return resolve_non_git_url(&url);
                }
            }
        };

        println!("[GIT] Using git operations for: {}", url);
        println!("[GIT] RepoRef: {:?}", repo_ref);

        let roms_dir =
            paths::uxntal_roms_get_path().ok_or("Failed to get uxntal roms directory")?;
        let cache_dir = roms_dir.join(format!("{}", hash_url(&repo_ref.url_git)));
        std::fs::create_dir_all(&cache_dir)?;

        match super::downloader::resolve_and_fetch_git_entry(
            &repo_ref.url_git,
            &repo_ref.branch,
            &repo_ref.path,
            &cache_dir,
        ) {
            Ok((entry_path, cache_dir)) => {
                println!(
                    "[GIT] Successfully resolved: entry_path={}, cache_dir={}",
                    entry_path.display(),
                    cache_dir.display()
                );
                return Ok((entry_path, cache_dir));
            }
            Err(e) => {
                // If git operations fail, fall through to regular handling
                eprintln!("[GIT] Git operations failed: {}", e);
                return Err(e);
            }
        }
    }

    resolve_non_git_url(&url)
}

fn resolve_non_git_url(url: &str) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let roms_dir = paths::uxntal_roms_get_path().ok_or("Failed to get uxntal roms directory")?;
    let rom_dir = roms_dir.join(format!("{}", hash_url(url)));
    std::fs::create_dir_all(&rom_dir)?;

    if super::parse_repo(url).is_some() {
        // For repos, always fetch (repo tree may change)
        println!("Fetching repo tree for URL: {}", url);
        let res = super::fetch_repo_tree(url, &rom_dir)?;
        Ok((res.entry_local, rom_dir))
    } else {
        let name = url
            .rsplit('/')
            .find(|s| !s.is_empty())
            .unwrap_or("downloaded.tal");
        let out = rom_dir.join(name);
        if !out.exists() {
            println!("Fetching single file for URL: {}", url);
            match downloader::http_get(url) {
                Ok(bytes) => {
                    downloader::write_bytes(&out, &bytes)?;
                    let _ = std::fs::write(
                        out.with_extension("url"),
                        format!("[InternetShortcut]\nURL={}\n", url),
                    );
                }
                Err(e) => {
                    if let Some(redirect) = e.downcast_ref::<DownloaderRedirect>() {
                        // Write the legacy error message to the cache file
                        let _ = std::fs::write(&out, e.to_string());
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
            println!("Using cached file: {}", out.display());
            // If cached file is HTML, check for LinkedIn-style redirect and re-exec if found
            let content = std::fs::read_to_string(&out).unwrap_or_default();
            let legacy_prefix = "HTML redirect detected: ";
            if let Some(stripped) = content.strip_prefix(legacy_prefix) {
                if let Some(url) = stripped.split_whitespace().next() {
                    eprintln!(
                        "[uxntal] Detected legacy redirect message, redirecting to: {}",
                        url
                    );
                    let _ = std::fs::write(
                        out.with_extension("url"),
                        format!("[InternetShortcut]\nURL={}\n", url),
                    );
                    // Re-exec self with the new URL and exit
                    let exe = std::env::current_exe()?;
                    let mut new_arg = url.to_string();
                    if let Some(orig_arg) = std::env::args().nth(1) {
                        if orig_arg.starts_with("uxntal:") {
                            if let Some(idx) = orig_arg.find("://") {
                                let prefix = &orig_arg[..idx + 3];
                                new_arg = format!("{}{}", prefix, url);
                            } else {
                                new_arg = format!("uxntal://{}", url);
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

            // 2. Otherwise, try HTML redirect detection
            let redirect_url = extract_linkedin_redirect(&content);
            eprintln!(
                "[uxntal] DEBUG: extract_linkedin_redirect result: {:?}",
                redirect_url
            );
            if let Some(real_url) = redirect_url {
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
            } else {
                // If file looks like HTML, refuse to assemble
                let content_trim = content.trim_start();
                if content_trim.to_ascii_lowercase().starts_with("<html")
                    || content_trim
                        .to_ascii_lowercase()
                        .starts_with("<!doctype html")
                {
                    eprintln!("[uxntal] ERROR: Cached file appears to be HTML, but no LinkedIn redirect was found. Refusing to assemble HTML file: {}", out.display());
                    pause_on_error();
                    std::process::exit(1);
                }
            }
        }
        Ok((out, rom_dir))
    }
}
