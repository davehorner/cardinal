use super::{
    downloader::{http_get, write_bytes},
    includes::{parse_includes, resolve_include},
    provider::{FetchResult, Provider, RepoRef},
};
use std::{
    collections::{HashSet, VecDeque},
    fs,
    path::{Path, PathBuf},
};
use url::Url;

pub struct GitHub;

impl GitHub {
    fn raw_url(r: &RepoRef, repo_rel: &str) -> String {
        format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            r.owner, r.repo, r.branch, repo_rel
        )
    }
    // fn try_entry(r: &RepoRef, out: &Path, repo_rel: &str) -> Option<PathBuf> {
    //     let url = Self::raw_url(r, repo_rel);
    //     if let Ok(bytes) = http_get(&url) {
    //         let local = out.join(repo_rel);
    //         if write_bytes(&local, &bytes).is_ok() { return Some(local); }
    //     }
    //     None
    // }
    // fn branch_candidates(branch: &str) -> Vec<String> {
    //     if branch != "HEAD" { return vec![branch.to_string()]; }
    //     vec!["main".into(), "master".into(), "trunk".into()]
    // }
    fn fetch_file(
        r: &RepoRef,
        out_root: &Path,
        repo_rel: &str,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let url = Self::raw_url(r, repo_rel);
        let bytes = http_get(&url)?;
        let local = out_root.join(repo_rel);
        // Check for HTML/404 response
        let is_html = bytes.starts_with(b"<!doctype html") || bytes.starts_with(b"<html");
        if is_html {
            let _ = std::fs::remove_file(&local);
            return Err(format!("GitHub: Got HTML/404 for {}", url).into());
        }
        if let Some(p) = local.parent() {
            fs::create_dir_all(p)?;
        }
        write_bytes(&local, &bytes)?;
        Ok(local)
    }
}

impl Provider for GitHub {
    fn parse_url(&self, url: &str) -> Option<RepoRef> {
        let u = Url::parse(url).ok()?;
        if u.domain()? != "github.com" {
            return None;
        }
        let segs = u.path_segments()?.collect::<Vec<_>>();
        if segs.len() < 2 {
            return None;
        }

        let owner = segs[0].to_string();
        let repo = segs[1].trim_end_matches(".git").to_string();

        if segs.len() == 2 {
            return Some(RepoRef {
                host: "github.com".into(),
                owner,
                repo,
                branch: "HEAD".into(),
                path: None,
            });
        }

        // Accept GitHub views: blob/tree/raw/blame
        match segs[2] {
            "blob" | "tree" | "raw" | "blame" if segs.len() >= 4 => {
                // <view>/<ref>/<path...>   where <ref> can be a branch name or a commit SHA
                let branch = segs[3].to_string();
                let path = if segs.len() > 4 {
                    Some(segs[4..].join("/"))
                } else {
                    None
                };
                Some(RepoRef {
                    host: "github.com".into(),
                    owner,
                    repo,
                    branch,
                    path,
                })
            }
            _ => {
                // Fallback: treat remainder as repo path with unknown ref
                let path = if segs.len() > 2 {
                    Some(segs[2..].join("/"))
                } else {
                    None
                };
                Some(RepoRef {
                    host: "github.com".into(),
                    owner,
                    repo,
                    branch: "HEAD".into(),
                    path,
                })
            }
        }
    }
    fn fetch_tal_tree(
        &self,
        r: &RepoRef,
        out_root: &Path,
    ) -> Result<FetchResult, Box<dyn std::error::Error>> {
        let entry_rel = match &r.path {
            Some(p)
                if p.to_ascii_lowercase().ends_with(".tal")
                    || p.to_ascii_lowercase().ends_with(".rom")
                    || p.to_ascii_lowercase().ends_with(".rom.txt")
                    || p.to_ascii_lowercase().ends_with(".orca")
                    || p.to_ascii_lowercase().ends_with(".bas") =>
            {
                p.replace('\\', "/")
            }
            _ => return Err(
                "github: URL must point to a .tal, .rom, .rom.txt, .orca, or .bas file; not guessing entries"
                    .into(),
            ),
        };

        // Check cache for entry file first
        let entry_local = out_root.join(&entry_rel);
        if entry_local.exists() {
            eprintln!("[GitHub] Using cached file: {}", entry_local.display());
            return Ok(FetchResult {
                entry_local: entry_local.clone(),
                all_files: vec![entry_local],
            });
        }
        let entry_local = Self::fetch_file(r, out_root, &entry_rel)?;
        let mut all = vec![entry_local.clone()];
        // Recursively fetch includes for .tal files
        if entry_rel.to_ascii_lowercase().ends_with(".tal") {
            let mut visited: HashSet<String> = [entry_rel.clone()].into_iter().collect();
            let mut q: VecDeque<(String, PathBuf)> =
                VecDeque::from([(entry_rel.clone(), entry_local.clone())]);
            let entry_dir = Path::new(&entry_rel)
                .parent()
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();
            while let Some((curr_rel, curr_local)) = q.pop_front() {
                let src = fs::read_to_string(&curr_local).unwrap_or_default();
                for inc in parse_includes(&src) {
                    let target = resolve_include(&curr_rel, &inc);
                    if !visited.insert(target.clone()) {
                        continue;
                    }
                    let mut attempts: Vec<(String, Option<std::path::PathBuf>)> = Vec::new();
                    let mut success = None;
                    let mut errors = vec![];
                    // First attempt: entry_dir + target, unless target already starts with entry_dir
                    let first_attempt = if !entry_dir.is_empty() && !target.starts_with(&entry_dir)
                    {
                        format!("{}/{}", entry_dir, target)
                    } else {
                        target.clone()
                    };
                    attempts.push((first_attempt.clone(), None));
                    match Self::fetch_file(r, out_root, &first_attempt) {
                        Ok(loc) => {
                            success = Some((first_attempt.clone(), loc));
                        }
                        Err(e) => {
                            errors.push((first_attempt.clone(), format!("{}", e)));
                        }
                    }
                    // Fallback 1: deduped path
                    if success.is_none() {
                        let parts: Vec<&str> = target.split('/').collect();
                        if parts.len() >= 3 && parts[0] == parts[1] {
                            let deduped = parts[1..].join("/");
                            attempts.push((deduped.clone(), None));
                            match Self::fetch_file(r, out_root, &deduped) {
                                Ok(loc) => {
                                    success = Some((deduped.clone(), loc));
                                }
                                Err(e) => {
                                    errors.push((deduped.clone(), format!("{}", e)));
                                }
                            }
                        }
                    }
                    // Fallback 2: walk up ancestor directories
                    if success.is_none() {
                        if let Some(fname) = Path::new(&target).file_name() {
                            let mut ancestor = Path::new(&target);
                            while let Some(parent) = ancestor.parent() {
                                let try_path = parent.join(fname);
                                let try_str = try_path.to_string_lossy().replace('\\', "/");
                                if try_str == target {
                                    break;
                                }
                                attempts.push((try_str.clone(), None));
                                match Self::fetch_file(r, out_root, &try_str) {
                                    Ok(loc) => {
                                        success = Some((try_str.clone(), loc));
                                        break;
                                    }
                                    Err(e) => {
                                        errors.push((try_str.clone(), format!("{}", e)));
                                    }
                                }
                                ancestor = parent;
                            }
                        }
                    }
                    // Fallback 3: try just the filename in the entry file's directory
                    if success.is_none() {
                        if let Some(fname) = Path::new(&target).file_name() {
                            if !entry_dir.is_empty() {
                                let entry_dir_path = Path::new(&entry_dir).join(fname);
                                let entry_dir_str =
                                    entry_dir_path.to_string_lossy().replace('\\', "/");
                                attempts.push((entry_dir_str.clone(), None));
                                match Self::fetch_file(r, out_root, &entry_dir_str) {
                                    Ok(loc) => {
                                        success = Some((entry_dir_str.clone(), loc));
                                    }
                                    Err(e) => {
                                        errors.push((entry_dir_str.clone(), format!("{}", e)));
                                    }
                                }
                            }
                        }
                    }
                    if let Some((path, loc)) = success {
                        all.push(loc.clone());
                        q.push_back((path, loc));
                        continue;
                    }
                    // No fallback possible, return error with all attempts
                    let error_msg = format!(
                        "Failed to fetch include '{}'. Attempts: {}. Errors: {}",
                        inc,
                        attempts
                            .iter()
                            .map(|(p, _)| format!("'{}'", p))
                            .collect::<Vec<_>>()
                            .join(", "),
                        errors
                            .iter()
                            .map(|(p, e)| format!("{}: {}", p, e))
                            .collect::<Vec<_>>()
                            .join("; ")
                    );
                    return Err(error_msg.into());
                }
            }
        }
        Ok(FetchResult {
            entry_local: all[0].clone(),
            all_files: all,
        })

        // for b in Self::branch_candidates(&r_in.branch) {
        //     let r = RepoRef { branch: b.clone(), ..r_in.clone() };

        //     let candidates: Vec<String> = match &r.path {
        //         Some(p) if p.ends_with(".tal") => vec![p.clone()],
        //         Some(p) if !p.is_empty()       => repo_entry_guesses().iter().map(|g| format!("{}/{}", p.trim_end_matches('/'), g)).collect(),
        //         _ => repo_entry_guesses().iter().map(|s| s.to_string()).collect(),
        //     };

        //     if let Some((entry_rel, entry_local)) =
        //         candidates.into_iter().find_map(|c| Self::try_entry(&r, out_root, &c).map(|l| (c, l)))
        //     {
        //         let mut visited = HashSet::new();
        //         let mut all = vec![entry_local.clone()];
        //         visited.insert(entry_rel.clone());
        //         let mut q = VecDeque::new();
        //         q.push_back((entry_rel, entry_local));

        //         let mut steps = 0usize;
        //         while let Some((repo_rel, local)) = q.pop_front() {
        //             if steps >= 200 { break; }
        //             steps += 1;

        //             let src = fs::read_to_string(&local).unwrap_or_default();
        //             for inc in parse_includes(&src) {
        //                 let target = resolve_include(&repo_rel, &inc);
        //                 if !visited.insert(target.clone()) { continue; }
        //                 let url = Self::raw_url(&r, &target);
        //                 println!("Fetching include from URL: {}", url);
        //                 match http_get(&url) {
        //                     Ok(bytes) => {
        //                         let loc = out_root.join(&target);
        //                         if write_bytes(&loc, &bytes).is_ok() {
        //                             all.push(loc.clone());
        //                             q.push_back((target, loc));
        //                         }
        //                     }
        //                     Err(_) => eprintln!("warning: include not found in GitHub repo: {} (branch {})", target, r.branch),
        //                 }
        //             }
        //         }
        //         return Ok(FetchResult { entry_local: all[0].clone(), all_files: all });
        //     }
        // }
        // Err("Could not find an entry .tal in GitHub repo across tried branches".into())
    }

    fn parse_git_url(&self, url: &str) -> Option<(RepoRef, String)> {
        // Normalize double git@ to single git@
        let normalized_url = if url.starts_with("git@git@") {
            &url[4..] // Remove one "git@" prefix
        } else {
            url
        };

        // Handle .git/ format: git@github.com:owner/repo.git/path/file.tal
        if let Some(git_pos) = normalized_url.find(".git/") {
            let repo_part = &normalized_url[..git_pos + 4]; // Include ".git"
            let file_part = &normalized_url[git_pos + 5..]; // Skip ".git/"

            if repo_part.starts_with("git@github.com:") {
                let path_part = &repo_part[15..repo_part.len() - 4]; // Remove "git@github.com:" and ".git"
                let segments: Vec<&str> = path_part.split('/').collect();

                if segments.len() >= 2 {
                    let owner = segments[0].to_string();
                    let repo = segments[1].to_string();

                    let repo_ref = RepoRef {
                        host: "github.com".to_string(),
                        owner: owner.clone(),
                        repo: repo.clone(),
                        branch: "main".to_string(), // Default to main for .git format
                        path: Some(file_part.to_string()),
                    };

                    let url_git = format!("git@github.com:{}/{}.git", owner, repo);
                    return Some((repo_ref, url_git));
                }
            }
        }

        // Handle git@github.com:owner/repo/branch/path/file.tal (SSH format)
        if let Some(path_part) = normalized_url.strip_prefix("git@github.com:") {
            // Remove "git@github.com:"
            let segments: Vec<&str> = path_part.split('/').collect();

            if segments.len() < 4 {
                return None;
            }

            let owner = segments[0].to_string();
            let repo = segments[1].to_string();

            // Skip "tree" segment if present
            let (branch, path_start) = if segments[2] == "tree" {
                (segments[3].to_string(), 4)
            } else {
                (segments[2].to_string(), 3)
            };

            let path = if segments.len() > path_start {
                Some(segments[path_start..].join("/"))
            } else {
                None
            };

            let repo_ref = RepoRef {
                host: "github.com".to_string(),
                owner,
                repo,
                branch,
                path,
            };

            let url_git = format!("git@github.com:{}/{}.git", repo_ref.owner, repo_ref.repo);
            return Some((repo_ref, url_git));
        }
        // Handle git@https://... or git@http://... (strip git@ prefix and parse as HTTPS)
        else if normalized_url.starts_with("git@https://")
            || normalized_url.starts_with("git@http://")
        {
            let stripped_url = &normalized_url[4..]; // Remove "git@" prefix
            if let Some(repo_ref) = self.parse_url(stripped_url) {
                let url_git = format!(
                    "https://github.com/{}/{}.git",
                    repo_ref.owner, repo_ref.repo
                );
                return Some((repo_ref, url_git));
            }
        }

        None
    }
}
