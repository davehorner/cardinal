use super::{
    downloader::{http_get, write_bytes},
    includes::{parse_includes, resolve_include},
    provider::{FetchResult, Provider, RepoRef},
};
use std::{
    collections::{HashSet, VecDeque},
    fs, path::{Path, PathBuf},
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

    fn fetch_file(r: &RepoRef, out_root: &Path, repo_rel: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let url = Self::raw_url(r, repo_rel);
        let bytes = http_get(&url)?;
        let local = out_root.join(repo_rel);
        if let Some(p) = local.parent() { fs::create_dir_all(p)?; }
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
        let repo  = segs[1].trim_end_matches(".git").to_string();

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
                    let path   = if segs.len() > 5 { Some(segs[5..].join("/")) } else { None };
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
                    let path   = if segs.len() > 5 { Some(segs[5..].join("/")) } else { None };
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
                let path = if segs.len() > 4 { Some(segs[4..].join("/")) } else { None };
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
                let path = if segs.len() > 2 { Some(segs[2..].join("/")) } else { None };
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
            Some(p) if p.to_ascii_lowercase().ends_with(".tal") => p.replace('\\', "/"),
            _ => return Err("codeberg: URL must point to a .tal file; not guessing entries".into()),
        };

        let entry_local = Self::fetch_file(r, out_root, &entry_rel)?;
        let mut visited: HashSet<String> = [entry_rel.clone()].into_iter().collect();
        let mut all = vec![entry_local.clone()];
        let mut q: VecDeque<(String, PathBuf)> = VecDeque::from([(entry_rel.clone(), entry_local)]);

        while let Some((curr_rel, curr_local)) = q.pop_front() {
            let src = fs::read_to_string(&curr_local).unwrap_or_default();
            for inc in parse_includes(&src) {
                let target = resolve_include(&curr_rel, &inc);
                if !visited.insert(target.clone()) { continue; }
                match Self::fetch_file(r, out_root, &target) {
                    Ok(loc) => { all.push(loc.clone()); q.push_back((target, loc)); }
                    Err(e)  => eprintln!("[codeberg] warn: missing include {} ({})", target, e),
                }
            }
        }

        Ok(FetchResult { entry_local: all[0].clone(), all_files: all })

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
}
