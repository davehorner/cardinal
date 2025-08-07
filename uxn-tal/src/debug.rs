use crate::{Assembler, AssemblerError};
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct DebugAssembleResult {
    pub rust_rom_path: String,
    pub wsl_rom_path: String,
    pub rust_rom_bytes: Vec<u8>,
    pub wsl_rom_bytes: Vec<u8>,
    pub rust_output: String,
    pub wsl_output: String,
    pub rust_dis_output: String,
    pub wsl_dis_output: String,
    pub first_line_difference_dis: Option<(usize, String, String)>,
}

pub struct DebugAssembler;

impl Default for DebugAssembler {
    fn default() -> Self {
        DebugAssembler
    }
}

impl DebugAssembler {
    pub fn assemble(
        &self,
        tal_file: &str,
        tal_source: &str,
    ) -> Result<(String, String, Vec<u8>, Vec<u8>, String, String, String, String), AssemblerError> {
        let _tal_file_name = Path::new(tal_file)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();

        let mut assembler = Assembler::new();
        let rom = match assembler.assemble(tal_source, Some(tal_file.to_owned())) {
            Ok(rom) => rom,
            Err(e) => {
                eprintln!("Rust assembler error: {:?}", e);
                Vec::new()
            }
        };
        let wsl_tal_file = tal_file.replace("\\", "/");
        let rust_rom_path = format!("{}_compare.rom", tal_file);
        fs::write(&rust_rom_path, &rom)?;

        // Detect if we're already in WSL environment
        let in_wsl = std::env::var("WSL_DISTRO_NAME").is_ok()
            || std::env::var("WSLENV").is_ok()
            || Path::new("/proc/version").exists()
                && fs::read_to_string("/proc/version")
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains("microsoft");

        // WSL uxnasm
        let wsl_rom_path = format!("{}_wsl.rom", wsl_tal_file);
        let wsl_file_name = Path::new(&wsl_rom_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();

        let status = if in_wsl {
            Command::new("uxnasm")
                .arg("--verbose")
                .arg(&wsl_tal_file)
                .arg(&wsl_rom_path)
                .status()?
        } else {
            Command::new("wsl")
                .arg("uxnasm")
                .arg("--verbose")
                .arg(&wsl_tal_file)
                .arg(&wsl_rom_path)
                .status()?
        };

        println!(
            "Ran command: {} uxnasm --verbose {} {}",
            if in_wsl { "" } else { "wsl " },
            wsl_tal_file,
            wsl_file_name
        );
        println!("Status: {}", status);
        let rust_rom_bytes = fs::read(&rust_rom_path).unwrap_or_default();
        let wsl_rom_bytes = fs::read(&wsl_rom_path).unwrap_or_default();

        // uxncli run
        let run_rom = |rom_path: &str| -> Result<String, std::io::Error> {
            let output = if in_wsl {
                Command::new("uxncli").arg(rom_path).output()?
            } else {
                Command::new("wsl").arg("uxncli").arg(rom_path).output()?
            };
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        };
        let rust_output = run_rom(&rust_rom_path).unwrap_or_default();
        let wsl_output = run_rom(&wsl_rom_path).unwrap_or_default();

        // uxncli uxndis.rom -- <rom>
        let run_dis = |rom_path: &str| -> Result<String, std::io::Error> {
            let sym_path = format!("{}.sym", rom_path);
            if Path::new(&sym_path).exists() {
                let _ = fs::remove_file(&sym_path);
            }
            let output = if in_wsl {
                Command::new("uxncli")
                    .arg("uxndis.rom")
                    .arg(rom_path)
                    .output()?
            } else {
                Command::new("wsl")
                    .arg("uxncli")
                    .arg("uxndis.rom")
                    .arg(rom_path)
                    .output()?
            };
            println!(
                "Ran command: {} uxncli uxndis.rom {}",
                if in_wsl { "" } else { "wsl " },
                rom_path
            );
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        };

        let rust_dis_output = run_dis(&rust_rom_path.replace("\\", "/")).unwrap_or_default();
        let wsl_dis_output = run_dis(&wsl_rom_path).unwrap_or_default();

        Ok((
            rust_rom_path,
            wsl_rom_path,
            rust_rom_bytes,
            wsl_rom_bytes,
            rust_output,
            wsl_output,
            rust_dis_output,
            wsl_dis_output,
        ))
    }

    pub fn compare(
        &self,
        rust_dis_output: &str,
        wsl_dis_output: &str,
    ) -> Option<(usize, String, String)> {
        // Find first line of difference in disassembly output
        let mut first_line_difference_dis = None;
        for (i, (r, w)) in rust_dis_output
            .lines()
            .zip(wsl_dis_output.lines())
            .enumerate()
        {
            if r != w {
                first_line_difference_dis = Some((i + 1, r.to_string(), w.to_string()));
                break;
            }
        }
        // If one output is longer, report the first extra line
        if first_line_difference_dis.is_none() {
            let rust_lines = rust_dis_output.lines().count();
            let wsl_lines = wsl_dis_output.lines().count();
            if rust_lines > wsl_lines {
                if let Some(extra) = rust_dis_output.lines().nth(wsl_lines) {
                    first_line_difference_dis =
                        Some((wsl_lines + 1, extra.to_string(), String::new()));
                }
            } else if wsl_lines > rust_lines {
                if let Some(extra) = wsl_dis_output.lines().nth(rust_lines) {
                    first_line_difference_dis =
                        Some((rust_lines + 1, String::new(), extra.to_string()));
                }
            }
        }
        first_line_difference_dis
    }

    pub fn assemble_and_compare(
        &self,
        tal_file: &str,
        tal_source: &str,
    ) -> Result<DebugAssembleResult, AssemblerError> {
        let (
            rust_rom_path,
            wsl_rom_path,
            rust_rom_bytes,
            wsl_rom_bytes,
            rust_output,
            wsl_output,
            rust_dis_output,
            wsl_dis_output,
        ) = self.assemble(tal_file, tal_source)?;

        let first_line_difference_dis = self.compare(&rust_dis_output, &wsl_dis_output);

        Ok(DebugAssembleResult {
            rust_rom_path,
            wsl_rom_path,
            rust_rom_bytes,
            wsl_rom_bytes,
            rust_output,
            wsl_output,
            rust_dis_output,
            wsl_dis_output,
            first_line_difference_dis,
        })
    }
    
}
