use crate::fetch::provider::{FetchResult, Provider, RepoRef};
use std::path::{Path, PathBuf};

pub struct LocalFile;

impl Provider for LocalFile {
    fn parse_url(&self, url: &str) -> Option<RepoRef> {
        if !url.starts_with("file://") {
            return None;
        }

        // Remove "file://" prefix and get the local path
        let path = &url[7..]; // Remove "file://"

        // For local files, we create a pseudo RepoRef structure
        Some(RepoRef {
            host: "localhost".to_string(),
            owner: "local".to_string(),
            repo: "files".to_string(),
            branch: "main".to_string(),
            path: Some(path.to_string()),
        })
    }

    fn fetch_tal_tree(
        &self,
        rf: &RepoRef,
        out_root: &Path,
    ) -> Result<FetchResult, Box<dyn std::error::Error>> {
        let file_path = rf.path.as_ref().ok_or("No file path provided")?;

        // Convert relative path to absolute if needed
        let absolute_path = if Path::new(file_path).is_absolute() {
            PathBuf::from(file_path)
        } else {
            // Resolve relative to current working directory
            std::env::current_dir()?.join(file_path)
        };

        // Check if file exists
        if !absolute_path.exists() {
            return Err(format!("File not found: {}", absolute_path.display()).into());
        }

        // For local files, we just copy the file to the output directory
        let file_name = absolute_path
            .file_name()
            .ok_or("Invalid file name")?
            .to_string_lossy()
            .to_string();

        let target_path = out_root.join(&file_name);

        // Create output directory if it doesn't exist
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Copy the file
        std::fs::copy(&absolute_path, &target_path)?;

        Ok(FetchResult {
            entry_local: target_path.clone(),
            all_files: vec![target_path],
        })
    }
}
