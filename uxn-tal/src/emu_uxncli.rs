use std::process::Command;

use crate::AssemblerError;

fn emu_err(path: &str, msg: &str) -> AssemblerError {
    AssemblerError::Backend {
        message: format!("{path}: {msg}"),
    }
}

pub fn run_uxncli_get_stdout(rom_path: &str) -> Result<String, AssemblerError> {
    if cfg!(windows) {
        let in_wsl = crate::wsl::detect_wsl();
        let output = if in_wsl {
            Command::new("uxncli").arg(rom_path).output()
        } else {
            Command::new("wsl").arg("uxncli").arg(rom_path).output()
        }
        .map_err(|e| emu_err(rom_path, &format!("uxncli failed: {e}")))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let output = Command::new("uxncli")
            .arg(rom_path)
            .output()
            .map_err(|e| emu_err(rom_path, &format!("uxncli failed: {e}")))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
