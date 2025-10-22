use crate::uxntal_protocol::EmulatorLauncher;
use crate::uxntal_protocol::{EmulatorArgMapper, ProtocolVarVar};
use std::path::PathBuf;
use std::process::Command;
#[cfg(not(target_arch = "wasm32"))]
use which::which;

impl crate::uxntal_protocol::EmulatorPathCheck for CuxnMapper {
    fn is_available_in_path(
        result: &crate::uxntal_protocol::ProtocolParseResult,
    ) -> Option<PathBuf> {
        let bin = {
            #[cfg(windows)]
            {
                if matches!(
                    result.proto_vars.get("debug"),
                    Some(crate::uxntal_protocol::ProtocolVarVar::Bool(true))
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

pub struct CuxnMapper;

impl EmulatorLauncher for CuxnMapper {
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

impl EmulatorArgMapper for CuxnMapper {
    fn map_args(&self, result: &crate::uxntal_protocol::ProtocolParseResult) -> Vec<String> {
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
            match result
                .query_vars
                .get("timeout")
                .or(result.query_vars.get("t"))
            {
                Some(ProtocolVarVar::Float(f)) => Some(f.to_string()),
                Some(ProtocolVarVar::Int(i)) => Some(i.to_string()),
                _ => None,
            }
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
            } else if let Some(val) = result.query_vars.get(key) {
                match val {
                    ProtocolVarVar::Int(i) => args.push(format!("{}={}", flag, i)),
                    ProtocolVarVar::Float(f) => args.push(format!("{}={}", flag, f)),
                    ProtocolVarVar::String(s) => args.push(format!("{}={}", flag, s)),
                    _ => {}
                }
            }
        }
        // Map fit to --fit argument
        if let Some(ProtocolVarVar::String(fit)) = result.proto_vars.get("fit") {
            args.push(format!("--fit={}", fit));
        } else if let Some(ProtocolVarVar::String(fit)) = result.query_vars.get("fit") {
            args.push(format!("--fit={}", fit));
        }
        if let Some(ProtocolVarVar::String(theme)) = result.query_vars.get("theme") {
            args.push(format!("--theme={}", theme));
        }
        // Map scale from protocol or query variables
        if let Some(ProtocolVarVar::Float(scale)) = result.proto_vars.get("scale") {
            args.push(format!("--scale={}", scale));
        } else if let Some(ProtocolVarVar::Float(scale)) = result.query_vars.get("scale") {
            args.push(format!("--scale={}", scale));
        }
        if let Some(ProtocolVarVar::Float(opacity)) = result.query_vars.get("opacity") {
            args.push(format!("--opacity={}", opacity));
        }
        if let Some(ProtocolVarVar::Bool(borderless)) = result.query_vars.get("borderless") {
            if *borderless {
                args.push("--borderless".to_string());
            }
        }
        if let Some(ProtocolVarVar::Bool(fullscreen)) = result.query_vars.get("fullscreen") {
            if *fullscreen {
                args.push("--fullscreen".to_string());
            }
        }
        if let Some(ProtocolVarVar::Bool(vsync)) = result.query_vars.get("vsync") {
            if *vsync {
                args.push("--vsync".to_string());
            }
        }
        if let Some(ProtocolVarVar::Int(monitor)) = result.query_vars.get("monitor") {
            args.push(format!("--monitor={}", monitor));
        }
        if let Some(ProtocolVarVar::Int(timeout)) = result.query_vars.get("timeout") {
            args.push(format!("--timeout={}", timeout));
        }
        if let Some(timeout) = timeout_val {
            args.push(format!("--timeout={}", timeout));
        }
        if let Some(ProtocolVarVar::String(id)) = result.query_vars.get("id") {
            args.push(format!("--id={}", id));
        }
        if let Some(ProtocolVarVar::Bool(keep_focus)) = result.query_vars.get("keep_focus") {
            if *keep_focus {
                args.push("--keep-focus".to_string());
            }
        }
        if let Some(ProtocolVarVar::Bool(screenshot)) = result.query_vars.get("screenshot") {
            if *screenshot {
                args.push("--screenshot".to_string());
            }
        }
        if let Some(ProtocolVarVar::Bool(openwindow)) = result.query_vars.get("openwindow") {
            if *openwindow {
                args.push("--openwindow".to_string());
            }
        }
        if let Some(ProtocolVarVar::Bool(close)) = result.query_vars.get("close") {
            if *close {
                args.push("--close".to_string());
            }
        }
        if let Some(ProtocolVarVar::Bool(refresh)) = result.query_vars.get("refresh") {
            if *refresh {
                args.push("--refresh".to_string());
            }
        }
        args
    }
}
