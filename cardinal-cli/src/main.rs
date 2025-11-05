use std::io::Read;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

use uxn::{Backend, Uxn, UxnRam};
use varvara::Varvara;

use anyhow::{Context, Result};
use clap::Parser;
use log::info;

/// Uxn runner
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// ROM to load and execute
    rom: PathBuf,

    /// Use the native Uxn implementation
    #[clap(long)]
    native: bool,

    /// Timeout in seconds after which the program will be terminated
    #[clap(long)]
    timeout: Option<f64>,

    /// Arguments to pass into the VM
    #[arg(last = true)]
    args: Vec<String>,
}

fn main() -> Result<()> {
    let env = env_logger::Env::default()
        .filter_or("UXN_LOG", "info")
        .write_style_or("UXN_LOG", "always");
    env_logger::init_from_env(env);

    let args = Args::parse();
    let mut f =
        std::fs::File::open(&args.rom).with_context(|| format!("failed to open {:?}", args.rom))?;

    let mut rom = vec![];
    f.read_to_end(&mut rom).context("failed to read file")?;

    let mut ram = UxnRam::new();
    let mut vm = Uxn::new(
        &mut ram,
        if args.native {
            #[cfg(not(target_arch = "aarch64"))]
            anyhow::bail!("no native implementation for this arch");

            #[cfg(target_arch = "aarch64")]
            Backend::Native
        } else {
            Backend::Interpreter
        },
    );
    let mut dev = Varvara::default();
    let data = vm.reset(&rom);
    dev.reset(data);
    dev.init_args(&mut vm, &args.args);

    // Run the reset vector
    let start = std::time::Instant::now();
    vm.run(&mut dev, 0x100);
    info!("startup complete in {:?}", start.elapsed());

    dev.output(&vm).check()?;
    dev.send_args(&mut vm, &args.args).check()?;

    // Set up timeout if specified
    let timeout_reached = Arc::new(AtomicBool::new(false));
    if let Some(timeout_secs) = args.timeout {
        let timeout_flag = timeout_reached.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs_f64(timeout_secs));
            timeout_flag.store(true, Ordering::Relaxed);
        });
    }

    fn run_console_loop(
        rx: std::sync::mpsc::Receiver<u8>,
        dev: &mut Varvara,
        vm: &mut Uxn,
        timeout_reached: &Arc<AtomicBool>,
    ) -> Result<()> {
        loop {
            if timeout_reached.load(Ordering::Relaxed) {
                info!("Timeout reached, exiting");
                break;
            }
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(c) => {
                    dev.console(vm, c);
                    dev.output(vm).check()?;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    continue;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    // Input source disconnected, exit
                    break;
                }
            }
        }
        Ok(())
    }

    if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        // Terminal: spawn console worker and run loop
        let (tx, rx) = std::sync::mpsc::channel();
        varvara::spawn_console_worker(move |e| tx.send(e));
        run_console_loop(rx, &mut dev, &mut vm, &timeout_reached)?;
    } else {
        // Piped stdin: spawn console worker and run loop, then flush output and exit
        let (tx, rx) = std::sync::mpsc::channel();
        varvara::spawn_console_worker(move |e| tx.send(e));
        run_console_loop(rx, &mut dev, &mut vm, &timeout_reached)?;
        // After EOF, flush output briefly
        for _ in 0..10 {
            if timeout_reached.load(Ordering::Relaxed) {
                info!("Timeout reached, exiting");
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
            dev.output(&vm).check()?;
        }
    }
    Ok(())
}
