use crate::ProtocolVarVar;
use crate::{
    EmulatorArgMapper, EmulatorLauncher, EmulatorPathCheck, ProtocolParseResult,
    ProtocolQueryVarVar,
};
use std::path::PathBuf;
use std::process::Command;
use uxn_tal_common::cache::RomCache;
#[cfg(not(target_arch = "wasm32"))]
use which::which;

impl<'a> EmulatorPathCheck for CuxnMapper<'a> {
    fn is_available_in_path(_result: &ProtocolParseResult) -> Option<PathBuf> {
        let bin = {
            #[cfg(windows)]
            {
                if matches!(
                    _result.proto_vars.get("debug"),
                    Some(ProtocolVarVar::Bool(true))
                ) {
                    "cardinal-gui"
                } else {
                    "cardinal-gui-win"
                }
            }
            #[cfg(not(windows))]
            {
                "cardinal-gui"
            }
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            match which(bin) {
                Ok(path) => Some(path),
                Err(_) => {
                    if let Ok(home) = std::env::var("HOME") {
                        let p = PathBuf::from(format!("{}/.cargo/bin/{}", home, bin));
                        if p.exists() {
                            Some(p)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            None
        }
    }
}

pub struct CuxnMapper<'a> {
    pub rom_cache: &'a dyn RomCache,
}

impl<'a> EmulatorLauncher<'a> for CuxnMapper<'a> {
    fn build_command(
        &self,
        result: &ProtocolParseResult,
        rom_path: &str,
        emulator_path: &std::path::Path,
    ) -> Command {
        if let Some(crate::ProtocolVarVar::Bool(true)) = result.proto_vars.get("orca") {
            use crate::consts::CANONICAL_ORCA;
            use std::fs;
            use std::path::Path;
            let canonical_rom_path = match self
                .rom_cache
                .get_or_write_cached_rom(CANONICAL_ORCA, Path::new("orca.rom"))
            {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("[CuxnMapper] Failed to cache/fetch canonical orca ROM: {e}");
                    Path::new("orca.rom").to_path_buf()
                }
            };
            let orca_file = Path::new(rom_path);
            let orca_rom = orca_file.with_file_name("orca.rom");
            if canonical_rom_path != orca_rom {
                if let Err(e) = fs::copy(&canonical_rom_path, &orca_rom) {
                    eprintln!("[CuxnMapper] Failed to copy canonical orca ROM to working dir: {e}");
                }
            }
            let mut cmd = Command::new(emulator_path);
            let mut args = self.map_args(result);
            args.insert(0, orca_file.to_string_lossy().to_string());
            args.insert(0, orca_rom.to_string_lossy().to_string());
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

impl<'a> EmulatorArgMapper for CuxnMapper<'a> {
    fn map_args(&self, result: &ProtocolParseResult) -> Vec<String> {
        let mut args = vec![];

        if let Some(ProtocolVarVar::String(color)) = result.proto_vars.get("transparent") {
            args.push(format!("--transparent={}", color));
        }
        // Timeout: support both t and timeout as protocol or query variable (float or int)
        let timeout_val = match result
            .proto_vars
            .get("timeout")
            .or(result.proto_vars.get("t"))
        {
            Some(ProtocolVarVar::Float(f)) => Some(f.to_string()),
            Some(ProtocolVarVar::Int(i)) => Some(i.to_string()),
            _ => None,
        }
        .or_else(|| {
            result
                .query_vars
                .get("timeout")
                .or(result.query_vars.get("t"))
                .and_then(|qv| match &qv.value {
                    ProtocolQueryVarVar::Float(f) => Some(f.to_string()),
                    ProtocolQueryVarVar::Int(i) => Some(i.to_string()),
                    ProtocolQueryVarVar::String(s) => s.parse::<f64>().ok().map(|f| f.to_string()),
                    _ => None,
                })
        });
        // Only map each protocol/query variable once
        if let Some(ProtocolVarVar::Enum(emu)) = result.proto_vars.get("emu") {
            args.push(format!("--emu={}", emu));
        }
        if let Some(ProtocolVarVar::Bool(widget)) = result.proto_vars.get("widget") {
            if *widget {
                args.push("--widget".to_string());
            }
        }
        if let Some(ProtocolVarVar::Bool(ontop)) = result.proto_vars.get("ontop") {
            args.push(format!("--ontop={}", ontop));
        }
        if let Some(ProtocolVarVar::Bool(debug)) = result.proto_vars.get("debug") {
            if *debug {
                args.push("--debug".to_string());
            }
        }
        if let Some(ProtocolVarVar::String(efx)) = result.proto_vars.get("efx") {
            args.push(format!("--efx={}", efx));
        }
        if let Some(ProtocolVarVar::String(efxmode)) = result.proto_vars.get("efxmode") {
            args.push(format!("--efxmode={}", efxmode));
        }
        // Map x, y, w, h from protocol variables (take precedence), else query variables
        for &key in &["x", "y", "w", "h"] {
            let flag = format!("--{}", key);
            if let Some(val) = result.proto_vars.get(key) {
                match val {
                    ProtocolVarVar::Int(i) => args.push(format!("{}={}", flag, i)),
                    ProtocolVarVar::Float(f) => args.push(format!("{}={}", flag, f)),
                    ProtocolVarVar::String(s) => args.push(format!("{}={}", flag, s)),
                    _ => {}
                }
            } else if let Some(qv) = result.query_vars.get(key) {
                match &qv.value {
                    ProtocolQueryVarVar::Int(i) => args.push(format!("{}={}", flag, i)),
                    ProtocolQueryVarVar::Float(f) => args.push(format!("{}={}", flag, f)),
                    ProtocolQueryVarVar::String(s) => args.push(format!("{}={}", flag, s)),
                    _ => args.push(format!("{}={:?}", flag, qv.value)),
                }
            }
        }
        // Map fit to --fit argument
        if let Some(ProtocolVarVar::String(fit)) = result.proto_vars.get("fit") {
            args.push(format!("--fit={}", fit));
        } else if let Some(qv) = result.query_vars.get("fit") {
            match &qv.value {
                ProtocolQueryVarVar::String(s) => args.push(format!("--fit={}", s)),
                _ => args.push(format!("--fit={:?}", qv.value)),
            }
        }
        if let Some(qv) = result.query_vars.get("theme") {
            match &qv.value {
                ProtocolQueryVarVar::String(s) => args.push(format!("--theme={}", s)),
                _ => args.push(format!("--theme={:?}", qv.value)),
            }
        }
        // Map scale from protocol or query variables
        if let Some(ProtocolVarVar::Float(scale)) = result.proto_vars.get("scale") {
            args.push(format!("--scale={}", scale));
        } else if let Some(qv) = result.query_vars.get("scale") {
            match &qv.value {
                ProtocolQueryVarVar::Float(f) => args.push(format!("--scale={}", f)),
                ProtocolQueryVarVar::String(s) => args.push(format!("--scale={}", s)),
                _ => args.push(format!("--scale={:?}", qv.value)),
            }
        }
        if let Some(qv) = result.query_vars.get("opacity") {
            match &qv.value {
                ProtocolQueryVarVar::Float(f) => args.push(format!("--opacity={}", f)),
                ProtocolQueryVarVar::String(s) => args.push(format!("--opacity={}", s)),
                _ => args.push(format!("--opacity={:?}", qv.value)),
            }
        }
        for flag in [
            ("borderless", "--borderless"),
            ("fullscreen", "--fullscreen"),
            ("vsync", "--vsync"),
            ("keep_focus", "--keep-focus"),
            ("screenshot", "--screenshot"),
            ("openwindow", "--openwindow"),
            ("close", "--close"),
            ("refresh", "--refresh"),
        ] {
            if let Some(qv) = result.query_vars.get(flag.0) {
                if let ProtocolQueryVarVar::Bool(true) = &qv.value {
                    args.push(flag.1.to_string())
                }
            }
        }
        if let Some(qv) = result.query_vars.get("monitor") {
            match &qv.value {
                ProtocolQueryVarVar::Int(i) => args.push(format!("--monitor={}", i)),
                ProtocolQueryVarVar::String(s) => args.push(format!("--monitor={}", s)),
                _ => args.push(format!("--monitor={:?}", qv.value)),
            }
        }
        if let Some(timeout) = timeout_val {
            args.push(format!("--timeout={}", timeout));
        }
        if let Some(qv) = result.query_vars.get("id") {
            match &qv.value {
                ProtocolQueryVarVar::String(s) => args.push(format!("--id={}", s)),
                _ => args.push(format!("--id={:?}", qv.value)),
            }
        }
        args
    }
}
