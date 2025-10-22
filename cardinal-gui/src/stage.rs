#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseResizeMode {
    None,
    MiddleDrag,
}

use log::{error, info};
use std::sync::mpsc;
use uxn::Uxn;
use varvara::{Key, Varvara};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseDragMode {
    None,
    DoubleDrag,
    // Only None and DoubleDrag are supported for window movement
}
pub struct EffectsConfig {
    pub efxt: f32,
    pub effect_mode: usize,
    pub effect_order: Vec<usize>,
    pub efx: Option<String>,
    pub efxmode: Option<String>,
    pub efx_ndx: usize,          // current effect index
    pub last_effect_switch: f64, // time of last effect switch
}

pub struct StageConfig {
    pub size: (u16, u16),
    pub scale: f32,
    pub rom_title: String,
    pub transparent: Option<String>,
    pub color_transform_name: String,
    pub color_params: Vec<f32>,
    pub effects: EffectsConfig,
    pub fit_mode: String,
    pub mouse_mode: MouseDragMode,
    pub mouse_resize: MouseResizeMode,
}

#[derive(Debug)]
pub enum Event {
    LoadRom(Vec<u8>),
    SetMuted(bool),
    Console(u8),
}

// Color transform trait and helpers
pub trait ColorTransform {
    fn apply(&self, r: u8, g: u8, b: u8, x: usize, y: usize, t: f32) -> (u8, u8, u8);
}

pub struct ColorShift {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}
impl ColorTransform for ColorShift {
    fn apply(&self, r: u8, g: u8, b: u8, _x: usize, _y: usize, _t: f32) -> (u8, u8, u8) {
        (
            ((r as f32 + self.r * 255.0).clamp(0.0, 255.0)) as u8,
            ((g as f32 + self.g * 255.0).clamp(0.0, 255.0)) as u8,
            ((b as f32 + self.b * 255.0).clamp(0.0, 255.0)) as u8,
        )
    }
}

pub struct Invert;
impl ColorTransform for Invert {
    fn apply(&self, r: u8, g: u8, b: u8, _x: usize, _y: usize, _t: f32) -> (u8, u8, u8) {
        (255 - r, 255 - g, 255 - b)
    }
}

pub struct Grayscale;
impl ColorTransform for Grayscale {
    fn apply(&self, r: u8, g: u8, b: u8, _x: usize, _y: usize, _t: f32) -> (u8, u8, u8) {
        let avg = ((r as u16 + g as u16 + b as u16) / 3) as u8;
        (avg, avg, avg)
    }
}

pub fn get_transform(name: &str, params: &[f32]) -> Box<dyn ColorTransform> {
    match name {
        "shift" => Box::new(ColorShift {
            r: params.first().copied().unwrap_or(0.0),
            g: params.get(1).copied().unwrap_or(0.0),
            b: params.get(2).copied().unwrap_or(0.0),
        }),
        "invert" => Box::new(Invert),
        "grayscale" => Box::new(Grayscale),
        _ => Box::new(ColorShift {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }),
    }
}

pub struct Stage<'a> {
    // zebra_offset moved to EffectsConfig
    pub should_exit: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    pub vm: Uxn<'a>,
    pub dev: Varvara,
    pub scale: f32,
    pub size: (u16, u16),
    pub next_frame: f64,
    pub scroll: (f32, f32),
    pub cursor_pos: Option<(f32, f32)>,
    pub texture: Option<egui::TextureHandle>,
    pub event_rx: mpsc::Receiver<Event>,
    pub resized: Option<Box<dyn FnMut(u16, u16)>>,
    pub transparent_rgb: Option<(u8, u8, u8)>,
    pub stopped: bool,
    pub drag_started: bool,
    pub color_transform: Box<dyn ColorTransform>,
    pub rom_title: String,
    pub config: StageConfig,
    pub drag_window: bool,
    pub drag_start_pos: Option<egui::Pos2>,
    pub resize_start_size: Option<(u16, u16)>,
    pub last_resize_size: Option<(u16, u16)>,
}

impl<'a> Stage<'a> {
    pub fn new(
        vm: Uxn<'a>,
        dev: Varvara,
        event_rx: mpsc::Receiver<Event>,
        config: StageConfig,
        should_exit: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    ) -> Self {
        // let image = egui::ColorImage::new(
        //     [usize::from(config.size.0), usize::from(config.size.1)],
        //     vec![egui::Color32::BLACK; (config.size.0 as usize) * (config.size.1 as usize)],
        // );
        // Texture will be created in update_texture when ctx is available
        let transparent_rgb = config.transparent.as_ref().and_then(|s| {
            if s.len() == 6 {
                let r = u8::from_str_radix(&s[0..2], 16).ok()?;
                let g = u8::from_str_radix(&s[2..4], 16).ok()?;
                let b = u8::from_str_radix(&s[4..6], 16).ok()?;
                Some((r, g, b))
            } else {
                None
            }
        });
        let color_transform = get_transform(&config.color_transform_name, &config.color_params);
        // Validate efxmode (blend mode)
        // let valid_efxmode = match config.efxmode.as_deref() {
        //     Some("blend") => Some(crate::effects::BlendMode::BlendMode),
        //     Some("full") => Some(crate::effects::BlendMode::FullMode),
        //     Some("random") => Some(crate::effects::BlendMode::Random),
        //     Some("normal") => Some(crate::effects::BlendMode::FullMode),
        //     None => None,
        //     _ => None,
        // };
        Stage {
            vm,
            dev,
            scale: config.scale,
            size: config.size,
            next_frame: 0.0,
            event_rx,
            resized: None,
            scroll: (0.0, 0.0),
            cursor_pos: None,
            texture: None,
            transparent_rgb,
            drag_window: false,
            drag_start_pos: None,
            stopped: false,
            drag_started: false,
            color_transform,
            rom_title: config.rom_title.clone(),
            should_exit,
            config,
            resize_start_size: None,
            last_resize_size: None,
        }
    }

    pub fn set_resize_callback(&mut self, f: Box<dyn FnMut(u16, u16)>) {
        self.resized = Some(f);
    }
    pub fn load_rom(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.vm.reset(data);
        self.dev.reset(data);
        self.vm.run(&mut self.dev, 0x100);
        let out = self.dev.output(&self.vm);
        self.size = out.size;
        Ok(())
    }
    pub fn step(&mut self) {
        // self.vm.run(&mut self.dev, 0x100);
        self.dev.redraw(&mut self.vm);
    }
    pub fn update_texture(&mut self, ctx: &egui::Context) {
        // Prepare image and pixel coordinates for effect loop
        let out = self.dev.output(&self.vm);
        let mut image = egui::ColorImage::new(
            [out.size.0 as usize, out.size.1 as usize],
            vec![egui::Color32::BLACK; (out.size.0 as usize) * (out.size.1 as usize)],
        );
        let mut x = 0;
        let mut y = 0;
        let w = out.size.0 as usize;
        let h = out.size.1 as usize;
        let effects = &self.config.effects;
        let effect_index = if let Some(ref efx_name) = effects.efx {
            let name = efx_name.to_ascii_lowercase();
            if name == "random" || name == "sequential" {
                effects.efx_ndx
            } else {
                crate::effects::effect_name_to_index(efx_name).unwrap_or(0)
            }
        } else {
            0
        };
        // Debug print: show current effect index and name during cycling
        // Print blend mode only when it changes
        // Print effect cycling info only when effect index changes
        {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static LAST_PRINTED_INDEX: AtomicUsize = AtomicUsize::new(usize::MAX);
            let blend_mode_str = self
                .config
                .effects
                .efxmode
                .clone()
                .unwrap_or_else(|| "none".to_string());
            if let Some(ref efx_name) = effects.efx {
                let name = efx_name.to_ascii_lowercase();
                if name == "random" || name == "sequential" {
                    let last = LAST_PRINTED_INDEX.load(Ordering::Relaxed);
                    if last != effect_index {
                        println!(
                            "Effect cycling: index {} - {} | blend mode: {}",
                            effect_index,
                            crate::effects::effect_name(effect_index),
                            blend_mode_str
                        );
                        LAST_PRINTED_INDEX.store(effect_index, Ordering::Relaxed);
                    }
                }
            }
        }
        // Use actual frame time for dynamic effects
        let t = ctx.input(|i| i.time) as f32;
        let zebra_offset = effects.efx_ndx;
        let blend_mode = if let Some(ref mode) = effects.efxmode {
            match mode.as_str() {
                "blend" => crate::effects::BlendMode::BlendMode,
                "full" => crate::effects::BlendMode::FullMode,
                "fullDyn" | "fulldyn" => crate::effects::BlendMode::FullDyn,
                "random" => crate::effects::BlendMode::Random,
                "diagonal" => crate::effects::BlendMode::Diagonal,
                "checker" => crate::effects::BlendMode::Checker,
                "threshold" => crate::effects::BlendMode::Threshold,
                "intensity" => crate::effects::BlendMode::Intensity,
                "anylit" => crate::effects::BlendMode::AnyLit,
                "optimal" => crate::effects::BlendMode::Optimal,
                "optimalperframe" => crate::effects::BlendMode::OptimalPerFrame,
                "randommode" => crate::effects::BlendMode::RandomMode,
                "randommodemode" => crate::effects::BlendMode::RandomModeMode,
                _ => crate::effects::BlendMode::Optimal,
            }
        } else {
            crate::effects::BlendMode::Optimal
        };

        // For optimal/optimalPerFrame, scan image for unique intensities
        let _selected_blend = blend_mode;
        use std::collections::HashSet;
        // BlendMode::Optimal: cache unique_colors/intensities only once
        use once_cell::sync::OnceCell;
        static OPTIMAL_COLORS: OnceCell<HashSet<(u8, u8, u8)>> = OnceCell::new();
        static OPTIMAL_INTENSITIES: OnceCell<HashSet<u16>> = OnceCell::new();
        let (unique_colors_len, unique_intensities_len) = match blend_mode {
            crate::effects::BlendMode::Optimal => {
                let colors = OPTIMAL_COLORS.get_or_init(|| {
                    let mut set = HashSet::new();
                    for i in out.frame.chunks(4) {
                        set.insert((i[2], i[1], i[0]));
                    }
                    set
                });
                let intensities = OPTIMAL_INTENSITIES.get_or_init(|| {
                    let mut set = HashSet::new();
                    for i in out.frame.chunks(4) {
                        let intensity = (i[2] as u16 + i[1] as u16 + i[0] as u16) / 3;
                        set.insert(intensity);
                    }
                    set
                });
                // Only print on first frame
                if !colors.is_empty()
                    && !intensities.is_empty()
                    && colors.len() + intensities.len() == colors.len() + intensities.len()
                {
                    // crude way to print only once
                    static PRINTED: std::sync::atomic::AtomicBool =
                        std::sync::atomic::AtomicBool::new(false);
                    if !PRINTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                        println!(
                            "[BlendMode::Optimal] unique_colors: {}, unique_intensities: {}",
                            colors.len(),
                            intensities.len()
                        );
                    }
                }
                (colors.len(), intensities.len())
            }
            crate::effects::BlendMode::OptimalPerFrame => {
                let mut unique_colors = HashSet::new();
                let mut unique_intensities = HashSet::new();
                for i in out.frame.chunks(4) {
                    unique_colors.insert((i[2], i[1], i[0]));
                    let intensity = (i[2] as u16 + i[1] as u16 + i[0] as u16) / 3;
                    unique_intensities.insert(intensity);
                }
                println!(
                    "[BlendMode::Optimal] unique_colors: {}, unique_intensities: {}",
                    unique_colors.len(),
                    unique_intensities.len()
                );
                (unique_colors.len(), unique_intensities.len())
            }
            _ => (0, 0),
        };

        // Only override efxmode if blend_mode is Optimal or OptimalPerFrame
        if matches!(
            blend_mode,
            crate::effects::BlendMode::Optimal | crate::effects::BlendMode::OptimalPerFrame
        ) {
            if unique_colors_len <= 3 || unique_intensities_len <= 3 {
                self.config.effects.efxmode = Some("none".to_string());
            } else if unique_colors_len < 10 || unique_intensities_len < 10 {
                self.config.effects.efxmode = Some("full".to_string());
            } else {
                self.config.effects.efxmode = Some("blend".to_string());
            }
        }

        for (i, o) in out.frame.chunks(4).zip(image.pixels.iter_mut()) {
            let r = i[2];
            let g = i[1];
            let b = i[0];
            let a = if let Some((tr, tg, tb)) = self.transparent_rgb {
                if r == tr && g == tg && b == tb {
                    0
                } else {
                    255
                }
            } else {
                255
            };
            *o = crate::effects::apply_effect_blend(
                effect_index,
                r,
                g,
                b,
                a,
                x,
                y,
                w,
                h,
                t,
                zebra_offset,
                blend_mode,
            );
            x += 1;
            if x >= w {
                x = 0;
                y += 1;
            }
        }
        if let Some(texture) = &mut self.texture {
            texture.set(image, egui::TextureOptions::NEAREST);
        } else {
            self.texture = Some(ctx.load_texture("frame", image, egui::TextureOptions::NEAREST));
        }
    }
    pub fn draw(&self, ui: &mut egui::Ui) {
        let window_size = ui.available_size();
        let rom_size = egui::Vec2::new(self.size.0 as f32, self.size.1 as f32) * self.scale;
        let fit_mode = self.config.fit_mode.as_str();
        let (_rect_id, _full_rect) = ui.allocate_space(window_size);
        let (draw_size, offset) = match fit_mode {
            "cover" => {
                let scale = f32::max(window_size.x / rom_size.x, window_size.y / rom_size.y);
                let size = rom_size * scale;
                let offset = (window_size - size) / 2.0;
                (size, offset)
            }
            "stretch" => (window_size, egui::Vec2::ZERO),
            "none" => {
                let size = egui::Vec2::new(
                    self.size.0 as f32 * self.scale,
                    self.size.1 as f32 * self.scale,
                );
                (size, egui::Vec2::ZERO)
            }
            _ => {
                let scale = f32::min(window_size.x / rom_size.x, window_size.y / rom_size.y);
                let size = rom_size * scale;
                let offset = (window_size - size) / 2.0;
                (size, offset)
            }
        };
        let rect = egui::Rect::from_min_size(ui.min_rect().min + offset, draw_size);
        if let Some(texture) = &self.texture {
            let mut mesh = egui::Mesh::with_texture(texture.id());
            mesh.add_rect_with_uv(
                rect,
                egui::Rect {
                    min: egui::Pos2::new(0.0, 0.0),
                    max: egui::Pos2::new(1.0, 1.0),
                },
                egui::Color32::WHITE,
            );
            ui.painter().add(egui::Shape::mesh(mesh));
        }
    }

    pub fn handle_usb_input(&mut self) {
        #[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
        {
            if let Some(controller_usb) = self.dev.controller_usb_mut() {
                let events = varvara::controller_usb::ControllerPollEvents::poll_usb_events(
                    controller_usb,
                    &mut self.vm,
                );
                for event in events {
                    self.dev.process_event(&mut self.vm, event);
                }
            }
        }
        // let k = varvara::Key::Right;
        // let repeat = false;
        // self.dev.pressed(&mut self.vm, k, repeat);
        // self.dev.released(&mut self.vm, k);
        // let k = varvara::Key::Char(b'A');
        //  self.dev
        //      .pressed(&mut self.vm, varvara::Key::Char(b'a'), false);
        // self.dev.char(&mut self.vm, b'a');
    }

    /// Load symbols from a byte slice (for embedded .sym files)
    pub fn load_symbols(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let map = varvara::Varvara::parse_symbols_from_bytes(data)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        self.dev.symbols = Some(map);
        Ok(())
    }
    /// Load a ROM from a file path and attempt to load a .sys symbol file if present
    pub fn load_rom_with_path<P: AsRef<std::path::Path>>(&mut self, path: P) {
        let path = path.as_ref();
        if let Ok(data) = std::fs::read(path) {
            let _ = self.vm.reset(&data);
            self.dev.reset(&data);
            self.vm.run(&mut self.dev, 0x100);
            let out = self.dev.output(&self.vm);
            self.size = out.size;
            // Look for a .sys file with the same file name as the ROM, e.g. orca.rom -> orca.rom.sys
            if let Some(file_name) = path.file_name() {
                use std::ffi::OsString;
                use std::path::PathBuf;
                let mut sys_file = OsString::from(file_name);
                sys_file.push(".sym");
                let mut sys_path = PathBuf::from(path);
                sys_path.set_file_name(sys_file);
                #[cfg(windows)]
                let mut sys_path_str = sys_path.to_string_lossy().to_string();
                #[cfg(windows)]
                {
                    sys_path_str = sys_path_str.replace("/", "\\");
                }
                #[cfg(not(windows))]
                let sys_path_str = sys_path.to_string_lossy().to_string();
                println!("[DEBUG][Stage] Attempting to load symbols from {sys_path_str}");
                if sys_path.exists() {
                    let _ = self.dev.load_symbols_into_self(&sys_path_str);
                    println!("[DEBUG][Stage] Loaded symbols from {sys_path_str}");
                }
            }
        }
    }

    /// Handle mouse events only (for hover)
    pub fn handle_mouse_input(
        &mut self,
        _ctx: &egui::Context,
        input: &egui::InputState,
        _response: &egui::Response,
        rect: egui::Rect,
    ) {
        if let Some(mouse_pos) = input.pointer.hover_pos() {
            if rect.contains(mouse_pos) {
                let rel_x = (mouse_pos.x - rect.min.x).max(0.0).min(rect.width());
                let rel_y = (mouse_pos.y - rect.min.y).max(0.0).min(rect.height());
                let mut buttons = 0u8;
                if input.pointer.button_down(egui::PointerButton::Primary) {
                    buttons |= 1;
                }
                if input.pointer.button_down(egui::PointerButton::Middle) {
                    buttons |= 2;
                }
                if input.pointer.button_down(egui::PointerButton::Secondary) {
                    buttons |= 4;
                }
                let mouse_state = varvara::MouseState {
                    pos: (rel_x, rel_y),
                    scroll: (0.0, 0.0),
                    buttons,
                };
                self.dev.mouse(&mut self.vm, mouse_state);

                // Window drag logic
                // Drag logic is handled in update() via StartDrag
            }
        }
        // Also handle pointer button events
        for event in &input.events {
            if let egui::Event::PointerButton {
                pos,
                button,
                pressed,
                ..
            } = event
            {
                if rect.contains(*pos) {
                    let rel_x = (pos.x - rect.min.x).max(0.0).min(rect.width());
                    let rel_y = (pos.y - rect.min.y).max(0.0).min(rect.height());
                    let mut buttons = 0u8;
                    if *button == egui::PointerButton::Primary && *pressed {
                        buttons |= 1;
                    }
                    if *button == egui::PointerButton::Middle && *pressed {
                        buttons |= 2;
                    }
                    if *button == egui::PointerButton::Secondary && *pressed {
                        buttons |= 4;
                    }
                    let mouse_state = varvara::MouseState {
                        pos: (rel_x, rel_y),
                        scroll: (0.0, 0.0),
                        buttons,
                    };
                    self.dev.mouse(&mut self.vm, mouse_state);
                }
            }
        }
    }

    /// Handle keyboard events only (for focus)
    pub fn handle_keyboard_input(&mut self, input: &egui::InputState) {
        for event in &input.events {
            if let egui::Event::Key { key, pressed, .. } = event {
                if let Some(varvara_key) = map_egui_key_to_varvara_key(*key) {
                    println!("[DEBUG][Stage] Forwarding key: {key:?} pressed={pressed} to VM as {varvara_key:?}");
                    if *pressed {
                        self.dev.pressed(&mut self.vm, varvara_key, false);
                    } else {
                        self.dev.released(&mut self.vm, varvara_key);
                    }
                } else {
                    println!("[DEBUG][Stage] Ignored key: {key:?}");
                }
            }
        }
    }

    /// Draw mesh at a specific rect (used by UxnPanel)
    pub fn draw_at(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let panel_size = egui::Vec2::new(self.size.0 as f32, self.size.1 as f32) * self.scale;
        if let Some(texture) = &self.texture {
            let tex_size = texture.size();
            println!("[draw_at] Texture size: {:?}", tex_size);
            let mut mesh = egui::Mesh::with_texture(texture.id());
            mesh.add_rect_with_uv(
                egui::Rect {
                    min: rect.min,
                    max: rect.min + panel_size,
                },
                egui::Rect {
                    min: egui::Pos2::new(0.0, 0.0),
                    max: egui::Pos2::new(1.0, 1.0),
                },
                egui::Color32::WHITE,
            );
            ui.painter().add(egui::Shape::mesh(mesh));
        }
    }

    /// Handle egui input events and forward to VM/device
    pub fn handle_input(&mut self, input: &egui::InputState, rect: egui::Rect) {
        // Forward text events as char events to the VM/device (for printable chars)
        for event in &input.events {
            if let egui::Event::Text(s) = event {
                for c in s.chars() {
                    // Only send printable ASCII (or adjust as needed for your ROM)
                    if c.is_ascii_graphic() || c == ' ' {
                        let byte = c as u8;
                        println!("[DEBUG][Stage] Forwarding char event: '{c}' (0x{byte:02x}) to VM (from egui::Event::Text)");
                        self.dev
                            .pressed(&mut self.vm, varvara::Key::Char(byte), false);
                        self.dev.char(&mut self.vm, byte);
                    } else {
                        println!(
                            "[DEBUG][Stage] Ignored char event: '{}' (0x{:02x}) (not printable)",
                            c, c as u8
                        );
                    }
                }
            }
        }
        // Only send key events for non-printable keys (arrows, enter, etc)
        use egui::Key;
        for event in &input.events {
            if let egui::Event::Key { key, pressed, .. } = event {
                let is_printable = matches!(
                    key,
                    Key::A
                        | Key::B
                        | Key::C
                        | Key::D
                        | Key::E
                        | Key::F
                        | Key::G
                        | Key::H
                        | Key::I
                        | Key::J
                        | Key::K
                        | Key::L
                        | Key::M
                        | Key::N
                        | Key::O
                        | Key::P
                        | Key::Q
                        | Key::R
                        | Key::S
                        | Key::T
                        | Key::U
                        | Key::V
                        | Key::W
                        | Key::X
                        | Key::Y
                        | Key::Z
                        | Key::Num0
                        | Key::Num1
                        | Key::Num2
                        | Key::Num3
                        | Key::Num4
                        | Key::Num5
                        | Key::Num6
                        | Key::Num7
                        | Key::Num8
                        | Key::Num9
                        | Key::Space
                        | Key::Tab
                        | Key::Backtick
                        | Key::Backslash
                        | Key::Pipe
                        | Key::Comma
                        | Key::Equals
                        | Key::Plus
                        | Key::OpenBracket
                        | Key::Minus
                        | Key::Period
                        | Key::CloseBracket
                        | Key::Semicolon
                        | Key::Colon
                        | Key::Slash
                        | Key::Questionmark
                );
                if !is_printable {
                    if let Some(varvara_key) = map_egui_key_to_varvara_key(*key) {
                        println!("[DEBUG][Stage] Forwarding non-printable key: {key:?} pressed={pressed} to VM as {varvara_key:?}");
                        if *pressed {
                            self.dev.pressed(&mut self.vm, varvara_key, false);
                        } else {
                            self.dev.released(&mut self.vm, varvara_key);
                        }
                    } else {
                        println!("[DEBUG][Stage] Ignored key: {key:?}");
                    }
                }
            }
        }
        // Forward mouse position and button state
        if let Some(mouse_pos) = input.pointer.hover_pos() {
            if rect.contains(mouse_pos) {
                let rel_x = (mouse_pos.x - rect.min.x).max(0.0).min(rect.width());
                let rel_y = (mouse_pos.y - rect.min.y).max(0.0).min(rect.height());
                let mut buttons = 0u8;
                if input.pointer.button_down(egui::PointerButton::Primary) {
                    buttons |= 1;
                }
                if input.pointer.button_down(egui::PointerButton::Middle) {
                    buttons |= 2;
                }
                if input.pointer.button_down(egui::PointerButton::Secondary) {
                    buttons |= 4;
                }
                let mouse_state = varvara::MouseState {
                    pos: (rel_x, rel_y),
                    scroll: (0.0, 0.0),
                    buttons,
                };
                self.dev.mouse(&mut self.vm, mouse_state);
            }
        }
        // Forward pointer button events (for clicks)
        for event in &input.events {
            if let egui::Event::PointerButton {
                pos,
                button,
                pressed,
                ..
            } = event
            {
                if rect.contains(*pos) {
                    let rel_x = (pos.x - rect.min.x).max(0.0).min(rect.width());
                    let rel_y = (pos.y - rect.min.y).max(0.0).min(rect.height());
                    let mut buttons = 0u8;
                    if *button == egui::PointerButton::Primary && *pressed {
                        buttons |= 1;
                    }
                    if *button == egui::PointerButton::Middle && *pressed {
                        buttons |= 2;
                    }
                    if *button == egui::PointerButton::Secondary && *pressed {
                        buttons |= 4;
                    }
                    let mouse_state = varvara::MouseState {
                        pos: (rel_x, rel_y),
                        scroll: (0.0, 0.0),
                        buttons,
                    };
                    println!("[DEBUG][Stage] Forwarding pointer button: {button:?} pressed={pressed} at rel=({rel_x:.1},{rel_y:.1})");
                    self.dev.mouse(&mut self.vm, mouse_state);
                }
            }
        }
    }

    /// Gracefully shutdown the stage (stop emulation and mute audio)
    pub fn shutdown(&mut self) {
        // mark stopped to prevent further updates
        self.stopped = true;
        // mute audio to avoid audio continuing after close
        self.dev.audio_set_muted(true);
    }
}

// Helper: Map egui::Key to Varvara::Key (basic ASCII)
fn map_egui_key_to_varvara_key(key: egui::Key) -> Option<varvara::Key> {
    use egui::Key;
    use varvara::Key as VKey;
    Some(match key {
        Key::ArrowUp => VKey::Up,
        Key::ArrowDown => VKey::Down,
        Key::ArrowLeft => VKey::Left,
        Key::ArrowRight => VKey::Right,
        Key::Home => VKey::Home,
        Key::End => VKey::End,
        Key::Tab => VKey::Char(b'\t'),
        Key::Backspace => VKey::Char(8),
        Key::Enter => VKey::Char(b'\n'),
        Key::Space => VKey::Char(b' '),
        Key::A => VKey::Char(b'a'),
        Key::B => VKey::Char(b'b'),
        Key::C => VKey::Char(b'c'),
        Key::D => VKey::Char(b'd'),
        Key::E => VKey::Char(b'e'),
        Key::F => VKey::Char(b'f'),
        Key::G => VKey::Char(b'g'),
        Key::H => VKey::Char(b'h'),
        Key::I => VKey::Char(b'i'),
        Key::J => VKey::Char(b'j'),
        Key::K => VKey::Char(b'k'),
        Key::L => VKey::Char(b'l'),
        Key::M => VKey::Char(b'm'),
        Key::N => VKey::Char(b'n'),
        Key::O => VKey::Char(b'o'),
        Key::P => VKey::Char(b'p'),
        Key::Q => VKey::Char(b'q'),
        Key::R => VKey::Char(b'r'),
        Key::S => VKey::Char(b's'),
        Key::T => VKey::Char(b't'),
        Key::U => VKey::Char(b'u'),
        Key::V => VKey::Char(b'v'),
        Key::W => VKey::Char(b'w'),
        Key::X => VKey::Char(b'x'),
        Key::Y => VKey::Char(b'y'),
        Key::Z => VKey::Char(b'z'),
        Key::Num0 => VKey::Char(b'0'),
        Key::Num1 => VKey::Char(b'1'),
        Key::Num2 => VKey::Char(b'2'),
        Key::Num3 => VKey::Char(b'3'),
        Key::Num4 => VKey::Char(b'4'),
        Key::Num5 => VKey::Char(b'5'),
        Key::Num6 => VKey::Char(b'6'),
        Key::Num7 => VKey::Char(b'7'),
        Key::Num8 => VKey::Char(b'8'),
        Key::Num9 => VKey::Char(b'9'),
        _ => return None,
    })
}

impl<'a> eframe::App for Stage<'a> {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Debug output for rendering and scaling
        let available = ctx.available_rect().size();
        let rom_size = egui::Vec2::new(self.size.0 as f32, self.size.1 as f32) * self.scale;
        let fit_mode = self.config.fit_mode.as_str();
        let (_draw_size, _offset) = match fit_mode {
            "cover" => {
                let scale = f32::max(available.x / rom_size.x, available.y / rom_size.y);
                let size = rom_size * scale;
                let offset = (available - size) / 2.0;
                (size, offset)
            }
            "stretch" => (available, egui::Vec2::ZERO),
            "none" => {
                let size = egui::Vec2::new(
                    self.size.0 as f32 * self.scale,
                    self.size.1 as f32 * self.scale,
                );
                (size, egui::Vec2::ZERO)
            }
            _ => {
                let scale = f32::min(available.x / rom_size.x, available.y / rom_size.y);
                let size = rom_size * scale;
                let offset = (available - size) / 2.0;
                (size, offset)
            }
        };
        // Call draw to ensure rendering and debug output
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                self.draw(ui);
            });
        if self.stopped {
            return;
        }
        // Timeout/exit flag: close window gracefully like Ctrl+C
        if let Some(flag) = &self.should_exit {
            if flag.load(std::sync::atomic::Ordering::Relaxed) {
                self.shutdown();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
        }
        let input = ctx.input(|i| i.clone());
        // Ctrl+Alt+drag OR mouse drag mode: use StartDrag for native drag
        let ctrl_alt_drag =
            input.modifiers.ctrl && input.modifiers.alt && input.pointer.primary_down();
        let mouse_drag_mode = self.config.mouse_mode;
        let mouse_drag_now = match mouse_drag_mode {
            MouseDragMode::DoubleDrag => {
                input.pointer.button_down(egui::PointerButton::Primary)
                    && input.pointer.button_down(egui::PointerButton::Secondary)
            }
            // Only None and DoubleDrag are supported for window movement
            MouseDragMode::None => false,
        };
        // Mouse resize logic: if mouse_resize is RightDrag and right button is down, resize window
        if self.config.mouse_resize == MouseResizeMode::MiddleDrag {
            if input.pointer.button_down(egui::PointerButton::Middle) {
                log::info!("Middle mouse button down for resize");
                if let Some(pos) = input.pointer.interact_pos() {
                    log::info!("interact_pos: {:?}", pos);
                    if self.drag_start_pos.is_none() {
                        log::info!("Starting middle drag at {:?}", pos);
                        self.drag_start_pos = Some(pos);
                        self.resize_start_size = Some(self.size);
                    }
                    if let (Some(start_pos), Some((start_w, start_h))) =
                        (self.drag_start_pos, self.resize_start_size)
                    {
                        let delta = pos - start_pos;
                        let new_w = (start_w as f32 + delta.x).max(100.0) as u16;
                        let new_h = (start_h as f32 + delta.y).max(100.0) as u16;
                        log::info!(
                            "Resizing: start=({},{}) delta=({:.2},{:.2}) new=({}, {})",
                            start_w,
                            start_h,
                            delta.x,
                            delta.y,
                            new_w,
                            new_h
                        );
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                            new_w as f32,
                            new_h as f32,
                        )));
                        self.last_resize_size = Some((new_w, new_h));
                    }
                } else {
                    log::info!("interact_pos is None");
                }
            } else if self.resize_start_size.is_some() {
                log::info!("Middle mouse button released");
                if let Some((new_w, new_h)) = self.last_resize_size {
                    log::info!("Finalizing resize to ({}, {})", new_w, new_h);
                    self.size = (new_w, new_h);
                    if let Some(f) = self.resized.as_mut() {
                        f(new_w, new_h);
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                        new_w as f32,
                        new_h as f32,
                    )));
                }
                self.drag_start_pos = None;
                self.resize_start_size = None;
                self.last_resize_size = None;
            }
        } else {
            self.drag_start_pos = None;
        }
        let dragging_now = ctrl_alt_drag || mouse_drag_now;
        if dragging_now && !self.drag_started {
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            self.drag_started = true;
        }
        if !dragging_now {
            self.drag_started = false;
        }
        while let Ok(e) = self.event_rx.try_recv() {
            match e {
                Event::LoadRom(data) => {
                    if let Err(e) = self.load_rom(&data) {
                        error!("could not load rom: {e:?}");
                    }
                    self.update_texture(ctx);
                }
                Event::SetMuted(m) => {
                    self.dev.audio_set_muted(m);
                }
                Event::Console(b) => {
                    self.dev.console(&mut self.vm, b);
                }
            }
        }

        // Repaint at vsync rate (60 FPS)
        ctx.request_repaint();
        let mut next_efx_ndx = self.config.effects.efx_ndx;
        let efx_name = self
            .config
            .effects
            .efx
            .as_ref()
            .map(|s| s.to_ascii_lowercase());
        let is_random = efx_name.as_deref() == Some("random");
        let is_sequential = efx_name.as_deref() == Some("sequential");
        while input.time >= self.next_frame {
            self.next_frame += 0.0166667;
            self.dev.redraw(&mut self.vm);
            // Only cycle effect if efx is 'random' or 'sequential' and efxt dwell time has elapsed
            if is_random {
                if self.config.effects.last_effect_switch == 0.0
                    || input.time - self.config.effects.last_effect_switch
                        >= self.config.effects.efxt as f64
                {
                    // Use getrandom-based helper for random effect index
                    next_efx_ndx = random_helpers::random_range(0, crate::effects::EFFECT_COUNT);
                    self.config.effects.last_effect_switch = input.time;
                }
            } else if is_sequential
                && (self.config.effects.last_effect_switch == 0.0
                    || input.time - self.config.effects.last_effect_switch
                        >= self.config.effects.efxt as f64)
            {
                next_efx_ndx = (next_efx_ndx + 1) % crate::effects::EFFECT_COUNT;
                self.config.effects.last_effect_switch = input.time;
            }
            self.update_texture(ctx);
        }

        if is_random || is_sequential {
            self.config.effects.efx_ndx = next_efx_ndx;
        }

        if input.raw.dropped_files.len() == 1 {
            let target = &input.raw.dropped_files[0];
            let r = if let Some(path) = &target.path {
                let data = std::fs::read(path).expect("failed to read file");
                info!("loading {} bytes from {path:?}", data.len());
                self.load_rom(&data)
            } else if let Some(data) = &target.bytes {
                self.load_rom(data)
            } else {
                Ok(())
            };
            if let Err(e) = r {
                error!("could not load ROM: {e:?}");
            }
        }

        let shift_held = input.modifiers.shift;
        for e in input.events.iter() {
            // Handle global shortcuts before passing to emulator
            if let egui::Event::Key { key, pressed, .. } = e {
                println!(
                    "Key event received: {:?}, pressed: {}, ctrl: {}",
                    key, pressed, input.modifiers.ctrl
                );
                if *key == egui::Key::C && input.modifiers.ctrl {
                    println!("Ctrl+C handler triggered");
                    self.shutdown();
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return;
                }
            }
            // Only pass non-global shortcut events to emulator
            match e {
                egui::Event::Text(s) => {
                    const RAW_CHARS: [u8; 16] = [
                        b'"', b'\'', b'{', b'}', b'_', b')', b'(', b'*', b'&', b'^', b'%', b'$',
                        b'#', b'@', b'!', b'~',
                    ];
                    for c in s.bytes() {
                        if RAW_CHARS.contains(&c) {
                            self.dev.char(&mut self.vm, c);
                        }
                    }
                }
                egui::Event::Key {
                    key,
                    pressed,
                    repeat,
                    ..
                } => {
                    // Only handle Ctrl+C on key press, not release
                    println!(
                        "Key event: {:?}, pressed: {}, ctrl: {}",
                        key, pressed, input.modifiers.ctrl
                    );
                    if *pressed && *key == egui::Key::F2 {
                        self.dev.system.debug(&mut self.vm);
                        #[cfg(target_os = "windows")]
                        unsafe {
                            winapi::um::winuser::MessageBeep(winapi::um::winuser::MB_OK);
                        }
                    }
                    if let Some(k) = decode_key(*key, shift_held) {
                        if *pressed {
                            self.dev.pressed(&mut self.vm, k, *repeat);
                        } else {
                            self.dev.released(&mut self.vm, k);
                        }
                    }
                }
                egui::Event::MouseWheel { delta, .. } => {
                    self.scroll.0 += delta.x;
                    self.scroll.1 -= delta.y;
                }
                _ => (),
            }
        }
        for (b, k) in [
            (input.modifiers.ctrl, varvara::Key::Ctrl),
            (input.modifiers.alt, varvara::Key::Alt),
            (input.modifiers.shift, varvara::Key::Shift),
        ] {
            if b {
                self.dev.pressed(&mut self.vm, k, false)
            } else {
                self.dev.released(&mut self.vm, k)
            }
        }

        let ptr = &input.pointer;
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
        let m = varvara::MouseState {
            pos: self.cursor_pos.unwrap_or((0.0, 0.0)),
            scroll: std::mem::take(&mut self.scroll),
            buttons,
        };
        self.dev.mouse(&mut self.vm, m);
        let m = varvara::TrackerState {
            pos: self.cursor_pos.unwrap_or((0.0, 0.0)),
            scroll: std::mem::take(&mut self.scroll),
            buttons,
        };
        self.dev.tracker(&mut self.vm, m);

        // Handle audio callback
        self.dev.audio(&mut self.vm);

        let out = self.dev.output(&self.vm);

        // Update our GUI based on current state
        if out.hide_mouse {
            ctx.set_cursor_icon(egui::CursorIcon::None);
        }
        // Only update self.size from emulator output if not currently resizing
        // let resizing_now = self.config.mouse_resize == MouseResizeMode::MiddleDrag && input.pointer.button_down(egui::PointerButton::Middle);
        //     if self.size != out.size && !resizing_now {
        //         info!("resizing window to {:?}", out.size);
        //         self.size = out.size;
        //         let size = egui::Vec2::new(out.size.0 as f32, out.size.1 as f32) * self.scale;
        //         ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
        //         if let Some(f) = self.resized.as_mut() {
        //             f(out.size.0, out.size.1);
        //         }
        //     }

        // TODO reduce allocation here?
        let mut image = egui::ColorImage::new(
            [out.size.0 as usize, out.size.1 as usize],
            vec![egui::Color32::BLACK; (out.size.0 as usize) * (out.size.1 as usize)],
        );
        let mut x = 0;
        let mut y = 0;
        let w = out.size.0 as usize;
        let _t = 0.0;
        let num_effects = 16; // 4 variants for each of 4 effect types
        for (i, o) in out.frame.chunks(4).zip(image.pixels.iter_mut()) {
            let r = i[2];
            let g = i[1];
            let b = i[0];
            let a;
            if let Some((tr, tg, tb)) = self.transparent_rgb {
                if r == tr && g == tg && b == tb {
                    a = 0;
                } else {
                    a = 255;
                }
            } else {
                a = 255;
            }
            if a != 0 {
                let t = self.config.effects.efxt;
                let fx = x as f32;
                let fy = y as f32;
                let cx = (w as f32) / 2.0;
                let cy = (out.size.1 as f32) / 2.0;
                let dist = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt();
                let diag = (fx + fy) * 0.5;
                let eff =
                    self.config.effects.effect_order[self.config.effects.effect_mode % num_effects];
                // Print current effect for debug
                let _effect_name = match eff {
                    0 => "Plasma (horizontal)",
                    1 => "Plasma (vertical)",
                    2 => "Plasma (circle)",
                    3 => "Plasma (diagonal)",
                    4 => "Rainbow (horizontal)",
                    5 => "Rainbow (vertical)",
                    6 => "Rainbow (circle)",
                    7 => "Rainbow (diagonal)",
                    8 => "Waves (horizontal)",
                    9 => "Waves (vertical)",
                    10 => "Waves (circle)",
                    11 => "Waves (diagonal)",
                    12 => "Noise (horizontal)",
                    13 => "Noise (vertical)",
                    14 => "Noise (circle)",
                    15 => "Noise (diagonal)",
                    _ => "Unknown",
                };
                match eff {
                    0 => {
                        // Plasma (horizontal)
                        let v = ((fx * 0.08 + t * 0.12).sin() + (fy * 0.08 + t * 0.15).cos()) * 0.5;
                        let r = ((v * 127.0 + 128.0) as u8).saturating_add((t as u8) % 128);
                        let g = ((v * 127.0 + 128.0) as u8).saturating_add((x as u8) % 128);
                        let b = ((v * 127.0 + 128.0) as u8).saturating_add((y as u8) % 128);
                        *o = egui::Color32::from_rgba_unmultiplied(r, g, b, a);
                        continue;
                    }
                    1 => {
                        // Plasma (vertical)
                        let v = ((fy * 0.08 + t * 0.12).sin() + (fx * 0.08 + t * 0.15).cos()) * 0.5;
                        let r = ((v * 127.0 + 128.0) as u8).saturating_add((t as u8) % 128);
                        let g = ((v * 127.0 + 128.0) as u8).saturating_add((y as u8) % 128);
                        let b = ((v * 127.0 + 128.0) as u8).saturating_add((x as u8) % 128);
                        *o = egui::Color32::from_rgba_unmultiplied(r, g, b, a);
                        continue;
                    }
                    2 => {
                        // Plasma (circle, enhanced)
                        let theta = (fy - cy).atan2(fx - cx);
                        let v =
                            ((dist * 0.15 + t * 0.12).sin() + (theta * 6.0 + t * 0.2).cos()) * 0.5;
                        let r = ((v * 127.0 + 128.0) as u8)
                            .saturating_add((theta.sin() * 127.0 + 128.0) as u8);
                        let g = ((v * 127.0 + 128.0) as u8)
                            .saturating_add((theta.cos() * 127.0 + 128.0) as u8);
                        let b = ((v * 127.0 + 128.0) as u8)
                            .saturating_add((dist.sin() * 127.0 + 128.0) as u8);
                        *o = egui::Color32::from_rgba_unmultiplied(r, g, b, a);
                        continue;
                    }
                    3 => {
                        // Plasma (diagonal)
                        let v = ((diag * 0.08 + t * 0.12).sin() + (t * 0.15).cos()) * 0.5;
                        let r = ((v * 127.0 + 128.0) as u8).saturating_add((t as u8) % 128);
                        let g = ((v * 127.0 + 128.0) as u8).saturating_add((x as u8) % 128);
                        let b = ((v * 127.0 + 128.0) as u8).saturating_add((y as u8) % 128);
                        *o = egui::Color32::from_rgba_unmultiplied(r, g, b, a);
                        continue;
                    }
                    4 => {
                        // Rainbow swirl (horizontal)
                        let r = ((fx * 0.12 + t * 0.2).sin() * 127.0 + 128.0) as u8;
                        let g = ((fy * 0.12 + t * 0.3).cos() * 127.0 + 128.0) as u8;
                        let b = (((fx + fy) * 0.07 + t * 0.4).sin() * 127.0 + 128.0) as u8;
                        *o = egui::Color32::from_rgba_unmultiplied(r, g, b, a);
                        continue;
                    }
                    5 => {
                        // Rainbow swirl (vertical)
                        let r = ((fy * 0.12 + t * 0.2).sin() * 127.0 + 128.0) as u8;
                        let g = ((fx * 0.12 + t * 0.3).cos() * 127.0 + 128.0) as u8;
                        let b = (((fx + fy) * 0.07 + t * 0.4).sin() * 127.0 + 128.0) as u8;
                        *o = egui::Color32::from_rgba_unmultiplied(r, g, b, a);
                        continue;
                    }
                    6 => {
                        // Rainbow swirl (circle, enhanced)
                        let theta = (fy - cy).atan2(fx - cx);
                        let r = ((dist * 0.18 + t * 0.2).sin() * 127.0 + 128.0) as u8;
                        let g = ((theta * 4.0 + t * 0.3).cos() * 127.0 + 128.0) as u8;
                        let b = ((dist * 0.09 + theta + t * 0.4).sin() * 127.0 + 128.0) as u8;
                        *o = egui::Color32::from_rgba_unmultiplied(r, g, b, a);
                        continue;
                    }
                    7 => {
                        // Rainbow swirl (diagonal)
                        let r = ((diag * 0.12 + t * 0.2).sin() * 127.0 + 128.0) as u8;
                        let g = ((diag * 0.12 + t * 0.3).cos() * 127.0 + 128.0) as u8;
                        let b = ((diag * 0.07 + t * 0.4).sin() * 127.0 + 128.0) as u8;
                        *o = egui::Color32::from_rgba_unmultiplied(r, g, b, a);
                        continue;
                    }
                    8 => {
                        // Waves (horizontal)
                        let v = ((fx * 0.2 + t * 0.25).sin() * 127.0 + 128.0) as u8;
                        *o = egui::Color32::from_rgba_unmultiplied(v, 255 - v, v / 2, a);
                        continue;
                    }
                    9 => {
                        // Waves (vertical)
                        let v = ((fy * 0.2 + t * 0.25).sin() * 127.0 + 128.0) as u8;
                        *o = egui::Color32::from_rgba_unmultiplied(v, 255 - v, v / 2, a);
                        continue;
                    }
                    10 => {
                        // Waves (circle, enhanced)
                        let v = ((dist * 0.25 + t * 0.25).sin() * 127.0 + 128.0) as u8;
                        let w = ((dist * 0.12 + t * 0.1).cos() * 127.0 + 128.0) as u8;
                        *o = egui::Color32::from_rgba_unmultiplied(v, 255 - w, w / 2, a);
                        continue;
                    }
                    11 => {
                        // Waves (diagonal)
                        let v = ((diag * 0.2 + t * 0.25).sin() * 127.0 + 128.0) as u8;
                        *o = egui::Color32::from_rgba_unmultiplied(v, 255 - v, v / 2, a);
                        continue;
                    }
                    12 => {
                        // Noise (horizontal)
                        let seed = (x as u32).wrapping_mul(73856093)
                            ^ (self.config.effects.effect_mode as u32).wrapping_mul(83492791);
                        let val = ((seed >> 3) ^ (seed << 7)).wrapping_add(seed) as u8;
                        *o = egui::Color32::from_rgba_unmultiplied(
                            val,
                            val.wrapping_mul(3),
                            val.wrapping_mul(7),
                            a,
                        );
                        continue;
                    }
                    13 => {
                        // Noise (vertical)
                        let seed = (y as u32).wrapping_mul(19349663)
                            ^ (self.config.effects.efx_ndx as u32).wrapping_mul(83492791);
                        let val = ((seed >> 3) ^ (seed << 7)).wrapping_add(seed) as u8;
                        *o = egui::Color32::from_rgba_unmultiplied(
                            val,
                            val.wrapping_mul(3),
                            val.wrapping_mul(7),
                            a,
                        );
                        continue;
                    }
                    14 => {
                        // Noise (circle, enhanced)
                        let theta = (fy - cy).atan2(fx - cx);
                        let seed = (dist as u32).wrapping_mul(1234567)
                            ^ (theta.to_bits().wrapping_mul(314159))
                            ^ (self.config.effects.effect_mode as u32).wrapping_mul(83492791);
                        let val = ((seed >> 3) ^ (seed << 7)).wrapping_add(seed) as u8;
                        *o = egui::Color32::from_rgba_unmultiplied(
                            val,
                            val.rotate_left(2),
                            val.rotate_left(4),
                            a,
                        );
                        continue;
                    }
                    15 => {
                        // Noise (diagonal)
                        let seed = (diag as u32).wrapping_mul(9876543)
                            ^ (self.config.effects.effect_mode as u32).wrapping_mul(83492791);
                        let val = ((seed >> 3) ^ (seed << 7)).wrapping_add(seed) as u8;
                        *o = egui::Color32::from_rgba_unmultiplied(
                            val,
                            val.wrapping_mul(3),
                            val.wrapping_mul(7),
                            a,
                        );
                        continue;
                    }
                    _ => {}
                }
            }
            *o = egui::Color32::from_rgba_unmultiplied(r, g, b, a);
            x += 1;
            if x >= w {
                x = 0;
                y += 1;
            }
        }

        // Update stdout / stderr / exiting
        out.check().expect("failed to print output?");
    }
}

fn decode_key(k: egui::Key, shift: bool) -> Option<Key> {
    let c = match (k, shift) {
        (egui::Key::ArrowUp, _) => Key::Up,
        (egui::Key::ArrowDown, _) => Key::Down,
        (egui::Key::ArrowLeft, _) => Key::Left,
        (egui::Key::ArrowRight, _) => Key::Right,
        (egui::Key::Home, _) => Key::Home,
        (egui::Key::Num0, false) => Key::Char(b'0'),
        (egui::Key::Num0, true) => Key::Char(b')'),
        (egui::Key::Num1, false) => Key::Char(b'1'),
        (egui::Key::Num1, true) => Key::Char(b'!'),
        (egui::Key::Num2, false) => Key::Char(b'2'),
        (egui::Key::Num2, true) => Key::Char(b'@'),
        (egui::Key::Num3, false) => Key::Char(b'3'),
        (egui::Key::Num3, true) => Key::Char(b'#'),
        (egui::Key::Num4, false) => Key::Char(b'4'),
        (egui::Key::Num4, true) => Key::Char(b'$'),
        (egui::Key::Num5, false) => Key::Char(b'5'),
        (egui::Key::Num5, true) => Key::Char(b'5'),
        (egui::Key::Num6, false) => Key::Char(b'6'),
        (egui::Key::Num6, true) => Key::Char(b'^'),
        (egui::Key::Num7, false) => Key::Char(b'7'),
        (egui::Key::Num7, true) => Key::Char(b'&'),
        (egui::Key::Num8, false) => Key::Char(b'8'),
        (egui::Key::Num8, true) => Key::Char(b'*'),
        (egui::Key::Num9, false) => Key::Char(b'9'),
        (egui::Key::Num9, true) => Key::Char(b'('),
        (egui::Key::A, false) => Key::Char(b'a'),
        (egui::Key::A, true) => Key::Char(b'A'),
        (egui::Key::B, false) => Key::Char(b'b'),
        (egui::Key::B, true) => Key::Char(b'B'),
        (egui::Key::C, false) => Key::Char(b'c'),
        (egui::Key::C, true) => Key::Char(b'C'),
        (egui::Key::D, false) => Key::Char(b'd'),
        (egui::Key::D, true) => Key::Char(b'D'),
        (egui::Key::E, false) => Key::Char(b'e'),
        (egui::Key::E, true) => Key::Char(b'E'),
        (egui::Key::F, false) => Key::Char(b'f'),
        (egui::Key::F, true) => Key::Char(b'F'),
        (egui::Key::G, false) => Key::Char(b'g'),
        (egui::Key::G, true) => Key::Char(b'G'),
        (egui::Key::H, false) => Key::Char(b'h'),
        (egui::Key::H, true) => Key::Char(b'H'),
        (egui::Key::I, false) => Key::Char(b'i'),
        (egui::Key::I, true) => Key::Char(b'I'),
        (egui::Key::J, false) => Key::Char(b'j'),
        (egui::Key::J, true) => Key::Char(b'J'),
        (egui::Key::K, false) => Key::Char(b'k'),
        (egui::Key::K, true) => Key::Char(b'K'),
        (egui::Key::L, false) => Key::Char(b'l'),
        (egui::Key::L, true) => Key::Char(b'L'),
        (egui::Key::M, false) => Key::Char(b'm'),
        (egui::Key::M, true) => Key::Char(b'M'),
        (egui::Key::N, false) => Key::Char(b'n'),
        (egui::Key::N, true) => Key::Char(b'N'),
        (egui::Key::O, false) => Key::Char(b'o'),
        (egui::Key::O, true) => Key::Char(b'O'),
        (egui::Key::P, false) => Key::Char(b'p'),
        (egui::Key::P, true) => Key::Char(b'P'),
        (egui::Key::Q, false) => Key::Char(b'q'),
        (egui::Key::Q, true) => Key::Char(b'Q'),
        (egui::Key::R, false) => Key::Char(b'r'),
        (egui::Key::R, true) => Key::Char(b'R'),
        (egui::Key::S, false) => Key::Char(b's'),
        (egui::Key::S, true) => Key::Char(b'S'),
        (egui::Key::T, false) => Key::Char(b't'),
        (egui::Key::T, true) => Key::Char(b'T'),
        (egui::Key::U, false) => Key::Char(b'u'),
        (egui::Key::U, true) => Key::Char(b'U'),
        (egui::Key::V, false) => Key::Char(b'v'),
        (egui::Key::V, true) => Key::Char(b'V'),
        (egui::Key::W, false) => Key::Char(b'w'),
        (egui::Key::W, true) => Key::Char(b'W'),
        (egui::Key::X, false) => Key::Char(b'x'),
        (egui::Key::X, true) => Key::Char(b'X'),
        (egui::Key::Y, false) => Key::Char(b'y'),
        (egui::Key::Y, true) => Key::Char(b'Y'),
        (egui::Key::Z, false) => Key::Char(b'z'),
        (egui::Key::Z, true) => Key::Char(b'Z'),
        // TODO missing Key::Quote
        (egui::Key::Backtick, false) => Key::Char(b'`'),
        (egui::Key::Backtick, true) => Key::Char(b'~'),
        (egui::Key::Backslash, _) => Key::Char(b'\\'),
        (egui::Key::Pipe, _) => Key::Char(b'|'),
        (egui::Key::Comma, false) => Key::Char(b','),
        (egui::Key::Comma, true) => Key::Char(b'<'),
        (egui::Key::Equals, _) => Key::Char(b'='),
        (egui::Key::Plus, _) => Key::Char(b'+'),
        (egui::Key::OpenBracket, false) => Key::Char(b'['),
        (egui::Key::OpenBracket, true) => Key::Char(b'{'),
        (egui::Key::Minus, false) => Key::Char(b'-'),
        (egui::Key::Minus, true) => Key::Char(b'_'),
        (egui::Key::Period, false) => Key::Char(b'.'),
        (egui::Key::Period, true) => Key::Char(b'>'),
        (egui::Key::CloseBracket, false) => Key::Char(b']'),
        (egui::Key::CloseBracket, true) => Key::Char(b'}'),
        (egui::Key::Semicolon, _) => Key::Char(b';'),
        (egui::Key::Colon, _) => Key::Char(b':'),
        (egui::Key::Slash, _) => Key::Char(b'/'),
        (egui::Key::Questionmark, _) => Key::Char(b'?'),
        (egui::Key::Space, _) => Key::Char(b' '),
        (egui::Key::Tab, _) => Key::Char(b'\t'),
        (egui::Key::Enter, _) => Key::Char(b'\r'),
        (egui::Key::Backspace, _) => Key::Char(0x08),
        _ => return None,
    };
    Some(c)
}

mod random_helpers {
    use getrandom::fill;

    pub fn random_range(start: usize, end: usize) -> usize {
        let mut bytes = [0u8; 8];
        fill(&mut bytes).expect("getrandom failed");
        let val = u64::from_le_bytes(bytes);
        let range = end - start;
        start + (val as usize % range)
    }
}
