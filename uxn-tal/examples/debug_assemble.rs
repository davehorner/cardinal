use std::{
    env, fs, io,
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};

use uxn_tal::{Assembler, AssemblerError};

const DIS_IGNORE_COMMENT_DIFF: bool = true; // Set false to count differing comments as diffs

fn main() -> Result<(), AssemblerError> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file.tal>", args[0]);
        std::process::exit(1);
    }
    let tal_path = &args[1];
    let source = fs::read_to_string(tal_path).map_err(|e| serr(tal_path, &format!("read: {e}")))?;

    println!("== Source: {} ==", tal_path);
    for (i, line) in source.lines().enumerate() {
        println!("{:4}: {}", i + 1, line);
    }

    // 1. Internal assembler (non-fatal on error now)
    let mut internal_rom: Option<Vec<u8>> = None;
    let mut internal_err: Option<String> = None;
    let internal_out_path = format!("{tal_path}.uxntal.rom");

    let mut asm = Assembler::new();
    match asm.assemble(&source, Some(tal_path.to_string())) {
        Ok(bytes) => {
            fs::write(&internal_out_path, &bytes).ok();
            println!(
                "\n[uxntal] OK -> {} ({} bytes)",
                internal_out_path,
                bytes.len()
            );
            internal_rom = Some(bytes);
            // Emit symbol file from the same assembler instance.
            let sym_path = format!("{}.sym", &internal_out_path);
            let sym_bytes = asm.generate_symbol_file();
            let _ = fs::write(&sym_path, &sym_bytes);
            if have_uxncli() && Path::new("uxndis.rom").exists() {
                dump_disassembly(&internal_out_path);
            }
        }
        Err(e) => {
            println!("\n[uxntal] FAIL: {e}");
            internal_err = Some(format!("{e}"));
        }
    }

    // 2. External backends (always attempted)
    println!("\nRunning external backends...");
    let uxna = run_uxnasm(tal_path);
    let drif = run_drifblim(tal_path); // optional
    if have_uxncli() && Path::new("uxndis.rom").exists() {
        if internal_rom.is_some() {
            dump_disassembly(&internal_out_path);
        }
        for r in [&uxna, &drif] {
            if r.ok {
                if let Some(ref rp) = r.rom_path {
                    dump_disassembly(rp);
                }
            }
        }
    }

    // 3. Summary
    println!("\n== Backend Summary ==");
    println!(
        "  {:<9} {:<5} {:>8}  output/summary",
        "backend", "stat", "bytes"
    );
    if let Some(ref rom) = internal_rom {
        println!(
            "  {:<9} {:<5} {:>8}  {}",
            "uxntal",
            "OK",
            rom.len(),
            internal_out_path
        );
    } else {
        // Show last 3 non-empty lines of internal error (previously only last 1)
        let summary = internal_err
            .as_ref()
            .map(|e| last_n_lines(e, 3))
            .unwrap_or_else(|| "-".into());
        println!("  {:<9} {:<5} {:>8}  {}", "uxntal", "FAIL", 0, summary);
    }
    for r in [&uxna, &drif] {
        let summary = if r.ok {
            r.rom_path.as_deref().unwrap_or("-").to_string()
        } else {
            let combined = if !r.stderr.trim().is_empty() {
                r.stderr.trim().to_string()
            } else {
                r.stdout.trim().to_string()
            };
            if combined.is_empty() {
                r.error.as_deref().unwrap_or("-").to_string()
            } else {
                last_n_lines(&combined, 3)
            }
        };
        println!(
            "  {:<9} {:<5} {:>8}  {}",
            r.name,
            if r.ok { "OK" } else { "FAIL" },
            r.bytes.len(),
            summary
        );
    }

    // 4. Detailed output (stdout/stderr trimmed)
    for r in [&uxna, &drif] {
        println!("\n== {} ==", r.name);
        if r.ok {
            println!(
                "{}: OK, {} bytes{}",
                r.name,
                r.bytes.len(),
                r.rom_path
                    .as_ref()
                    .map(|p| format!(", rom={}", p))
                    .unwrap_or_default()
            );
        } else {
            println!("{}: FAIL: {}", r.name, r.error.as_deref().unwrap_or("-"));
        }
        if !r.stdout.trim().is_empty() {
            println!("-- stdout --");
            if r.name == "uxnasm" {
                println!("{}", tail_block(&r.stdout, 4000));
            } else {
                println!("{}", trim_block(&r.stdout, 4000));
            }
        }
        if !r.stderr.trim().is_empty() {
            println!("-- stderr --");
            if r.name == "uxnasm" {
                println!("{}", tail_block(&r.stderr, 4000));
            } else {
                println!("{}", trim_block(&r.stderr, 4000));
            }
        }
    }

    // 5. Diffs (only if internal succeeded and external ok)
    if let Some(ref int_rom) = internal_rom {
        println!("\n== Byte Diffs vs uxntal ==");
        for r in [&uxna, &drif] {
            if r.ok {
                print!("uxntal vs {:<9}: ", r.name);
                if let Some(d) = first_byte_diff(int_rom, &r.bytes) {
                    println!(
                        "first diff at 0x{:04X}: {:02X} != {:02X}",
                        d.index, d.a, d.b
                    );
                } else {
                    println!("identical");
                }
            } else {
                println!("uxntal vs {:<9}: skipped ({} failed)", r.name, r.name);
            }
        }

        // Optional disassembly diff (needs uxndis.rom & uxncli)
        if have_uxncli() && Path::new("uxndis.rom").exists() {
            println!("\n== Disassembly Diffs (first differing line) ==");
            if dis_ok(&internal_out_path) {
                let uxntal_dis = disassemble(&internal_out_path).unwrap_or_default();
                // Print the internal ROM path for clarity
                println!("(disassemble) backend=uxntal rom={}", internal_out_path);
                for r in [&uxna, &drif] {
                    if r.ok {
                        if let Some(ref rp) = r.rom_path {
                            if dis_ok(rp) {
                                // NOTE: This is where each external backend (including 'drifblim') is disassembled.
                                // The next call to disassemble(rp) produces the B side of the diff.
                                println!("(disassemble) backend={} rom={}", r.name, rp);
                                match disassemble(rp) {
                                    Some(other_dis) => {
                                        match first_line_diff(&uxntal_dis, &other_dis) {
                                            Some((ln, a, b)) => {
                                                println!(
                                                    "uxntal vs {:<9} line {}:\n  uxntal: {}\n  {:<9}: {}",
                                                    r.name, ln, a, r.name, b
                                                );
                                            }
                                            None => println!("uxntal vs {:<9} identical", r.name),
                                        }
                                    }
                                    None => {
                                        println!("uxntal vs {:<9} disassembly unavailable (empty output)", r.name);
                                    }
                                }
                            } else {
                                println!("uxntal vs {:<9} disassembly unavailable", r.name);
                            }
                        }
                    }
                }
            } else {
                println!("(uxntal disassembly unavailable, skipping)");
            }
        } else {
            println!("\n(disassembly skipped: need uxndis.rom + uxncli)");
        }
    } else {
        println!("\nSkipping diffs (internal uxntal failed).");
    }

    // 6. Explicit note if user expected relative $label support (internal failure hint)
    if internal_err
        .as_ref()
        .map(|e| e.contains("Skip directive requires hex value"))
        .unwrap_or(false)
    {
        println!("\nNOTE: Internal assembler currently rejects $label (relative padding) which uxnasm accepts.");
    }

    Ok(())
}

/* ---------------- Backend runners ---------------- */

struct BackendResult {
    name: &'static str,
    ok: bool,
    rom_path: Option<String>,
    bytes: Vec<u8>,
    stdout: String,
    stderr: String,
    error: Option<String>,
}

fn run_uxnasm(tal: &str) -> BackendResult {
    let out = format!("{tal}.uxnasm.rom");
    let (cmd, mut args): (&str, Vec<String>) = if in_wsl() {
        (
            "uxnasm",
            vec!["--verbose".to_string(), tal.to_string(), out.to_string()],
        )
    } else {
        // Convert Windows paths to WSL paths so uxnasm inside WSL can access them
        let wsl_tal = wslize(tal);
        let wsl_out = wslize(&out);
        (
            "wsl",
            vec![
                "uxnasm".to_string(),
                "--verbose".to_string(),
                wsl_tal,
                wsl_out,
            ],
        )
    };
    match spawn_capture(cmd, &mut args) {
        Ok((status, so, se)) if status.success() && Path::new(&out).exists() => {
            let out_clone = out.clone();
            BackendResult {
                name: "uxnasm",
                ok: true,
                rom_path: Some(out),
                bytes: fs::read(&out_clone).unwrap_or_default(),
                stdout: so,
                stderr: se,
                error: None,
            }
        }
        Ok((_s, so, se)) => BackendResult {
            name: "uxnasm",
            ok: false,
            rom_path: None,
            bytes: vec![],
            stdout: so,
            stderr: se,
            error: Some("uxnasm failed".into()),
        },
        Err(e) => BackendResult {
            name: "uxnasm",
            ok: false,
            rom_path: None,
            bytes: vec![],
            stdout: String::new(),
            stderr: String::new(),
            error: Some(format!("spawn error: {e}")),
        },
    }
}

fn run_drifblim(tal: &str) -> BackendResult {
    // Optional: must exist drifblim.rom driver
    if !Path::new("drifblim.rom").exists() {
        return BackendResult {
            name: "drifblim",
            ok: false,
            rom_path: None,
            bytes: vec![],
            stdout: String::new(),
            stderr: String::new(),
            error: Some("drifblim.rom missing".into()),
        };
    }
    let out = format!("{tal}.drifblim.rom");
    let (cmd, mut args): (&str, Vec<String>) = if in_wsl() {
        (
            "uxncli",
            vec!["drifblim.rom".to_string(), tal.to_string(), out.to_string()],
        )
    } else {
        let wsl_tal = wslize(tal);
        let wsl_out = wslize(&out);
        (
            "wsl",
            vec![
                "uxncli".to_string(),
                "drifblim.rom".to_string(),
                wsl_tal,
                wsl_out,
            ],
        )
    };
    match spawn_capture(cmd, &mut args) {
        Ok((status, so, se)) if status.success() && Path::new(&out).exists() => {
            let out_clone = out.clone();
            BackendResult {
                name: "drifblim",
                ok: true,
                rom_path: Some(out),
                bytes: fs::read(out_clone).unwrap_or_default(),
                stdout: so,
                stderr: se,
                error: None,
            }
        }
        Ok((_s, so, se)) => BackendResult {
            name: "drifblim",
            ok: false,
            rom_path: None,
            bytes: vec![],
            stdout: so,
            stderr: se,
            error: Some("drifblim run failed".into()),
        },
        Err(e) => BackendResult {
            name: "drifblim",
            ok: false,
            rom_path: None,
            bytes: vec![],
            stdout: String::new(),
            stderr: String::new(),
            error: Some(format!("spawn error: {e}")),
        },
    }
}

/* ---------------- Diff helpers ---------------- */

struct ByteDiff {
    index: usize,
    a: u8,
    b: u8,
}
fn first_byte_diff(a: &[u8], b: &[u8]) -> Option<ByteDiff> {
    let n = a.len().min(b.len());
    for i in 0..n {
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
            index: n,
            a: a.get(n).copied().unwrap_or(0),
            b: b.get(n).copied().unwrap_or(0),
        })
    } else {
        None
    }
}

fn first_line_diff(a: &str, b: &str) -> Option<(usize, String, String)> {
    for (i, (la, lb)) in a.lines().zip(b.lines()).enumerate() {
        if la == lb {
            continue;
        }
        if DIS_IGNORE_COMMENT_DIFF {
            let na = strip_dis_comment(la);
            let nb = strip_dis_comment(lb);
            if na == nb {
                // differ only in trailing comment -> ignore
                continue;
            }
        }
        return Some((i + 1, la.to_string(), lb.to_string()));
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

fn strip_dis_comment(line: &str) -> &str {
    match line.find('(') {
        Some(idx) => line[..idx].trim_end(),
        None => line.trim_end(),
    }
}

/* ---------------- Disassembly ---------------- */
fn disassemble(rom_path: &str) -> Option<String> {
    if !have_uxncli() || !Path::new("uxndis.rom").exists() {
        return None;
    }
    let (cmd, mut args): (&str, Vec<String>) = if in_wsl() {
        (
            "uxncli",
            vec!["uxndis.rom".to_string(), rom_path.to_string()],
        )
    } else {
        let wsl_rom_path = wslize(rom_path);
        (
            "wsl",
            vec!["uxncli".to_string(), "uxndis.rom".to_string(), wsl_rom_path],
        )
    };
    if let Ok((status, out, _)) = spawn_capture(cmd, &mut args) {
        if status.success() {
            // Treat empty output as failure to avoid blank B-side in diff
            if out.trim().is_empty() {
                return None;
            }
            return Some(out);
        }
    }
    None
}

fn dis_ok(rom_path: &str) -> bool {
    have_uxncli() && Path::new("uxndis.rom").exists() && Path::new(rom_path).exists()
}

fn dump_disassembly(rom_path: &str) {
    if !dis_ok(rom_path) {
        return;
    }
    if let Some(text) = disassemble(rom_path) {
        let path = format!("{rom_path}.dis.txt");
        let _ = std::fs::write(&path, text);
    }
}

/* ---------------- System helpers ---------------- */

fn spawn_capture(
    cmd: &str,
    args: &mut [impl AsRef<str>],
) -> io::Result<(std::process::ExitStatus, String, String)> {
    let mut c = Command::new(cmd);
    for a in args {
        c.arg(a.as_ref());
    }
    c.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = c.spawn()?;
    let stdout = {
        let mut s = String::new();
        if let Some(mut out) = child.stdout.take() {
            let _ = io::Read::read_to_string(&mut out, &mut s);
        }
        s
    };
    let stderr = {
        let mut s = String::new();
        if let Some(mut er) = child.stderr.take() {
            let _ = io::Read::read_to_string(&mut er, &mut s);
        }
        s
    };
    let status = child.wait()?;
    // Tiny delay to let filesystem flush outputs (mostly for WSL)
    std::thread::sleep(Duration::from_millis(5));
    Ok((status, stdout, stderr))
}

fn in_wsl() -> bool {
    std::env::var("WSL_DISTRO_NAME").is_ok()
        || (Path::new("/proc/version").exists()
            && fs::read_to_string("/proc/version")
                .unwrap_or_default()
                .to_lowercase()
                .contains("microsoft"))
}

fn which(bin: &str) -> Option<String> {
    let path = env::var_os("PATH")?;
    for p in env::split_paths(&path) {
        let cand = p.join(bin);
        if cand.is_file() {
            return cand.to_str().map(|s| s.to_string());
        }
    }
    None
}

fn have_uxncli() -> bool {
    if in_wsl() {
        which("uxncli").is_some()
    } else {
        which("wsl").is_some()
    }
}

/* ---------- Path translation (Windows host -> WSL) ---------- */
fn wslize(p: &str) -> String {
    // Fast path: already looks like a Unix path
    if p.starts_with('/') {
        return p.to_string();
    }
    // Drive letter?
    if p.len() > 2 && p.as_bytes()[1] == b':' {
        let drive = p.chars().next().unwrap().to_ascii_lowercase();
        let rest = p[2..].replace('\\', "/");
        if rest.is_empty() {
            return format!("/mnt/{drive}");
        }
        return format!("/mnt/{drive}/{}", rest.trim_start_matches('/'));
    }
    p.replace('\\', "/")
}

/* ---------------- Misc helpers ---------------- */

fn serr(path: &str, msg: &str) -> AssemblerError {
    AssemblerError::SyntaxError {
        path: path.to_string(),
        line: 0,
        position: 0,
        message: msg.to_string(),
        source_line: String::new(),
    }
}

fn trim_block(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...\n[truncated {} bytes]", &s[..max], s.len() - max)
    }
}

// NEW: tail printer for uxnasm output
fn tail_block(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let start = s.len() - max;
        format!("...\n{}\n[truncated first {} bytes]", &s[start..], start)
    }
}

fn last_n_lines(s: &str, n: usize) -> String {
    let mut lines: Vec<&str> = s.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return String::new();
    }
    if lines.len() > n {
        lines = lines.split_off(lines.len() - n);
    }
    lines.join(" | ")
}
