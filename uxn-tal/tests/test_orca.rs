use uxn_tal::fetch::downloader::resolve_and_fetch_entry;
#[test]
#[ignore = "requires network access to git.sr.ht, not available on GitHub CI"]
fn test_srht_orca_download_and_cache() {
    let url = "uxntal://https://git.sr.ht/~rabbits/orca-examples/tree/master/item/basics/a.orca";
    let result = resolve_and_fetch_entry(url);
    assert!(
        result.is_ok(),
        "resolve_and_fetch_entry should succeed: {:?}",
        result
    );
    let (entry_path, cache_dir) = result.unwrap();
    // The entry_path should exist and be a file
    assert!(
        entry_path.exists(),
        "Downloaded .orca file should exist: {:?}",
        entry_path
    );
    assert!(
        entry_path.is_file(),
        "Downloaded .orca entry should be a file: {:?}",
        entry_path
    );
    // The cache_dir should exist and be a directory
    assert!(
        cache_dir.exists(),
        "Cache directory should exist: {:?}",
        cache_dir
    );
    assert!(
        cache_dir.is_dir(),
        "Cache directory should be a directory: {:?}",
        cache_dir
    );
    // The entry_path should end with .orca
    assert!(
        entry_path.extension().map(|e| e == "orca").unwrap_or(false),
        "Entry path should have .orca extension: {:?}",
        entry_path
    );
}

#[test]
#[ignore = "requires network access to GitHub, not available on GitHub CI"]
fn test_github_orca_download_and_cache() {
    let url = "uxntal://https://github.com/hundredrabbits/Orca-c/blob/master/examples/misc/chromatic.orca";
    let result = resolve_and_fetch_entry(url);
    assert!(
        result.is_ok(),
        "resolve_and_fetch_entry should succeed: {:?}",
        result
    );
    let (entry_path, cache_dir) = result.unwrap();
    // The entry_path should exist and be a file
    assert!(
        entry_path.exists(),
        "Downloaded .orca file should exist: {:?}",
        entry_path
    );
    assert!(
        entry_path.is_file(),
        "Downloaded .orca entry should be a file: {:?}",
        entry_path
    );
    // The cache_dir should exist and be a directory
    assert!(
        cache_dir.exists(),
        "Cache directory should exist: {:?}",
        cache_dir
    );
    assert!(
        cache_dir.is_dir(),
        "Cache directory should be a directory: {:?}",
        cache_dir
    );
    // The entry_path should end with .orca
    assert!(
        entry_path.extension().map(|e| e == "orca").unwrap_or(false),
        "Entry path should have .orca extension: {:?}",
        entry_path
    );
}
