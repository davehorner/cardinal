use crate::uxntal_protocol::EmulatorArgMapper;
use crate::uxntal_protocol::EmulatorLauncher;
use std::path::PathBuf;
use std::process::Command;
#[cfg(not(target_arch = "wasm32"))]
use which::which;

impl crate::uxntal_protocol::EmulatorPathCheck for UxnMapper {
    fn is_available_in_path(
        _result: &crate::uxntal_protocol::ProtocolParseResult,
    ) -> Option<PathBuf> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            which("uxnemu").ok()
        }
        #[cfg(target_arch = "wasm32")]
        {
            None
        }
    }
}
pub struct UxnMapper;

impl EmulatorLauncher for UxnMapper {
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

impl EmulatorArgMapper for UxnMapper {
    fn map_args(&self, _result: &crate::uxntal_protocol::ProtocolParseResult) -> Vec<String> {
        let mut _args = vec![];

        // Add more mappings as needed
        _args
    }
}
