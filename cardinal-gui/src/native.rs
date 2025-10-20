use anyhow::{anyhow, Context};
use std::{io::Read, sync::mpsc};

use uxn::{Backend, Uxn, UxnRam};
use varvara::Varvara;

use anyhow::Result;
use eframe::egui;
use log::info;

use clap::Parser;

use crate::Stage;
use cardinal_gui::uxn::audio_setup; // Removed due to unresolved import error

/// Uxn runner
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// ROM to load and execute
    rom: std::path::PathBuf,

    /// Scale factor for the window
    #[clap(long)]
    scale: Option<f32>,

    /// Use the native assembly Uxn implementation
    #[clap(long)]
    native: bool,

    /// Make the window background fully transparent (hex RGB, e.g. ffffff for white)
    #[clap(long, value_name = "COLOR")]
    transparent: Option<String>,

    /// Disable window decorations (frameless window)
    #[clap(long)]
    no_decorations: bool,

    /// Widget mode: implies --transparent=ffffff, --no-decorations, and enables ctrl-move
    #[clap(long)]
    widget: bool,

    /// Arguments to pass into the VM
    #[arg(trailing_var_arg = true)]
    args: Vec<String>,
}

pub fn run() -> Result<()> {
    let env = env_logger::Env::default()
        .filter_or("UXN_LOG", "info")
        .write_style_or("UXN_LOG", "always");
    env_logger::init_from_env(env);

    let args = Args::parse();
    let mut f =
        std::fs::File::open(&args.rom).with_context(|| format!("failed to open {:?}", args.rom))?;

    let mut rom = vec![];
    f.read_to_end(&mut rom).context("failed to read file")?;

    let ram = UxnRam::new();
    let mut vm = Uxn::new(
        ram.leak(),
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

    let extra = vm.reset(&rom);
    dev.reset(extra);
    dev.init_args(&mut vm, &args.args);

    let _audio = audio_setup(dev.audio_streams());

    // Run the reset vector
    let start = std::time::Instant::now();
    vm.run(&mut dev, 0x100);
    info!("startup complete in {:?}", start.elapsed());

    dev.output(&vm).check()?;
    dev.send_args(&mut vm, &args.args).check()?;

    let size @ (width, height) = dev.output(&vm).size;
    let scale = args.scale.unwrap_or(if width < 320 { 2.0 } else { 1.0 });
    info!("creating window with size ({width}, {height}) and scale {scale}");
    let rom_title = args
        .rom
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    // Determine window options
    let transparent = args.widget || args.transparent.is_some();
    let transparent_color = if args.widget {
        Some("ffffff")
    } else {
        args.transparent.as_deref()
    };
    let no_decorations = args.widget || args.no_decorations;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([width as f32 * scale, height as f32 * scale])
            .with_transparent(transparent)
            .with_decorations(!no_decorations), // .with_title(Some(rom_title))
        ..Default::default()
    };

    let (tx, rx) = mpsc::channel();
    varvara::spawn_console_worker(move |c| tx.send(crate::Event::Console(c)));
    eframe::run_native(
        "Varvara",
        options,
        Box::new(move |cc| {
            Ok(Box::new(Stage::new(
                vm,
                dev,
                size,
                scale,
                rx,
                &cc.egui_ctx,
                rom_title.clone(),
                transparent_color.map(|s| s.to_string()),
            )))
        }),
    )
    .map_err(|e| anyhow!("got egui error: {e:?}"))
}
