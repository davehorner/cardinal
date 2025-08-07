use std::io::Read;
use std::path::PathBuf;

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
    // If the ROM file extension is .tal, pass its path as an argument to drifblim.rom
    let mut extra_args = args.args.clone();
    if args
        .rom
        .extension()
        .map(|ext| ext == "tal")
        .unwrap_or(false)
    {
        extra_args.insert(0, args.rom.to_string_lossy().to_string());
        extra_args.insert(
            1,
            args.rom
                .with_extension("drifblim.rom")
                .to_string_lossy()
                .to_string(),
        );
        extra_args = extra_args
            .into_iter()
            .map(|s| s.replace('\\', "/"))
            .collect();
    }
    let mut rom = vec![];
    if extra_args.is_empty() {
        let mut f = std::fs::File::open(&args.rom)
            .with_context(|| format!("failed to open {:?}", args.rom))?;
        f.read_to_end(&mut rom).context("failed to read file")?;
    }
    // Embed drifblim.rom if no ROM is provided
    let rom = if rom.is_empty() {
        if !extra_args.is_empty() {
            println!("Using embedded drifblim.rom with args: {:?}", extra_args);
        }
        include_bytes!("drifblim.rom").to_vec()
    } else {
        rom
    };
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
    dev.init_args(&mut vm, &extra_args);

    // Run the reset vector
    let start = std::time::Instant::now();
    vm.run(&mut dev, 0x100);
    info!("startup complete in {:?}", start.elapsed());

    dev.output(&vm).check()?;
    dev.send_args(&mut vm, &extra_args).check()?;

    // Blocking loop, listening to the stdin reader thread
    let (tx, rx) = std::sync::mpsc::channel();
    varvara::spawn_console_worker(move |e| tx.send(e));
    while let Ok(c) = rx.recv() {
        dev.console(&mut vm, c);
        dev.output(&vm).check()?;
    }

    Ok(())
}
