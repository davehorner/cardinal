//! Test for argument passing in buxn-cli

#[cfg(test)]
mod buxn_tests {
    use std::process::Command;

    #[ignore = "This test requires buxn-cli to be installed and available in PATH."]
    #[test]
    fn test_argument_passing_buxn() {
        // This test fetches a remote .rom.txt via uxntal:// and writes it to a temp file
        let url =
            "uxntal://https://git.sr.ht/~rabbits/drifblim/tree/main/item/etc/drifblim.rom.txt";
        let tmp: tempfile::TempDir = tempfile::tempdir().expect("failed to create tempdir");
        let rom = tmp.path().join("seed.rom");
        uxn_tal::bkend_drif::write_rom_resolved(url, &rom).expect("failed to fetch and write rom");
        assert!(rom.exists(), "ROM file was not written");
        let data = std::fs::read(&rom).expect("failed to read written rom");
        assert!(!data.is_empty(), "ROM file is empty");
        // Optionally, check a few bytes for expected content (magic, etc.)
        println!("Fetched ROM size: {} bytes", data.len());

        // Print the current working directory for debugging
        let cwd = std::env::current_dir().expect("Failed to get current working directory");
        println!("Current working directory: {}", cwd.display());
        // Copy the .tal file to the current directory (runner's dir) before running buxn-cli
        let src_tal = std::path::Path::new("tests/helloworld.tal");
        let dst_tal = std::path::Path::new("helloworld.tal");
        if dst_tal.exists() {
            std::fs::remove_file(dst_tal).expect("Failed to remove pre-existing helloworld.tal");
        }
        std::fs::copy(src_tal, dst_tal).expect("Failed to copy helloworld.tal to runner dir");
        // Remove the .rom file if it exists
        let rom_path = std::path::Path::new("helloworld.rom");
        if rom_path.exists() {
            std::fs::remove_file(rom_path).expect("Failed to remove pre-existing helloworld.rom");
        }
        // Run buxn-cli and check output
        let exe = "buxn-cli";
        let args = ["helloworld.tal", "helloworld.rom"];
        let output = Command::new(exe)
            .arg(&rom)
            .args(args)
            .output()
            .expect("failed to run buxn-cli");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Print both outputs for debugging
        println!("buxn-cli stdout:\n{}", stdout);
        println!("buxn-cli stderr:\n{}", stderr);
        // The output should mention assembling the ROM (check both stdout and stderr, allow for line ending differences)
        let re = regex::Regex::new(r"Assembled helloworld\.rom in \d+ bytes\.").unwrap();
        assert!(
            re.is_match(&stdout) || re.is_match(&stderr),
            "Expected assembly message not found.\nstdout:\n{}\nstderr:\n{}",
            stdout,
            stderr
        );
        // The file should now exist
        assert!(
            rom_path.exists(),
            "helloworld.rom was not created by buxn-cli"
        );
    }
}
