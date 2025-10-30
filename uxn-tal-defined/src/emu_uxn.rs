use crate::v1::EmulatorLauncher;
use std::process::Command;

impl EmulatorLauncher for UxnMapper {
    fn build_command(
        &self,
        _result: &crate::v1::ProtocolParseResult,
        rom_path: &str,
        emulator_path: &std::path::Path,
    ) -> Command {
        let mut cmd = Command::new(emulator_path);
        let mut args = self.map_args(_result);
        args.push(rom_path.to_string());
        cmd.args(&args);
        cmd
    }
}
use crate::v1::{EmulatorArgMapper, ProtocolParseResult};

impl EmulatorArgMapper for UxnMapper {
    fn map_args(&self, _result: &ProtocolParseResult) -> Vec<String> {
        vec![]
    }
}
// Uxn emulator argument/feature mapping for protocol variables
// Adapted from uxn-tal/src/emu_uxn.rs

use std::path::PathBuf;
// use std::process::Command;

pub struct UxnMapper;

impl UxnMapper {
    pub fn is_available_in_path() -> Option<PathBuf> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            which::which("uxnemu").ok()
        }
        #[cfg(target_arch = "wasm32")]
        {
            None
        }
    }
    // Add feature support table and mapping logic here
}
