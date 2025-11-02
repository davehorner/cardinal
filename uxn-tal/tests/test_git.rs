//! Tests for git@ URL support for each provider (GitHub, SourceHut, Codeberg)

use std::sync::Mutex;

static TEST_LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
mod tests {

    #[test]
    #[ignore = "requires git clone and network access to GitHub, not available on GitHub CI"]
    fn test_git_clone_github() {
        let _lock = super::TEST_LOCK.lock().unwrap();
        // Test git@github.com:davehorner/uxn-games (should work on Windows)
        let url = "uxntal://git@github.com:davehorner/uxn-games/tree/main/flap/flap.tal";
        let parsed = uxn_tal::parse_uxntal_url(url);
        // For git@ URLs, url_git should be Some(...)
        assert_eq!(
            parsed.repo_ref.as_ref().map(|r| r.url_git.clone()),
            Some("git@github.com:davehorner/uxn-games".to_string())
        );
        assert_eq!(parsed.url_raw, url);
        let cache_dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from(".cache"));
        clean_cache(&cache_dir);
        let result = resolve_and_fetch_entry(url);
        assert!(
            result.is_ok(),
            "GitHub git@ clone failed: {:?}",
            result.err()
        );
        let (entry, cache) = result.unwrap();
        assert!(entry.exists(), "Entry file not found after git clone");
        assert!(cache.exists(), "Cache dir not found after git clone");
    }

    #[test]
    #[ignore = "requires network access to GitHub, not available on GitHub CI"]
    fn test_https_github() {
        // Test https://github.com/davehorner/uxn-games (should work on Windows)
        let url = "https://github.com/davehorner/uxn-games/blob/main/flap/flap.tal";
        let parsed = uxn_tal::parse_uxntal_url(url);
        // For https URLs, url_git should be None or empty string
        assert!(parsed
            .repo_ref
            .as_ref()
            .map(|r| r.url_git.clone())
            .unwrap_or_default()
            .is_empty());
        let cache_dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from(".cache"));
        clean_cache(&cache_dir);
        let result = resolve_and_fetch_entry(url);
        assert!(
            result.is_ok(),
            "GitHub https fetch failed: {:?}",
            result.err()
        );
        let (entry, cache) = result.unwrap();
        assert!(entry.exists(), "Entry file not found after https fetch");
        assert!(cache.exists(), "Cache dir not found after https fetch");
    }
    #[test]
    #[ignore = "requires git clone and network access to git.sr.ht, not available on GitHub CI"]
    fn test_parser_git_at_srht() {
        let url = "uxntal://git@git.sr.ht:~rabbits/noodle/main/src/noodle.tal";
        let parsed = uxn_tal::parse_uxntal_url(url);
        // For git@ SSH URLs, url_git should remain in SSH format
        assert_eq!(
            parsed.repo_ref.as_ref().map(|r| r.url_git.clone()),
            Some("git@git.sr.ht:~rabbits/noodle".to_string())
        );
        // The normalized url should be the git@... form (no uxntal:// prefix)
        assert_eq!(
            parsed.url,
            "git@git.sr.ht:~rabbits/noodle/main/src/noodle.tal"
        );
    }

    #[test]
    #[ignore = "requires git clone and network access to git.sr.ht, not available on GitHub CI"]
    fn test_parser_git_at_https_srht() {
        let url = "uxntal://git@https://git.sr.ht/~rabbits/noodle/main/src/noodle.tal";
        let parsed = uxn_tal::parse_uxntal_url(url);
        // For git@https URLs, url_git should be Some(...)
        assert_eq!(
            parsed.repo_ref.as_ref().map(|r| r.url_git.clone()),
            Some("https://git.sr.ht/~rabbits/noodle".to_string())
        );
        // The normalized url should be the git@https form (no uxntal:// prefix)
        assert_eq!(
            parsed.url,
            "git@https://git.sr.ht/~rabbits/noodle/main/src/noodle.tal"
        );
    }
    use std::fs;
    use std::path::PathBuf;
    use uxn_tal::fetch::downloader::resolve_and_fetch_entry;

    // Helper: remove cache dir if exists
    fn clean_cache(path: &PathBuf) {
        if path.exists() {
            let _ = fs::remove_dir_all(path);
        }
    }

    #[test]
    #[ignore = "requires git clone and network access to git.sr.ht, not available on GitHub CI"]
    fn test_git_clone_srht() {
        let _lock = super::TEST_LOCK.lock().unwrap();
        // Use the canonical SourceHut test URL for all tests
        let url = "uxntal://git@git.sr.ht:~rabbits/noodle/main/src/noodle.tal";
        let parsed = uxn_tal::parse_uxntal_url(url);
        // For git@ URLs, url_git should be Some(...)
        assert_eq!(
            parsed.repo_ref.as_ref().map(|r| r.url_git.clone()),
            Some("git@git.sr.ht:~rabbits/noodle".to_string())
        );
        assert_eq!(parsed.url_raw, url);
        // Clean the deterministic cache dir for this URL
        use uxn_tal::paths::uxntal_roms_get_path;
        use uxn_tal_common::cache::hash_url;
        let norm_url = parsed.url.as_str();
        let cache_base = uxntal_roms_get_path().unwrap_or_else(|| PathBuf::from(".cache"));
        let cache_dir = cache_base.join(format!("{}", hash_url(norm_url)));
        println!(
            "[TEST] Cleaning cache dir: {} (for normalized url: {})",
            cache_dir.display(),
            norm_url
        );
        clean_cache(&cache_dir);
        let result = resolve_and_fetch_entry(url);
        assert!(
            result.is_ok(),
            "SourceHut git clone failed: {:?}",
            result.err()
        );
        let (entry, cache) = result.unwrap();
        assert!(entry.exists(), "Entry file not found after git clone");
        assert!(cache.exists(), "Cache dir not found after git clone");
        // Print recursive contents of cache dir for debugging (before any assembler/parse step)
        fn print_dir_recursive(path: &std::path::Path, prefix: &str) {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    println!(
                        "[TEST] {}{}",
                        prefix,
                        p.file_name().unwrap().to_string_lossy()
                    );
                    if p.is_dir() {
                        print_dir_recursive(&p, &format!("{}  ", prefix));
                    }
                }
            }
        }
        use std::io::Write;
        println!(
            "[TEST] Recursive cache dir listing after clone (before assembler): {}",
            cache.display()
        );
        print_dir_recursive(&cache, "");
        std::io::stdout().flush().unwrap();
    }

    #[test]
    #[ignore = "requires git clone and network access to git.sr.ht, not available on GitHub CI"]
    fn test_git_https_srht() {
        let _lock = super::TEST_LOCK.lock().unwrap();
        // Use the canonical SourceHut test URL for all tests
        let url = "uxntal://git@git.sr.ht:~rabbits/noodle/main/src/noodle.tal";
        let parsed = uxn_tal::parse_uxntal_url(url);
        // For git@ URLs, url_git should be Some(...)
        assert_eq!(
            parsed.repo_ref.as_ref().map(|r| r.url_git.clone()),
            Some("git@git.sr.ht:~rabbits/noodle".to_string())
        );
        // Clean the deterministic cache dir for this URL
        use uxn_tal::paths::uxntal_roms_get_path;
        use uxn_tal_common::cache::hash_url;
        let norm_url = parsed.url.as_str();
        let cache_base = uxntal_roms_get_path().unwrap_or_else(|| PathBuf::from(".cache"));
        let cache_dir = cache_base.join(format!("{}", hash_url(norm_url)));
        println!(
            "[TEST] Cleaning cache dir: {} (for normalized url: {})",
            cache_dir.display(),
            norm_url
        );
        clean_cache(&cache_dir);
        let result = resolve_and_fetch_entry(url);
        assert!(
            result.is_ok(),
            "SourceHut git clone failed: {:?}",
            result.err()
        );
        let (entry, cache) = result.unwrap();
        assert!(entry.exists(), "Entry file not found after git clone");
        assert!(cache.exists(), "Cache dir not found after git clone");
    }

    #[test]
    #[ignore = "requires git clone and network access to GitHub, not available on GitHub CI"]
    fn test_protocol_parser_standard_api() {
        // Test that the standard ProtocolParser::parse now includes git support automatically
        let raw_url = "uxntal://git@github.com:davehorner/uxn-games/tree/main/flap/flap.tal";

        let result = uxn_tal::ProtocolParser::parse(raw_url);

        // Should detect git repository automatically
        assert!(
            result.repo_ref.is_some(),
            "Git repository should be detected"
        );
        let repo_ref = result.repo_ref.unwrap();
        assert_eq!(repo_ref.provider, "github");
        assert_eq!(repo_ref.owner, "davehorner");
        assert_eq!(repo_ref.repo, "uxn-games");
        assert_eq!(repo_ref.branch, "main");
        assert_eq!(repo_ref.path, "flap/flap.tal");
        assert_eq!(repo_ref.url_git, "git@github.com:davehorner/uxn-games");
    }

    #[test]
    #[ignore = "requires git clone and network access to GitHub, not available on GitHub CI"]
    fn test_api_equivalence() {
        // Test that both APIs (ProtocolParser::parse and parse_uxntal_url) produce identical results
        let raw_url = "uxntal://git@github.com:davehorner/uxn-games/tree/main/flap/flap.tal";

        let result1 = uxn_tal::ProtocolParser::parse(raw_url);
        let result2 = uxn_tal::parse_uxntal_url(raw_url);

        // Both should have the same repo_ref
        assert_eq!(result1.repo_ref.is_some(), result2.repo_ref.is_some());
        if let (Some(ref1), Some(ref2)) = (&result1.repo_ref, &result2.repo_ref) {
            assert_eq!(ref1.provider, ref2.provider);
            assert_eq!(ref1.owner, ref2.owner);
            assert_eq!(ref1.repo, ref2.repo);
            assert_eq!(ref1.branch, ref2.branch);
            assert_eq!(ref1.path, ref2.path);
            assert_eq!(ref1.url_git, ref2.url_git);
        }

        // Other fields should also match
        assert_eq!(result1.url_raw, result2.url_raw);
        assert_eq!(result1.url, result2.url);
        assert_eq!(result1.protocol, result2.protocol);
    }

    #[test]
    #[ignore = "requires git clone and network access to multiple git providers, not available on GitHub CI"]
    fn test_comprehensive_url_parsing() {
        // Test various URL formats to ensure they all parse correctly
        let test_cases = vec![
            ("SourceHut SSH", "uxntal://git@git.sr.ht:~rabbits/noodle/main/src/noodle.tal", "sourcehut", "~rabbits", "noodle"),
            ("SourceHut HTTPS", "uxntal://git@https://git.sr.ht/~rabbits/noodle/main/src/noodle.tal", "sourcehut", "~rabbits", "noodle"),
            ("GitHub SSH", "uxntal://git@github.com:davehorner/uxn-games/tree/main/flap/flap.tal", "github", "davehorner", "uxn-games"),
            ("GitHub HTTPS", "uxntal://git@https://github.com/davehorner/uxn-games/tree/main/flap/flap.tal", "github", "davehorner", "uxn-games"),
            ("GitHub .git format", "uxntal://git@github.com:davehorner/uxn-cats.git/catclock.tal", "github", "davehorner", "uxn-cats"),
            ("Codeberg HTTPS", "uxntal://git@https://codeberg.org/yorshex/minesweeper-uxn/tree/main/src/minesweeper.tal", "codeberg", "yorshex", "minesweeper-uxn"),
        ];

        for (description, url, expected_provider, expected_owner, expected_repo) in test_cases {
            println!("Testing: {}", description);
            let parsed = uxn_tal::ProtocolParser::parse(url);

            assert!(
                parsed.repo_ref.is_some(),
                "Git repository should be detected for {}",
                description
            );
            let repo_ref = parsed.repo_ref.unwrap();
            assert_eq!(
                repo_ref.provider, expected_provider,
                "Provider mismatch for {}",
                description
            );
            assert_eq!(
                repo_ref.owner, expected_owner,
                "Owner mismatch for {}",
                description
            );
            assert_eq!(
                repo_ref.repo, expected_repo,
                "Repo mismatch for {}",
                description
            );
        }
    }

    #[test]
    #[ignore = "requires git clone and network access to multiple git providers, not available on GitHub CI"]
    fn test_git_integration_comprehensive() {
        let _lock = super::TEST_LOCK.lock().unwrap();
        // Test comprehensive git integration with actual repository operations
        // This combines URL parsing with actual git functionality
        let test_urls = vec![
            "uxntal://git@github.com:davehorner/uxn-games/tree/main/flap/flap.tal",
            "uxntal://git@github.com:davehorner/uxn-cats.git/catclock.tal",
        ];

        for url in test_urls {
            println!("Testing comprehensive git integration for: {}", url);

            // 1. Test parsing
            let parsed = uxn_tal::ProtocolParser::parse(url);
            assert!(parsed.repo_ref.is_some(), "Should parse git URL: {}", url);

            // 2. Test resolution and fetching (this actually clones the repo)
            let cache_dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from(".cache"));
            clean_cache(&cache_dir);

            let result = resolve_and_fetch_entry(url);
            assert!(
                result.is_ok(),
                "Git resolution should succeed for {}: {:?}",
                url,
                result.err()
            );

            let (entry_path, cache_path) = result.unwrap();
            assert!(
                entry_path.exists(),
                "Entry file should exist after git clone: {}",
                entry_path.display()
            );
            assert!(
                cache_path.exists(),
                "Cache dir should exist after git clone: {}",
                cache_path.display()
            );

            // 3. Test that the file has content
            if entry_path.extension().and_then(|s| s.to_str()) == Some("tal") {
                let content =
                    std::fs::read_to_string(&entry_path).expect("Should be able to read .tal file");
                assert!(!content.trim().is_empty(), "TAL file should have content");
            }
        }
    }
}
