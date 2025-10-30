use std::path::PathBuf;
use uxn_tal::fetch::fetch_repo_tree;

#[test]
fn fetch_patchstorage_orca() {
    let url = "https://patchstorage.com/phase-sequencer/";
    let out_dir = PathBuf::from("test-out-patchstorage");
    let result = fetch_repo_tree(url, &out_dir);
    assert!(result.is_ok(), "fetch_repo_tree failed: {:?}", result);
    let fetch_result = result.unwrap();
    assert!(!fetch_result.all_files.is_empty(), "No files fetched");
    let found_orca = fetch_result
        .all_files
        .iter()
        .any(|f| f.extension().map(|e| e == "orca").unwrap_or(false));
    assert!(
        found_orca,
        "No .orca file fetched: files = {:?}",
        fetch_result.all_files
    );
}

#[test]
fn fetch_uxntal_url_caches_file() {
    use uxn_tal::fetch::resolver::resolve_entry_from_url;
    // Example uxntal:// URL pointing to a real .tal or .orca file
    let url = "uxntal://https://patchstorage.com/phase-sequencer/";
    let result = resolve_entry_from_url(url);
    assert!(
        result.is_ok(),
        "resolve_entry_from_url failed: {:?}",
        result
    );
    let (entry_local, rom_dir) = result.unwrap();
    // The file should exist in the cache directory
    assert!(
        entry_local.exists(),
        "Cached file does not exist: {:?}",
        entry_local
    );
    // Optionally, check that the file is an .orca or .tal file
    let ext = entry_local
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    assert!(
        ext == "orca" || ext == "tal" || ext == "rom",
        "Unexpected file extension: {}",
        ext
    );
    // Optionally, print the cache directory for debugging
    println!("Cached file: {:?} in dir {:?}", entry_local, rom_dir);
}

#[test]
fn fetch_uxntal_url_scales_caches_file() {
    use uxn_tal::fetch::resolver::resolve_entry_from_url;
    // Example uxntal:// URL pointing to a real .orca file on PatchStorage
    let url = "uxntal://https://patchstorage.com/scales/";
    let result = resolve_entry_from_url(url);
    assert!(
        result.is_ok(),
        "resolve_entry_from_url failed: {:?}",
        result
    );
    let (entry_local, rom_dir) = result.unwrap();
    // The file should exist in the cache directory
    assert!(
        entry_local.exists(),
        "Cached file does not exist: {:?}",
        entry_local
    );
    // Optionally, check that the file is an .orca or .tal file
    let ext = entry_local
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    assert!(
        ext == "orca" || ext == "tal" || ext == "rom",
        "Unexpected file extension: {}",
        ext
    );
    // Optionally, print the cache directory for debugging
    println!("Cached file: {:?} in dir {:?}", entry_local, rom_dir);
}
