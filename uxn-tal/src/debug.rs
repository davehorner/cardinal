use crate::{Assembler, AssemblerError};
use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus};

pub trait AssemblerBackend {
    fn name(&self) -> &'static str;
    fn assemble(&self, tal_file: &str, tal_source: &str) -> Result<AssemblyOutput, AssemblerError>;
}

pub struct AssemblyOutput {
    pub rom_path: String,
    pub rom_bytes: Vec<u8>,
    pub stdout: String,
    pub disassembly: String,
}

impl Default for AssemblyOutput {
    fn default() -> Self {
        Self {
            rom_path: String::new(),
            rom_bytes: vec![],
            stdout: String::new(),
            disassembly: String::new(),
        }
    }
}

pub struct DebugAssembleResult {
    pub uxntal_rom_path: String,
    pub uxnasm_rom_path: String,
    pub drifblim_rom_path: String,
    pub uxntal_rom_bytes: Vec<u8>,
    pub uxnasm_rom_bytes: Vec<u8>,
    pub drifblim_rom_bytes: Vec<u8>,
    pub uxntal_output: String,
    pub uxnasm_output: String,
    pub drifblim_output: String,
    pub uxntal_dis_output: String,
    pub uxnasm_dis_output: String,
    pub drifblim_dis_output: String,
    pub diff_uxntal_uxnasm: Option<(usize, String, String)>,
    pub diff_uxntal_drifblim: Option<(usize, String, String)>,
    pub diff_uxnasm_drifblim: Option<(usize, String, String)>,
    pub backend_errors: Vec<(String, String)>,
}

pub struct UxntalBackend;
impl AssemblerBackend for UxntalBackend {
    fn name(&self) -> &'static str {
        "uxntal"
    }
    fn assemble(&self, tal_file: &str, tal_source: &str) -> Result<AssemblyOutput, AssemblerError> {
        let mut assembler = Assembler::new();
        let rom = assembler.assemble(tal_source, Some(tal_file.to_string()))?;
        let rom_path = format!("{}_{}.rom", tal_file, self.name());
        fs::write(&rom_path, &rom).map_err(|e| io_err(&rom_path, e))?;
        Ok(AssemblyOutput {
            rom_path: rom_path.clone(),
            rom_bytes: rom.clone(),
            stdout: run_vm_last(&rom)?,
            disassembly: run_dis_file(&rom_path)?,
        })
    }
}

pub struct UxnasmBackend;
impl AssemblerBackend for UxnasmBackend {
    fn name(&self) -> &'static str {
        "uxnasm"
    }
    fn assemble(&self, tal_file: &str, _src: &str) -> Result<AssemblyOutput, AssemblerError> {
        let input = tal_file.replace('\\', "/");
        let rom_path = format!("{}_{}.rom", input, self.name());
        let status = spawn_cmd("uxnasm", &["--verbose", &input, &rom_path])?;
        if !status.success() {
            return Err(syntax_err(&input, "uxnasm failed"));
        }
        let bytes = fs::read(&rom_path).unwrap_or_default();
        Ok(AssemblyOutput {
            rom_path: rom_path.clone(),
            rom_bytes: bytes.clone(),
            stdout: run_vm_path(&rom_path)?,
            disassembly: run_dis_file(&rom_path)?,
        })
    }
}

pub struct DrifblimBackend;
impl AssemblerBackend for DrifblimBackend {
    fn name(&self) -> &'static str {
        "drifblim"
    }
    fn assemble(&self, tal_file: &str, _src: &str) -> Result<AssemblyOutput, AssemblerError> {
        let input = tal_file.replace('\\', "/");
        let rom_path = format!("{}_{}.rom", input, self.name());
        let stdout = Self::run_drif(tal_file, &rom_path)?;
        let bytes = fs::read(&rom_path).unwrap_or_default();
        Ok(AssemblyOutput {
            rom_path: rom_path.clone(),
            rom_bytes: bytes,
            stdout,
            disassembly: run_dis_file(&rom_path)?,
        })
    }
}
impl DrifblimBackend {
    fn run_drif(tal_file: &str, rom_path: &str) -> Result<String, AssemblerError> {
        let in_wsl = detect_wsl();
        let output = if in_wsl {
            Command::new("uxncli")
                .arg("drifblim.rom")
                .arg(tal_file)
                .arg(rom_path)
                .output()
        } else {
            Command::new("wsl")
                .arg("uxncli")
                .arg("drifblim.rom")
                .arg(tal_file)
                .arg(rom_path)
                .output()
        }
        .map_err(|e| syntax_err(rom_path, &format!("drifblim failed: {e}")))?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

fn detect_wsl() -> bool {
    std::env::var("WSL_DISTRO_NAME").is_ok()
        || std::env::var("WSLENV").is_ok()
        || (Path::new("/proc/version").exists()
            && fs::read_to_string("/proc/version")
                .unwrap_or_default()
                .to_lowercase()
                .contains("microsoft"))
}

fn spawn_cmd(cmd: &str, args: &[&str]) -> Result<ExitStatus, AssemblerError> {
    let in_wsl = detect_wsl();
    let status = if in_wsl {
        Command::new(cmd).args(args).status()
    } else {
        let mut all = vec![cmd];
        all.extend(args);
        Command::new("wsl").args(all).status()
    }
    .map_err(|e| syntax_err("", &format!("Failed to spawn {cmd}: {e}")))?;
    Ok(status)
}

fn run_vm_path(rom_path: &str) -> Result<String, AssemblerError> {
    let in_wsl = detect_wsl();
    let output = if in_wsl {
        Command::new("uxncli").arg(rom_path).output()
    } else {
        Command::new("wsl").arg("uxncli").arg(rom_path).output()
    }
    .map_err(|e| syntax_err(rom_path, &format!("uxncli failed: {e}")))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run_vm_last(bytes: &[u8]) -> Result<String, AssemblerError> {
    let tmp = ".__temp_uxntal_exec.rom";
    fs::write(tmp, bytes).map_err(|e| io_err(tmp, e))?;
    let out = run_vm_path(tmp)?;
    let _ = fs::remove_file(tmp);
    Ok(out)
}

fn run_dis_file(rom_path: &str) -> Result<String, AssemblerError> {
    let sym = format!("{rom_path}.sym");
    if Path::new(&sym).exists() {
        let _ = fs::remove_file(&sym);
    }
    let in_wsl = detect_wsl();
    let output = if in_wsl {
        Command::new("uxncli")
            .arg("uxndis.rom")
            .arg(rom_path)
            .output()
    } else {
        Command::new("wsl")
            .arg("uxncli")
            .arg("uxndis.rom")
            .arg(rom_path)
            .output()
    }
    .map_err(|e| syntax_err(rom_path, &format!("uxndis failed: {e}")))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn syntax_err(path: &str, msg: &str) -> AssemblerError {
    AssemblerError::SyntaxError {
        path: path.to_string(),
        line: 0,
        position: 0,
        message: msg.to_string(),
        source_line: String::new(),
    }
}

fn io_err(path: &str, e: std::io::Error) -> AssemblerError {
    syntax_err(path, &format!("IO error: {e}"))
}

pub struct DebugAssembler;

impl Default for DebugAssembler {
    fn default() -> Self {
        DebugAssembler
    }
}

impl DebugAssembler {
    pub fn assemble_and_compare(
        &self,
        tal_file: &str,
        tal_source: &str,
        verbose: bool,
    ) -> Result<DebugAssembleResult, AssemblerError> {
        let backends: Vec<Box<dyn AssemblerBackend>> = vec![
            Box::new(UxntalBackend),
            Box::new(UxnasmBackend),
            Box::new(DrifblimBackend),
        ];
        let mut uxntal = None;
        let mut uxnasm = None;
        let mut drif = None;
        let mut backend_errors = Vec::new();

        for b in backends {
            match b.assemble(tal_file, tal_source) {
                Ok(out) => {
                    // Always write .sym file for each backend's ROM if possible
                    let sym_path = format!("{}.sym", out.rom_path);
                    if !out.rom_bytes.is_empty() {
                        // Use the internal assembler for symbol generation if possible
                        if b.name() == "uxntal" {
                            let mut assembler = Assembler::new();
                            if let Ok(_) = assembler.assemble(tal_source, Some(tal_file.to_string())) {
                                let sym_bytes = assembler.generate_symbol_file();
                                let _ = fs::write(&sym_path, &sym_bytes);
                            }
                        }
                    }
                    match b.name() {
                        "uxntal" => uxntal = Some(out),
                        "uxnasm" => uxnasm = Some(out),
                        "drifblim" => drif = Some(out),
                        _ => {}
                    }
                }
                Err(e) => backend_errors.push((b.name().to_string(), e.to_string())),
            }
        }

        // Print summary and diffs for all three backends, even if one fails
        if verbose {
            println!("\n== Backend Summary ==");
            println!(
                "  {:<9} {:<5} {:>8}  {}",
                "backend", "stat", "bytes", "output/summary"
            );
            for (name, out) in [
                ("uxntal", &uxntal),
                ("uxnasm", &uxnasm),
                ("drifblim", &drif),
            ] {
                if let Some(ref o) = out {
                    println!("  {:<9} {:<5} {:>8}  {}", name, "OK", o.rom_bytes.len(), o.rom_path);
                } else {
                    let summary = backend_errors
                        .iter()
                        .find(|(n, _)| n == name)
                        .map(|(_, e)| e.as_str())
                        .unwrap_or("-");
                    println!("  {:<9} {:<5} {:>8}  {}", name, "FAIL", 0, summary);
                }
            }

            // Print byte diffs for all pairs
            println!("\n== Byte Diffs ==");
            let pairs = [
                ("uxntal", &uxntal, "uxnasm", &uxnasm),
                ("uxntal", &uxntal, "drifblim", &drif),
                ("uxnasm", &uxnasm, "drifblim", &drif),
            ];
            for (a_name, a, b_name, b) in pairs {
                print!("{:<8} vs {:<9}: ", a_name, b_name);
                if let (Some(ref a), Some(ref b)) = (a, b) {
                    if let Some(d) = first_byte_diff(&a.rom_bytes, &b.rom_bytes) {
                        println!(
                            "first diff at 0x{:04X}: {:02X} != {:02X}",
                            d.index, d.a, d.b
                        );
                    } else {
                        println!("identical");
                    }
                } else {
                    println!("skipped (missing backend)");
                }
            }

            // Print disassembly diffs for all pairs
            println!("\n== Disassembly Diffs (first differing line) ==");
            let dis_pairs = [
                ("uxntal", &uxntal, "uxnasm", &uxnasm),
                ("uxntal", &uxntal, "drifblim", &drif),
                ("uxnasm", &uxnasm, "drifblim", &drif),
            ];
            for (a_name, a, b_name, b) in dis_pairs {
                print!("{:<8} vs {:<9}: ", a_name, b_name);
                if let (Some(ref a), Some(ref b)) = (a, b) {
                    match diff_first(&a.disassembly, &b.disassembly) {
                        Some((ln, la, lb)) => {
                            println!("line {}:\n  {}: {}\n  {}: {}", ln, a_name, la, b_name, lb);
                        }
                        None => println!("identical"),
                    }
                } else {
                    println!("skipped (missing backend)");
                }
            }

            // Print backend errors if any
            if !backend_errors.is_empty() {
                println!("\n== Backend Errors ==");
                for (name, err) in &backend_errors {
                    println!("{}: {}", name, err);
                }
            }
        }

        // Prepare summary fields
        let uxntal_rom_path = uxntal.as_ref().map(|x| x.rom_path.clone()).unwrap_or_default();
        let uxnasm_rom_path = uxnasm.as_ref().map(|x| x.rom_path.clone()).unwrap_or_default();
        let drifblim_rom_path = drif.as_ref().map(|x| x.rom_path.clone()).unwrap_or_default();
        let uxntal_rom_bytes = uxntal.as_ref().map(|x| x.rom_bytes.clone()).unwrap_or_default();
        let uxnasm_rom_bytes = uxnasm.as_ref().map(|x| x.rom_bytes.clone()).unwrap_or_default();
        let drifblim_rom_bytes = drif.as_ref().map(|x| x.rom_bytes.clone()).unwrap_or_default();
        let uxntal_output = uxntal.as_ref().map(|x| x.stdout.clone()).unwrap_or_default();
        let uxnasm_output = uxnasm.as_ref().map(|x| x.stdout.clone()).unwrap_or_default();
        let drifblim_output = drif.as_ref().map(|x| x.stdout.clone()).unwrap_or_default();
        let uxntal_dis_output = uxntal.as_ref().map(|x| x.disassembly.clone()).unwrap_or_default();
        let uxnasm_dis_output = uxnasm.as_ref().map(|x| x.disassembly.clone()).unwrap_or_default();
        let drifblim_dis_output = drif.as_ref().map(|x| x.disassembly.clone()).unwrap_or_default();

        // Disassembly diffs for all pairs
        let diff_uxntal_uxnasm = diff_first(&uxntal_dis_output, &uxnasm_dis_output);
        let diff_uxntal_drifblim = diff_first(&uxntal_dis_output, &drifblim_dis_output);
        let diff_uxnasm_drifblim = diff_first(&uxnasm_dis_output, &drifblim_dis_output);

        Ok(DebugAssembleResult {
            uxntal_rom_path,
            uxnasm_rom_path,
            drifblim_rom_path,
            uxntal_rom_bytes,
            uxnasm_rom_bytes,
            drifblim_rom_bytes,
            uxntal_output,
            uxnasm_output,
            drifblim_output,
            uxntal_dis_output,
            uxnasm_dis_output,
            drifblim_dis_output,
            diff_uxntal_uxnasm,
            diff_uxntal_drifblim,
            diff_uxnasm_drifblim,
            backend_errors,
        })
    }
}

fn diff_first(a: &str, b: &str) -> Option<(usize, String, String)> {
    for (i, (la, lb)) in a.lines().zip(b.lines()).enumerate() {
        if la != lb {
            return Some((i + 1, la.to_string(), lb.to_string()));
        }
    }
    let ac = a.lines().count();
    let bc = b.lines().count();
    if ac > bc {
        a.lines().nth(bc).map(|extra| (bc + 1, extra.to_string(), String::new()))
    } else if bc > ac {
        b.lines().nth(ac).map(|extra| (ac + 1, String::new(), extra.to_string()))
    } else {
        None
    }
}

fn first_byte_diff(a: &[u8], b: &[u8]) -> Option<ByteDiff> {
    let len = std::cmp::min(a.len(), b.len());
    for i in 0..len {
        if a[i] != b[i] {
            return Some(ByteDiff {
                index: i,
                a: a[i],
                b: b[i],
            });
        }
    }
    if a.len() != b.len() {
        Some(ByteDiff {
            index: len,
            a: if a.len() > len { a[len] } else { 0 },
            b: if b.len() > len { b[len] } else { 0 },
        })
    } else {
        None
    }
}

struct ByteDiff {
    index: usize,
    a: u8,
    b: u8,
}


// debug_preprocess.rs: Compare chocolatal (Rust) vs deluge (Docker) preprocessors
//
// This module provides a CLI and helpers to run both preprocessors on the same input,
// diff the results, and print diagnostics for debugging and regression testing.

use std::io::{self, Write};
use crate::chocolatal::{preprocess, deluge_preprocess};

/// Run both preprocessors on the given file and print a diff.
pub fn compare_preprocessors(input_path: &str) -> io::Result<()> {
    // Read input file
    let input = fs::read_to_string(input_path)?;
    // Run chocolatal (Rust)
    let rust_out = match preprocess(&input, input_path) {
        Ok(s) => s,
        Err(e) => format!("[chocolatal error: {:?}]\n", e),
    };
    let rust_path = "chocolatal.tal";
    let mut file = fs::File::create(rust_path)?;
    file.write_all(rust_out.as_bytes())?;
    // Run deluge (Docker)
    let deluge_out = deluge_preprocess(input_path)?;
    let deluge_pp_path = "deluge_pp_cmp.tal";
    let mut file = fs::File::create(deluge_pp_path)?;
    file.write_all(deluge_out.as_bytes())?;
    // Diff the outputs (simple line-by-line)
    let rust_lines: Vec<_> = rust_out.lines().collect();
    let deluge_lines: Vec<_> = deluge_out.lines().collect();
    let max = rust_lines.len().max(deluge_lines.len());
    let mut found = false;
    for i in 0..max {
        let r = rust_lines.get(i).map_or("", |v| v);
        let d = deluge_lines.get(i).map_or("", |v| v);
        if r != d {
            println!("First difference at line {}:{}:\n  chocolatal : {}\n  deluge_pp  : {}\n{}:{}:\n", rust_path, i+1, r, d, deluge_pp_path, i+1);
            found = true;
            break;
        }
    }
    if !found {
        println!("No differences found.");
    }
    Ok(())
}