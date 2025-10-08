use std::{fs, path::{Path, PathBuf}, process::Command};

use crate::{bkend::{AssemblerBackend, AssemblyOutput}, dis_uxndis::run_dis_file, hexrev::HexRev, rom, Assembler, AssemblerError};


fn bkend_err(path: &std::path::Path, msg: &str) -> AssemblerError {
    AssemblerError::Backend { message: msg.to_string() }
}

/// Returns the path to buxn.rom in the user's home directory, or just "buxn.rom" if not found.
pub fn buxn_repo_get_path() -> Option<PathBuf> {
    let home_dir = dirs::home_dir();
    if let Some(home) = home_dir {
        let buxn_path = home.join(".uxntal").join(".buxn");
        if buxn_path.exists() {
            return Some(buxn_path);
        }
    }
    None
}

pub fn ensure_docker_buxn_image() -> Result<(), AssemblerError> {
    let docker_path = which::which("docker")
        .map_err(|_| bkend_err(Path::new("."), "docker not found in PATH"))?;

    let info_output = Command::new(&docker_path)
            .arg("info")
            .output()
            .map_err(|e| bkend_err(Path::new("."), &format!("failed to run 'docker info': {e}")))?;

    if !info_output.status.success() {
       return Err(bkend_err(Path::new("."), "docker daemon does not appear to be running or accessible"));
    }
    let images_output = Command::new(&docker_path)
        .arg("images")
        .arg("-q")
        .arg("buxn-linux")
        .output()
        .map_err(|e| bkend_err(Path::new("."), &format!("failed to check docker images: {e}")))?;

    if !images_output.stdout.is_empty() {
        println!("Docker buxn-linux image already exists.");
        // Image already exists
        return Ok(());
    }
    let buxn_path = buxn_repo_get_path().ok_or_else(|| bkend_err(Path::new("."), "buxn repository not found; cannot build docker image"))?;
    println!("Building Docker buxn-linux image. {}  Be patient this can take some time.", buxn_path.display());
    let status = Command::new(&docker_path)
        .current_dir(buxn_path)
        .arg("build")
        .arg("--no-cache")
        .arg("--progress=plain")
        .arg("-t")
        .arg("buxn-linux")
        .arg(".")
        .output()
        .map_err(|e| bkend_err(Path::new("."), &format!("failed to build docker image: {e}")))?;
    if !status.status.success() {
        println!("output: {}", String::from_utf8_lossy(&status.stdout));
        println!("error: {}", String::from_utf8_lossy(&status.stderr));
        eprintln!("Failed to build buxn-linux docker image");
    } else {
        println!("Successfully built buxn-linux docker image");
    }
    Ok(())
}

pub fn ensure_buxn_repo() -> Result<Option<PathBuf>, AssemblerError> {
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
    let home_dir = dirs::home_dir().ok_or_else(|| bkend_err(Path::new("~/.uxntal/.buxn"), "failed to get home directory"))?;
    let uxntal_path = home_dir.join(".uxntal");
    let buxn_path = uxntal_path.join(".buxn");
    if !buxn_path.exists() {
        let status = Command::new("git")
            .arg("clone")
            .arg("https://github.com/davehorner/buxn.git")
            .arg(&buxn_path)
            .status()
            .map_err(|e| bkend_err(&buxn_path, &format!("failed to execute git clone: {e}")))?;
        if !status.success() {
            eprintln!("Failed to clone buxn repository");
            return Err(bkend_err(&buxn_path, "failed to clone repository"));
        }
    } else {
        // If already exists, do a git pull
        let status = Command::new("git")
            .arg("-C")
            .arg(&buxn_path)
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
    if !buxn_path.exists() {
        eprintln!("buxn repository not found after clone/pull");
        return Err(bkend_err(&buxn_path, "buxn repository not found after clone/pull"));
    }
    let _guard = DirGuard::new(&buxn_path);

    Ok(Some(buxn_path))
}

pub struct UxnBuxnBackend;
impl AssemblerBackend for UxnBuxnBackend {
    fn name(&self) -> &'static str {
        "buxn"
    }
    fn assemble(&self, tal_file: &str, _src: &str) -> Result<AssemblyOutput, AssemblerError> {
        // let input = tal_file.replace('\\', "/");
        // let input = input.replace("//?/C:", "/mnt/c"); // Handle Windows long path prefix

        let input = if tal_file.starts_with(r"\\?\") {
            // Remove Windows long path prefix
            tal_file.trim_start_matches(r"\\?\").replace('\\', "/")
        } else {
            tal_file.replace('\\', "/")
        };
        let tal_file = &input;
        let rom_path = format!("{}_{}.rom", tal_file, self.name());
        let docker_path = which::which("docker")
        .map_err(|_| bkend_err(Path::new("."), "docker not found in PATH"))?;
        let cwd_path = std::env::current_dir()?.display().to_string().replace(r"\\?\", "").replace("\\", "/").replace("c:", "C:");
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
            .arg("buxn-linux")
            .arg("/app/bin/Release/linux/buxn-asm")
            .arg(tal_file)
            .arg(&rom_path)
            .output()
            .map_err(|e| AssemblerError::Backend { message: format!("Failed to run docker buxn-asm: {e}") })?;
            println!(
                "buxn: Running docker command: docker run --rm -v {}:/src -w /src buxn-linux /app/bin/Release/linux/buxn-asm {} {}",
                cwd_path,
                tal_file,
                rom_path
            );
            println!("buxn: Arguments: {:?}", docker_cmd);
        if !docker_cmd.status.success() {
            return Err(bkend_err(
                std::path::Path::new(&tal_file),
                &format!(
                    "docker buxn-asm failed {} {tal_file} {:?} stderr: {} stdout: {}",
                    std::env::current_dir()?.display(),
                    docker_cmd.status,
                    String::from_utf8_lossy(&docker_cmd.stderr),
                    String::from_utf8_lossy(&docker_cmd.stdout)
                ),
            ));
        }
        let bytes = fs::read(&rom_path).unwrap_or_default();
        Ok(AssemblyOutput {
            rom_path: rom_path.to_string(),
            rom_bytes: bytes.clone(),
            stdout: crate::emu_uxncli::run_uxncli_get_stdout(&rom_path)?,
            disassembly: run_dis_file(&rom_path)?,
        })
    }
}