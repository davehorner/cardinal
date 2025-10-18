/// Returns a reference to the build_orca_inject_queue function for use in event injection
pub fn build_orca_inject_queue_ref() -> Option<fn(&str) -> std::collections::VecDeque<InjectEvent>>
{
    Some(build_orca_inject_queue)
}
/// Build an InjectEvent queue for orca file injection with rectangle and efficient movement
pub fn build_orca_inject_queue(file_path: &str) -> std::collections::VecDeque<InjectEvent> {
    use std::collections::VecDeque;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use varvara::Key;

    let mut queue = VecDeque::new();
    const CTRL_H: Key = Key::Ctrl;
    const RIGHT: Key = Key::Right;
    const LEFT: Key = Key::Left;
    const UP: Key = Key::Up;
    const DOWN: Key = Key::Down;
    // Read file into lines
    let mut lines: Vec<Vec<char>> = Vec::new();
    let mut max_len = 0;
    if let Ok(file) = File::open(file_path) {
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            let chars: Vec<char> = line.chars().collect();
            max_len = max_len.max(chars.len());
            lines.push(chars);
        }
    }
    let rows = lines.len();
    let cols = max_len;
    // Build rectangle with '/' border
    let mut grid = vec![vec!['/'; cols + 2]; rows + 2];
    for i in 1..=rows {
        for j in 1..=cols {
            grid[i][j] = if j - 1 < lines[i - 1].len() {
                lines[i - 1][j - 1]
            } else {
                ' '
            };
        }
    }
    // Start at (1,1)
    let mut cur_row = 1;
    let mut cur_col = 1;
    queue.push_back(InjectEvent::KeyPress(CTRL_H));
    queue.push_back(InjectEvent::KeyRelease(CTRL_H));
    // Visit all non '.' cells efficiently (row-major order)
    for (r, row) in grid.iter().enumerate().take(rows + 2) {
        for (c, &ch) in row.iter().enumerate().take(cols + 2) {
            if ch != '.' {
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
                queue.push_back(InjectEvent::Char(ch as u8));
                // If this is a border '/' (not top/bottom), write hex x,y to the right
                if ch == '/' && r != 0 && r != rows + 1 {
                    // x = c, y = r
                    let hex = format!("{c:02X}{r:02X}");
                    for b in hex.bytes() {
                        queue.push_back(InjectEvent::Char(b));
                    }
                }
            }
        }
    }
    queue
}

#[cfg(not(target_arch = "wasm32"))]
use getrandom;
#[derive(Debug, Clone)]
pub enum InjectEvent {
    Char(u8),
    KeyPress(Key),
    KeyRelease(Key),
    Sleep(u64),      // milliseconds
    Chord(Vec<Key>), // Simultaneous keypresses
}
// uxn.rs - Uxn integration for e_window

#[allow(clippy::module_inception)]
pub mod uxn {

    use std::path::Path;
    use std::sync::{Arc, Mutex};
    use uxn::{Backend, Uxn};
    use varvara::Varvara;
    /// UxnModule: Encapsulates a Uxn VM and its state for e_window
    pub struct UxnModule {
        pub uxn: Arc<Mutex<Uxn<'static>>>,
        pub varvara: Option<Varvara>,
    }

    impl UxnModule {
        /// Create a new UxnModule, optionally loading a ROM file
        pub fn new(rom_path: Option<&Path>) -> Result<Self, String> {
            // Use a static RAM buffer for the Uxn VM
            static mut RAM: [u8; 65536] = [0; 65536];
            #[allow(static_mut_refs)]
            let ram: &'static mut [u8; 65536] = unsafe { &mut RAM };
            let mut uxn = Uxn::new(ram, Backend::Interpreter);
            let mut varvara = Varvara::default();

            // varvara.screen.buffer.resize(1024 * 768 * 4, 0);
            // varvara.screen.changed = true;
            if let Some(path) = rom_path {
                let rom = std::fs::read(path).map_err(|e| format!("Failed to read ROM: {e}"))?;
                let _ = uxn.reset(&rom);
                varvara.reset(&rom);

                varvara.load_sym_with_rom_path(path);
            }

            Ok(UxnModule {
                uxn: Arc::new(Mutex::new(uxn)),
                varvara: Some(varvara),
            })
        }

        // /// Reset the Uxn VM (clears memory and state)
        // pub fn reset(&self, rom: &[u8]) {
        //     let mut uxn = self.uxn.lock().unwrap();
        //     let _ = uxn.reset(rom);
        // }

        /// Load a new ROM into the Uxn VM (resets VM)
        pub fn load_rom(&mut self, rom_path: &Path) -> Result<(), String> {
            let rom = std::fs::read(rom_path).map_err(|e| format!("Failed to read ROM: {e}"))?;
            let mut uxn = self.uxn.lock().unwrap();
            let _ = uxn.reset(&rom);
            if let Some(varvara) = self.varvara.as_mut() {
                varvara.load_sym_with_rom_path(rom_path);
            }
            Ok(())
        }
    }
}

use log::{error, info};
use std::path::Path;
use std::sync::{Arc, Mutex};
// Re-export the cardinal-uxn and cardinal-varvara crates for Uxn VM and utilities
use ::uxn::{Backend, Uxn};
use varvara::Key;
use varvara::MouseState;
use varvara::TrackerState;
use varvara::Varvara;
/// UxnModule: Encapsulates a Uxn VM and its state for e_window
pub struct UxnModule {
    pub uxn: Arc<Mutex<Uxn<'static>>>,
    pub varvara: Option<Varvara>,
}

impl UxnModule {
    /// Create a new UxnModule, optionally loading a ROM file
    pub fn new(rom_path: Option<&Path>) -> Result<Self, String> {
        // Use a static RAM buffer for the Uxn VM
        static mut RAM: [u8; 65536] = [0; 65536];
        #[allow(static_mut_refs)]
        let ram: &'static mut [u8; 65536] = unsafe { &mut RAM };
        let mut uxn = Uxn::new(ram, Backend::Interpreter);
        let mut varvara = Varvara::default();
        if let Some(path) = rom_path {
            let rom = std::fs::read(path).map_err(|e| format!("Failed to read ROM: {e}"))?;
            let _ = uxn.reset(&rom);
            varvara.load_sym_with_rom_path(path);
        }
        Ok(UxnModule {
            uxn: Arc::new(Mutex::new(uxn)),
            varvara: Some(varvara),
        })
    }

    // Step/run methods are not available in cardinal-uxn. Use run/reset as needed.

    // No run_cycles method; use run/reset as needed.

    /// Reset the Uxn VM (clears memory and state)
    pub fn reset(&self, rom: &[u8]) {
        let mut uxn = self.uxn.lock().unwrap();
        let _ = uxn.reset(rom);
    }

    /// Load a new ROM into the Uxn VM (resets VM)
    pub fn load_rom(&mut self, rom_path: &Path) -> Result<(), String> {
        let rom = std::fs::read(rom_path).map_err(|e| format!("Failed to read ROM: {e}"))?;
        if let Some(varvara) = self.varvara.as_mut() {
            varvara.load_sym_with_rom_path(rom_path);
        }
        self.reset(&rom);
        Ok(())
    }

    // No get_state method; UxnState is not available in cardinal-uxn.
}

// Optionally, add egui integration for UxnModule (UI panel, etc.)
pub mod egui_ui {
    use super::*;
    use egui::{CollapsingHeader, Ui};

    pub fn show_uxn_panel(ui: &mut Ui, _uxn_mod: &UxnModule) {
        CollapsingHeader::new("Uxn VM State").show(ui, |ui| {
            ui.label("Uxn state display not implemented (no UxnState in _uxn)");
        });
    }
}
use eframe::egui;
use std::sync::mpsc;
// use log::{error, info};

#[derive(Debug)]
pub enum Event {
    LoadRom(Vec<u8>),
    SetMuted(bool),
    Console(u8),
}

pub struct UxnApp<'a> {
    pub vm: Uxn<'a>,
    pub dev: Varvara,
    scale: f32,
    size: (u16, u16),
    next_frame: f64,
    scroll: (f32, f32),
    cursor_pos: Option<(f32, f32)>,
    texture: egui::TextureHandle,
    event_rx: mpsc::Receiver<Event>,
    resized: Option<Box<dyn FnMut(u16, u16)>>,
    window_mode: String,
    aspect_ratio: f32,
    // For auto ROM cycling
    auto_rom_select: bool,
    auto_timer: f64,
    auto_index: usize,
    auto_roms: Vec<varvara::RomData>,
    /// Parallel to auto_roms: labels or filenames for each ROM
    auto_rom_labels: Vec<String>,
    /// Callback for when the ROM changes (filename or label)
    #[allow(clippy::type_complexity)]
    on_rom_change: Option<Box<dyn Fn(&str) + Send + Sync>>,
    /// Callback for first update/frame (for deferred actions)
    #[allow(clippy::type_complexity)]
    on_first_update: Option<Box<dyn FnOnce(&mut UxnApp<'a>) + Send + 'a>>,
    first_update_done: bool,
    /// The current ROM label or filename (if available)
    current_rom_label: Option<String>,
    /// Queue for deferred input events (for orca injection)
    input_queue: std::collections::VecDeque<InjectEvent>,
    /// The last ROM file path loaded (if any)
    last_rom_path: Option<std::path::PathBuf>,
    // --- Hot-reload support ---
    pub reload_rx: std::sync::mpsc::Receiver<()>,
    pub rom_path_arc: std::sync::Arc<std::sync::Mutex<Option<std::path::PathBuf>>>,
    #[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
    pub usb_controller: Option<UsbControllerHandle>,
    #[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
    pub last_usb_event: Option<(u8, Vec<u8>)>, // (pedal_state, raw data)
}

#[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
pub struct UsbControllerHandle {
    pub rx: std::sync::mpsc::Receiver<varvara::controller_usb::UsbControllerMessage>,
}

impl<'a> UxnApp<'a> {
    /// Set a callback to be called on the first update/frame (for deferred actions)
    pub fn set_on_first_update(&mut self, f: Box<dyn FnOnce(&mut UxnApp<'a>) + Send + 'a>) {
        self.on_first_update = Some(f);
        self.first_update_done = false;
    }
    /// Set the current ROM label or filename (for title/callback)
    pub fn set_rom_label<S: Into<String>>(&mut self, label: S) {
        self.current_rom_label = Some(label.into());
    }
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_mode(
        mut vm: Uxn<'a>,
        mut dev: Varvara,
        mut size: (u16, u16),
        scale: f32,
        event_rx: mpsc::Receiver<Event>,
        ctx: &egui::Context,
        window_mode: String,
        auto_roms: Vec<varvara::RomData>,
        auto_rom_labels: Vec<String>,
        auto_rom_select: bool,
        reload_rx: std::sync::mpsc::Receiver<()>,
        rom_path_arc: std::sync::Arc<std::sync::Mutex<Option<std::path::PathBuf>>>,
    ) -> Self {
        if let Some(path) = rom_path_arc.lock().unwrap().as_ref() {
            if path.exists() {
                dev.load_sym_with_rom_path(path);
            }
        }
        // Run the VM and redraw once to initialize the framebuffer
        vm.run(&mut dev, 0x100);

        dev.redraw(&mut vm);

        let w = 2048_usize; //size.0);
        let h = 2048_usize;
        size.0 = w as u16;
        size.1 = h as u16;
        let image = egui::ColorImage::new([w, h], vec![egui::Color32::BLACK; w * h]);
        let texture = ctx.load_texture("frame", image, egui::TextureOptions::NEAREST);
        let aspect_ratio = w as f32 / h as f32;
        let mut auto_index = 0;
        let mut auto_timer = 0.0;
        let mut current_rom_label = None;
        if auto_rom_select && !auto_roms.is_empty() {
            println!(
                "[AUTO ROM CYCLING] Enabled. Cycling through {} ROMs every 10s.",
                auto_roms.len()
            );
            // Load the first ROM immediately
            let rom = &auto_roms[0];
            let _ = vm.reset(&rom.rom);
            dev.reset(&rom.rom);
            vm.run(&mut dev, 0x100);
            dev.redraw(&mut vm);

            auto_index = 0;
            auto_timer = 0.0;
            // Use the provided label if available, else fallback
            if !auto_rom_labels.is_empty() {
                current_rom_label = Some(auto_rom_labels[0].clone());
            } else {
                current_rom_label = Some("ROM 1".to_string());
            }
        }
        UxnApp {
            vm,
            dev,
            scale,
            size,
            next_frame: 0.0,
            event_rx,
            resized: None,
            scroll: (0.0, 0.0),
            cursor_pos: None,
            texture,
            window_mode,
            aspect_ratio,
            auto_rom_select,
            auto_timer,
            auto_index,
            auto_roms,
            on_rom_change: None,
            current_rom_label,
            auto_rom_labels,
            on_first_update: None,
            first_update_done: false,
            input_queue: std::collections::VecDeque::new(),
            reload_rx,
            rom_path_arc,
            last_rom_path: None,
            #[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
            usb_controller: None,
            #[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
            last_usb_event: None,
        }
    }
    /// Set a callback to be called when the ROM changes (filename or label)
    pub fn set_on_rom_change<F>(&mut self, f: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_rom_change = Some(Box::new(f));
    }

    pub fn load_symbols(&mut self, path: &std::path::Path) -> Result<(), String> {
        println!("[UxnApp] Loading symbols from: {}", path.display());
        let _ = self.dev.load_symbols_into_self(path.to_str().unwrap());
        Ok(())
    }

    /// Reload the ROM from the given path (for hot-reload)
    pub fn reload_rom(&mut self, path: &std::path::Path) -> Result<(), String> {
        println!("[UxnApp] Reloading ROM from: {}", path.display());
        let rom = std::fs::read(path).map_err(|e| format!("Failed to read ROM: {e}"))?;
        let _ = self.vm.reset(&rom);
        self.dev.reset(&rom);
        self.vm.run(&mut self.dev, 0x100);
        self.dev.redraw(&mut self.vm);

        self.load_symbols(path)?;

        // Explicitly beep after reload
        #[cfg(windows)]
        {
            unsafe {
                winapi::um::winuser::MessageBeep(0xFFFFFFFF);
            }
        }
        Ok(())
    }

    pub fn set_resize_callback(&mut self, f: Box<dyn FnMut(u16, u16)>) {
        self.resized = Some(f);
    }

    /// Load a ROM from a file path, tracking the file path for symbol support
    pub fn load_rom_with_path<P: AsRef<std::path::Path>>(&mut self, path: P) -> anyhow::Result<()> {
        let path = path.as_ref();
        let data = std::fs::read(path)?;
        self.last_rom_path = Some(path.to_path_buf());
        let _ = self.vm.reset(&data);
        self.dev.reset(&data);
        self.vm.run(&mut self.dev, 0x100);
        // Try to load .sym file if ROM was loaded from a file
        let sym_path = path.with_extension("sym");
        if sym_path.exists() {
            let _ = self.dev.load_symbols_into_self(sym_path.to_str().unwrap());
        }
        // Optionally update the ROM label
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            self.set_rom_label(name);
        }
        let out = self.dev.output(&self.vm);
        out.check()?;
        Ok(())
    }

    /// Backward-compatible load_rom for in-memory loads (no file path)
    fn load_rom(&mut self, data: &[u8]) -> anyhow::Result<()> {
        self.last_rom_path = None;
        let _ = self.vm.reset(data);
        self.dev.reset(data);
        self.vm.run(&mut self.dev, 0x100);
        let out = self.dev.output(&self.vm);
        out.check()?;
        Ok(())
    }

    // Helper to get the last ROM path if available (for symbol loading)
    // fn get_last_rom_path(&self) -> Option<std::path::PathBuf> {
    //     self.last_rom_path.clone()
    // }

    /// Queue a sequence of input events to be sent per frame
    pub fn queue_input<I: IntoIterator<Item = InjectEvent>>(&mut self, input: I) {
        self.input_queue.extend(input);
    }
}

impl eframe::App for UxnApp<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- ROM hot-reload check ---
        if self.reload_rx.try_recv().is_ok() {
            let path_opt = { self.rom_path_arc.lock().unwrap().clone() };
            if let Some(ref path) = path_opt {
                println!(
                    "[ROM WATCHER] Detected ROM file change, reloading: {}",
                    path.display()
                );
                if let Err(e) = self.reload_rom(path) {
                    eprintln!("[ROM WATCHER] Failed to reload ROM in app: {e}");
                } else {
                    println!("[ROM WATCHER] ROM reloaded in app successfully.");
                }
            }
        }
        // --- First update callback for deferred actions (e.g., orca file injection) ---
        if !self.first_update_done {
            if let Some(cb) = self.on_first_update.take() {
                cb(self);
            }
            self.first_update_done = true;
        }
        // --- Ctrl+C and Ctrl+R event handling ---
        let orca_dir = r"C:\w\music\Orca-c\examples\basics";
        let files: Vec<std::path::PathBuf> = std::fs::read_dir(orca_dir)
            .map(|read_dir| {
                read_dir
                    .filter_map(|entry| {
                        entry.ok().and_then(|e| {
                            let path = e.path();
                            if path.extension().and_then(|ext| ext.to_str()) == Some("orca") {
                                Some(path)
                            } else {
                                None
                            }
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|_| Vec::new());
        static mut LAST_CTRL_R: Option<std::time::Instant> = None;
        let mut exit_requested = false;
        for event in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Key { key, pressed, .. } = event {
                if key == egui::Key::C && ctx.input(|i| i.modifiers.ctrl) && pressed {
                    exit_requested = true;
                }
                if key == egui::Key::R && ctx.input(|i| i.modifiers.ctrl) && pressed {
                    let now = std::time::Instant::now();
                    let last = unsafe { LAST_CTRL_R };
                    let allow = match last {
                        Some(t) => now.duration_since(t).as_millis() > 500,
                        None => true,
                    };
                    if allow {
                        unsafe {
                            LAST_CTRL_R = Some(now);
                        }
                        if !files.is_empty() {
                            #[cfg(target_arch = "wasm32")]
                            {
                                // On wasm, just cycle sequentially through the files
                                static mut LAST_IDX: usize = 0;
                                let idx = unsafe {
                                    let i = LAST_IDX;
                                    LAST_IDX = (LAST_IDX + 1) % files.len();
                                    i
                                };
                                let random_file = &files[idx];
                                let queue = build_orca_inject_queue(random_file.to_str().unwrap());
                                self.queue_input(queue);
                            }
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                // On native, pick a random file
                                if let Ok(idx) =
                                    get_random_u128().map(|v| (v as usize) % files.len())
                                {
                                    let idx: usize = idx;
                                    let random_file = &files[idx];
                                    let queue =
                                        build_orca_inject_queue(random_file.to_str().unwrap());
                                    self.queue_input(queue);
                                }
                            }
                        }
                    }
                }
            }
        }

        /// Returns a random u128 using getrandom
        #[cfg(not(target_arch = "wasm32"))]
        pub fn get_random_u128() -> Result<u128, getrandom::Error> {
            let mut buf = [0u8; 16];
            getrandom::fill(&mut buf)?;
            Ok(u128::from_ne_bytes(buf))
        }

        #[cfg(target_arch = "wasm32")]
        #[allow(dead_code)]
        pub fn get_random_u128() -> Result<u128, Result<u128, ()>> {
            // WASM: fallback to a deterministic value or use JS random if needed
            Ok(42)
        }
        if exit_requested {
            std::process::exit(0);
        }
        // --- Per-frame input injection ---
        if let Some(event) = self.input_queue.pop_front() {
            match event {
                InjectEvent::Char(c) => self.dev.char(&mut self.vm, c),
                InjectEvent::KeyPress(k) => self.dev.pressed(&mut self.vm, k, false),
                InjectEvent::KeyRelease(k) => self.dev.released(&mut self.vm, k),
                InjectEvent::Sleep(ms) => {
                    std::thread::sleep(std::time::Duration::from_millis(ms));
                }
                InjectEvent::Chord(keys) => {
                    for k in &keys {
                        self.dev.pressed(&mut self.vm, *k, false);
                    }
                }
            }
        }
        // --- AUTO ROM CYCLING: Switch between ROMs every 10 seconds if enabled ---
        if self.auto_rom_select && !self.auto_roms.is_empty() {
            let dt = ctx.input(|i| i.stable_dt) as f64;
            log::debug!(
                "[AUTO ROM CYCLING] auto_timer: {:.3}, dt: {:.3}, auto_index: {}, roms: {}",
                self.auto_timer,
                dt,
                self.auto_index,
                self.auto_roms.len()
            );
            self.auto_timer += dt;
            if self.auto_timer == 0.0 && self.auto_index == 0 {
                log::debug!("[AUTO ROM CYCLING] First ROM already loaded.");
                // Already loaded first ROM in new_with_mode
            } else if self.auto_timer > 10.0 {
                log::debug!(
                    "[AUTO ROM CYCLING] Switching ROM: {} -> {}",
                    self.auto_index,
                    (self.auto_index + 1) % self.auto_roms.len()
                );
                self.auto_timer = 0.0;
                self.auto_index = (self.auto_index + 1) % self.auto_roms.len();
                let rom = self.auto_roms[self.auto_index].clone();
                // Use the correct label for the new ROM
                let label = if self.auto_index < self.auto_rom_labels.len() {
                    self.auto_rom_labels[self.auto_index].clone()
                } else {
                    format!("ROM {}", self.auto_index + 1)
                };
                self.set_rom_label(label);
                let _ = self.load_rom(&rom.rom);
                if rom.sym.is_some() {
                    // Load symbols if available
                    if let Some(sym) = &rom.sym {
                        let _ = self.dev.load_symbols_from_vec(sym);
                    }
                }
            }
        }
        while let Ok(e) = self.event_rx.try_recv() {
            match e {
                Event::LoadRom(data) => {
                    if let Err(e) = self.load_rom(&data) {
                        error!("could not load rom: {e:?}");
                    }
                }
                Event::SetMuted(m) => {
                    self.dev.audio_set_muted(m);
                }
                Event::Console(b) => {
                    self.dev.console(&mut self.vm, b);
                }
            }
        }
        // if let Some(ref mut varvara_controller) =
        //     self.dev
        //         .controller
        //         .as_any()
        //         .downcast_mut::<varvara::controller::Controller>()
        // {
        //     // Example pedal event mapping (replace with your actual pedal state logic)
        //     // Assume pedal_state is a u8 bitmask from your input source
        //     let pedal_state: u8 = 0; // TODO: get real pedal state
        //     static mut PREV_PEDAL: u8 = 0;
        //     let _prev;
        //     unsafe {
        //         _prev = PREV_PEDAL;
        //         PREV_PEDAL = pedal_state;
        //     }
        //     // Use canonical helper from varvara::controller
        //     varvara::controller::inject_pedal_keys(
        //         varvara_controller,
        //         &mut self.vm,
        //         _prev,
        //         pedal_state,
        //     );
        // }
        #[cfg(target_arch = "wasm32")]
        let events = Vec::new();
        #[cfg(not(target_arch = "wasm32"))]
        let mut events = Vec::new();
        #[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
        {
            // Only borrow self.dev.controller mutably once in this block
            let mut last_pedal = None;
            if let Some(controller_usb) = self
                .dev
                .controller
                .as_any()
                .downcast_mut::<varvara::controller_usb::ControllerUsb>()
            {
                events = varvara::controller_usb::ControllerPollEvents::poll_usb_events(
                    controller_usb,
                    &mut self.vm,
                );
                last_pedal = controller_usb.last_pedal;
            }
            // Now, after the mutable borrow is done, update last_usb_event, etc.
            static mut PREV_PEDAL: u8 = 0;
            if let Some(pedal) = last_pedal {
                // Update last_usb_event for debug panel
                self.last_usb_event = Some((pedal, vec![]));
                let _prev;
                unsafe {
                    _prev = PREV_PEDAL;
                    PREV_PEDAL = pedal;
                }
                // Use shared controller pedal key injection logic
                // Controller::inject_pedal_keys(&mut self.vm, prev, pedal);
            }
        }
        let pedal_data: Vec<u8> = events
            .iter()
            .flat_map(|e: &varvara::Event| e.data.iter().map(|b| b.value))
            .collect();
        for value in pedal_data {
            self.dev
                .pressed(&mut self.vm, varvara::Key::Char(value), true);
            self.dev.char(&mut self.vm, value);
        }

        ctx.request_repaint();
        ctx.input(|i| {
            while i.time >= self.next_frame {
                self.next_frame += 0.0166667;
                self.dev.redraw(&mut self.vm);
            }
            if i.raw.dropped_files.len() == 1 {
                let target = &i.raw.dropped_files[0];
                let r = if let Some(path) = &target.path {
                    info!("loading ROM from {path:?}");
                    self.load_rom_with_path(path)
                } else if let Some(data) = &target.bytes {
                    self.load_rom(data)
                } else {
                    Ok(())
                };
                if let Err(e) = r {
                    error!("could not load ROM: {e:?}");
                }
            }
            let shift_held = i.modifiers.shift;
            for e in i.events.iter() {
                match e {
                    egui::Event::Text(s) => {
                        for c in s.bytes() {
                            self.dev.char(&mut self.vm, c);
                        }
                    }
                    egui::Event::Key {
                        key,
                        pressed,
                        repeat,
                        ..
                    } => {
                        if let Some(k) = decode_key(*key, shift_held) {
                            if *pressed {
                                self.dev.pressed(&mut self.vm, k, *repeat);
                            } else {
                                self.dev.released(&mut self.vm, k);
                            }
                        }
                    }
                    _ => (),
                }
            }
            for (b, k) in [
                (i.modifiers.ctrl, Key::Ctrl),
                (i.modifiers.alt, Key::Alt),
                (i.modifiers.shift, Key::Shift),
            ] {
                if b {
                    self.dev.pressed(&mut self.vm, k, false)
                } else {
                    self.dev.released(&mut self.vm, k)
                }
            }
            let ptr = &i.pointer;
            if let Some(p) = ptr.latest_pos() {
                self.cursor_pos = Some((p.x / self.scale, p.y / self.scale));
            }
            let buttons = [
                egui::PointerButton::Primary,
                egui::PointerButton::Middle,
                egui::PointerButton::Secondary,
            ]
            .into_iter()
            .enumerate()
            .map(|(i, b)| (ptr.button_down(b) as u8) << i)
            .fold(0, |a, b| a | b);
            let m = MouseState {
                pos: self.cursor_pos.unwrap_or((0.0, 0.0)),
                scroll: std::mem::take(&mut self.scroll),
                buttons,
            };
            self.dev.mouse(&mut self.vm, m);
            let m = TrackerState {
                pos: self
                    .cursor_pos
                    .map(|(x, y)| (x + 16.0, y + 16.0))
                    .unwrap_or((16.0, 16.0)),
                scroll: std::mem::take(&mut self.scroll),
                buttons,
            };
            self.dev.tracker(&mut self.vm, m);
            i.time
        });
        self.dev.audio(&mut self.vm);
        let out = self.dev.output(&self.vm);
        if out.hide_mouse {
            ctx.set_cursor_icon(egui::CursorIcon::None);
        }
        if self.size != out.size {
            // Get current window size in logical points
            let current_window_size = ctx.input(|i| {
                i.viewport()
                    .inner_rect
                    .map_or(egui::Vec2::ZERO, |rect| rect.size())
            });
            let new_size = egui::Vec2::new(out.size.0 as f32, out.size.1 as f32) * self.scale;
            // let should_resize = new_size.x > current_window_size.x || new_size.y > current_window_size.y
            //     || new_size.x < current_window_size.x || new_size.y < current_window_size.y;
            // // Only resize if the new frame is larger, or if it's smaller than the window
            // if should_resize {
            // Only resize if the new frame is larger than the current window
            if new_size.x > current_window_size.x || new_size.y > current_window_size.y {
                info!("resizing window to {:?}", out.size);
                self.size = out.size;
                let mut size = new_size;
                // Enforce proportional resizing if needed
                if self.window_mode == "proportional" {
                    let aspect = self.aspect_ratio;
                    let w = size.x;
                    let h = size.y;
                    let (new_w, new_h) = if w / h > aspect {
                        (h * aspect, h)
                    } else {
                        (w, w / aspect)
                    };
                    size = egui::Vec2::new(new_w, new_h);
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
                if let Some(f) = self.resized.as_mut() {
                    f(out.size.0, out.size.1);
                }
            }
        }
        let w = out.size.0 as usize;
        let h = out.size.1 as usize;
        let mut image = egui::ColorImage::new([w, h], vec![egui::Color32::BLACK; w * h]);
        for (i, o) in out.frame.chunks(4).zip(image.pixels.iter_mut()) {
            *o = egui::Color32::from_rgba_unmultiplied(i[2], i[1], i[0], i[3]);
        }
        self.texture.set(image, egui::TextureOptions::NEAREST);
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                let available = ui.available_size();
                let frame_size = egui::Vec2::new(
                    out.size.0 as f32 * self.scale,
                    out.size.1 as f32 * self.scale,
                );
                let offset = egui::Vec2::new(
                    (available.x - frame_size.x) * 0.5,
                    (available.y - frame_size.y) * 0.5,
                );
                let top_left = ui.min_rect().min + offset;
                let mut mesh = egui::Mesh::with_texture(self.texture.id());
                mesh.add_rect_with_uv(
                    egui::Rect {
                        min: top_left,
                        max: top_left + frame_size,
                    },
                    egui::Rect {
                        min: egui::Pos2::new(0.0, 0.0),
                        max: egui::Pos2::new(1.0, 1.0),
                    },
                    egui::Color32::WHITE,
                );
                ui.painter().add(egui::Shape::mesh(mesh));
                // --- USB Debug Panel ---
                #[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
                egui::CollapsingHeader::new("USB Pedal Debug")
                    .default_open(true)
                    .show(ui, |ui| {
                        if let Some((pedal_state, ref data)) = self.last_usb_event {
                            ui.label(format!(
                                "[USB] Last pedal event: state=0x{pedal_state:02X}, data={data:?}"
                            ));
                        } else {
                            ui.label("[USB] No pedal events received yet.");
                        }
                    });
                // Show auto ROM cycling info
                if self.auto_rom_select && !self.auto_roms.is_empty() {
                    ui.label(format!(
                        "[AUTO ROM CYCLING] Switching ROM every 10s. ROMs loaded: {}",
                        self.auto_roms.len()
                    ));
                }
            });
        out.check().ok();
    }
}

pub fn decode_key(k: egui::Key, shift: bool) -> Option<Key> {
    match (k, shift) {
        (egui::Key::ArrowUp, _) => Some(Key::Up),
        (egui::Key::ArrowDown, _) => Some(Key::Down),
        (egui::Key::ArrowLeft, _) => Some(Key::Left),
        (egui::Key::ArrowRight, _) => Some(Key::Right),
        (egui::Key::Home, _) => Some(Key::Home),
        (egui::Key::Num0, false) => Some(Key::Char(b'0')),
        (egui::Key::Num0, true) => Some(Key::Char(b')')),
        (egui::Key::Num1, false) => Some(Key::Char(b'1')),
        (egui::Key::Num1, true) => Some(Key::Char(b'!')),
        (egui::Key::Num2, false) => Some(Key::Char(b'2')),
        (egui::Key::Num2, true) => Some(Key::Char(b'@')),
        (egui::Key::Num3, false) => Some(Key::Char(b'3')),
        (egui::Key::Num3, true) => Some(Key::Char(b'#')),
        (egui::Key::Num4, false) => Some(Key::Char(b'4')),
        (egui::Key::Num4, true) => Some(Key::Char(b'$')),
        (egui::Key::Num5, false) => Some(Key::Char(b'5')),
        (egui::Key::Num5, true) => Some(Key::Char(b'%')),
        (egui::Key::Num6, false) => Some(Key::Char(b'6')),
        (egui::Key::Num6, true) => Some(Key::Char(b'^')),
        (egui::Key::Num7, false) => Some(Key::Char(b'7')),
        (egui::Key::Num7, true) => Some(Key::Char(b'&')),
        (egui::Key::Num8, false) => Some(Key::Char(b'8')),
        (egui::Key::Num8, true) => Some(Key::Char(b'*')),
        (egui::Key::Num9, false) => Some(Key::Char(b'9')),
        (egui::Key::Num9, true) => Some(Key::Char(b'(')),
        (egui::Key::A, false) => Some(Key::Char(b'a')),
        (egui::Key::A, true) => Some(Key::Char(b'A')),
        (egui::Key::B, false) => Some(Key::Char(b'b')),
        (egui::Key::B, true) => Some(Key::Char(b'B')),
        (egui::Key::C, false) => Some(Key::Char(b'c')),
        (egui::Key::C, true) => Some(Key::Char(b'C')),
        (egui::Key::D, false) => Some(Key::Char(b'd')),
        (egui::Key::D, true) => Some(Key::Char(b'D')),
        (egui::Key::E, false) => Some(Key::Char(b'e')),
        (egui::Key::E, true) => Some(Key::Char(b'E')),
        (egui::Key::F, false) => Some(Key::Char(b'f')),
        (egui::Key::F, true) => Some(Key::Char(b'F')),
        (egui::Key::G, false) => Some(Key::Char(b'g')),
        (egui::Key::G, true) => Some(Key::Char(b'G')),
        (egui::Key::H, false) => Some(Key::Char(b'h')),
        (egui::Key::H, true) => Some(Key::Char(b'H')),
        (egui::Key::I, false) => Some(Key::Char(b'i')),
        (egui::Key::I, true) => Some(Key::Char(b'I')),
        (egui::Key::J, false) => Some(Key::Char(b'j')),
        (egui::Key::J, true) => Some(Key::Char(b'J')),
        (egui::Key::K, false) => Some(Key::Char(b'k')),
        (egui::Key::K, true) => Some(Key::Char(b'K')),
        (egui::Key::L, false) => Some(Key::Char(b'l')),
        (egui::Key::L, true) => Some(Key::Char(b'L')),
        (egui::Key::M, false) => Some(Key::Char(b'm')),
        (egui::Key::M, true) => Some(Key::Char(b'M')),
        (egui::Key::N, false) => Some(Key::Char(b'n')),
        (egui::Key::N, true) => Some(Key::Char(b'N')),
        (egui::Key::O, false) => Some(Key::Char(b'o')),
        (egui::Key::O, true) => Some(Key::Char(b'O')),
        (egui::Key::P, false) => Some(Key::Char(b'p')),
        (egui::Key::P, true) => Some(Key::Char(b'P')),
        (egui::Key::Q, false) => Some(Key::Char(b'q')),
        (egui::Key::Q, true) => Some(Key::Char(b'Q')),
        (egui::Key::R, false) => Some(Key::Char(b'r')),
        (egui::Key::R, true) => Some(Key::Char(b'R')),
        (egui::Key::S, false) => Some(Key::Char(b's')),
        (egui::Key::S, true) => Some(Key::Char(b'S')),
        (egui::Key::T, false) => Some(Key::Char(b't')),
        (egui::Key::T, true) => Some(Key::Char(b'T')),
        (egui::Key::U, false) => Some(Key::Char(b'u')),
        (egui::Key::U, true) => Some(Key::Char(b'U')),
        (egui::Key::V, false) => Some(Key::Char(b'v')),
        (egui::Key::V, true) => Some(Key::Char(b'V')),
        (egui::Key::W, false) => Some(Key::Char(b'w')),
        (egui::Key::W, true) => Some(Key::Char(b'W')),
        (egui::Key::X, false) => Some(Key::Char(b'x')),
        (egui::Key::X, true) => Some(Key::Char(b'X')),
        (egui::Key::Y, false) => Some(Key::Char(b'y')),
        (egui::Key::Y, true) => Some(Key::Char(b'Y')),
        (egui::Key::Z, false) => Some(Key::Char(b'z')),
        (egui::Key::Z, true) => Some(Key::Char(b'Z')),
        (egui::Key::Backtick, false) => Some(Key::Char(b'`')),
        (egui::Key::Backtick, true) => Some(Key::Char(b'~')),
        (egui::Key::Backslash, _) => Some(Key::Char(b'\\')),
        (egui::Key::Pipe, _) => Some(Key::Char(b'|')),
        (egui::Key::Comma, false) => Some(Key::Char(b',')),
        (egui::Key::Comma, true) => Some(Key::Char(b'<')),
        (egui::Key::Equals, _) => Some(Key::Char(b'=')),
        (egui::Key::Plus, _) => Some(Key::Char(b'+')),
        (egui::Key::OpenBracket, false) => Some(Key::Char(b'[')),
        (egui::Key::OpenBracket, true) => Some(Key::Char(b'{')),
        (egui::Key::Minus, false) => Some(Key::Char(b'-')),
        (egui::Key::Minus, true) => Some(Key::Char(b'_')),
        (egui::Key::Period, false) => Some(Key::Char(b'.')),
        (egui::Key::Period, true) => Some(Key::Char(b'>')),
        (egui::Key::CloseBracket, false) => Some(Key::Char(b']')),
        (egui::Key::CloseBracket, true) => Some(Key::Char(b'}')),
        (egui::Key::Semicolon, _) => Some(Key::Char(b';')),
        (egui::Key::Colon, _) => Some(Key::Char(b':')),
        (egui::Key::Slash, _) => Some(Key::Char(b'/')),
        (egui::Key::Questionmark, _) => Some(Key::Char(b'?')),
        (egui::Key::Space, _) => Some(Key::Char(b' ')),
        (egui::Key::Tab, _) => Some(Key::Char(b'\t')),
        (egui::Key::Enter, _) => Some(Key::Char(b'\r')),
        (egui::Key::Backspace, _) => Some(Key::Char(0x08)),
        _ => None,
    }
}

/// Stub for audio_setup on wasm32
// #[cfg(target_arch = "wasm32")]
// pub fn audio_setup(
//     _data: [Arc<Mutex<varvara::StreamData>>; 4],
// ) -> Option<()> {
//     None
// }
/// Shared audio setup for Varvara audio streams
// #[cfg(not(target_arch = "wasm32"))]
pub fn audio_setup(
    data: [Arc<Mutex<varvara::StreamData>>; 4],
) -> Option<(cpal::Device, [cpal::Stream; 4])> {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");
    let supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs")
        .collect::<Vec<_>>();

    let mut supported_config = None;
    let mut used_rate = 0;
    for &rate in &[48000, 44100] {
        supported_config = supported_configs_range
            .iter()
            .filter(|c| usize::from(c.channels()) == varvara::AUDIO_CHANNELS)
            .filter(|c| c.sample_format() == cpal::SampleFormat::F32)
            .find_map(|c| c.try_with_sample_rate(cpal::SampleRate(rate)));
        if supported_config.is_some() {
            used_rate = rate;
            break;
        }
    }
    let supported_config = match supported_config {
        Some(cfg) => cfg,
        None => {
            log::error!(
                "could not find supported audio config ({} channels, 48000 or 44100 Hz, f32)",
                varvara::AUDIO_CHANNELS
            );
            log::error!("available configs:");
            for c in &supported_configs_range {
                if c.min_sample_rate() == c.max_sample_rate() {
                    log::error!(
                        "  channels: {}, sample_rate: {} Hz, {}",
                        c.channels(),
                        c.min_sample_rate().0,
                        c.sample_format(),
                    );
                } else {
                    log::error!(
                        "  channels: {}, sample_rate: {} - {} Hz, {}",
                        c.channels(),
                        c.min_sample_rate().0,
                        c.max_sample_rate().0,
                        c.sample_format(),
                    );
                }
            }
            return None;
        }
    };
    // Set the sample rate in the audio engine
    varvara::set_sample_rate(used_rate);
    let config = supported_config.config();

    let streams = data.map(|d| {
        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _opt: &cpal::OutputCallbackInfo| {
                    d.lock().unwrap().next(data);
                },
                move |err| {
                    panic!("{err}");
                },
                None,
            )
            .expect("could not build stream");
        cpal::traits::StreamTrait::play(&stream).unwrap();
        stream
    });
    Some((device, streams))
}

/// Inject key events from egui input into the VM, handling press/release and modifiers
pub fn inject_key_events(
    vm: &mut ::uxn::Uxn,
    dev: &mut varvara::Varvara,
    events: &[egui::Event],
    modifiers: &egui::Modifiers,
    _repeat: bool,
) {
    use varvara::Key;
    let shift_held = modifiers.shift;
    for e in events {
        match e {
            egui::Event::Text(s) => {
                const RAW_CHARS: [u8; 16] = [
                    b'"', b'\'', b'{', b'}', b'_', b')', b'(', b'*', b'&', b'^', b'%', b'$', b'#',
                    b'@', b'!', b'~',
                ];
                for c in s.bytes() {
                    if RAW_CHARS.contains(&c) {
                        dev.char(vm, c);
                    }
                }
            }
            egui::Event::Key {
                key,
                pressed,
                repeat,
                ..
            } => {
                if let Some(k) = decode_key(*key, shift_held) {
                    if *pressed {
                        dev.pressed(vm, k, *repeat);
                    } else {
                        dev.released(vm, k);
                    }
                }
            }
            egui::Event::MouseWheel { delta, .. } => {
                dev.mouse(
                    vm,
                    varvara::MouseState {
                        pos: (0.0, 0.0),
                        scroll: (delta.x, -delta.y),
                        buttons: 0,
                    },
                );
            }
            _ => (),
        }
    }
    for (b, k) in [
        (modifiers.ctrl, Key::Ctrl),
        (modifiers.alt, Key::Alt),
        (modifiers.shift, Key::Shift),
    ] {
        if b {
            dev.pressed(vm, k, false)
        } else {
            dev.released(vm, k)
        }
    }
}
