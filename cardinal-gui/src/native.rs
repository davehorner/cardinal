use anyhow::Context;
use std::{io::Read, sync::mpsc};

use uxn::{Backend, Uxn, UxnRam};
use varvara::Varvara;

use anyhow::Result;
use eframe::egui;
use log::info;

use clap::Parser;

use crate::uxn::audio_setup;

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_args_parsing() {
        let argv = vec![
            "cardinal-gui",
            "--timeout=30",
            "--widget",
            "--ontop=true",
            "--debug",
            "--transparent=ffffff",
            "--no-decorations",
            "--scale=2.5",
            "--native",
            "--color_transform=invert",
            "--color_params=1.0,2.0,3.0",
            "test.rom",
            "extra1",
            "extra2",
        ];
        let args = Args::parse_from(argv);
        assert_eq!(args.timeout, Some(30.0));
        assert!(args.widget);
        assert_eq!(args.ontop, Some(true));
        assert!(args.debug);
        assert_eq!(args.transparent.as_deref(), Some("ffffff"));
        assert!(args.no_decorations);
        assert_eq!(args.scale, Some(2.5));
        assert!(args.native);
        assert_eq!(args.color_transform, "invert");
        assert_eq!(args.color_params, vec![1.0, 2.0, 3.0]);
        assert_eq!(args.rom, std::path::PathBuf::from("test.rom"));
        assert_eq!(args.args, vec!["extra1", "extra2"]);
    }
}

/// Uxn runner
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, disable_help_flag = true)]
struct Args {
    /// Mouse resize mode (none, middledrag)
    #[clap(long)]
    mouse_resize: Option<String>,
    /// Effect dwell time in seconds (for random effect cycling)
    #[clap(long, default_value = "3")]
    efxt: Option<f32>,
    /// Effect name (case-insensitive)
    #[clap(long)]
    efx: Option<String>,

    /// Effect mode (e.g. blend, full)
    #[clap(long)]
    efxmode: Option<String>,
    /// Timeout in seconds before the app exits
    #[clap(long, short = 't')]
    timeout: Option<f32>,
    /// ROM to load and execute
    rom: std::path::PathBuf,

    /// Scale factor for the window
    #[clap(long)]
    scale: Option<f32>,

    /// Window position X
    #[clap(long, short = 'x')]
    x: Option<i32>,
    /// Window position Y
    #[clap(long, short = 'y')]
    y: Option<i32>,
    /// Window width
    #[clap(long, short = 'w')]
    w: Option<u32>,
    /// Window height
    #[clap(long, short = 'h')]
    h: Option<u32>,

    /// Fit mode for ROM display (none, contain, cover, stretch)
    #[clap(long)]
    fit: Option<String>,

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

    /// Make the window always on top (true/false). If not set, widget implies ontop.
    #[clap(long)]
    ontop: Option<bool>,

    /// Show debug console
    #[clap(long, short = 'd')]
    debug: bool,

    /// Mouse drag mode for window movement (none, doubledrag)
    #[clap(long)]
    mouse: Option<String>,

    /// Arguments to pass into the VM
    #[arg(trailing_var_arg = true)]
    args: Vec<String>,

    /// Color transform type (e.g. shift, invert, grayscale)
    #[clap(long, alias = "color_transform", default_value = "none")]
    color_transform: String,

    /// Parameters for color transform (comma or space separated)
    #[clap(long, alias = "color_params", value_delimiter = ',')]
    color_params: Vec<f32>,
}

pub fn run() -> Result<()> {
    let args = Args::parse();
    // Determine mouse resize mode
    let mouse_resize = if let Some(ref resize_str) = args.mouse_resize {
        match resize_str.to_lowercase().as_str() {
            "none" => crate::stage::MouseResizeMode::None,
            "middledrag" => crate::stage::MouseResizeMode::MiddleDrag,
            _ => crate::stage::MouseResizeMode::None,
        }
    } else if args.widget {
        crate::stage::MouseResizeMode::MiddleDrag
    } else {
        crate::stage::MouseResizeMode::None
    };
    let env = env_logger::Env::default()
        .filter_or("UXN_LOG", "info")
        .write_style_or("UXN_LOG", "always");
    env_logger::init_from_env(env);

    let args = Args::parse();
    println!(
        "[cardinal-gui] CLI args: {:?}",
        std::env::args().collect::<Vec<_>>()
    );
    println!("[cardinal-gui] Parsed Args struct: {:?}", args);
    println!(
        "[cardinal-gui] Requested window size: w={:?}, h={:?}, scale={:?}, fit_mode={:?}",
        args.w, args.h, args.scale, args.fit
    );

    // Determine mouse drag mode
    let mouse_mode = if let Some(ref mouse_str) = args.mouse {
        match mouse_str.to_lowercase().as_str() {
            "none" => crate::stage::MouseDragMode::None,
            "doubledrag" => crate::stage::MouseDragMode::DoubleDrag,
            // Only none and doubledrag are supported
            _ => crate::stage::MouseDragMode::None,
        }
    } else if args.widget {
        crate::stage::MouseDragMode::DoubleDrag
    } else {
        crate::stage::MouseDragMode::None
    };
    println!("[cardinal-gui] Mouse drag mode: {:?}", mouse_mode);
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

    // // Hide the console after ROM is loaded, unless --debug is present
    // #[cfg(windows)]
    // {
    //     if !args.debug {
    //         unsafe { winapi::um::wincon::FreeConsole(); }
    //     }
    // }

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
    let transparent_color = if let Some(color) = args.transparent.as_deref() {
        Some(color)
    } else if args.widget {
        Some("ffffff")
    } else {
        None
    };
    let no_decorations = args.widget || args.no_decorations;

    // Window position and size from CLI only
    let window_x = args.x.unwrap_or(0);
    let window_y = args.y.unwrap_or(0);
    let window_w = if args.w.is_some() {
        args.w.unwrap()
    } else {
        (width as f32 * scale) as u32
    };
    let window_h = if args.h.is_some() {
        args.h.unwrap()
    } else {
        (height as f32 * scale) as u32
    };
    let fit_mode = args.fit.clone().unwrap_or_else(|| "contain".to_string());
    println!(
        "[cardinal-gui] Requested window size: w={}, h={}, scale={}, fit_mode={}",
        window_w, window_h, scale, fit_mode
    );

    let mut viewport_builder = egui::ViewportBuilder::default()
        .with_inner_size([window_w as f32, window_h as f32])
        .with_position([window_x as f32, window_y as f32])
        .with_transparent(transparent)
        .with_decorations(!no_decorations);
    let ontop = match args.ontop {
        Some(val) => val,
        None => args.widget,
    };
    if ontop {
        viewport_builder = viewport_builder.with_always_on_top();
    }
    let options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };

    let (tx, rx) = mpsc::channel();
    varvara::spawn_console_worker(move |c| tx.send(crate::stage::Event::Console(c)));
    let color_transform = args.color_transform.clone();
    let color_params = args.color_params.clone();
    let efx = args.efx.clone();
    let efxt = args.efxt.unwrap_or(3.0);
    let efxmode = args.efxmode.clone();
    let fit_mode = args.fit.clone().unwrap_or_else(|| "contain".to_string());
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    let should_exit = Arc::new(AtomicBool::new(false));
    let should_exit_clone = should_exit.clone();
    let timeout_opt = args.timeout;
    let run_result = eframe::run_native(
        "Varvara",
        options,
        Box::new(move |cc| {
            if let Some(timeout_secs) = timeout_opt {
                println!("[cardinal-gui] Timeout set: {} seconds", timeout_secs);
                let ctx = cc.egui_ctx.clone();
                let should_exit = should_exit_clone.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs_f32(timeout_secs));
                    should_exit.store(true, Ordering::Relaxed);
                    ctx.request_repaint();
                });
            }
            Ok(Box::new(crate::stage::Stage::new(
                vm,
                dev,
                rx,
                crate::stage::StageConfig {
                    size,
                    scale,
                    rom_title: rom_title.clone(),
                    transparent: transparent_color.map(|s| s.to_string()),
                    color_transform_name: color_transform,
                    color_params,
                    effects: crate::stage::EffectsConfig {
                        effect_mode: 0,
                        effect_order: (0..crate::effects::EFFECT_COUNT).collect(),
                        efx,
                        efxt,
                        efxmode,
                        efx_ndx: 0,
                        last_effect_switch: 0.0,
                    },
                    fit_mode,
                    mouse_mode,
                    mouse_resize,
                },
                Some(should_exit_clone.clone()),
            )))
        }),
    );
    run_result.map_err(|e| anyhow::anyhow!("got egui error: {e:?}"))?;
    // After eframe exits, check if we should exit due to timeout
    if should_exit.load(std::sync::atomic::Ordering::Relaxed) {
        std::process::exit(0);
    }
    Ok(())
}
