#![allow(unused_imports)]
use cardinal_gui::uxn::audio_setup;
use cardinal_gui::uxn::InjectEvent;
#[cfg(not(target_arch = "wasm32"))]
use eframe::NativeOptions;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Url;
#[cfg(target_arch = "wasm32")]
use reqwest_wasm::Client;
#[cfg(target_arch = "wasm32")]
use reqwest_wasm::Url;
use std::thread;
use varvara::Key;

/// Injects a string as Orca events, simulating character entry with right/left/down arrows
#[allow(dead_code)]
fn build_orca_inject_queue_from_chars(
    chars: &str,
) -> std::collections::VecDeque<InjectEvent> {
    use std::collections::VecDeque;
    let mut queue = VecDeque::new();
    #[allow(dead_code)]
    const CTRL_H: Key = Key::Ctrl;
    const RIGHT: Key = Key::Right;
    #[allow(dead_code)]
    const LEFT: Key = Key::Left;
    const DOWN: Key = Key::Down;
    // Send Ctrl + . four times instead of Ctrl+H
    const CTRL: Key = Key::Ctrl;
    const COMMA: Key = Key::Char(b',');

    // Inject Ctrl+Comma as a chord 7 times, then release both
    for _ in 0..7 {
        queue.push_back(InjectEvent::Chord(vec![CTRL, COMMA]));
        queue.push_back(InjectEvent::KeyRelease(COMMA));
        queue.push_back(InjectEvent::KeyRelease(CTRL));
    }
    // Inject each character, move right after each
    let mut arrow_count = 0;
    for ch in chars.chars() {
        let byte = ch as u8;
        queue.push_back(InjectEvent::Char(byte));
        queue.push_back(InjectEvent::KeyPress(RIGHT));
        queue.push_back(InjectEvent::KeyRelease(RIGHT));
        arrow_count += 1;
    }

    // After all chars, send LEFT_ARROW 'arrow_count' times to return to column 0
    for _ in 0..arrow_count {
        queue.push_back(InjectEvent::KeyPress(RIGHT));
        queue.push_back(InjectEvent::KeyRelease(RIGHT));
    }
    // Send DOWN_ARROW to move to next line
    queue.push_back(InjectEvent::KeyPress(DOWN));
    queue.push_back(InjectEvent::KeyRelease(DOWN));
    queue.push_back(InjectEvent::KeyPress(DOWN));
    queue.push_back(InjectEvent::KeyRelease(DOWN));

    queue
}
#[allow(dead_code)]
/// Build an InjectEvent queue for orca file injection with rectangle and efficient movement
fn build_orca_inject_queue(
    file_path: &str,
) -> std::collections::VecDeque<InjectEvent> {
    use std::collections::VecDeque;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    let mut queue = VecDeque::new();
    const CTRL_H: Key = Key::Ctrl;
    const RIGHT: Key = Key::Right;
    #[allow(dead_code)]
    const LEFT: Key = Key::Left;
    const UP: Key = Key::Up;
    const DOWN: Key = Key::Down;
    // Read file into lines
    let mut lines: Vec<Vec<char>> = Vec::new();
    let mut max_len = 0;
    if let Ok(file) = File::open(file_path) {
        let reader = BufReader::new(file);
        for line_result in reader.lines() {
            match line_result {
                Ok(line) => {
                    let chars: Vec<char> = line.chars().collect();
                    max_len = max_len.max(chars.len());
                    lines.push(chars);
                }
                Err(e) => {
                    eprintln!("Error reading line from file: {e}");
                    break;
                }
            }
        }
    }
    let rows = lines.len();
    let cols = max_len;
    // Build rectangle with '/' border
    let mut grid = vec![vec![' '; cols + 2]; rows + 2];
    // Fill top and bottom borders
    for c in 0..cols + 2 {
        grid[0][c] = '/';
        grid[rows + 1][c] = '/';
    }
    // Fill left and right borders with '/' (actual border logic handled in event queue below)
    // Fill file contents
    for (i, line) in lines.iter().enumerate() {
        for (j, &ch) in line.iter().enumerate() {
            grid[i + 1][j + 1] = ch;
        }
    }
    // Start at (1,1)
    let mut cur_row = 1;
    let mut cur_col = 1;
    queue.push_back(InjectEvent::KeyPress(CTRL_H));
    queue.push_back(InjectEvent::KeyRelease(CTRL_H));
    // Visit all non '.' cells efficiently
    let mut visited = vec![vec![false; cols + 2]; rows + 2];
    for r in 0..rows + 2 {
        for c in 0..cols + 2 {
            if grid[r][c] != '.' && !visited[r][c] {
                // Move to (r,c)
                let dr = r as isize - cur_row as isize;
                let dc = c as isize - cur_col as isize;
                for _ in 0..dr.abs() {
                    queue.push_back(if dr > 0 {
                        InjectEvent::KeyPress(DOWN)
                    } else {
                        InjectEvent::KeyPress(UP)
                    });
                    queue.push_back(if dr > 0 {
                        InjectEvent::KeyRelease(DOWN)
                    } else {
                        InjectEvent::KeyRelease(UP)
                    });
                }
                for _ in 0..dc.abs() {
                    queue.push_back(if dc > 0 {
                        InjectEvent::KeyPress(RIGHT)
                    } else {
                        InjectEvent::KeyPress(LEFT)
                    });
                    queue.push_back(if dc > 0 {
                        InjectEvent::KeyRelease(RIGHT)
                    } else {
                        InjectEvent::KeyRelease(LEFT)
                    });
                }
                cur_row = r;
                cur_col = c;
                // Print char
                if r == 0 || r == rows + 1 {
                    // Top or bottom border: just '/'
                    if grid[r][c] == '/' {
                        queue.push_back(InjectEvent::Char(b'/'));
                    } else {
                        queue.push_back(InjectEvent::Char(grid[r][c] as u8)); // char to u8: only safe for ASCII
                    }
                } else if c == 0 {
                    // Left border: '/' then row and col as two hex digits each
                    queue.push_back(InjectEvent::Char(b'/'));
                    queue.push_back(InjectEvent::KeyPress(RIGHT));
                    let hex = format!("{r:01X}{c:01X}");
                    for b in hex.bytes() {
                        queue.push_back(InjectEvent::Char(b));

                        queue.push_back(InjectEvent::KeyRelease(RIGHT));
                    }
                } else if c == cols + 1 {
                    // Right border: '/' then row and col as two hex digits each
                    queue.push_back(InjectEvent::Char(b'/'));
                    queue.push_back(InjectEvent::KeyPress(RIGHT));
                    let hex = format!("{r:01X}{c:01X}");
                    for b in hex.bytes() {
                        queue.push_back(InjectEvent::Char(b));
                        queue.push_back(InjectEvent::KeyRelease(RIGHT));
                    }
                    // After right border, return to start of next row
                    for _ in 0..(cols + 1) {
                        queue.push_back(InjectEvent::KeyPress(LEFT));
                        queue.push_back(InjectEvent::KeyRelease(LEFT));
                    }
                    queue.push_back(InjectEvent::KeyPress(DOWN));
                } else {
                    // File contents
                    queue.push_back(InjectEvent::Char(grid[r][c] as u8)); // char to u8: only safe for ASCII
                }
                visited[r][c] = true;
            }
        }
    }
    queue
}
// cardinal-demo.rs - Uxn GUI runner for crate, with ROM selection and download

use cardinal_gui::uxn::{UxnApp, UxnModule};
#[cfg(not(target_arch = "wasm32"))]
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
#[cfg(not(target_arch = "wasm32"))]
use reqwest::blocking::Client;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

#[cfg(feature = "uses_e_midi")]
use cardinal_gui::e_midi::MidiPlayerThread;

#[allow(dead_code)]
struct AppState {
    // ...existing fields...
    #[cfg(feature = "uses_e_midi")]
    midi_thread: Option<MidiPlayerThread>,
    #[cfg(feature = "uses_e_midi")]
    should_exit: Arc<Mutex<bool>>,
}

fn egui_close_requested(ctx: &egui::Context) -> bool {
    ctx.input(|i| i.viewport().close_requested())
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut rom_path: Option<PathBuf> = None;
    let mut scale: f32 = 2.0;
    let mut window_size = (640, 480);
    let mut title = "crate Uxn".to_string();
    let mut window_mode = "free".to_string(); // static, free, proportional

    // Parse CLI args (simple version, extend as needed)
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--rom" => {
                if i + 1 < args.len() {
                    rom_path = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--scale" => {
                if i + 1 < args.len() {
                    scale = args[i + 1].parse().unwrap_or(scale);
                    i += 1;
                }
            }
            "--width" => {
                if i + 1 < args.len() {
                    window_size.0 =
                        args[i + 1].parse().unwrap_or(window_size.0);
                    i += 1;
                }
            }
            "--height" => {
                if i + 1 < args.len() {
                    window_size.1 =
                        args[i + 1].parse().unwrap_or(window_size.1);
                    i += 1;
                }
            }
            "--title" => {
                if i + 1 < args.len() {
                    title = args[i + 1].clone();
                    i += 1;
                }
            }
            "--window-mode" => {
                if i + 1 < args.len() {
                    window_mode = args[i + 1].to_lowercase();
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    // --- Static ROM URLs ---
    let static_rom_urls = vec![
        (
            "orca-toy/orca.rom",
            "https://rabbits.srht.site/orca-toy/orca.rom",
        ),
        // (
        //     "orca-toy/bin/orca.rom",
        //     "file:///C:/w/music/orca-toy/bin/orca.rom",
        // ),
        // (
        //     "write_to_file.rom",
        //     "file:///C:/w/music/orca-toy/bin/write_to_file.rom",
        // ),
        ("potato.rom", "https://rabbits.srht.site/potato/potato.rom"),
        //("uxn.rom", "https://rabbits.srht.site/uxn/uxn.rom"),
        ("oekaki.rom", "https://rabbits.srht.site/oekaki/oekaki.rom"),
        ("flick.rom", "https://rabbits.srht.site/flick/flick.rom"),
        //("adelie.rom", "https://rabbits.srht.site/adelie/adelie.rom"),
        //("nasu.rom", "https://hundredrabbits.itch.io/nasu"),
        // ("noodle.rom", "https://hundredrabbits.itch.io/noodle"),
        // ("left.rom", "http://hundredrabbits.itch.io/Left")
        // Add more static ROMs here: ("label", "url")
    ];

    // Download static ROMs and collect their names and paths
    let mut static_rom_names = Vec::new();
    let mut static_rom_paths = Vec::new();
    for (label, url) in &static_rom_urls {
        let path = download_static_rom(label, url)?;
        static_rom_names.push(label.to_string());
        static_rom_paths.push(path.clone());
    }

    // If no ROM is selected, fetch ROM list and prompt user
    let mut auto_rom_select = false;
    let mut selected_rom_label: Option<String> = None;
    if rom_path.is_none() {
        let mut roms = static_rom_names.clone();
        let github_roms = fetch_rom_list()?;
        roms.extend(github_roms.iter().cloned());

        // let cwd_roms = std::env::current_dir()
        //     .ok()
        //     .map(|mut dir| {
        //         dir.push("roms");
        //         dir
        //     })
        //     .filter(|dir| dir.is_dir());

        // if let Some(roms_dir) = cwd_roms {
        //     if let Ok(entries) = std::fs::read_dir(&roms_dir) {
        //         for entry in entries.flatten() {
        //             let path = entry.path();
        //             if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        //                 if ext.eq_ignore_ascii_case("rom") {
        //                     if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        //                         roms.push(name.to_string());
        //                         static_rom_paths.push(path.clone());
        //                         static_rom_names.push(name.to_string());
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // }2

        let selected = prompt_rom_selection(&roms)?;
        if selected == "__AUTO__" {
            // User hit return: enable auto ROM cycling
            auto_rom_select = true;
        } else {
            let idx = roms.iter().position(|r| r == &selected).unwrap_or(0);
            selected_rom_label = Some(roms[idx].clone());
            if idx < static_rom_paths.len() {
                rom_path = Some(static_rom_paths[idx].clone());
                title = format!("cardinal-demo - {}", roms[idx]);
            } else {
                let github_idx = idx - static_rom_paths.len();
                let rom_file = download_rom(&github_roms[github_idx])?;
                println!("[DEBUG] Downloaded ROM: {}", rom_file.display());
                let _bytes = std::fs::read(&rom_file)?;
                // Try to find a .sym file for this ROM in github_roms
                let rom_name = &github_roms[github_idx];
                let sym_name = if rom_name.ends_with(".rom") {
                    let base = rom_name.trim_end_matches(".rom");
                    // Look for base.rom.sym or base.sym in github_roms
                    let sym1 = format!("{base}.rom.sym");
                    let sym2 = format!("{base}.sym");
                    github_roms
                        .iter()
                        .find(|n| *n == &sym1 || *n == &sym2)
                        .cloned()
                } else {
                    None
                };
                if let Some(sym_file) = sym_name {
                    if let Ok(sym_path) = download_sym(&sym_file) {
                        let sym = std::fs::read(&sym_path)?;
                        // Write the .sym to a temp file with the same base as the ROM temp file, adding ".sym" extension
                        let sym_path = rom_file.clone();
                        let sym_path = sym_path.with_extension(format!(
                            "{}.sym",
                            sym_path
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("")
                        ));
                        std::fs::write(&sym_path, &sym)
                            .map_err(|e| e.to_string())?;
                        println!(
                            "[DEBUG] Downloaded .sym file: {}",
                            sym_path.display()
                        );
                    }
                }
                title = format!("cardinal-demo - {}", github_roms[github_idx]);
                rom_path = Some(rom_file);
            }
        }
    } else if let Some(path) = &rom_path {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            selected_rom_label = Some(name.to_string());
            title = format!("cardinal-demo - {name}");
        }
    }

    // Build all_roms and all_labels if needed (for auto ROM cycling)
    let mut all_roms: Vec<varvara::RomData> = Vec::new();
    let mut all_labels: Vec<String> = Vec::new();
    if auto_rom_select {
        // Add static ROMs first
        for (i, path) in static_rom_paths.iter().enumerate() {
            let bytes = std::fs::read(path)?;
            println!("[DEBUG] Loaded static ROM: {}", path.display());

            all_roms.push(varvara::RomData {
                rom: bytes,
                sym: None,
            });
            all_labels.push(static_rom_names[i].clone());
        }
        // Then add GitHub ROMs
        let github_roms = fetch_rom_list()?;
        for name in &github_roms {
            let rom_file = download_rom(name)?;
            let bytes = std::fs::read(&rom_file)?;
            println!("[DEBUG] Downloaded ROM: {}", rom_file.display());
            if let Ok(_sym) = download_sym(name) {
                let sym = std::fs::read(&rom_file)?;
                all_roms.push(varvara::RomData {
                    rom: bytes,
                    sym: Some(sym),
                });
            } else {
                all_roms.push(varvara::RomData {
                    rom: bytes,
                    sym: None,
                });
            }
            all_labels.push(name.clone());
        }
    }

    // --- Helper to download static ROMs by URL ---
    fn download_static_rom(
        _label: &str,
        url: &str,
    ) -> Result<std::path::PathBuf, String> {
        if let Some(stripped) = url.strip_prefix("file://") {
            println!("[DEBUG] Using local file path: {stripped}");
            // Use the local file path directly
            #[cfg(windows)]
            let mut path_str = stripped;
            // On Windows, strip leading slash if path is like /C:/...
            #[cfg(windows)]
            {
                if path_str.len() > 2
                    && path_str.as_bytes()[0] == b'/'
                    && path_str.as_bytes()[2] == b':'
                {
                    path_str = &path_str[1..];
                }
            }
            #[cfg(not(windows))]
            let path_str = stripped;
            let path = std::path::PathBuf::from(path_str);
            if path.exists() {
                Ok(path)
            } else {
                Err(format!("Local file not found: {}", path.display()))
            }
        } else {
            println!("[DEBUG] Downloading static ROM from: {url}");

            #[cfg(target_arch = "wasm32")]
            let client = reqwest_wasm::Client::builder()
                .user_agent("cardinal-demo")
                .build()
                .map_err(|e| e.to_string())?;
            #[cfg(not(target_arch = "wasm32"))]
            let client = reqwest::blocking::Client::builder()
                .user_agent("cardinal-demo")
                .build()
                .map_err(|e| e.to_string())?;

            let resp = client.get(url).send().map_err(|e| e.to_string())?;
            let bytes = resp.bytes().map_err(|e| e.to_string())?;
            // let mut file =
            //     tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
            //     let label_filename = _label.rsplit('/').next().unwrap_or(_label);
            let label_filename = _label.rsplit('/').next().unwrap_or(_label);
            let mut file = tempfile::Builder::new()
                .prefix(label_filename)
                .suffix("")
                .rand_bytes(3)
                .tempfile_in(".")
                .expect("Failed to create tempfile for static ROM");

            file.write_all(&bytes).map_err(|e| e.to_string())?;
            let path =
                file.into_temp_path().keep().map_err(|e| e.to_string())?;
            Ok(path)
        }
    }

    // Create UxnModule
    let rom_path_arc = Arc::new(Mutex::new(rom_path.clone()));
    let mut uxn_mod = UxnModule::new(rom_path.as_deref())?;
    // --- ROM File Watcher ---
    let (reload_tx, reload_rx) = std::sync::mpsc::channel();
    if let Some(ref rom_path) = rom_path {
        let rom_path_buf = rom_path.clone();
        thread::spawn(move || {
            let mut watcher = RecommendedWatcher::new(
                move |res: Result<Event, notify::Error>| {
                    if let Ok(event) = res {
                        if event.kind.is_modify() {
                            reload_tx.send(()).ok();
                            println!("[ROM WATCHER] Detected change in ROM file: {:?}", event.paths);
                        }
                    }
                },
                Config::default(),
            ).expect("Failed to create watcher");
            watcher
                .watch(&rom_path_buf, RecursiveMode::NonRecursive)
                .expect("Failed to watch ROM file");
            // Keep the watcher alive
            loop {
                std::thread::sleep(Duration::from_secs(60));
            }
        });
    }
    // // Add reload_rom method to UxnModule if not present
    // trait ReloadableUxnModule {
    //     fn reload_rom(&mut self, path: &std::path::Path) -> Result<(), String>;
    // }

    // impl ReloadableUxnModule for UxnModule {
    //     fn reload_rom(&mut self, path: &std::path::Path) -> Result<(), String> {
    //         // Replace the ROM in the VM
    //         // If load_rom expects a path, use it directly
    //         self.load_rom(path).map_err(|e| e.to_string())?;
    //         // Beep twice on reload
    //         beep();
    //         beep();
    //         Ok(())
    //     }
    // }

    // Set up event channel for UxnApp
    let (_event_tx, event_rx) = mpsc::channel();

    let vm = Arc::clone(&uxn_mod.uxn);
    let mut vm = vm.lock().unwrap();
    let mut dev = uxn_mod.varvara.take().expect("Varvara device missing");
    let _audio = audio_setup(dev.audio_streams());

    // --- Load .sym file if it exists next to the ROM ---
    println!("[DEBUG] Checking for .sym file next to ROM...");
    if let Some(ref rom_path) = rom_path {
        let mut sym_path = rom_path.clone();
        sym_path.set_extension("sym");
        println!("[DEBUG] Looking for .sym file at: {}", sym_path.display());
        if sym_path.exists() {
            match std::fs::read_to_string(&sym_path) {
                Ok(_sym_contents) => {
                    if let Some(ref mut varvara) = uxn_mod.varvara {
                        let _ = varvara
                            .load_symbols_into_self(sym_path.to_str().unwrap());
                        println!("[DEBUG] Loaded symbols from {sym_path:?}");
                    }
                }
                Err(e) => {
                    eprintln!("[DEBUG] Failed to read .sym file: {e}");
                }
            }
        }
    }

    // Use selected_rom_label for matching, not temp file name
    println!("[DEBUG] selected_rom_label: {selected_rom_label:?}");
    if let Some(label) = &selected_rom_label {
        if label.contains("orca.rom") {
            // println!("[DEBUG] ROM matched orca.rom by label: {}", label);
            // let dir_path = r"C:\w\music\Orca-c\examples\basics";
            // let entries = fs::read_dir(dir_path)
            //     .map_err(|e| format!("Failed to read directory: {}", e))
            //     .and_then(|read_dir| {
            //     let files: Vec<_> = read_dir
            //         .filter_map(|entry| {
            //         entry.ok().and_then(|e| {
            //             let path = e.path();
            //             if path.extension().and_then(|ext| ext.to_str()) == Some("orca") {
            //             Some(path)
            //             } else {
            //             None
            //             }
            //         })
            //         })
            //         .collect();
            //     if files.is_empty() {
            //         Err("No .orca files found".to_string())
            //     } else {
            //         Ok(files)
            //     }
            //     });

            // match entries {
            //     Ok(files) => {
            //     let mut rng = rand::thread_rng();
            //     if let Some(random_file) = files.choose(&mut rng) {
            //         println!("[DEBUG] Detected orca.rom, sending {:?} to console...", random_file);
            //         match send_orca_file_to_console(&mut dev, &mut vm, random_file.to_str().unwrap()) {
            //         Ok(_) => println!("[DEBUG] {:?} sent to console successfully.", random_file),
            //         Err(e) => eprintln!("Failed to send file: {}", e),
            //         }
            //     }
            //     }
            //     Err(e) => {
            //     eprintln!("[DEBUG] Could not select random .orca file: {}", e);
            //     }
            // }
        } else {
            println!("[DEBUG] ROM label did not match orca.rom: {label}");
        }
    } else {
        println!("[DEBUG] selected_rom_label is None");
    }
    #[cfg(windows)]
    fn beep() {
        unsafe {
            winapi::um::winuser::MessageBeep(0xFFFFFFFF);
        }
    }

    #[cfg(not(windows))]
    fn beep() {
        print!("\x07");
    }
    // Register listeners for console output
    dev.console.register_stdout_listener(|byte| {
        eprint!("{byte:02X} ");
        eprintln!();
        beep();
    });
    dev.console.register_stderr_listener(|byte| {
        println!("Console stderr: {byte}");
    });

    dev.audio(&mut vm);
    let size = dev.output(&vm).size;
    drop(vm); // Release lock

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([
            (window_size.0 as f32 * scale),
            (window_size.1 as f32 * scale),
        ])
        .with_title(&title);

    match window_mode.as_str() {
        "static" => {
            viewport = viewport.with_resizable(false);
        }
        _ => {
            viewport = viewport.with_resizable(true);
        }
    }

    let options = NativeOptions {
        viewport,
        ..Default::default()
    };

    // If auto_rom_select, pass all_roms and set auto_rom_select flag, else pass empty vec and false
    let app_all_roms = if auto_rom_select {
        all_roms.clone()
    } else {
        Vec::new()
    };
    let app_auto_rom_select = auto_rom_select;

    // Start the MIDI player thread when the GUI starts
    #[cfg(feature = "uses_e_midi")]
    let midi_thread = Some(MidiPlayerThread::start());


    let result = eframe::run_native(
        &title,
        options,
        Box::new(move |cc| {
            let ctx = &cc.egui_ctx;
            let vm = Arc::clone(&uxn_mod.uxn);
            let mut vm = vm.lock().unwrap();
            static mut RAM: [u8; 65536] = [0; 65536];
            #[allow(static_mut_refs)]
            let ram: &'static mut [u8; 65536] = unsafe { &mut RAM };
            let new_uxn = uxn::Uxn::new(ram, uxn::Backend::Interpreter);
            let mut app = UxnApp::new_with_mode(
                std::mem::replace(&mut *vm, new_uxn),
                dev,
                size,
                scale,
                event_rx,
                ctx,
                window_mode.clone(),
                app_all_roms,
                if app_auto_rom_select {
                    all_labels.clone()
                } else {
                    Vec::new()
                },
                app_auto_rom_select,
                reload_rx,
                rom_path_arc,
            );
            if app_auto_rom_select {
                let ctx = cc.egui_ctx.clone();
                app.set_on_rom_change(move |rom_name| {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                        format!("cardinal-demo - {rom_name}"),
                    ));
                });
            }

            // --- Orca file injection after UI is ready ---
            // let mut pending_orca_inject = false;
            // let orca_dir = r"C:\w\music\Orca-c\examples\basics";
            // let orca_path = format!("{}/k.orca", orca_dir);
            // if let Some(label) = &selected_rom_label {
            //     if label.contains("orca.rom") {
            //         pending_orca_inject = true;
            //     }
            // }
            // if pending_orca_inject {
            //     let orca_path = orca_path.clone();
            //     app.set_on_first_update(Box::new(move |app_ref: &mut UxnApp| {
            //         let queue = build_orca_inject_queue(&orca_path);
            //         app_ref.queue_input(queue);
            //     }));
            // }

            // --- Inject HornersOrca file if selected ROM is orca.rom ---
            if let Some(label) = &selected_rom_label {
                if label.contains("orca.rom") {
                    let horners_orca_chars = "horners.orca";
                    let queue =
                        build_orca_inject_queue_from_chars(horners_orca_chars);
                    app.queue_input(queue);
                }
            }

            // Wrap the app in a struct that checks for close and signals MIDI thread shutdown
            struct AppWithClose<A> {
                inner: A,
                #[cfg(feature = "uses_e_midi")]
                midi_thread: Option<MidiPlayerThread>,
            }

            impl<A: eframe::App> eframe::App for AppWithClose<A> {
                fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
                    self.inner.update(ctx, frame);
                    println!("[DEBUG] AppWithClose update called");
                    #[cfg(feature = "uses_e_midi")]
                    if egui_close_requested(ctx) {
                        if let Some(thread) = self.midi_thread.take() {
                            thread.shutdown();
                        }
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
                // ...other trait methods can delegate to self.inner...
            }

            Ok(Box::new(AppWithClose {
                inner: app,
                #[cfg(feature = "uses_e_midi")]
                midi_thread,
            }) as Box<dyn eframe::App>)
        }),
    );

    result?;
    Ok(())
}

/// Fetch the list of ROMs from the GitHub directory listing
#[cfg(not(target_arch = "wasm32"))]
fn fetch_rom_list() -> Result<Vec<String>, String> {
    let url = "https://api.github.com/repos/davehorner/cardinal/contents/roms";
    println!("[DEBUG] Fetching ROM list from: {url}");
    let client = Client::builder()
        .user_agent("cardinal-demo")
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get(url)
        .send()
        .map_err(|e| e.to_string())?
        .json::<serde_json::Value>()
        .map_err(|e| e.to_string())?;
    let mut roms = Vec::new();
    if let Some(arr) = resp.as_array() {
        for entry in arr {
            if let Some(name) = entry.get("name").and_then(|n| n.as_str()) {
                if name.ends_with(".rom") || name.ends_with(".sym") {
                    roms.push(name.to_string());
                }
            }
        }
    }
    Ok(roms)
}

/// Prompt the user to select a ROM (simple CLI prompt)
#[allow(dead_code)]
fn prompt_rom_selection(roms: &[String]) -> Result<String, String> {
    println!("Available ROMs: (hit return for auto ROM cycling)");
    for (i, rom) in roms.iter().enumerate() {
        println!("  [{i}] {rom}", i = i + 1, rom = rom);
    }
    println!(
        "  [Return] Enable AUTO ROM CYCLING mode (cycle all ROMs every 10s)"
    );
    print!("Select a ROM by number, or hit return for auto cycling: ");
    let _ = std::io::stdout().flush();
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
    let trimmed = input.trim();
    if trimmed.is_empty() {
        println!(
            "[AUTO ROM CYCLING] You selected auto mode. All ROMs will cycle every 10 seconds."
        );
        return Ok("__AUTO__".to_string());
    }
    let idx: usize = trimmed.parse().unwrap_or(1);
    let idx = idx.saturating_sub(1).min(roms.len().saturating_sub(1));
    Ok(roms[idx].clone())
}

/// Download the selected ROM to a temp file and return its path
#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
fn download_rom(_rom_name: &str) -> Result<PathBuf, String> {
    Err("ROM download not supported on wasm32 targets.".to_string())
}
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
fn download_rom(rom_name: &str) -> Result<PathBuf, String> {
    let url = format!(
        "https://raw.githubusercontent.com/davehorner/cardinal/main/roms/{rom_name}");

    let client = Client::builder()
        .user_agent("cardinal-demo")
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get(Url::parse(&url).map_err(|e| e.to_string())?)
        .send()
        .map_err(|e| e.to_string())?;
    let bytes = resp.bytes().map_err(|e| e.to_string())?;
    let label_filename = rom_name.rsplit('/').next().unwrap_or(rom_name);
    let _file = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
    let mut file = tempfile::Builder::new()
        .prefix(label_filename)
        .suffix(".rom")
        .rand_bytes(6)
        .tempfile()
        .map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;
    let path = file.into_temp_path().keep().map_err(|e| e.to_string())?;
    Ok(path)
}

fn download_sym(sym_name: &str) -> Result<PathBuf, String> {
    let url = format!(
        "https://raw.githubusercontent.com/davehorner/cardinal/main/roms/{sym_name}");
    println!("[DEBUG] Downloading .sym file from: {url}");
    #[cfg(target_arch = "wasm32")]
    use reqwest_wasm::Client;

    #[cfg(target_arch = "wasm32")]
    let body = reqwest_wasm::blocking::get(url)?.text()?;
    #[cfg(target_arch = "wasm32")]
    let bytes = body.as_bytes();

    #[cfg(not(target_arch = "wasm32"))]
    let client = reqwest::blocking::Client::builder()
        .user_agent("cardinal-demo")
        .build()
        .map_err(|e| e.to_string())?;
    #[cfg(not(target_arch = "wasm32"))]
    let resp = client
        .get(Url::parse(&url).map_err(|e| e.to_string())?)
        .send()
        .map_err(|e| e.to_string())?;
    #[cfg(not(target_arch = "wasm32"))]
    let bytes = resp.bytes().map_err(|e| e.to_string())?;
    let mut file = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;
    let path = file.into_temp_path().keep().map_err(|e| e.to_string())?;
    Ok(path)
}

#[allow(dead_code)]
/// Send an orca file to the VM console, simulating character entry with right/left/down arrows
fn send_orca_file_to_console(
    dev: &mut varvara::Varvara,
    vm: &mut uxn::Uxn,
    file_path: &str,
) -> Result<(), String> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    // Key codes (may need adjustment for your VM)
    // const CTRL_H: u8 = 0x08; // Ctrl+H (backspace, often used for home)
    const RIGHT_ARROW: u8 = 0x1B; // Example: ESC for right arrow (replace with actual code)
    const LEFT_ARROW: u8 = 0x1A; // Example: SUB for left arrow (replace with actual code)
    const DOWN_ARROW: u8 = 0x0A; // LF for down arrow (replace with actual code)

    // Send Ctrl+H to start
    // dev.console(vm, CTRL_H);
    // println!("[DEBUG] Sent Ctrl+H to console");

    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);

    for line_res in reader.lines() {
        let line = line_res.map_err(|e| e.to_string())?;
        let mut arrow_count = 0;
        for ch in line.chars() {
            let byte = ch as u8;
            dev.console(vm, byte);
            dev.console(vm, RIGHT_ARROW);
            arrow_count += 1;
        }
        for _ in 0..arrow_count {
            dev.console(vm, LEFT_ARROW);
        }
        dev.console(vm, DOWN_ARROW);
    }
    Ok(())
}
// ...removed UxnEguiApp, now using UxnApp from uxn.rs...

#[cfg(target_arch = "wasm32")]
fn main() {
    eprintln!("cardinal-demo does not support wasm32 targets.");
}
