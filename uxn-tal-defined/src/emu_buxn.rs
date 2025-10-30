use crate::v1::EmulatorLauncher;
use std::process::Command;
use uxn_tal_common::cache::RomCache;

pub struct BuxnMapper<'a> {
    pub rom_cache: &'a dyn RomCache,
}

impl<'a> EmulatorLauncher<'a> for BuxnMapper<'a> {
    fn build_command(
        &self,
        result: &crate::v1::ProtocolParseResult,
        rom_path: &str,
        emulator_path: &std::path::Path,
    ) -> Command {
        if let Some(crate::v1::ProtocolVarVar::Bool(true)) = result.proto_vars.get("orca") {
            use crate::consts::CANONICAL_ORCA;
            use std::fs;
            use std::path::Path;
            let canonical_rom_path = match self
                .rom_cache
                .get_or_write_cached_rom(CANONICAL_ORCA, Path::new("orca.rom"))
            {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("[BuxnMapper] Failed to cache/fetch canonical orca ROM: {e}");
                    Path::new("orca.rom").to_path_buf()
                }
            };
            let orca_file = Path::new(rom_path);
            let orca_rom = orca_file.with_file_name("orca.rom");
            if canonical_rom_path != orca_rom {
                if let Err(e) = fs::copy(&canonical_rom_path, &orca_rom) {
                    eprintln!("[BuxnMapper] Failed to copy canonical orca ROM to working dir: {e}");
                }
            }
            let mut cmd = Command::new(emulator_path);
            let mut args = self.map_args(result);
            args.push(orca_rom.to_string_lossy().to_string());
            args.push(rom_path.to_string());
            cmd.args(&args);
            return cmd;
        }
        let mut cmd = Command::new(emulator_path);
        let mut args = self.map_args(result);
        args.push(rom_path.to_string());
        cmd.args(&args);
        cmd
    }
}
use crate::v1::{EmulatorArgMapper, ProtocolParseResult};

impl<'a> EmulatorArgMapper for BuxnMapper<'a> {
    fn map_args(&self, _result: &ProtocolParseResult) -> Vec<String> {
        vec![]
    }
}
// Buxn emulator argument/feature mapping for protocol variables
// Adapted from uxn-tal/src/emu_buxn.rs

use std::path::PathBuf;
// use std::process::Command;

impl<'a> BuxnMapper<'a> {
    pub fn is_available_in_path() -> Option<PathBuf> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            which::which("buxn-gui").ok()
        }
        #[cfg(target_arch = "wasm32")]
        {
            None
        }
    }
    // Add feature support table and mapping logic here
}
