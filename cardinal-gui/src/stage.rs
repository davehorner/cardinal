use log::{error, info};
use std::sync::mpsc;
use uxn::Uxn;
use varvara::{Key, Varvara};

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
    pub texture: Option<egui::TextureHandle>,
    pub event_rx: mpsc::Receiver<Event>,
    pub resized: Option<Box<dyn FnMut(u16, u16)>>,
    pub transparent_rgb: Option<(u8, u8, u8)>,
    pub stopped: bool,
    pub drag_started: bool,
}

impl<'a> Stage<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        vm: Uxn<'a>,
        dev: Varvara,
        size: (u16, u16),
        scale: f32,
        event_rx: mpsc::Receiver<Event>,
        ctx: &egui::Context,
        _rom_title: String,
        transparent_color: Option<String>,
    ) -> Self {
        let image = egui::ColorImage::new(
            [usize::from(size.0), usize::from(size.1)],
            vec![egui::Color32::BLACK; (size.0 as usize) * (size.1 as usize)],
        );
        let texture = Some(ctx.load_texture("frame", image, egui::TextureOptions::NEAREST));
        let transparent_rgb = transparent_color.as_ref().and_then(|s| {
            if s.len() == 6 {
                let r = u8::from_str_radix(&s[0..2], 16).ok()?;
                let g = u8::from_str_radix(&s[2..4], 16).ok()?;
                let b = u8::from_str_radix(&s[4..6], 16).ok()?;
                Some((r, g, b))
            } else {
                None
            }
        });
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
            transparent_rgb,
            stopped: false,
            drag_started: false,
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
        let out = self.dev.output(&self.vm);
        let mut image = egui::ColorImage::new(
            [out.size.0 as usize, out.size.1 as usize],
            vec![egui::Color32::BLACK; (out.size.0 as usize) * (out.size.1 as usize)],
        );
        for (i, o) in out.frame.chunks(4).zip(image.pixels.iter_mut()) {
            // Fix channel order: assume frame is BGRA, convert to RGBA
            *o = egui::Color32::from_rgba_unmultiplied(i[2], i[1], i[0], i[3]);
        }
        if let Some(texture) = &mut self.texture {
            texture.set(image, egui::TextureOptions::NEAREST);
        } else {
            self.texture = Some(ctx.load_texture("frame", image, egui::TextureOptions::NEAREST));
        }
    }
    pub fn draw(&self, ui: &mut egui::Ui) {
        let panel_size = egui::Vec2::new(self.size.0 as f32, self.size.1 as f32) * self.scale;
        let (_rect_id, rect) = ui.allocate_space(panel_size);
        if let Some(texture) = &self.texture {
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
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 0),
            );
            ui.painter().add(egui::Shape::mesh(mesh));
        }
    }

    pub fn handle_usb_input(&mut self) {
        if let Some(controller_usb) = self.dev.controller_usb_mut() {
            let events = varvara::controller_usb::ControllerPollEvents::poll_usb_events(
                controller_usb,
                &mut self.vm,
            );
            for event in events {
                self.dev.process_event(&mut self.vm, event);
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
        if self.stopped {
            return;
        }
        let input = ctx.input(|i| i.clone());
        // Ctrl+drag window move: use StartDrag for native drag, no manual offset or snap
        let dragging_now =
            input.modifiers.ctrl && input.modifiers.alt && input.pointer.primary_down();
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
        while input.time >= self.next_frame {
            self.next_frame += 0.0166667;
            self.dev.redraw(&mut self.vm);
            self.update_texture(ctx);
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
        if self.size != out.size {
            info!("resizing window to {:?}", out.size);
            self.size = out.size;
            let size = egui::Vec2::new(out.size.0 as f32, out.size.1 as f32) * self.scale;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            if let Some(f) = self.resized.as_mut() {
                f(out.size.0, out.size.1);
            }
        }

        // TODO reduce allocation here?
        let mut image = egui::ColorImage::new(
            [out.size.0 as usize, out.size.1 as usize],
            vec![egui::Color32::BLACK; (out.size.0 as usize) * (out.size.1 as usize)],
        );
        if let Some((r, g, b)) = self.transparent_rgb {
            // Make only the specified color transparent
            for (i, o) in out.frame.chunks(4).zip(image.pixels.iter_mut()) {
                if i[2] == r && i[1] == g && i[0] == b {
                    *o = egui::Color32::from_rgba_unmultiplied(i[2], i[1], i[0], 0);
                } else {
                    *o = egui::Color32::from_rgba_unmultiplied(i[2], i[1], i[0], i[3]);
                }
            }
        } else {
            // Render all pixels as-is (white is visible)
            for (i, o) in out.frame.chunks(4).zip(image.pixels.iter_mut()) {
                *o = egui::Color32::from_rgba_unmultiplied(i[2], i[1], i[0], i[3]);
            }
        }
        if let Some(texture) = &mut self.texture {
            texture.set(image, egui::TextureOptions::NEAREST);
        }
        let panel_frame = egui::Frame::default().fill(egui::Color32::TRANSPARENT);
        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |_ui| {
                let panel_frame = egui::Frame::default().fill(egui::Color32::TRANSPARENT);
                egui::CentralPanel::default()
                    .frame(panel_frame)
                    .show(ctx, |_ui| {
                        let panel_frame = egui::Frame::default().fill(egui::Color32::TRANSPARENT);
                        egui::CentralPanel::default()
                            .frame(panel_frame)
                            .show(ctx, |ui| {
                                if let Some(texture) = &self.texture {
                                    let mut mesh = egui::Mesh::with_texture(texture.id());
                                    mesh.add_rect_with_uv(
                                        egui::Rect {
                                            min: egui::Pos2::new(0.0, 0.0),
                                            max: egui::Pos2::new(
                                                out.size.0 as f32 * self.scale,
                                                out.size.1 as f32 * self.scale,
                                            ),
                                        },
                                        egui::Rect {
                                            min: egui::Pos2::new(0.0, 0.0),
                                            max: egui::Pos2::new(1.0, 1.0),
                                        },
                                        egui::Color32::WHITE,
                                    );
                                    ui.painter().add(egui::Shape::mesh(mesh));
                                }
                            });
                    });
            });

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
