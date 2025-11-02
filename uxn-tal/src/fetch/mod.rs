pub mod patchstorage;
use patchstorage::PatchStorage;
pub mod codeberg;
pub mod downloader;
pub mod github;
pub mod html_redirect;
pub mod includes;
pub mod provider;
pub mod resolver;
pub mod srht;

use codeberg::Codeberg;
use github::GitHub;
use provider::{FetchResult, Provider, RepoRef};
use srht::SourceHut;
use uxn_tal_defined::v1::ProtocolParseResult;

/// Parse a uxntal protocol URL with git URL enhancement
/// This is the primary parsing function that should be used instead of ProtocolParser::parse directly
pub fn parse_uxntal_url(raw_url: &str) -> ProtocolParseResult {
    let mut result = uxn_tal_defined::v1::ProtocolParser::parse(raw_url);

    // If repo_ref is already populated, don't override it
    if result.repo_ref.is_some() {
        return result;
    }

    // Try to parse the URL as a git@ URL using each provider
    if result.url.starts_with("git@") {
        let providers: Vec<Box<dyn Provider>> = vec![
            Box::new(SourceHut),
            Box::new(GitHub),
            Box::new(Codeberg),
            Box::new(PatchStorage),
        ];

        for provider in providers {
            if let Some((repo_ref, url_git)) = provider.parse_git_url(&result.url) {
                result.repo_ref = Some(uxn_tal_defined::v1::RepoRef {
                    provider: match repo_ref.host.as_str() {
                        "git.sr.ht" => "sourcehut".to_string(),
                        "github.com" => "github".to_string(),
                        "codeberg.org" => "codeberg".to_string(),
                        _ => "unknown".to_string(),
                    },
                    owner: repo_ref.owner,
                    repo: repo_ref.repo,
                    branch: repo_ref.branch,
                    path: repo_ref.path.unwrap_or_default(),
                    url_git,
                });
                break;
            }
        }
    }

    result
}

pub fn parse_repo(url: &str) -> Option<(Box<dyn Provider>, RepoRef)> {
    let provs: Vec<Box<dyn Provider>> = vec![
        Box::new(SourceHut),
        Box::new(GitHub),
        Box::new(Codeberg),
        Box::new(PatchStorage),
    ];
    for p in provs.into_iter() {
        let res = p.parse_url(url);
        eprintln!(
            "[parse_repo] provider: {} url: {} match: {}",
            std::any::type_name::<Box<dyn Provider>>(),
            url,
            res.is_some()
        );
        if let Some(r) = res {
            return Some((p, r));
        }
    }
    eprintln!("[parse_repo] no provider matched for url: {}", url);
    None
}

pub fn parse_git_url_direct(url: &str) -> Option<RepoRef> {
    let provs: Vec<Box<dyn Provider>> = vec![
        Box::new(SourceHut),
        Box::new(GitHub),
        Box::new(Codeberg),
        Box::new(PatchStorage),
    ];
    for p in provs.into_iter() {
        if let Some((repo_ref, _url_git)) = p.parse_git_url(url) {
            return Some(repo_ref);
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
