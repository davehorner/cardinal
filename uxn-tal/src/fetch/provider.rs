use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct RepoRef {
    pub host: String,         // "git.sr.ht" | "github.com"
    pub owner: String,        // "~rabbits" or "rabbits"
    pub repo: String,         // "drifblim"
    pub branch: String,       // "main" | "master" | "HEAD"
    pub path: Option<String>, // repo-relative file/dir (if any)
}

#[derive(Debug)]
pub struct FetchResult {
    pub entry_local: PathBuf, // local .tal to compile
    pub all_files: Vec<PathBuf>,
}

pub trait Provider: Send + Sync {
    fn parse_url(&self, url: &str) -> Option<RepoRef>;
    fn fetch_tal_tree(
        &self,
        rf: &RepoRef,
        out_root: &Path,
    ) -> Result<FetchResult, Box<dyn std::error::Error>>;

    /// Parse a git@ URL and return a RepoRef with url_git populated if this provider can handle it
    fn parse_git_url(&self, _url: &str) -> Option<(RepoRef, String)> {
        None // Default implementation returns None for providers that don't support git@ URLs
    }
}
