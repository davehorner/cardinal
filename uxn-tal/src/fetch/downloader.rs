use crate::fetch::html_redirect::extract_linkedin_redirect;
use crate::{fetch, resolve_entry_from_url};
use std::fmt;
use std::{fs, io::Write, path::Path};

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
    // 1. Parse protocol and enhance with git parsing
    let parsed = crate::fetch::parse_uxntal_url(raw);

    // 2. Check if we have git repository information
    if let Some(repo_ref) = &parsed.repo_ref {
        eprintln!("[GIT] Using git operations for: {}", raw);
        use uxn_tal_common::hash_url;
        let roms_dir =
            crate::paths::uxntal_roms_get_path().ok_or("Failed to get uxntal roms directory")?;
        let cache_dir = roms_dir.join(format!("{}", hash_url(raw)));
        std::fs::create_dir_all(&cache_dir)?;

        return resolve_and_fetch_git_entry(
            &repo_ref.url_git,
            &repo_ref.branch,
            &repo_ref.path,
            &cache_dir,
        );
    }

    // 3. Fall back to existing logic for non-git URLs
    let url = if !parsed.url.is_empty() {
        parsed.url.clone()
    } else {
        raw.to_string()
    };

    // Use fetch_repo_tree if it's a repo, else download single file
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

/// Ensure a git repository is cloned and up to date in the cache directory.
/// Returns the path to the repository root.
pub fn ensure_git_repo(
    url_git: &str,
    branch: &str,
    cache_dir: &Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = cache_dir;

    // Check if this is already a git repository
    let git_dir = repo_dir.join(".git");

    if git_dir.exists() {
        eprintln!("[GIT] Repository exists, updating: {}", repo_dir.display());
        update_git_repo(repo_dir, branch)?;
    } else {
        eprintln!(
            "[GIT] Cloning repository: {} -> {}",
            url_git,
            repo_dir.display()
        );
        clone_git_repo(url_git, repo_dir, branch)?;
    }

    Ok(repo_dir.to_path_buf())
}

/// Clone a git repository to the specified directory
fn clone_git_repo(
    url_git: &str,
    target_dir: &Path,
    branch: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure parent directory exists
    if let Some(parent) = target_dir.parent() {
        fs::create_dir_all(parent)?;
    }

    // Remove target directory if it exists but isn't a git repo
    if target_dir.exists() {
        fs::remove_dir_all(target_dir)?;
    }

    // Check for common SSH issues
    if url_git.starts_with("git@") && url_git.contains("codeberg.org") {
        return Err("Codeberg does not support SSH protocol. Use HTTPS instead (git@https://codeberg.org/...)".into());
    }

    // Clone with specific branch
    let output = crate::util::create_git_command()
        .args(["clone", "--branch", branch, "--single-branch", url_git])
        .arg(target_dir)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Provide more helpful error messages
        let error_msg = if stderr.contains("Repository not found") {
            format!("Repository not found: {}. Check if the repository exists and is publicly accessible.", url_git)
        } else if stderr.contains("Host key verification failed") {
            format!("SSH host key verification failed for {}. Add the host to your known_hosts file or use HTTPS instead.", url_git)
        } else if stderr.contains("Permission denied") {
            format!(
                "Permission denied for {}. Check SSH keys or use HTTPS for public repositories.",
                url_git
            )
        } else if stderr.contains("Could not read from remote repository") {
            format!(
                "Could not access repository: {}. Check URL, permissions, or try HTTPS.",
                url_git
            )
        } else {
            format!("Git clone failed for {}: {}", url_git, stderr.trim())
        };

        return Err(error_msg.into());
    }

    eprintln!("[GIT] Successfully cloned {} (branch: {})", url_git, branch);
    Ok(())
}

/// Update an existing git repository
fn update_git_repo(repo_dir: &Path, branch: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Change to repo directory and fetch
    let output = crate::util::create_git_command()
        .args(["fetch", "origin"])
        .current_dir(repo_dir)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("[GIT] Warning: fetch failed: {}", stderr);
    }

    // Checkout the specified branch
    let output = crate::util::create_git_command()
        .args(["checkout", branch])
        .current_dir(repo_dir)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_msg = if stderr.contains("did not match any file(s) known to git") {
            format!("Branch '{}' does not exist in repository. Available branches can be checked with 'git branch -a'", branch)
        } else {
            format!(
                "Git checkout failed for branch '{}': {}",
                branch,
                stderr.trim()
            )
        };
        return Err(error_msg.into());
    }

    // Pull latest changes
    let output = crate::util::create_git_command()
        .args(["pull", "origin", branch])
        .current_dir(repo_dir)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("[GIT] Warning: pull failed: {}", stderr);
    }

    eprintln!("[GIT] Successfully updated repository (branch: {})", branch);
    Ok(())
}

/// Resolve and fetch entry for git URLs with RepoRef
pub fn resolve_and_fetch_git_entry(
    url_git: &str,
    branch: &str,
    file_path: &str,
    cache_dir: &Path,
) -> Result<(std::path::PathBuf, std::path::PathBuf), Box<dyn std::error::Error>> {
    // Clone or update the repository
    let repo_dir = ensure_git_repo(url_git, branch, cache_dir)?;

    // Resolve the specific file path within the repo
    let target_file = repo_dir.join(file_path);

    if !target_file.exists() {
        // Try to provide helpful suggestions for common issues
        let suggestions = if file_path.starts_with("src/") {
            let alt_path = file_path.strip_prefix("src/").unwrap();
            let alt_file = repo_dir.join(alt_path);
            if alt_file.exists() {
                format!(
                    "File found at root level: try '{}' instead of '{}'",
                    alt_path, file_path
                )
            } else {
                "Try checking the repository structure or branch name".to_string()
            }
        } else {
            let src_path = format!("src/{}", file_path);
            let src_file = repo_dir.join(&src_path);
            if src_file.exists() {
                format!(
                    "File found in src directory: try '{}' instead of '{}'",
                    src_path, file_path
                )
            } else {
                "Try checking the repository structure or branch name".to_string()
            }
        };

        return Err(format!(
            "File not found in repository: {} (looking for: {})\nRepository: {}\nSuggestion: {}",
            target_file.display(),
            file_path,
            url_git,
            suggestions
        )
        .into());
    }

    eprintln!("[GIT] Resolved file: {}", target_file.display());
    Ok((target_file, cache_dir.to_path_buf()))
}
