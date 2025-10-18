use std::{fs, io::Write, path::Path, process::Command};

pub fn http_get(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let resp = reqwest::blocking::get(url)?;
    if resp.status().as_u16() == 403 {
        if let Ok(out) = Command::new("curl").args(["-L", "-sS", url]).output() {
            if out.status.success() {
                return Ok(out.stdout);
            } else {
                return Err(format!("curl failed with exit code {} for {}", out.status.code().unwrap_or(-1), url).into());
            }
        }
    }
    if resp.status().as_u16() == 404 {
        return Err(format!("HTTP 404 Not Found for {}", url).into());
    }
    if !resp.status().is_success() {
        return Err(format!("HTTP {} for {}", resp.status(), url).into());
    }
    Ok(resp.bytes()?.to_vec())
}

pub fn write_bytes(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(p) = path.parent() { fs::create_dir_all(p)?; }
    let mut f = fs::File::create(path)?;
    f.write_all(bytes)?;
    Ok(())
}
