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

pub struct Codeberg;

impl Codeberg {
    // Gitea/Codeberg raw:
    //   https://codeberg.org/<owner>/<repo>/raw/<branch>/<path>
    fn raw_url(r: &RepoRef, repo_rel: &str) -> String {
        format!(
            "https://codeberg.org/{}/{}/raw/{}/{}",
            r.owner, r.repo, r.branch, repo_rel
        )
    }

    // fn try_entry(r: &RepoRef, out: &Path, repo_rel: &str) -> Option<PathBuf> {
    //     let url = Self::raw_url(r, repo_rel);
    //     if let Ok(bytes) = http_get(&url) {
    //         let local = out.join(repo_rel);
    //         if write_bytes(&local, &bytes).is_ok() {
    //             return Some(local);
    //         }
    //     }
    //     None
    // }

    // fn branch_candidates(branch: &str) -> Vec<String> {
    //     if branch != "HEAD" {
    //         return vec![branch.to_string()];
    //     }
    //     // common defaults
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
            return Err(format!("Codeberg: Got HTML/404 for {}", url).into());
        }
        if let Some(p) = local.parent() {
            fs::create_dir_all(p)?;
        }
        write_bytes(&local, &bytes)?;
        Ok(local)
    }
}

impl Provider for Codeberg {
    fn parse_url(&self, url: &str) -> Option<RepoRef> {
        let u = Url::parse(url).ok()?;
        if u.domain()? != "codeberg.org" {
            return None;
        }
        let segs = u.path_segments()?.collect::<Vec<_>>();
        if segs.len() < 2 {
            return None;
        }
        let owner = segs[0].to_string();
        let repo = segs[1].trim_end_matches(".git").to_string();

        if segs.len() == 2 {
            // repo root
            return Some(RepoRef {
                host: "codeberg.org".into(),
                owner,
                repo,
                branch: "HEAD".into(),
                path: None,
            });
        }

        // Accept Gitea view routes: src/raw/tree/blob/blame and also tag/commit forms
        match segs[2] {
            // Standard: /<view>/<branch>/<path...>
            "src" | "raw" | "tree" | "blob" | "blame" if segs.len() >= 4 => {
                // handle multi-part ref: tag/<tag> or commit/<sha>
                if segs[3] == "tag" && segs.len() >= 6 {
                    let branch = format!("tag/{}", segs[4]);
                    let path = if segs.len() > 5 {
                        Some(segs[5..].join("/"))
                    } else {
                        None
                    };
                    return Some(RepoRef {
                        host: "codeberg.org".into(),
                        owner,
                        repo,
                        branch,
                        path,
                    });
                }
                if segs[3] == "commit" && segs.len() >= 6 {
                    let branch = format!("commit/{}", segs[4]);
                    let path = if segs.len() > 5 {
                        Some(segs[5..].join("/"))
                    } else {
                        None
                    };
                    return Some(RepoRef {
                        host: "codeberg.org".into(),
                        owner,
                        repo,
                        branch,
                        path,
                    });
                }
                // Normal branch name
                let branch = segs[3].to_string();
                let path = if segs.len() > 4 {
                    Some(segs[4..].join("/"))
                } else {
                    None
                };
                Some(RepoRef {
                    host: "codeberg.org".into(),
                    owner,
                    repo,
                    branch,
                    path,
                })
            }
            _ => {
                // Fallback: treat the rest as a path, unknown ref
                let path = if segs.len() > 2 {
                    Some(segs[2..].join("/"))
                } else {
                    None
                };
                Some(RepoRef {
                    host: "codeberg.org".into(),
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
                "codeberg: URL must point to a .tal, .rom, .rom.txt, .orca, or .bas file; not guessing entries"
                    .into(),
            ),
        };

        // Check cache for entry file first
        let entry_local = out_root.join(&entry_rel);
        if entry_local.exists() {
            eprintln!("[Codeberg] Using cached file: {}", entry_local.display());
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
            while let Some((curr_rel, curr_local)) = q.pop_front() {
                let src = fs::read_to_string(&curr_local).unwrap_or_default();
                for inc in parse_includes(&src) {
                    let target = resolve_include(&curr_rel, &inc);
                    if !visited.insert(target.clone()) {
                        continue;
                    }
                    let mut attempts: Vec<(String, Option<std::path::PathBuf>)> =
                        vec![(target.clone(), None)];
                    let mut success: Option<(String, PathBuf)> = None;
                    let mut errors: Vec<(String, String)> = vec![];
                    // Try original
                    match Self::fetch_file(r, out_root, &target) {
                        Ok(loc) => {
                            success = Some((target.clone(), loc));
                        }
                        Err(e) => {
                            let local = out_root.join(&target);
                            let _ = std::fs::remove_file(&local);
                            errors.push((target.clone(), format!("{}", e)));
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
                                    let local = out_root.join(&deduped);
                                    let _ = std::fs::remove_file(&local);
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
                                        let local = out_root.join(&try_str);
                                        let _ = std::fs::remove_file(&local);
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
                            if let Some(entry_dir) = Path::new(&entry_rel).parent() {
                                let entry_dir_path = entry_dir.join(fname);
                                let entry_dir_str =
                                    entry_dir_path.to_string_lossy().replace('\\', "/");
                                attempts.push((entry_dir_str.clone(), None));
                                match Self::fetch_file(r, out_root, &entry_dir_str) {
                                    Ok(loc) => {
                                        success = Some((entry_dir_str.clone(), loc));
                                    }
                                    Err(e) => {
                                        let local = out_root.join(&entry_dir_str);
                                        let _ = std::fs::remove_file(&local);
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
        //     let r = RepoRef {
        //         branch: b.clone(),
        //         ..r_in.clone()
        //     };

        //     // figure entry candidates
        //     let candidates: Vec<String> = match &r.path {
        //         Some(p) if p.ends_with(".tal") => vec![p.clone()],
        //         Some(p) if !p.is_empty() => repo_entry_guesses()
        //             .iter()
        //             .map(|g| format!("{}/{}", p.trim_end_matches('/'), g))
        //             .collect(),
        //         _ => repo_entry_guesses().iter().map(|s| s.to_string()).collect(),
        //     };

        //     if let Some((entry_rel, entry_local)) = candidates
        //         .into_iter()
        //         .find_map(|c| Self::try_entry(&r, out_root, &c).map(|l| (c, l)))
        //     {
        //         // BFS includes
        //         let mut visited = HashSet::new();
        //         let mut all = vec![entry_local.clone()];
        //         visited.insert(entry_rel.clone());

        //         let mut q = VecDeque::new();
        //         q.push_back((entry_rel, entry_local));

        //         let mut steps = 0usize;
        //         while let Some((repo_rel, local)) = q.pop_front() {
        //             if steps >= 200 {
        //                 break;
        //             }
        //             steps += 1;

        //             let src = fs::read_to_string(&local).unwrap_or_default();
        //             for inc in parse_includes(&src) {
        //                 let target = resolve_include(&repo_rel, &inc);
        //                 if !visited.insert(target.clone()) {
        //                     continue;
        //                 }
        //                 let url = Self::raw_url(&r, &target);
        //                 match http_get(&url) {
        //                     Ok(bytes) => {
        //                         let loc = out_root.join(&target);
        //                         if write_bytes(&loc, &bytes).is_ok() {
        //                             all.push(loc.clone());
        //                             q.push_back((target, loc));
        //                         }
        //                     }
        //                     Err(_) => eprintln!(
        //                         "warning: include not found in Codeberg repo: {} (branch {})",
        //                         target, r.branch
        //                     ),
        //                 }
        //             }
        //         }

        //         return Ok(FetchResult {
        //             entry_local: all[0].clone(),
        //             all_files: all,
        //         });
        //     }
        // }

        // Err("Could not find an entry .tal in Codeberg repo across tried branches".into())
    }

    fn parse_git_url(&self, url: &str) -> Option<(RepoRef, String)> {
        // Handle git@codeberg.org:owner/repo/branch/path/file.tal (SSH format)
        if let Some(path_part) = url.strip_prefix("git@codeberg.org:") {
            // Remove "git@codeberg.org:"
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
                host: "codeberg.org".to_string(),
                owner,
                repo,
                branch,
                path,
            };

            let url_git = format!("git@codeberg.org:{}/{}", repo_ref.owner, repo_ref.repo);
            return Some((repo_ref, url_git));
        }
        // Handle git@https://... or git@http://... (strip git@ prefix and parse as HTTPS)
        else if url.starts_with("git@https://") || url.starts_with("git@http://") {
            let stripped_url = &url[4..]; // Remove "git@" prefix
            if let Some(repo_ref) = self.parse_url(stripped_url) {
                let url_git = format!("https://codeberg.org/{}/{}", repo_ref.owner, repo_ref.repo);
                return Some((repo_ref, url_git));
            }
        }

        None
    }
}
