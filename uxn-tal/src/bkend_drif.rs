/// Returns the path to drifloon.rom in the user's home directory, or just "drifloon.rom" if not found.
pub fn drifblim_repo_get_drifloon() -> PathBuf {
    let home_dir = dirs::home_dir();
    if let Some(home) = home_dir {
        let drifloon_path = home
            .join(".uxntal")
            .join(".drifblim")
            .join("src")
            .join("drifloon.rom");
        if drifloon_path.exists() {
            return drifloon_path;
        }
    }
    PathBuf::from("drifloon.rom")
}

/// Returns the path to drifblim.rom in the user's home directory, or just "drifblim.rom" if not found.
pub fn drifblim_repo_get_drifblim() -> PathBuf {
    let home_dir = dirs::home_dir();
    if let Some(home) = home_dir {
        let drifblim_path = home
            .join(".uxntal")
            .join(".drifblim")
            .join("src")
            .join("drifblim.rom");
        if drifblim_path.exists() {
            return drifblim_path;
        }
    }
    PathBuf::from("drifblim.rom")
}

/// Returns the path to drifblim-seed.rom in the user's home directory, or just "drifblim-seed.rom" if not found.
pub fn drifblim_repo_get_drifblim_seed() -> PathBuf {
    let home_dir = dirs::home_dir();
    if let Some(home) = home_dir {
        let drifblim_seed_path = home
            .join(".uxntal")
            .join(".drifblim")
            .join("bin")
            .join("drifblim-seed.rom");
        if drifblim_seed_path.exists() {
            return drifblim_seed_path;
        }
    }
    PathBuf::from("drifblim-seed.rom")
}
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

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

pub fn ensure_drifblim_repo() -> Result<Option<PathBuf>, AssemblerError> {
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
    let home_dir = dirs::home_dir().ok_or_else(|| {
        simple_err(
            Path::new("~/.uxntal/.drifblim"),
            "failed to get home directory",
        )
    })?;
    let uxntal_path = home_dir.join(".uxntal");
    let drifblim_path = uxntal_path.join(".drifblim");
    if !drifblim_path.exists() {
        let status = Command::new("git")
            .arg("clone")
            .arg("https://git.sr.ht/~rabbits/drifblim")
            .arg(&drifblim_path)
            .status()
            .map_err(|e| {
                simple_err(&drifblim_path, &format!("failed to execute git clone: {e}"))
            })?;
        if !status.success() {
            eprintln!("Failed to clone drifblim repository");
            return Err(simple_err(&drifblim_path, "failed to clone repository"));
        }
    } else {
        // If already exists, do a git pull
        let status = Command::new("git")
            .arg("-C")
            .arg(&drifblim_path)
            .arg("pull")
            .status()
            .ok();
        if let Some(status) = status {
            if !status.success() {
                eprintln!("Failed to pull drifblim repository");
            }
        } else {
            eprintln!("Failed to execute git pull for drifblim repository");
        }
    }
    if !drifblim_path.exists() {
        eprintln!("drifblim repository not found after clone/pull");
        return Err(simple_err(
            &drifblim_path,
            "drifblim repository not found after clone/pull",
        ));
    }
    let _guard = DirGuard::new(&drifblim_path);

    let drifblim_seed_path = uxntal_path
        .join(".drifblim")
        .join("bin")
        .join("drifblim-seed.rom");
    let drifblim_seed_txt_path = uxntal_path
        .join(".drifblim")
        .join("etc")
        .join("drifblim.rom.txt");
    HexRev::hex_to_bin_paths(&drifblim_seed_txt_path, &drifblim_seed_path).map_err(|e| {
        simple_err(
            &drifblim_seed_txt_path,
            &format!("failed to convert hex to bin: {e}"),
        )
    })?;

    let drifloon_rom = drifblim_path.join("src").join("drifloon.rom");
    if !drifloon_rom.exists() {
        let mut asm = Assembler::new();

        let drifloon_tal = drifblim_path.join("src").join("drifloon.tal");
        if !drifloon_tal.exists() {
            eprintln!("drifloon.tal not found in drifblim repository");
            return Err(simple_err(&drifblim_path, "drifloon.tal not found"));
        }
        eprintln!("Assembling drifloon.tal to drifloon.rom...");
        let drifloon_tal_contents = match fs::read_to_string(&drifloon_tal) {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("Failed to read drifloon.tal: {:?}", e);
                return Err(simple_err(&drifloon_tal, "failed to read drifloon.tal"));
            }
        };
        let ret = asm.assemble(
            &drifloon_tal_contents,
            Some(drifloon_rom.display().to_string()),
        );
        match ret {
            Ok(rom) => {
                fs::write(&drifloon_rom, &rom)
                    .map_err(|e| simple_err(&drifloon_rom, &format!("failed to write rom: {e}")))
                    .ok();
                if drifloon_rom.exists() {
                    eprintln!(
                        "Successfully assembled drifloon.tal {}",
                        drifloon_rom.display()
                    );
                } else {
                    eprintln!(
                        "Assembly succeeded but drifloon.rom not found at {}",
                        drifloon_rom.display()
                    );
                    return Err(simple_err(&drifloon_rom, "drifloon.rom not found"));
                }
            }
            Err(e) => {
                eprintln!("Failed to assemble drifloon.tal: {:?}", e);
                return Err(simple_err(&drifloon_tal, "failed to assemble drifloon.tal"));
            }
        }
    }

    let drifblim_rom = drifblim_path.join("src").join("drifblim.rom");
    if !drifblim_rom.exists() {
        let mut asm = Assembler::new();

        let drifblim_tal = drifblim_path.join("src").join("drifblim.tal");
        if !drifblim_tal.exists() {
            eprintln!("drifblim.tal not found in drifblim repository");
            return Err(simple_err(&drifblim_tal, "drifblim.tal not found"));
        }
        eprintln!("Assembling drifblim.tal to drifblim.rom...");
        let drifblim_tal_contents = match fs::read_to_string(&drifblim_tal) {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("Failed to read drifblim.tal: {:?}", e);
                return Err(simple_err(&drifblim_tal, "failed to read drifblim.tal"));
            }
        };
        let ret = asm.assemble(
            &drifblim_tal_contents,
            Some(drifblim_rom.display().to_string()),
        );
        match ret {
            Ok(rom) => {
                fs::write(&drifblim_rom, &rom)
                    .map_err(|e| simple_err(&drifblim_rom, &format!("failed to write rom: {e}")))
                    .ok();
                if drifblim_rom.exists() {
                    eprintln!(
                        "Successfully assembled drifblim.tal {}",
                        drifblim_rom.display()
                    );
                } else {
                    eprintln!(
                        "Assembly succeeded but drifblim.rom not found at {}",
                        drifblim_rom.display()
                    );
                    return Err(simple_err(&drifblim_rom, "drifblim.rom not found"));
                }
            }
            Err(e) => {
                eprintln!("Failed to assemble drifblim.tal: {:?}", e);
                return Err(simple_err(&drifblim_tal, "failed to assemble drifblim.tal"));
            }
        }
    }

    Ok(Some(drifblim_path))
}
