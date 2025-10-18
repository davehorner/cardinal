pub mod codeberg;
pub mod downloader;
pub mod github;
pub mod includes;
pub mod provider;
pub mod resolver;
pub mod srht;

use codeberg::Codeberg;
use github::GitHub;
use provider::{FetchResult, Provider, RepoRef};
use srht::SourceHut;

pub fn parse_repo(url: &str) -> Option<(Box<dyn Provider>, RepoRef)> {
    let provs: Vec<Box<dyn Provider>> =
        vec![Box::new(SourceHut), Box::new(GitHub), Box::new(Codeberg)];
    for p in provs.into_iter() {
        if let Some(r) = p.parse_url(url) {
            return Some((p, r));
        }
    }
    None
}

pub fn fetch_repo_tree(
    url: &str,
    out_root: &std::path::Path,
) -> Result<FetchResult, Box<dyn std::error::Error>> {
    let (prov, rf) = parse_repo(url).ok_or("unsupported repo host or bad URL")?;
    prov.fetch_tal_tree(&rf, out_root)
}
