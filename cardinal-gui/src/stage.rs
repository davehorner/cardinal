impl<'a> Stage<'a> {
    /// Load symbols from a byte slice (for embedded .sym files)
    pub fn load_symbols(&mut self, data: &[u8]) {
        if let Ok(map) = varvara::Varvara::parse_symbols_from_bytes(data) {
            self.dev.symbols = Some(map);
        }
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
                println!(
                    "[DEBUG][Stage] Attempting to load symbols from {sys_path_str}");
                if sys_path.exists() {
                    let _ = self.dev.load_symbols_into_self(&sys_path_str);
                    println!(
                        "[DEBUG][Stage] Loaded symbols from {sys_path_str}"
                    );
                }
            }
        }
    }

    /// Handle mouse events only (for hover)
    pub fn handle_mouse_input(
        &mut self,
        input: &egui::InputState,
        _response: &egui::Response,
        rect: egui::Rect,
    ) {
        if let Some(mouse_pos) = input.pointer.hover_pos() {
            if rect.contains(mouse_pos) {
                let rel_x =
                    (mouse_pos.x - rect.min.x).max(0.0).min(rect.width());
                let rel_y =
                    (mouse_pos.y - rect.min.y).max(0.0).min(rect.height());
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
                    let rel_y =
                        (pos.y - rect.min.y).max(0.0).min(rect.height());
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
        let panel_size =
            egui::Vec2::new(self.size.0 as f32, self.size.1 as f32)
                * self.scale;
        let mut mesh = egui::Mesh::with_texture(self.texture.id());
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

    /// Handle egui input events and forward to VM/device
    pub fn handle_input(
        &mut self,
        input: &egui::InputState,
        _response: &egui::Response,
        rect: egui::Rect,
    ) {
        // Forward text events as char events to the VM/device (for printable chars)
        for event in &input.events {
            if let egui::Event::Text(s) = event {
                for c in s.chars() {
                    // Only send printable ASCII (or adjust as needed for your ROM)
                    if c.is_ascii_graphic() || c == ' ' {
                        let byte = c as u8;
                        println!("[DEBUG][Stage] Forwarding char event: '{c}' (0x{byte:02x}) to VM (from egui::Event::Text)");
                        self.dev.pressed(
                            &mut self.vm,
                            varvara::Key::Char(byte),
                            false,
                        );
                        self.dev.char(&mut self.vm, byte);
                    } else {
                        println!("[DEBUG][Stage] Ignored char event: '{}' (0x{:02x}) (not printable)", c, c as u8);
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
                    if let Some(varvara_key) = map_egui_key_to_varvara_key(*key)
                    {
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
                let rel_x =
                    (mouse_pos.x - rect.min.x).max(0.0).min(rect.width());
                let rel_y =
                    (mouse_pos.y - rect.min.y).max(0.0).min(rect.height());
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
                    let rel_y =
                        (pos.y - rect.min.y).max(0.0).min(rect.height());
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
use eframe::egui;
use std::sync::mpsc;
use uxn::Uxn;
use varvara::Varvara;

#[derive(Debug)]
pub enum Event {
    LoadRom(Vec<u8>),
    SetMuted(bool),
    Console(u8),
}

pub struct Stage<'a> {
    pub vm: Uxn<'a>,
    pub dev: Varvara,
    pub scale: f32,
    pub size: (u16, u16),
    pub next_frame: f64,
    pub scroll: (f32, f32),
    pub cursor_pos: Option<(f32, f32)>,
    pub texture: egui::TextureHandle,
    pub event_rx: mpsc::Receiver<Event>,
    pub resized: Option<Box<dyn FnMut(u16, u16)>>,
}

impl<'a> Stage<'a> {
    pub fn new(
        vm: Uxn<'a>,
        dev: Varvara,
        size: (u16, u16),
        scale: f32,
        event_rx: mpsc::Receiver<Event>,
        ctx: &egui::Context,
        texture_name: String,
    ) -> Self {
        let image = egui::ColorImage::new(
            [usize::from(size.0), usize::from(size.1)],
            vec![egui::Color32::BLACK; (size.0 as usize) * (size.1 as usize)],
        );
        let texture = ctx.load_texture(
            &texture_name,
            image,
            egui::TextureOptions::NEAREST,
        );
        Stage {
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
        }
    }
    pub fn set_resize_callback(&mut self, f: Box<dyn FnMut(u16, u16)>) {
        self.resized = Some(f);
    }
    pub fn load_rom(&mut self, data: &[u8]) {
        let _ = self.vm.reset(data);
        self.dev.reset(data);
        self.vm.run(&mut self.dev, 0x100);
        let out = self.dev.output(&self.vm);
        self.size = out.size;
    }
    pub fn step(&mut self) {
        self.vm.run(&mut self.dev, 0x100);
        self.dev.redraw(&mut self.vm);
    }
    pub fn update_texture(&mut self) {
        let out = self.dev.output(&self.vm);
        let mut image = egui::ColorImage::new(
            [out.size.0 as usize, out.size.1 as usize],
            vec![
                egui::Color32::BLACK;
                (out.size.0 as usize) * (out.size.1 as usize)
            ],
        );
        for (i, o) in out.frame.chunks(4).zip(image.pixels.iter_mut()) {
            *o = egui::Color32::from_rgba_unmultiplied(i[0], i[1], i[2], i[3]);
        }
        self.texture.set(image, egui::TextureOptions::NEAREST);
    }
    pub fn draw(&self, ui: &mut egui::Ui) {
        let panel_size =
            egui::Vec2::new(self.size.0 as f32, self.size.1 as f32)
                * self.scale;
        let (_rect_id, rect) = ui.allocate_space(panel_size);
        let mut mesh = egui::Mesh::with_texture(self.texture.id());
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

    pub fn handle_usb_input(&mut self) {
        // let k = varvara::Key::Right;
        // let repeat = false;
        // self.dev.pressed(&mut self.vm, k, repeat);
        // self.dev.released(&mut self.vm, k);
        // let k = varvara::Key::Char(b'A');
        self.dev
            .pressed(&mut self.vm, varvara::Key::Char(b'a'), false);
        self.dev.char(&mut self.vm, b'a');
    }
}
