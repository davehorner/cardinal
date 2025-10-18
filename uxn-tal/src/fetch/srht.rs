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

pub struct SourceHut;

impl SourceHut {
    fn raw_url(r: &RepoRef, repo_rel: &str) -> String {
        format!("https://git.sr.ht/{}/{}/blob/{}/{}?raw=1",
            r.owner, r.repo, r.branch, repo_rel)
    }
    // fn try_entry(r: &RepoRef, out: &Path, repo_rel: &str) -> Option<PathBuf> {
    //     let url = Self::raw_url(r, repo_rel);
    //     if let Ok(bytes) = http_get(&url) {
    //         let local = out.join(repo_rel);
    //         if write_bytes(&local, &bytes).is_ok() { return Some(local); }
    //     }
    //     None
    // }
    fn fetch_file(r: &RepoRef, out_root: &Path, repo_rel: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let url = Self::raw_url(r, repo_rel);
        let bytes = http_get(&url)?;
        // Check for HTML/404 response
        let is_html = bytes.starts_with(b"<!doctype html") || bytes.starts_with(b"<html");
        if is_html {
            return Err(format!("SourceHut: Got HTML/404 for {}", url).into());
        }
        let local = out_root.join(repo_rel);
        if let Some(p) = local.parent() { fs::create_dir_all(p)?; }
        write_bytes(&local, &bytes)?;
        Ok(local)
    }
}

impl Provider for SourceHut {
    fn parse_url(&self, url: &str) -> Option<RepoRef> {
        let u = Url::parse(url).ok()?;
        if u.domain()? != "git.sr.ht" { return None; }
        let segs = u.path_segments()?.collect::<Vec<_>>();
        if segs.len() < 2 || !segs[0].starts_with('~') { return None; }

        let owner = segs[0].to_string();
        let repo  = segs[1].to_string();

        if segs.len() == 2 {
            return Some(RepoRef { host: "git.sr.ht".into(), owner, repo, branch: "HEAD".into(), path: None });
        }

        match segs[2] {
            // Accept blob/tree/blame; normalize tree/.../item/... (UI-only)
            "blob" | "tree" | "blame" | "log" if segs.len() >= 4 => {
                let branch = segs[3].to_string();
                // compute start of repo path
                let mut path_start = 4;
                if (segs[2] == "tree" || segs[2]=="log") && segs.get(4) == Some(&"item") {
                    path_start = 5;
                }
                let path = if segs.len() > path_start {
                    Some(segs[path_start..].join("/"))
                } else {
                    None
                };
                Some(RepoRef { host: "git.sr.ht".into(), owner, repo, branch, path })
            }
            _ => {
                let path = if segs.len() > 2 { Some(segs[2..].join("/")) } else { None };
                Some(RepoRef { host: "git.sr.ht".into(), owner, repo, branch: "HEAD".into(), path })
            }
        }
    }


    fn fetch_tal_tree(&self, r: &RepoRef, out_root: &Path) -> Result<FetchResult, Box<dyn std::error::Error>> {
        // Strict: must point to a file
        let entry_rel = match &r.path {
            Some(p) if p.to_ascii_lowercase().ends_with(".tal") || p.to_ascii_lowercase().ends_with(".rom") => p.replace('\\', "/"),
            _ => return Err("sr.ht: URL must point to a .tal or .rom file; not guessing entries".into()),
        };

        // Fetch entry and walk includes
        let entry_local = Self::fetch_file(r, out_root, &entry_rel)?;
        let mut all = vec![entry_local.clone()];
        // Recursively fetch includes for .tal files
        if entry_rel.to_ascii_lowercase().ends_with(".tal") {
            let mut visited: HashSet<String> = [entry_rel.clone()].into_iter().collect();
            let mut q: VecDeque<(String, PathBuf)> = VecDeque::from([(entry_rel.clone(), entry_local.clone())]);
            while let Some((curr_rel, curr_local)) = q.pop_front() {
                let src = fs::read_to_string(&curr_local).unwrap_or_default();
                for inc in parse_includes(&src) {
                    let target = resolve_include(&curr_rel, &inc);
                    if !visited.insert(target.clone()) { continue; }
                    let mut attempts: Vec<(String, Option<std::path::PathBuf>)> = vec![(target.clone(), None)];
                    let mut success: Option<(String, PathBuf)> = None;
                    let mut errors: Vec<(String, String)> = vec![];
                    // Try original
                    match Self::fetch_file(r, out_root, &target) {
                        Ok(loc) => { success = Some((target.clone(), loc)); }
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
                                Ok(loc) => { success = Some((deduped.clone(), loc)); }
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
                                if try_str == target { break; }
                                attempts.push((try_str.clone(), None));
                                match Self::fetch_file(r, out_root, &try_str) {
                                    Ok(loc) => { success = Some((try_str.clone(), loc)); break; }
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
                                let entry_dir_str = entry_dir_path.to_string_lossy().replace('\\', "/");
                                attempts.push((entry_dir_str.clone(), None));
                                match Self::fetch_file(r, out_root, &entry_dir_str) {
                                    Ok(loc) => { success = Some((entry_dir_str.clone(), loc)); }
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
                        attempts.iter().map(|(p, _)| format!("'{}'", p)).collect::<Vec<_>>().join(", "),
                        errors.iter().map(|(p, e)| format!("{}: {}", p, e)).collect::<Vec<_>>().join("; ")
                    );
                    return Err(error_msg.into());
                }
            }
        }
        Ok(FetchResult { entry_local: all[0].clone(), all_files: all })
    
        // FAST PATH: if the URL points to a .tal, fetch it directly
        // if let Some(p) = &r.path {
        //     if p.ends_with(".tal") {
        //         let url = Self::raw_url(r, p);
        //         eprintln!("[srht] entry url: {}", url); // debug
        //         if let Ok(bytes) = super::downloader::http_get(&url) {
        //             let entry_local = out_root.join(p);
        //             super::downloader::write_bytes(&entry_local, &bytes)?;
        //             // then fall through to include BFS using this as the entry
        //             return Ok(FetchResult { entry_local, all_files: vec![] });
        //         }
        //     }
        // }

        // let candidates: Vec<String> = match &r.path {
        //     Some(p) if p.ends_with(".tal") => vec![p.clone()],
        //     Some(p) if !p.is_empty()       => repo_entry_guesses().iter().map(|g| format!("{}/{}", p.trim_end_matches('/'), g)).collect(),
        //     _ => repo_entry_guesses().iter().map(|s| s.to_string()).collect(),
        // };

        // let (entry_rel, entry_local) = candidates
        //     .into_iter()
        //     .find_map(|c| Self::try_entry(r, out_root, &c).map(|l| (c, l)))
        //     .ok_or("Could not locate entry .tal in SourceHut repo")?;

        // let mut visited = HashSet::new();
        // let mut all = vec![entry_local.clone()];
        // visited.insert(entry_rel.clone());
        // let mut q = VecDeque::new();
        // q.push_back((entry_rel, entry_local));

        // let mut steps = 0usize;
        // while let Some((repo_rel, local)) = q.pop_front() {
        //     if steps >= 200 { break; }
        //     steps += 1;

        //     let src = fs::read_to_string(&local).unwrap_or_default();
        //     for inc in parse_includes(&src) {
        //         let target = resolve_include(&repo_rel, &inc);
        //         if !visited.insert(target.clone()) { continue; }
        //         let url = Self::raw_url(r, &target);
        //         match http_get(&url) {
        //             Ok(bytes) => {
        //                 let loc = out_root.join(&target);
        //                 if write_bytes(&loc, &bytes).is_ok() {
        //                     all.push(loc.clone());
        //                     q.push_back((target, loc));
        //                 }
        //             }
        //             Err(_) => eprintln!("warning: include not found in SourceHut repo: {}", target),
        //         }
        //     }
        // }
        // Ok(FetchResult { entry_local: all[0].clone(), all_files: all })
    }
}


// // BFS include walker for sr.ht
// fn bfs_includes_srht(
//     r: &RepoRef,
//     out_root: &Path,
//     entry_rel: String,
//     entry_local: PathBuf,
// ) -> Result<FetchResult, Box<dyn std::error::Error>>
// {
//     let mut visited: HashSet<String> = HashSet::new();
//     let mut all: Vec<PathBuf> = vec![entry_local.clone()];
//     visited.insert(entry_rel.clone());

//     let mut q: VecDeque<(String, PathBuf)> = VecDeque::new();
//     q.push_back((entry_rel.clone(), entry_local));

//     let mut steps = 0usize;
//     while let Some((curr_rel, curr_local)) = q.pop_front() {
//         if steps > 2000 { break; } // simple safety cap
//         steps += 1;

//         let src = fs::read_to_string(&curr_local).unwrap_or_default();
//         for inc in parse_includes(&src) {
//             // Turn "~src/drif.util.tal" into "src/drif.util.tal", "foo.tal" into "dir/foo.tal", etc.
//             let target = resolve_include(&curr_rel, &inc);

//             if !visited.insert(target.clone()) {
//                 continue;
//             }

//             let url = SourceHut::raw_url(r, &target);
//             eprintln!("[srht] include GET {}", url);
//             match http_get(&url) {
//                 Ok(bytes) => {
//                     let loc = out_root.join(&target);
//                     if let Some(parent) = loc.parent() { let _ = fs::create_dir_all(parent); }
//                     if write_bytes(&loc, &bytes).is_ok() {
//                         all.push(loc.clone());
//                         q.push_back((target, loc));
//                     }
//                 }
//                 Err(e) => {
//                     eprintln!("[srht] warn: include missing {} ({})", target, e);
//                 }
//             }
//         }
//     }

//     Ok(FetchResult { entry_local: out_root.join(&visited.iter().next().unwrap()), all_files: all })
// }