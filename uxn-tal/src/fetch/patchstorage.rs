use super::{
    downloader::{http_get, write_bytes},
    provider::{FetchResult, Provider, RepoRef},
};
use std::path::{Path, PathBuf};
use url::Url;

pub struct PatchStorage;

impl PatchStorage {
    fn extract_download_url(html: &str) -> Option<String> {
        // Use scraper to find the download <a> tag with .orca href
        use scraper::{Html, Selector};
        let document = Html::parse_document(html);
        let selector = Selector::parse("a.ps-patch-download").ok()?;
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                if href.ends_with(".orca") {
                    if href.starts_with("http") {
                        return Some(href.to_string());
                    } else {
                        return Some(format!("https://patchstorage.com{}", href));
                    }
                }
            }
        }
        None
    }

    fn fetch_file(url: &str, out_root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let html = http_get(url)?;
        let html_str = String::from_utf8_lossy(&html);
        let download_url = Self::extract_download_url(&html_str)
            .ok_or_else(|| "PatchStorage: Download link not found".to_string())?;
        let file_bytes = http_get(&download_url)?;
        let filename = download_url
            .split('/')
            .next_back()
            .ok_or("PatchStorage: Invalid download URL")?;
        let local = out_root.join(filename);
        write_bytes(&local, &file_bytes)?;
        Ok(local)
    }
}

impl Provider for PatchStorage {
    fn parse_url(&self, url: &str) -> Option<RepoRef> {
        let u = Url::parse(url).ok()?;
        if u.domain()? != "patchstorage.com" {
            return None;
        }
        Some(RepoRef {
            host: "patchstorage.com".into(),
            owner: "".into(),
            repo: "".into(),
            branch: "".into(),
            path: Some(u.path().trim_start_matches('/').to_string()),
        })
    }

    fn fetch_tal_tree(
        &self,
        r: &RepoRef,
        out_root: &Path,
    ) -> Result<FetchResult, Box<dyn std::error::Error>> {
        let url = format!(
            "https://patchstorage.com/{}",
            r.path.as_deref().unwrap_or("")
        );
        let filename = url.split('/').next_back().unwrap_or("");
        let mut entry_file: Option<PathBuf> = None;
        if !filename.is_empty() {
            let local = out_root.join(filename);
            if local.is_file() {
                eprintln!("[PatchStorage] Using cached file: {}", local.display());
                entry_file = Some(local);
            }
        }
        // If filename is empty or not a file, look for any .orca file in the cache dir
        if entry_file.is_none() {
            if let Ok(entries) = std::fs::read_dir(out_root) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "orca").unwrap_or(false) && path.is_file() {
                        eprintln!("[PatchStorage] Using cached file: {}", path.display());
                        entry_file = Some(path);
                        break;
                    }
                }
            }
        }
        if let Some(local) = entry_file {
            return Ok(FetchResult {
                entry_local: local.clone(),
                all_files: vec![local],
            });
        }
        // Otherwise, fetch and cache
        let local = Self::fetch_file(&url, out_root)?;
        Ok(FetchResult {
            entry_local: local.clone(),
            all_files: vec![local],
        })
    }
}
