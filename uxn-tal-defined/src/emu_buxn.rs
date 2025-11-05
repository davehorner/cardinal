use crate::v1::{EmulatorArgMapper, EmulatorLauncher, ProtocolParseResult};
use std::process::Command;

pub struct BuxnMapper;

impl<'a> EmulatorLauncher<'a> for BuxnMapper {
    fn build_command(
        &self,
        result: &crate::v1::ProtocolParseResult,
        rom_path: &str,
        emulator_path: &std::path::Path,
        cache_dir: Option<&std::path::Path>,
    ) -> Command {
        let mut cmd = Command::new(emulator_path);
        let mut args = self.map_args(result);
        args.push(rom_path.to_string());
        let post_args = self.map_post_args(result);
        args.extend(post_args);
        cmd.args(&args);

        // Set working directory to cache_dir if provided
        if let Some(dir) = cache_dir {
            cmd.current_dir(dir);
        }

        cmd
    }

    fn timeout_follow_forceexit(&self, _emulator_path: &std::path::Path) -> bool {
        true // buxn doesn't support --timeout flag
    }

    fn cli_executable_name(&self) -> Option<&'static str> {
        Some("buxn-cli")
    }
}

impl EmulatorArgMapper for BuxnMapper {
    fn map_args(&self, _result: &ProtocolParseResult) -> Vec<String> {
        let args = vec![];
        args
    }

    fn map_post_args(&self, result: &ProtocolParseResult) -> Vec<String> {
        let mut args = vec![];

        // Handle !arg1 as an argument after the ROM (no -- separator for buxn)
        if let Some(qv) = result.bang_vars.get("arg1") {
            match &qv.value {
                crate::v1::ProtocolQueryVarVar::String(s) => args.push(s.clone()),
                _ => args.push(format!("{:?}", qv.value)),
            }
        }

        args
    }
}
// Buxn emulator argument/feature mapping for protocol variables
// Adapted from uxn-tal/src/emu_buxn.rs

use std::path::PathBuf;
// use std::process::Command;

impl BuxnMapper {
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
