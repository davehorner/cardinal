//! emu_buxn.rs
//! Argument mapping for buxn emulator

use crate::uxntal_protocol::EmulatorArgMapper;
use crate::uxntal_protocol::EmulatorLauncher;
use std::path::PathBuf;
use std::process::Command;
#[cfg(not(target_arch = "wasm32"))]
use which::which;
pub struct BuxnMapper;

impl EmulatorLauncher for BuxnMapper {
    fn build_command(
        &self,
        result: &crate::uxntal_protocol::ProtocolParseResult,
        rom_path: &str,
        emulator_path: &std::path::Path,
    ) -> Command {
        let mut cmd = Command::new(emulator_path);
        let mut args = self.map_args(result);
        args.push(rom_path.to_string());
        cmd.args(&args);
        cmd
    }
}

impl EmulatorArgMapper for BuxnMapper {
    fn map_args(&self, _result: &crate::uxntal_protocol::ProtocolParseResult) -> Vec<String> {
        let args = vec![];

        // Add more mappings as needed
        args
    }
}

impl crate::uxntal_protocol::EmulatorPathCheck for BuxnMapper {
    fn is_available_in_path(
        _result: &crate::uxntal_protocol::ProtocolParseResult,
    ) -> Option<PathBuf> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            which("buxn-gui").ok()
        }
        #[cfg(target_arch = "wasm32")]
        {
            None
        }
    }
}
