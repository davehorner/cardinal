#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "github actions noget disable"]
    fn test_fetch_remote_romtxt_via_uxntal() {
        // This test fetches a remote .rom.txt via uxntal:// and writes it to a temp file
        let url =
            "uxntal://https://git.sr.ht/~rabbits/drifblim/tree/main/item/etc/drifblim.rom.txt";
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        let out_path = tmp.path().join("seed.rom");
        uxn_tal::bkend_drif::write_rom_resolved(url, &out_path)
            .expect("failed to fetch and write rom");
        assert!(out_path.exists(), "ROM file was not written");
        let data = std::fs::read(&out_path).expect("failed to read written rom");
        assert!(!data.is_empty(), "ROM file is empty");
        // Optionally, check a few bytes for expected content (magic, etc.)
        println!("Fetched ROM size: {} bytes", data.len());
    }

    #[test]
    #[ignore = "github actions noget disable"]
    fn test_fetch_and_run_remote_romtxt_for_usage() {
        // Fetch remote .rom.txt via uxntal:// and write to temp file
        let url =
            "uxntal://https://git.sr.ht/~rabbits/drifblim/tree/main/item/etc/drifblim.rom.txt";
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        let out_path = tmp.path().join("seed.rom");
        uxn_tal::bkend_drif::write_rom_resolved(url, &out_path)
            .expect("failed to fetch and write rom");
        assert!(out_path.exists(), "ROM file was not written");
        let data = std::fs::read(&out_path).expect("failed to read written rom");
        assert!(!data.is_empty(), "ROM file is empty");
        println!("Fetched ROM size: {} bytes", data.len());
        println!("Seed ROM path: {}", out_path.display());
        // Run cardinal-cli with no args to check for usage output
        let exe = assert_cmd::cargo::cargo_bin("cardinal-cli");
        let mut cmd = std::process::Command::new(&exe);
        // Only pass the ROM path, no other args, should trigger usage
        cmd.arg(&out_path);
        let output = cmd.output().expect("failed to run cardinal-cli");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("cardinal-cli stdout:\n{}", stdout);
        println!("cardinal-cli stderr:\n{}", stderr);
        // Check for usage/help in output (stdout or stderr)
        let usage_found =
            stdout.to_lowercase().contains("usage") || stderr.to_lowercase().contains("usage");
        assert!(usage_found, "Expected usage/help output from cardinal-cli when run with only ROM.\nstdout:\n{}\nstderr:\n{}", stdout, stderr);
    }

    #[ignore = "this demonstrates an issue with raven and drifblim argument passing leading to early termination with no output i found that I couldnt get drifblim-seed to run with raven. https://github.com/mkeeter/raven/issues/30"]
    #[test]
    fn test_argument_passing() {
        // This test fetches a remote .rom.txt via uxntal:// and writes it to a temp file
        let url =
            "uxntal://https://git.sr.ht/~rabbits/drifblim/tree/main/item/etc/drifblim.rom.txt";
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        let seed_rom = tmp.path().join("drifblim-seed.rom");
        uxn_tal::bkend_drif::write_rom_resolved(url, &seed_rom)
            .expect("failed to fetch and write rom");
        assert!(seed_rom.exists(), "ROM file was not written");
        let data = std::fs::read(&seed_rom).expect("failed to read written rom");
        assert!(!data.is_empty(), "ROM file is empty");
        // Optionally, check a few bytes for expected content (magic, etc.)
        println!("Fetched ROM size: {} bytes", data.len());

        // Print the current working directory for debugging
        let cwd = std::env::current_dir().expect("Failed to get current working directory");
        println!("Current working directory: {}", cwd.display());
        // Remove the .rom file if it exists, so the CLI must create it
        let dst_rom = std::path::Path::new("tests/helloworld.rom");
        if dst_rom.exists() {
            std::fs::remove_file(dst_rom).expect("Failed to remove pre-existing helloworld.rom");
        }
        println!("helloworld.rom exists before run: {}", dst_rom.exists());
        // Run cardinal-cli and check output
        let exe = assert_cmd::cargo::cargo_bin("cardinal-cli");
        let mut cmd = std::process::Command::new(&exe);
        println!("ROM path: {}", seed_rom.display());
        println!("ROM exists before run: {}", seed_rom.exists());
        assert!(
            seed_rom.exists(),
            "drifblim-seed.rom is missing in the test directory: {}",
            seed_rom.display()
        );
        let args = ["helloworld.tal", "helloworld.rom"];
        // Pass ROM as first positional arg, then --, then input/output files
        println!("Running: cardinal-cli {} -- {:?}", seed_rom.display(), args);
        let output = cmd
            .current_dir("tests")
            .arg(seed_rom)
            .arg("--")
            .args(args)
            .output()
            .expect("failed to run cardinal-cli");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("cardinal-cli stdout:\n{}", stdout);
        println!("cardinal-cli stderr:\n{}", stderr);
        // The output ROM should be created
        println!("helloworld.rom exists after run: {}", dst_rom.exists());
        assert!(
            dst_rom.exists(),
            "helloworld.rom was not created by cardinal-cli"
        );
        // The output should not be empty
        let metadata = std::fs::metadata(dst_rom).expect("Failed to stat output ROM");
        assert!(metadata.len() > 0, "Output ROM file is empty");
        // Check that the ROM file exists
        assert!(
            std::path::Path::new("tests/helloworld.rom").exists(),
            "helloworld.rom does not exist in the test directory"
        );

        // Check that stdout is not empty (should match buxn behavior)
        assert!(
            !stdout.trim().is_empty(),
            "stdout is empty, expected output from ROM execution"
        );
        // The output should mention assembling the ROM (check both stdout and stderr, allow for line ending differences)
        let re = regex::Regex::new(r"Assembled helloworld\.rom in \d+ bytes\.").unwrap();
        assert!(
            re.is_match(&stdout) || re.is_match(&stderr),
            "Expected assembly message not found.\nstdout:\n{}\nstderr:\n{}",
            stdout,
            stderr
        );
    }
}
