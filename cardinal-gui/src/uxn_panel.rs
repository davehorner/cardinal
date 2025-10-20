use crate::stage::Stage;
use ::uxn::{Backend, Uxn};
use eframe::egui;
use varvara::Varvara;

pub struct UxnPanel<'a> {
    pub stage: Stage<'a>,
    focused: bool,
    last_response_id: Option<egui::Id>,
}

impl<'a> UxnPanel<'a> {
    pub fn last_response_id(&self) -> egui::Id {
        self.last_response_id
            .unwrap_or_else(|| egui::Id::new("uxn_panel_default"))
    }
    /// Set the ROM data for this panel from a byte slice
    pub fn set_rom_bytes(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        self.stage.load_rom(data)
    }

    /// Set the symbol data for this panel from a byte slice
    pub fn set_sym_bytes(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        self.stage.load_symbols(data)
    }
    /// Set the ROM path for this panel and load the ROM and .sys symbol file if present
    pub fn set_rom_path<P: AsRef<std::path::Path>>(&mut self, path: P) {
        self.stage.load_rom_with_path(path);
    }
    pub fn new(
        ctx: &egui::Context,
        rom: Option<&[u8]>,
        size: (u16, u16),
        texture_name: String,
        transparent_color: Option<String>,
    ) -> Self {
        let ram = Box::new([0u8; 65536]);
        let ram_for_leak = ram.clone();
        let ram_static: &'a mut [u8; 65536] = Box::leak(ram_for_leak);
        let vm = Uxn::new(ram_static, Backend::Interpreter);
        let dev = Varvara::default();
        let (_tx, rx) = std::sync::mpsc::channel();
        let mut stage = Stage::new(vm, dev, size, 1.0, rx, ctx, texture_name, transparent_color);
        if let Some(rom) = rom {
            let _ = stage.load_rom(rom);
        }
        Self {
            stage,
            focused: false,
            last_response_id: None,
        }
    }

    /// Returns true if the panel is currently focused for keyboard input
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Forward input events to the panel (keyboard and mouse)
    pub fn handle_input(&mut self, input: &egui::InputState, rect: egui::Rect) {
        self.stage.handle_input(input, rect);
    }

    /// Show the panel and forward input events if hovered/focused
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        // Allocate space and get response for interaction
        let panel_size =
            egui::Vec2::new(self.stage.size.0 as f32, self.stage.size.1 as f32) * self.stage.scale;
        let (rect_id, rect) = ui.allocate_space(panel_size);
        let response = ui.interact(rect, rect_id, egui::Sense::click_and_drag());

        // Request keyboard focus when clicked
        if response.clicked() {
            response.request_focus();
        }
        let input = ui.ctx().input(|i| i.clone());
        // Mouse events: always forward when hovered
        if response.hovered() {
            self.stage.handle_mouse_input(&input, &response, rect);
        }
        // Keyboard focus state
        self.focused = response.has_focus();

        // If focused, process keyboard input (including char/text events)
        if self.focused {
            self.stage.handle_input(&input, rect);
            self.stage.handle_usb_input();
        }

        self.stage.step();
        self.stage.update_texture(ui.ctx());
        // Draw mesh at rect
        self.stage.draw_at(ui, rect);
        self.last_response_id = Some(response.id);
        response
    }
}
