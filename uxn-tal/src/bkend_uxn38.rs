use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    bkend::{AssemblerBackend, AssemblyOutput},
    AssemblerError,
};

fn simple_err(path: &std::path::Path, msg: &str) -> AssemblerError {
    AssemblerError::SyntaxError {
        path: path.display().to_string(),
        line: 0,
        position: 0,
        message: msg.to_string(),
        source_line: String::new(),
    }
}

pub fn uxn38_repo_get_path() -> Option<PathBuf> {
    let home_dir = dirs::home_dir();
    if let Some(home) = home_dir {
        let uxn38_path = home.join(".uxntal").join(".uxn38");
        if uxn38_path.exists() {
            return Some(uxn38_path);
        }
    }
    None
}

pub fn ensure_uxn38_repo() -> Result<Option<PathBuf>, AssemblerError> {
    struct DirGuard {
        original: PathBuf,
    }

    impl DirGuard {
        fn new(target: &Path) -> Option<Self> {
            let original = std::env::current_dir().ok()?;
            if std::env::set_current_dir(target).is_ok() {
                Some(DirGuard { original })
            } else {
                None
            }
        }
    }

    impl Drop for DirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original);
        }
    }
    let home_dir = dirs::home_dir()
        .ok_or_else(|| simple_err(Path::new("~/.uxntal/.bxn"), "failed to get home directory"))?;
    let uxntal_path = home_dir.join(".uxntal");
    let uxn38_path = uxntal_path.join(".uxn38");
    if !uxn38_path.exists() {
        let status = Command::new("git")
            .arg("clone")
            .arg("https://github.com/davehorner/uxn38.git")
            .arg(&uxn38_path)
            .status()
            .map_err(|e| simple_err(&uxn38_path, &format!("failed to execute git clone: {e}")))?;
        if !status.success() {
            eprintln!("Failed to clone uxn38 repository");
            return Err(simple_err(&uxn38_path, "failed to clone repository"));
        }
    } else {
        // If already exists, do a git pull
        let status = Command::new("git")
            .arg("-C")
            .arg(&uxn38_path)
            .arg("pull")
            .status()
            .ok();
        if let Some(status) = status {
            if !status.success() {
                eprintln!("Failed to pull uxndis repository");
            }
        } else {
            eprintln!("Failed to execute git pull for uxndis repository");
        }
    }
    if !uxn38_path.exists() {
        eprintln!("uxn38 repository not found after clone/pull");
        return Err(simple_err(
            &uxn38_path,
            "uxn38 repository not found after clone/pull",
        ));
    }
    let _guard = DirGuard::new(&uxn38_path);

    Ok(Some(uxn38_path))
}

pub fn ensure_docker_uxn38_image() -> Result<(), AssemblerError> {
    #[cfg(all(target_family = "wasm", target_os = "unknown"))]
    {
        return Err(simple_err(
            Path::new("."),
            "docker not available in browser WASM",
        ));
    }
    #[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
    {
        let docker_path = which::which("docker")
            .map_err(|_| simple_err(Path::new("."), "docker not found in PATH"))?;
        let images_output = Command::new(&docker_path)
            .arg("images")
            .arg("-q")
            .arg("uxn38-linux")
            .output()
            .map_err(|e| {
                simple_err(
                    Path::new("."),
                    &format!("failed to check docker images: {e}"),
                )
            })?;

        if !images_output.stdout.is_empty() {
            println!("Docker uxn38-linux image already exists.");
            // Image already exists
            return Ok(());
        }
        let uxn_path = uxn38_repo_get_path().ok_or_else(|| {
            simple_err(
                Path::new("."),
                "uxn38 repository not found; cannot build docker image",
            )
        })?;
        println!(
            "Building Docker uxn38-linux image. {}  Be patient this can take some time.",
            uxn_path.display()
        );
        let status = Command::new(&docker_path)
            .current_dir(uxn_path)
            .arg("build")
            .arg("--no-cache")
            .arg("--progress=plain")
            .arg("-t")
            .arg("uxn38-linux")
            .arg(".")
            .output()
            .map_err(|e| {
                simple_err(
                    Path::new("."),
                    &format!("failed to build docker image: {e}"),
                )
            })?;
        if !status.status.success() {
            println!("output: {}", String::from_utf8_lossy(&status.stdout));
            println!("error: {}", String::from_utf8_lossy(&status.stderr));
            eprintln!("Failed to build uxn38-linux docker image");
        } else {
            println!("Successfully built uxn38-linux docker image");
        }
        Ok(())
    }
}

fn bkend_err(_path: &std::path::Path, msg: &str) -> AssemblerError {
    AssemblerError::Backend {
        message: msg.to_string(),
    }
}

pub struct UxnUxn38Backend;
impl AssemblerBackend for UxnUxn38Backend {
    fn name(&self) -> &'static str {
        "uxn38"
    }
    fn assemble(&self, tal_file: &str, _src: &str) -> Result<AssemblyOutput, AssemblerError> {
        #[cfg(all(target_family = "wasm", target_os = "unknown"))]
        {
            return Err(bkend_err(
                Path::new("."),
                "docker not available in browser WASM",
            ));
        }
        #[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
        {
            // ...existing code...
            let input = if tal_file.starts_with(r"\\?\") {
                tal_file.trim_start_matches(r"\\?\").replace('\\', "/")
            } else {
                tal_file.replace('\\', "/")
            };
            let tal_file = &input;
            let rom_path = format!("{}_{}.rom", tal_file, self.name());
            let docker_path = which::which("docker")
                .map_err(|_| bkend_err(Path::new("."), "docker not found in PATH"))?;
            let cwd_path = std::env::current_dir()?
                .display()
                .to_string()
                .replace(r"\\?\", "")
                .replace("\\", "/")
                .replace("c:", "C:");
            let tal_file = tal_file.strip_prefix(&cwd_path).unwrap_or(tal_file);
            let rom_path = rom_path.strip_prefix(&cwd_path).unwrap_or(&rom_path);
            let tal_file = tal_file.trim_start_matches('/');
            let rom_path = rom_path.trim_start_matches('/');
            let docker_cmd = Command::new(docker_path)
                .arg("run")
                .arg("--rm")
                .arg("-v")
                .arg(format!("{}:/src", &cwd_path))
                .arg("-w")
                .arg("/src")
                .arg("uxn38-linux")
                .arg("/app/uxnasm")
                .arg(tal_file)
                .arg(rom_path)
                .output()
                .map_err(|e| AssemblerError::Backend {
                    message: format!("Failed to run docker uxn38asm: {e}"),
                })?;

            println!("uxn38: Arguments: {:?}", docker_cmd);
            if !docker_cmd.status.success() {
                return Err(bkend_err(
                    std::path::Path::new(&tal_file),
                    &format!(
                        "docker uxn38-asm failed {} {tal_file} {:?} stderr: {} stdout: {}",
                        std::env::current_dir()?.display(),
                        docker_cmd.status,
                        String::from_utf8_lossy(&docker_cmd.stderr),
                        String::from_utf8_lossy(&docker_cmd.stdout)
                    ),
                ));
            }
            let bytes = fs::read(rom_path).unwrap_or_default();
            Ok(AssemblyOutput {
                rom_path: rom_path.to_string(),
                rom_bytes: bytes.clone(),
                stdout: crate::emu_uxncli::run_uxncli_get_stdout(rom_path)?,
                disassembly: crate::dis_uxndis::run_dis_file(rom_path)?,
            })
        }
    }
}
