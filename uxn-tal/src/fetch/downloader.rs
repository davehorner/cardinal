use crate::fetch::html_redirect::extract_linkedin_redirect;
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

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
pub fn http_get(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
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
