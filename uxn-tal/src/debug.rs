use crate::dis_uxndis::run_dis_file;
use crate::{bkend_drif, rom, Assembler, AssemblerError};
use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus};
use crate::bkend::{AssemblerBackend, AssemblyOutput};

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

pub struct UxntalBackend {
    pub drif_mode: bool,
}

impl UxntalBackend {
    pub fn new() -> Self {
        Self { drif_mode: false }
    }
    
    pub fn with_drif_mode(drif_mode: bool) -> Self {
        Self { drif_mode }
    }
}

impl AssemblerBackend for UxntalBackend {
    fn name(&self) -> &'static str {
        "uxntal"
    }
    fn assemble(&self, tal_file: &str, tal_source: &str) -> Result<AssemblyOutput, AssemblerError> {
        let mut assembler = if self.drif_mode {
            Assembler::with_drif_mode(true)
        } else {
            Assembler::new()
        };
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
        // let input = tal_file.replace('\\', "/");
        // let input = input.replace("//?/C:", "/mnt/c"); // Handle Windows long path prefix
        let input = wslpath::windows_to_wsl(&tal_file)
            .map_err(|e| syntax_err(&tal_file, &format!("Could not convert TAL path to WSL: {e}")))?;
        //let input = tal_file;//.replace("\\\\?\\", ""); // Handle Windows long path prefix
            // let input = match Path::new(&input).canonicalize() {
            //     Ok(abs_path) => {
            //         let cwd = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
            //         match abs_path.strip_prefix(&cwd) {
            //             Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
            //             Err(_) => abs_path.to_string_lossy().replace('\\', "/"),
            //         }
            //     }
            //     Err(_) => input,
            // };
        let rom_path = format!("{}_{}.rom", input, self.name());
        let mut cmd = spawn_cmd("uxnasm", &["--verbose", &input, &rom_path]);
        let output = cmd.output().map_err(|e| syntax_err(&input, &format!("Failed to run uxnasm: {e}")))?;
        if !output.status.success() {
            return Err(syntax_err(
                &input,
                &format!(
                    "uxnasm failed {:?} {:?} stderr: {} stdout: {}",
                    output.status,
                    std::env::current_dir(),
                    String::from_utf8_lossy(&output.stderr),
                    String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .rev()
                        .take(3)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
            ));
        }
        // let rom_path = rom_path.replace("/mnt/c/", "C:/"); // Handle Windows long path prefix
        let rom_path = wslpath::wsl_to_windows(&rom_path)
            .map_err(|e| syntax_err(&rom_path, &format!("Could not convert ROM path to Windows: {e}")))?;
        let bytes = fs::read(&rom_path).unwrap_or_default();
        Ok(AssemblyOutput {
            rom_path: rom_path.clone(),
            rom_bytes: bytes.clone(),
            stdout: crate::emu_uxncli::run_uxncli_get_stdout(&rom_path)?,
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

        // let mut temp_file = tempfile::NamedTempFile::new().map_err(|e| syntax_err(tal_file, &format!("tempfile error: {e}")))?;
        // temp_file.write_all(_src.as_bytes()).map_err(|e| syntax_err(tal_file, &format!("tempfile write error: {e}")))?;
        // let tal_path = temp_file.path().to_string_lossy().to_string();
        // let tal_path = tal_path.replace('\\', "/");
        // let rom_path = format!("{}_{}.rom", tal_path, self.name());
        // println!("rom_path: {}", rom_path);

        let input = tal_file.replace('\\', "/");
        let input = input.replace("//?/C:", "/mnt/c"); // Handle Windows long path prefix
        //let input = tal_file;//.replace("\\\\?\\", ""); // Handle Windows long path prefix
        let rom_path = format!("{}_{}.rom", input, self.name());
        println!("rom_path: {}", rom_path);
        println!("tal_file: {}", tal_file);
        println!("input: {}", input);
        let stdout = Self::run_drif(&input, &input, &rom_path)?;
        let rom_path = rom_path.replace("/mnt/c/", "C:/"); // Handle Windows long path prefix
        let rom_path = rom_path.replace('/', "\\");
        let bytes = fs::read(&rom_path).unwrap_or_default();
        if bytes.is_empty() {
            return Err(syntax_err(&rom_path, "drifblim produced empty ROM"));
        }
        Ok(AssemblyOutput {
            rom_path: rom_path.clone(),
            rom_bytes: bytes,
            stdout,
            disassembly: run_dis_file(&rom_path)?,
        })
    }
}
impl DrifblimBackend {
    fn run_drif(tal_path: &str, tal_file: &str, rom_path: &str) -> Result<String, AssemblerError> {
            println!("tal_path: {}", tal_path);
    println!("rom_path: {}", rom_path);
    let drifblim_rom = crate::bkend_drif::drifblim_repo_get_drifblim();
    let drifblim_rom = wslpath::windows_to_wsl(&drifblim_rom.to_string_lossy()).or_else(|_| Err(syntax_err(rom_path, "Could not convert drifblim path to WSL")))?;
        let in_wsl = detect_wsl();
        let output = if in_wsl {
            Command::new("uxncli")
                .arg(drifblim_rom)
                .arg(tal_path)
                .arg(rom_path)
                .output()
        } else {
            Command::new("wsl")
                .arg("uxncli")
                .arg(drifblim_rom)
                .arg(tal_path)
                .arg(rom_path)
                .output()
        }
        .map_err(|e| syntax_err(rom_path, &format!("drifblim failed: {e}")))?;
println!("drifblim stdout: {:?}", output);
        if output.stderr.len() > 0 {
            let stderr_str = String::from_utf8_lossy(&output.stderr);
            if stderr_str.contains("Assembled ") {
                // Write tal_file to a temp file with a short name
                println!("Path exceeded error from drifblim, retrying with short path");
                // let tal_path_rel = Path::new(tal_path)
                //     .file_name()
                //     .map(|f| f.to_string_lossy().to_string())
                //     .unwrap_or_else(|| "temp.tal".to_string());
                // let temp_path = format!("./{}", tal_path_rel);
                // let rom_path = format!("{}_{}.rom", temp_path, "drifblim");
                // let tal_path = wslpath::wsl_to_windows(tal_path)
                //     .or_else(|_| Err(syntax_err(&rom_path, "Could not convert TAL path to WSL")))?;
                // fs::write(&temp_path, tal_file.as_bytes()).map_err(|e| syntax_err(&temp_path, &format!("Failed to write to temp path: {e}")))?;
                // return Self::run_drif(&temp_path, &tal_file, &rom_path);
            } else {
            return Err(syntax_err(
                rom_path,
                &format!(
                    "drifblim stderr: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
            }

        }   
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
/////
/// 
pub struct DrifblimSeedBackend;
impl AssemblerBackend for DrifblimSeedBackend {
    fn name(&self) -> &'static str {
        "drifseed"
    }
    fn assemble(&self, tal_file: &str, _src: &str) -> Result<AssemblyOutput, AssemblerError> {

        // let mut temp_file = tempfile::NamedTempFile::new().map_err(|e| syntax_err(tal_file, &format!("tempfile error: {e}")))?;
        // temp_file.write_all(_src.as_bytes()).map_err(|e| syntax_err(tal_file, &format!("tempfile write error: {e}")))?;
        // let tal_path = temp_file.path().to_string_lossy().to_string();
        // let tal_path = tal_path.replace('\\', "/");
        // let rom_path = format!("{}_{}.rom", tal_path, self.name());
        // println!("rom_path: {}", rom_path);
        let tal_file = Path::new(tal_file)
            .canonicalize()
            .ok()
            .and_then(|abs| {
            std::env::current_dir()
                .ok()
                .and_then(|cwd| abs.strip_prefix(&cwd).ok().map(|rel| rel.to_path_buf()))
                .or(Some(abs))
            })
            .unwrap_or_else(|| Path::new(tal_file).to_path_buf());
        let cwd_wsl = std::env::current_dir()
            .ok()
            .and_then(|cwd| wslpath::windows_to_wsl(&cwd.display().to_string()).ok())
            .unwrap_or_else(|| ".".to_string());
        let input = wslpath::windows_to_wsl(&tal_file.display().to_string())
            .map_err(|e| syntax_err(&tal_file.display().to_string(), &format!("Could not convert TAL path to WSL: {e}")))?;
        let input = if input.starts_with(&cwd_wsl) {
            input[cwd_wsl.len()..].trim_start_matches('/').to_string()
        } else {
            input
        };
        //let input = tal_file;//.replace("\\\\?\\", ""); // Handle Windows long path prefix
        let rom_path = format!("{}_{}.rom", input, self.name());
        println!("cwd_wsl: {}", cwd_wsl);
        println!("rom_path: {}", rom_path);
        println!("tal_file: {}", tal_file.display());
        println!("input: {}", input);
                        // std::process::exit(0);
        let stdout = Self::run_drif(&input, &input, &rom_path)?;
        let rom_path = wslpath::wsl_to_windows(&rom_path)
            .map_err(|e| syntax_err(&rom_path, &format!("Could not convert ROM path to Windows: {e}")))?;   
        let bytes = fs::read(&rom_path).unwrap_or_default();
        println!("rom_path: {}", rom_path);

        if bytes.is_empty() {
            return Err(syntax_err(&rom_path, "drifblim produced empty ROM"));
        }

        Ok(AssemblyOutput {
            rom_path: rom_path.clone(),
            rom_bytes: bytes,
            stdout,
            disassembly: run_dis_file(&rom_path)?,
        })
    }
}
impl DrifblimSeedBackend {
    fn run_drif(tal_path: &str, tal_file: &str, rom_path: &str) -> Result<String, AssemblerError> {
            println!("tal_path: {}", tal_path);
    println!("rom_path: {}", rom_path);
    let drifblim_rom = crate::bkend_drif::drifblim_repo_get_drifblim_seed();
    let drifblim_rom = wslpath::windows_to_wsl(&drifblim_rom.to_string_lossy()).or_else(|_| Err(syntax_err(rom_path, "Could not convert drifblim path to WSL")))?;
        let in_wsl = detect_wsl();
        let output = if in_wsl {
            Command::new("uxncli")
                .arg(drifblim_rom)
                .arg(tal_path)
                .arg(rom_path)
                .output()
        } else {
            Command::new("wsl")
                .arg("uxncli")
                .arg(drifblim_rom)
                .arg(tal_path)
                .arg(rom_path)
                .output()
        }
        .map_err(|e| syntax_err(rom_path, &format!("drifblim failed: {e}")))?;
println!("drifblim stdout: {:?}", output);
        if output.stderr.len() > 0 {
            let stderr_str = String::from_utf8_lossy(&output.stderr);
            if stderr_str.contains("Assembled ") {
                // Write tal_file to a temp file with a short name
                // println!("Path exceeded error from drifblim, retrying with short path");
                // let tal_path_rel = Path::new(tal_path)
                //     .file_name()
                //     .map(|f| f.to_string_lossy().to_string())
                //     .unwrap_or_else(|| "temp.tal".to_string());
                // let temp_path = format!("./{}", tal_path_rel);
                // let rom_path = format!("{}_{}.rom", temp_path, "drifblim");
                // let tal_path = wslpath::wsl_to_windows(tal_path)
                //     .or_else(|_| Err(syntax_err(&rom_path, "Could not convert TAL path to WSL")))?;
                // fs::write(&temp_path, tal_file.as_bytes()).map_err(|e| syntax_err(&temp_path, &format!("Failed to write to temp path: {e}")))?;
                // return Self::run_drif(&temp_path, &tal_file, &rom_path);
            } else {
            return Err(syntax_err(
                rom_path,
                &format!(
                    "drifblim stderr: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
            }

        }   
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}



pub struct DrifloonBackend;
impl AssemblerBackend for DrifloonBackend {
    fn name(&self) -> &'static str {
        "drifloon"
    }
    fn assemble(&self, tal_file: &str, _src: &str) -> Result<AssemblyOutput, AssemblerError> {

        // let mut temp_file = tempfile::NamedTempFile::new().map_err(|e| syntax_err(tal_file, &format!("tempfile error: {e}")))?;
        // temp_file.write_all(_src.as_bytes()).map_err(|e| syntax_err(tal_file, &format!("tempfile write error: {e}")))?;
        // let tal_path = temp_file.path().to_string_lossy().to_string();
        // let tal_path = tal_path.replace('\\', "/");
        // let rom_path = format!("{}_{}.rom", tal_path, self.name());
        // println!("rom_path: {}", rom_path);


                let input = tal_file.replace('\\', "/");
        let input = input.replace("//?/C:", "/mnt/c"); // Handle Windows long path prefix
        //let input = tal_file;//.replace("\\\\?\\", ""); // Handle Windows long path prefix
        let rom_path = format!("{}_{}.rom", input, self.name());
        println!("rom_path: {}", rom_path);
        println!("tal_file: {}", tal_file);
        println!("input: {}", input);
        let stdout = Self::run_drifloon(&input, &_src, &rom_path)?;
        let rom_path = rom_path.replace("/mnt/c/", "C:/"); // Handle Windows long path prefix
        let rom_path = rom_path.replace('/', "\\");
        let bytes = fs::read(&rom_path).unwrap_or_default();
        if bytes.is_empty() {
            return Err(syntax_err(&rom_path, "drifloon produced empty ROM"));
        }
        Ok(AssemblyOutput {
            rom_path: rom_path.clone(),
            rom_bytes: bytes,
            stdout,
            disassembly: run_dis_file(&rom_path)?,
        })
    }
}
impl DrifloonBackend {
    fn run_drifloon(tal_path: &str, tal_file: &str, rom_path: &str) -> Result<String, AssemblerError> {
            println!("tal_path: {}", tal_path);
    println!("rom_path: {}", rom_path);
    let home_dir = dirs::home_dir().ok_or_else(|| syntax_err(rom_path, "Could not determine home directory"))?;
    let uxntal_path = home_dir.join(".uxntal");
    let drifblim_path = uxntal_path.join(".drifblim");
    let drifloon_path = drifblim_path.join("src").join("drifloon.rom");
    let drifloon_path = wslpath::windows_to_wsl(&drifloon_path.to_string_lossy()).or_else(|_| Err(syntax_err(rom_path, "Could not convert drifloon path to WSL")))?;
        let in_wsl = detect_wsl();
        let output = if in_wsl {
            let mut child = Command::new("uxncli")
                .arg(&drifloon_path)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| syntax_err(rom_path, &format!("drifloon failed to spawn: {e}")))?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(tal_file.as_bytes())
                    .map_err(|e| syntax_err(rom_path, &format!("drifloon failed to write to stdin: {e}")))?;
            }
            child.wait_with_output()
        } else {
            let mut child = Command::new("wsl")
                .arg("uxncli")
                .arg(&drifloon_path)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| syntax_err(rom_path, &format!("drifloon failed to spawn: {e}")))?;
            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(tal_file.as_bytes())
                    .map_err(|e| syntax_err(rom_path, &format!("drifloon failed to write to stdin: {e}")))?;
            }
            child.wait_with_output()
        }
        .map_err(|e| syntax_err(rom_path, &format!("drifloon failed: {e}")))?;
    println!("drifloon path: {}", &drifloon_path);
println!("drifloon stdout: {:?}", output);
println!("drifloon stderr: {:?}", output.stderr);
println!("rom_path: {}", rom_path);
// Write output to rom_path
if !output.stdout.is_empty() {
    let rom_path = wslpath::wsl_to_windows(rom_path)
        .map_err(|e| syntax_err(rom_path, &format!("Could not convert ROM path to Windows: {e}")))?;
    fs::write(&rom_path, &output.stdout)
        .map_err(|e| syntax_err(&rom_path, &format!("Failed to write ROM: {e}")))?;
}

// println!("{:?}", tal_file);
        if output.stderr.len() > 0 && output.stdout.len() == 0 {
            return Err(syntax_err(
                rom_path,
                &format!(
                    "drifloon stderr: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        }   
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

fn spawn_cmd(cmd: &str, args: &[&str]) -> Command {
    let in_wsl = detect_wsl();
    if in_wsl {
        let mut command = Command::new(cmd);
        command.args(args);
        command
    } else {
        let mut all = vec![cmd];
        all.extend(args);
        let mut command = Command::new("wsl");
        command.args(all);
        command
    }
}


fn run_vm_last(bytes: &[u8]) -> Result<String, AssemblerError> {
    let tmp = ".__temp_uxntal_exec.rom";
    fs::write(tmp, bytes).map_err(|e| io_err(tmp, e))?;
    let out = crate::emu_uxncli::run_uxncli_get_stdout(tmp)?;
    let _ = fs::remove_file(tmp);
    Ok(out)
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

pub struct DebugAssembler {
    pub drif_mode: bool,
}

impl Default for DebugAssembler {
    fn default() -> Self {
        DebugAssembler { drif_mode: false }
    }
}

impl DebugAssembler {
    pub fn new() -> Self {
        Self { drif_mode: false }
    }
    
    pub fn with_drif_mode(drif_mode: bool) -> Self {
        Self { drif_mode }
    }
    
    pub fn assemble_and_compare(
        &self,
        tal_file: &str,
        tal_source: &str,
        verbose: bool,
    ) -> Result<DebugAssembleResult, AssemblerError> {
        let backends: Vec<Box<dyn AssemblerBackend>> = vec![
            Box::new(UxntalBackend::with_drif_mode(self.drif_mode)),
            Box::new(DrifloonBackend),
            Box::new(UxnasmBackend),
            Box::new(DrifblimBackend),
            Box::new(DrifblimSeedBackend),
            Box::new(crate::bkend_buxn::UxnBuxnBackend),
            Box::new(crate::bkend_uxn::UxnDUxnAsmBackend),
        ];
        let mut uxntal = None;
        let mut uxnasm = None;
        let mut drif = None;
        let mut drifloon = None;
        let mut drifblim_seed = None;
        let mut buxn = None;
        let mut uxn38 = None;
        let mut duxnasm = None;
        let mut backend_errors = Vec::new();

        for b in backends {
            match b.assemble(tal_file, tal_source) {
                Ok(out) => {
                    // Always write .sym file for each backend's ROM if possible
                    let sym_path = format!("{}.sym", out.rom_path);
                    if !out.rom_bytes.is_empty() {
                        // Use the internal assembler for symbol generation if possible
                        if b.name() == "uxntal" {
                            let mut assembler = if self.drif_mode {
                                Assembler::with_drif_mode(true)
                            } else {
                                Assembler::new()
                            };
                            if let Ok(_) =
                                assembler.assemble(tal_source, Some(tal_file.to_string()))
                            {
                                let sym_bytes = assembler.generate_symbol_file();
                                let _ = fs::write(&sym_path, &sym_bytes);
                            }
                        }
                    }
                    match b.name() {
                        "uxntal" => uxntal = Some(out),
                        "uxnasm" => uxnasm = Some(out),
                        "drifblim" => drif = Some(out),
                        "drifloon" => drifloon = Some(out),
                        "drifseed" => drifblim_seed = Some(out),
                        "buxn" => buxn = Some(out),
                        "uxn38" => uxn38 = Some(out),
                        "duxnasm" => duxnasm = Some(out),
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
                ("drifloon", &drifloon),
                ("drifseed", &drifblim_seed),
                ("buxn", &buxn),
                ("uxn38", &uxn38),
                ("duxnasm", &duxnasm),
            ] {
                if let Some(ref o) = out {
                    println!(
                        "  {:<9} {:<5} {:>8}  {}",
                        name,
                        "OK",
                        o.rom_bytes.len(),
                        o.rom_path
                    );
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
                ("uxntal", &uxntal, "drifloon", &drifloon),
                ("uxnasm", &uxnasm, "drifloon", &drifloon),
                ("drifblim", &drif, "drifloon", &drifloon),
                ("drifblim", &drif, "drifseed", &drifblim_seed),
                ("uxntal", &uxntal, "drifseed", &drifblim_seed),
                ("uxnasm", &uxnasm, "drifseed", &drifblim_seed),
                ("drifloon", &drifloon, "drifseed", &drifblim_seed),
                ("buxn", &buxn, "uxntal", &uxntal),
                ("buxn", &buxn, "uxnasm", &uxnasm),
                ("buxn", &buxn, "drifblim", &drif),
                ("buxn", &buxn, "drifloon", &drifloon),
                ("buxn", &buxn, "drifseed", &drifblim_seed),
                ("uxn38", &uxn38, "uxntal", &uxntal),
                ("uxn38", &uxn38, "uxnasm", &uxnasm),
                ("uxn38", &uxn38, "drifblim", &drif),
                ("uxn38", &uxn38, "drifloon", &drifloon),
                ("uxn38", &uxn38, "drifseed", &drifblim_seed),
                ("uxn38", &uxn38, "buxn", &buxn),
                ("duxnasm", &duxnasm, "uxntal", &uxntal),
                ("duxnasm", &duxnasm, "uxnasm", &uxnasm),
                ("duxnasm", &duxnasm, "drifblim", &drif),
                ("duxnasm", &duxnasm, "drifloon", &drifloon),
                ("duxnasm", &duxnasm, "drifseed", &drifblim_seed),
                ("duxnasm", &duxnasm, "buxn", &buxn),
                ("duxnasm", &duxnasm, "uxn38", &uxn38),
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
                ("uxntal", &uxntal, "drifloon", &drifloon),
                ("uxnasm", &uxnasm, "drifloon", &drifloon),
                ("drifblim", &drif, "drifloon", &drifloon),
                ("drifblim", &drif, "drifseed", &drifblim_seed),
                ("uxntal", &uxntal, "drifseed", &drifblim_seed),
                ("uxnasm", &uxnasm, "drifseed", &drifblim_seed),
                ("drifloon", &drifloon, "drifseed", &drifblim_seed),
                ("buxn", &buxn, "uxntal", &uxntal),
                ("buxn", &buxn, "uxnasm", &uxnasm),
                ("buxn", &buxn, "drifblim", &drif),
                ("buxn", &buxn, "drifloon", &drifloon),
                ("buxn", &buxn, "drifseed", &drifblim_seed),
                ("uxn38", &uxn38, "uxntal", &uxntal),
                ("uxn38", &uxn38, "uxnasm", &uxnasm),
                ("uxn38", &uxn38, "drifblim", &drif),
                ("uxn38", &uxn38, "drifloon", &drifloon),
                ("uxn38", &uxn38, "drifseed", &drifblim_seed),
                ("uxn38", &uxn38, "buxn", &buxn),
                ("duxnasm", &duxnasm, "uxntal", &uxntal),
                ("duxnasm", &duxnasm, "uxnasm", &uxnasm),
                ("duxnasm", &duxnasm, "drifblim", &drif),
                ("duxnasm", &duxnasm, "drifloon", &drifloon),
                ("duxnasm", &duxnasm, "drifseed", &drifblim_seed),
                ("duxnasm", &duxnasm, "buxn", &buxn),
                ("duxnasm", &duxnasm, "uxn38", &uxn38),
            ];
            for (a_name, a, b_name, b) in dis_pairs {
                print!("{:<8} vs {:<9}: ", a_name, b_name);
                if let (Some(ref a), Some(ref b)) = (a, b) {
                    
                    if a.disassembly.is_empty() {
                        println!("skipped (empty disassembly from {})", a_name);
                        continue;
                    }
                    if b.disassembly.is_empty() {
                        println!("skipped (empty disassembly from {})", b_name);
                        continue;
                    }
                    match diff_first(&a.disassembly, &b.disassembly) {
                        Some((ln, la, lb)) => {
                            println!("line {}:\n  {}: {}\n  {}: {}", ln, a_name, la, b_name, lb);
                        }
                        None => {
                            let lines: Vec<_> = a.disassembly.lines().take(3).collect();
                            if lines.is_empty() {
                                println!("identical (no disassembly output)");
                            } else {
                                println!("identical");
                                // for l in lines {
                                //     println!("    {}", l);
                                // }
                                // if a.disassembly.lines().count() > 3 {
                                //     println!("    ...");
                                // }
                                // println!(
                                //     "identical (first lines):\n    {}: {}\n    {}: {}",
                                //     a_name, a.rom_path, b_name, b.rom_path
                                // );
                            }
                        }
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
        let uxntal_rom_path = uxntal
            .as_ref()
            .map(|x| x.rom_path.clone())
            .unwrap_or_default();
        let uxnasm_rom_path = uxnasm
            .as_ref()
            .map(|x| x.rom_path.clone())
            .unwrap_or_default();
        let drifblim_rom_path = drif
            .as_ref()
            .map(|x| x.rom_path.clone())
            .unwrap_or_default();
        let uxntal_rom_bytes = uxntal
            .as_ref()
            .map(|x| x.rom_bytes.clone())
            .unwrap_or_default();
        let uxnasm_rom_bytes = uxnasm
            .as_ref()
            .map(|x| x.rom_bytes.clone())
            .unwrap_or_default();
        let drifblim_rom_bytes = drif
            .as_ref()
            .map(|x| x.rom_bytes.clone())
            .unwrap_or_default();
        let uxntal_output = uxntal
            .as_ref()
            .map(|x| x.stdout.clone())
            .unwrap_or_default();
        let uxnasm_output = uxnasm
            .as_ref()
            .map(|x| x.stdout.clone())
            .unwrap_or_default();
        let drifblim_output = drif.as_ref().map(|x| x.stdout.clone()).unwrap_or_default();
        let uxntal_dis_output = uxntal
            .as_ref()
            .map(|x| x.disassembly.clone())
            .unwrap_or_default();
        let uxnasm_dis_output = uxnasm
            .as_ref()
            .map(|x| x.disassembly.clone())
            .unwrap_or_default();
        let drifblim_dis_output = drif
            .as_ref()
            .map(|x| x.disassembly.clone())
            .unwrap_or_default();

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
            // Check if the difference is only in symbol comments (parts in parentheses)
            if let (Some(a_without_comment), Some(b_without_comment)) = (strip_symbol_comment(la), strip_symbol_comment(lb)) {
                if a_without_comment == b_without_comment {
                    // Only symbol comment differs, skip this difference
                    continue;
                }
            }
            return Some((i + 1, la.to_string(), lb.to_string()));
        }
    }
    let ac = a.lines().count();
    let bc = b.lines().count();
    if ac > bc {
        a.lines()
            .nth(bc)
            .map(|extra| (bc + 1, extra.to_string(), String::new()))
    } else if bc > ac {
        b.lines()
            .nth(ac)
            .map(|extra| (ac + 1, String::new(), extra.to_string()))
    } else {
        None
    }
}

/// Strip symbol comments from disassembly lines to compare only the actual bytecode
/// Returns the line without the comment part if it has one, otherwise returns the original line
fn strip_symbol_comment(line: &str) -> Option<&str> {
    if let Some(comment_start) = line.find('(') {
        if let Some(comment_end) = line.rfind(')') {
            if comment_start < comment_end {
                // Extract the part before the comment and trim whitespace
                return Some(line[..comment_start].trim_end());
            }
        }
    }
    Some(line)
}

fn first_byte_diff(a: &[u8], b: &[u8]) -> Option<ByteDiff> {
    let len = std::cmp::min(a.len(), b.len());
    for i in 0..len {
        if a[i] != b[i] {
            let rom_address = i + 0x100; // Convert file offset to ROM address
            // Add debug for the specific problematic offset
            if rom_address == 0x0B48 {
                eprintln!("DEBUG: [Byte comparison] At ROM address 0x{:04X} (file offset 0x{:04X}): a[{}]=0x{:02X}, b[{}]=0x{:02X}", rom_address, i, i, a[i], i, b[i]);
                // Show context around the problematic byte
                let start = (i as i32 - 8).max(0) as usize;
                let end = (i + 8).min(len);
                eprint!("DEBUG: [Context a]: ");
                for j in start..end {
                    if j == i {
                        eprint!("[{:02X}] ", a[j]);
                    } else {
                        eprint!("{:02X} ", a[j]);
                    }
                }
                eprintln!();
                eprint!("DEBUG: [Context b]: ");
                for j in start..end {
                    if j == i {
                        eprint!("[{:02X}] ", b[j]);
                    } else {
                        eprint!("{:02X} ", b[j]);
                    }
                }
                eprintln!();
            }
            return Some(ByteDiff {
                index: rom_address, // Report ROM address, not file offset
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

use crate::chocolatal::{deluge_preprocess, preprocess};
use std::io::{self, Write};
use std::path::PathBuf;

/// Run both preprocessors on the given file and print a diff.
pub fn compare_preprocessors(input_path: &str, root_dir: &PathBuf) -> io::Result<()> {
    // Read input file
    let input = fs::read_to_string(input_path)?;
    // Run chocolatal (Rust)
    let rust_out = match preprocess(&input, input_path, root_dir) {
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
            println!(
                "First difference at line {}:{}:\n  chocolatal : {}\n  deluge_pp  : {}\n{}:{}:\n",
                rust_path,
                i + 1,
                r,
                d,
                deluge_pp_path,
                i + 1
            );
            found = true;
            break;
        }
    }
    if !found {
        println!("No differences found.");
    }
    Ok(())
}
