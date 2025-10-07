/// Returns the path to drifloon.rom in the user's home directory, or just "drifloon.rom" if not found.
pub fn uxndis_repo_get_drifloon() -> PathBuf {
    let home_dir = dirs::home_dir();
    if let Some(home) = home_dir {
        let drifloon_path = home.join(".uxntal").join(".uxndis").join("src").join("drifloon.rom");
        if drifloon_path.exists() {
            return drifloon_path;
        }
    }
    PathBuf::from("drifloon.rom")
}

/// Returns the path to uxndis.rom in the user's home directory, or just "uxndis.rom" if not found.
pub fn uxndis_repo_get_uxndis() -> PathBuf {
    let home_dir = dirs::home_dir();
    if let Some(home) = home_dir {
        let uxndis_path = home.join(".uxntal").join(".uxndis").join("src").join("uxndis.rom");
        if uxndis_path.exists() {
            return uxndis_path;
        }
    }
    PathBuf::from("uxndis.rom")
}


/// Returns the path to uxndis-seed.rom in the user's home directory, or just "uxndis-seed.rom" if not found.
pub fn uxndis_repo_get_uxndis_seed() -> PathBuf {
    let home_dir = dirs::home_dir();
    if let Some(home) = home_dir {
        let uxndis_seed_path = home.join(".uxntal").join(".uxndis").join("bin").join("uxndis-seed.rom");
        if uxndis_seed_path.exists() {
            return uxndis_seed_path;
        }
    }
    PathBuf::from("uxndis-seed.rom")
}
use std::{fs, path::{Path, PathBuf}, process::Command};

use crate::{hexrev::HexRev, Assembler, AssemblerError};


fn simple_err(path: &std::path::Path, msg: &str) -> AssemblerError {
    AssemblerError::SyntaxError {
        path: path.display().to_string(),
        line: 0,
        position: 0,
        message: msg.to_string(),
        source_line: String::new(),
    }
}

pub fn ensure_uxndis_repo() -> Result<Option<PathBuf>, AssemblerError> {
    struct DirGuard {
        original: PathBuf,
    }

    impl DirGuard {
        fn new(target: &Path) -> Option<Self> {
            let original = std::env::current_dir().ok()?;
            if std::env::set_current_dir(target).is_ok() {
                Some(DirGuard { original })
            } else {
                None
            }
        }
    }

    impl Drop for DirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original);
        }
    }
    let home_dir = dirs::home_dir().ok_or_else(|| simple_err(Path::new("~/.uxntal/.uxndis"), "failed to get home directory"))?;
    let uxntal_path = home_dir.join(".uxntal");
    let uxndis_path = uxntal_path.join(".uxndis");
    if !uxndis_path.exists() {
        let status = Command::new("git")
            .arg("clone")
            .arg("https://git.sr.ht/~rabbits/uxndis")
            .arg(&uxndis_path)
            .status()
            .map_err(|e| simple_err(&uxndis_path, &format!("failed to execute git clone: {e}")))?;
        if !status.success() {
            eprintln!("Failed to clone uxndis repository");
            return Err(simple_err(&uxndis_path, "failed to clone repository"));
        }
    } else {
        // If already exists, do a git pull
        let status = Command::new("git")
            .arg("-C")
            .arg(&uxndis_path)
            .arg("pull")
            .status()
            .ok();
        if let Some(status) = status {
            if !status.success() {
            eprintln!("Failed to pull uxndis repository");
            }
        } else {
            eprintln!("Failed to execute git pull for uxndis repository");
        }
    }
    if !uxndis_path.exists() {
        eprintln!("uxndis repository not found after clone/pull");
        return Err(simple_err(&uxndis_path, "uxndis repository not found after clone/pull"));
    }
    let _guard = DirGuard::new(&uxndis_path);

    let uxndis_rom = uxndis_path.join("src").join("uxndis.rom");
    if !uxndis_rom.exists() {
        let mut asm = Assembler::new();

        let uxndis_tal = uxndis_path.join("src").join("uxndis.tal");
        if !uxndis_tal.exists() {
            eprintln!("uxndis.tal not found in uxndis repository");
            return Err(simple_err(&uxndis_tal, "uxndis.tal not found"));
        }
        eprintln!("Assembling uxndis.tal to uxndis.rom...");
        let uxndis_tal_contents = match fs::read_to_string(&uxndis_tal) {
            Ok(contents) => contents,
            Err(e) => {
            eprintln!("Failed to read uxndis.tal: {:?}", e);
            return Err(simple_err(&uxndis_tal, "failed to read uxndis.tal"));
            }
        };
        let ret = asm.assemble(&uxndis_tal_contents, Some(uxndis_rom.display().to_string()));
        match ret {
            Ok(rom) => {
                fs::write(&uxndis_rom, &rom)
                    .map_err(|e| simple_err(&uxndis_rom, &format!("failed to write rom: {e}"))).ok();
                if uxndis_rom.exists() {
                    eprintln!("Successfully assembled uxndis.tal {}", uxndis_rom.display());
                } else {
                    eprintln!("Assembly succeeded but uxndis.rom not found at {}", uxndis_rom.display());
                    return Err(simple_err(&uxndis_rom, "uxndis.rom not found"));
                }
            }
            Err(e) => {
                eprintln!("Failed to assemble uxndis.tal: {:?}", e);
                return Err(simple_err(&uxndis_tal, "failed to assemble uxndis.tal"));
            }
        }
    }
    
    Ok(Some(uxndis_path))
}