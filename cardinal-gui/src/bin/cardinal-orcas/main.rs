use std::cell::RefCell;
use std::rc::Rc;
impl<'a> CardinalViewportsApp<'a> {
    /// Create a UxnPanel for a given PanelState (by id), using persistent data.
    /// This does not store the panel, but reconstructs it as needed for the UI layer.
    pub fn panel_for_state(
        &self,
        panel_state: &PanelState,
        ctx: &'a egui::Context,
        panel_size: (u32, u32),
        panel_scale: f32,
    ) -> uxn_panel::UxnPanel<'a> {
        let panel_size_u16 = (panel_size.0 as u16, panel_size.1 as u16);
        let mut panel = uxn_panel::UxnPanel::new(
            ctx,
            None,
            panel_size_u16,
            format!("uxn_panel_texture_{:?}", panel_state.id),
            None,
        );
        panel.set_rom_bytes(CARDINAL_ORCAS_ROM).ok();
        panel.set_sym_bytes(CARDINAL_ORCAS_SYM).ok();
        panel.stage.scale = panel_scale;
        panel
    }
}
// Persistent per-panel state (no egui/eframe references)
pub struct PanelState {
    pub id: egui::Id,
    /// This field is reserved for future use (e.g. lazy panel initialization or migration).
    /// It is not currently used, but kept for compatibility with previous state management logic.
    pub initialized: bool,
    // Add any persistent data you need here (e.g. ROM, VM state, etc.)
}
/// Example: cardinal-orcas
/// Spawns animated viewport windows in N, S, W, E directions on key press.
use cardinal_gui::uxn_panel;
use eframe::egui;
use egui::{StrokeKind, ViewportBuilder, ViewportId}; // Import the UxnPanel module
mod monitor_info;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Direction {
    North,
    South,
    West,
    East,
}

impl Direction {
    fn from_key(key: egui::Key) -> Option<Self> {
        match key {
            egui::Key::N => Some(Direction::North),
            egui::Key::S => Some(Direction::South),
            egui::Key::W => Some(Direction::West),
            egui::Key::E => Some(Direction::East),
            _ => None,
        }
    }

    fn vector(&self) -> (f32, f32) {
        match self {
            Direction::North => (0.0, -1.0), // move up
            Direction::South => (0.0, 1.0),  // move down
            Direction::West => (-1.0, 0.0),  // move left
            Direction::East => (1.0, 0.0),   // move right
        }
    }
}

struct CardinalViewport {
    direction: Direction,
    position: egui::Pos2,
    velocity: f32,
    open: bool,
    monitor_rect: egui::Rect,
    movement: (f32, f32),
}

pub struct CardinalViewportsApp<'a> {
    viewports: Vec<CardinalViewport>,
    collision_enabled: bool,
    wrap_mode: WrapMode,
    panels: Vec<PanelState>,
    uxn_panels: Vec<Rc<RefCell<uxn_panel::UxnPanel<'a>>>>,
    grid_cols: usize,
    grid_rows: usize,
    focused_panel: Option<usize>,
    all_panels_receive_input: bool,
    all_panels_receive_mouse: bool,
    #[cfg(feature = "uses_usb")]
    #[allow(dead_code)]
    last_usb_pedal: Option<u8>,
    pending_focus_panel: Option<usize>,
}

#[derive(PartialEq, Eq, Debug)]
enum WrapMode {
    ParentRect,
    MonitorOfSpawn,
    AllMonitorsSequential,
    AllMonitorsGeometric,
}

impl<'a> Default for CardinalViewportsApp<'a> {
    fn default() -> Self {
        let grid_cols = 2;
        let grid_rows = 2;
        let mut panels = Vec::new();
        for i in 0..(grid_cols * grid_rows) {
            panels.push(PanelState {
                id: egui::Id::new(format!("uxn_panel_{i}")),
                initialized: false,
            });
        }
        Self {
            viewports: Vec::new(),
            collision_enabled: true,
            wrap_mode: WrapMode::ParentRect,
            panels,
            uxn_panels: Vec::new(),
            grid_cols,
            grid_rows,
            focused_panel: Some(0), // Focus the first panel by default
            all_panels_receive_input: false,
            all_panels_receive_mouse: false,
            #[cfg(feature = "uses_usb")]
            last_usb_pedal: None,
            pending_focus_panel: None,
        }
    }
}

impl<'a> eframe::App for CardinalViewportsApp<'a> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Initialize uxn_panels only once, with the real egui::Context
        if self.uxn_panels.is_empty() {
            let panel_size = (680u16, 456u16);
            for (i, panel_state) in self.panels.iter_mut().enumerate() {
                if !panel_state.initialized {
                    let texture_name = format!("uxn_panel_texture_{i}");
                    let mut panel =
                        uxn_panel::UxnPanel::new(ctx, None, panel_size, texture_name, None);
                    panel.set_rom_bytes(CARDINAL_ORCAS_ROM).ok();
                    panel.set_sym_bytes(CARDINAL_ORCAS_SYM).ok();
                    self.uxn_panels.push(Rc::new(RefCell::new(panel)));
                    panel_state.initialized = true;
                } else {
                    // Already initialized, reuse existing
                    // (Assumes panels and uxn_panels are kept in sync by index)
                }
            }
        }
        // --- Ctrl+Q to exit the app ---
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Q)) {
            std::process::exit(0); // not nice to do this, sorry
        }
        // Listen for key presses and button clicks
        let mut spawn_direction: Option<Direction> = None;
        egui::TopBottomPanel::top("cardinal_controls_top").show(ctx, |ui| {
            ui.scope(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("cardinal-orcas");
                    ui.label("Press N, S, W, E or use the buttons below to spawn animated viewports in each direction.");
                });
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("North").frame(false)).clicked() {
                        spawn_direction = Some(Direction::North);
                    }
                    if ui.add(egui::Button::new("South").frame(false)).clicked() {
                        spawn_direction = Some(Direction::South);
                    }
                    if ui.add(egui::Button::new("West").frame(false)).clicked() {
                        spawn_direction = Some(Direction::West);
                    }
                    if ui.add(egui::Button::new("East").frame(false)).clicked() {
                        spawn_direction = Some(Direction::East);
                    }
                    ui.separator();
                    ui.checkbox(&mut self.collision_enabled, "Collision Detection");
                    ui.separator();
                    egui::ComboBox::from_label("Wrap Mode")
                        .selected_text(match self.wrap_mode {
                            WrapMode::ParentRect => "Parent Rect",
                            WrapMode::MonitorOfSpawn => "Monitor of Spawn",
                            WrapMode::AllMonitorsSequential => "All Monitors (Sequential)",
                            WrapMode::AllMonitorsGeometric => "All Monitors (Geometric)",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.wrap_mode, WrapMode::ParentRect, "Parent Rect");
                            ui.selectable_value(&mut self.wrap_mode, WrapMode::MonitorOfSpawn, "Monitor of Spawn");
                            ui.selectable_value(&mut self.wrap_mode, WrapMode::AllMonitorsSequential, "All Monitors (Sequential)");
                            ui.selectable_value(&mut self.wrap_mode, WrapMode::AllMonitorsGeometric, "All Monitors (Geometric)");
                        });
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.all_panels_receive_input, "All panels receive keyboard input");
                    ui.checkbox(&mut self.all_panels_receive_mouse, "All panels receive mouse input");
                });
            });
        });

        for key in [egui::Key::N, egui::Key::S, egui::Key::W, egui::Key::E] {
            if ctx.input(|i| i.key_pressed(key)) {
                spawn_direction = Direction::from_key(key);
            }
        }

        if let Some(direction) = spawn_direction {
            if let Some(parent_rect) = ctx.input(|i| i.viewport().outer_rect) {
                let monitor_rects: Vec<egui::Rect> =
                    monitor_info::MONITOR_RECTS.lock().unwrap().clone();
                let collision_radius = 100.0;
                let (dx, dy) = direction.vector();
                let start_pos = egui::pos2(
                    parent_rect.center().x + dx * (collision_radius + 8.0),
                    parent_rect.center().y + dy * (collision_radius + 8.0),
                );
                let spawn_monitor_rect = monitor_rects
                    .iter()
                    .find(|r| r.contains(start_pos))
                    .cloned()
                    .unwrap_or_else(|| {
                        monitor_rects.first().cloned().unwrap_or_else(|| {
                            egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::Vec2::new(1920.0, 1080.0),
                            )
                        })
                    });
                self.viewports.push(CardinalViewport {
                    direction,
                    position: start_pos,
                    velocity: 8.0,
                    open: true,
                    monitor_rect: spawn_monitor_rect,
                    movement: (dx, dy),
                });
                // After spawning, set focus to the first panel (if any) only if none is focused
                if self.focused_panel.is_none() {
                    self.focused_panel = Some(0);
                }
                // Defer egui focus request for the currently focused panel (if any)
                if let Some(idx) = self.focused_panel {
                    self.pending_focus_panel = Some(idx);
                }
            }
        }

        // Handle Tab key for panel focus cycling
        let input = ctx.input(|i| i.clone());
        let tab_pressed = input.events.iter().any(|e| {
            matches!(
                e,
                egui::Event::Key {
                    key: egui::Key::Tab,
                    pressed: true,
                    ..
                }
            )
        });
        if tab_pressed {
            let panel_count = self.grid_cols * self.grid_rows;
            let next_panel = match self.focused_panel {
                Some(idx) => (idx + 1) % panel_count,
                None => 0,
            };
            self.focused_panel = Some(next_panel);
        }

        if let Some(parent_rect) = ctx.input(|i| i.viewport().outer_rect) {
            let monitor_rects = monitor_info::MONITOR_RECTS.lock().unwrap().clone();
            let _all_monitors_rect = monitor_rects.iter().skip(1).fold(
                monitor_rects
                    .first()
                    .cloned()
                    .unwrap_or(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::Vec2::new(1920.0, 1080.0),
                    )),
                |acc, r| acc.union(*r),
            );
            // Debug: print monitor rects
            #[cfg(debug_assertions)]
            {
                // println!("[DEBUG] Monitor rects:");
                // for (idx, r) in monitor_rects.iter().enumerate() {
                //     println!(
                //         " {} Monitor {idx}: min=({:.1},{:.1}) max=({:.1},{:.1}) size=({:.1},{:.1})",
                //         idx,
                //         r.min.x,
                //         r.min.y,
                //         r.max.x,
                //         r.max.y,
                //         r.width(),
                //         r.height()
                //     );
                // }
                // println!("[DEBUG] All monitors union: min=({:.1},{:.1}) max=({:.1},{:.1}) size=({:.1},{:.1})", all_monitors_rect.min.x, all_monitors_rect.min.y, all_monitors_rect.max.x, all_monitors_rect.max.y, all_monitors_rect.width(), all_monitors_rect.height());
            }
            for (i, viewport) in self.viewports.iter_mut().enumerate() {
                if !viewport.open {
                    continue;
                }
                // Animate movement
                let (dx, dy) = viewport.movement;
                viewport.position.x += dx * viewport.velocity;
                viewport.position.y += dy * viewport.velocity;

                // Wrapping logic
                let wrap_rect = match self.wrap_mode {
                    WrapMode::ParentRect => parent_rect,
                    WrapMode::MonitorOfSpawn => viewport.monitor_rect,
                    WrapMode::AllMonitorsSequential | WrapMode::AllMonitorsGeometric => {
                        let mut current_monitor_idx = monitor_rects
                            .iter()
                            .position(|r| r.contains(viewport.position));
                        if current_monitor_idx.is_none() {
                            // Snap to closest
                            current_monitor_idx = monitor_rects
                                .iter()
                                .enumerate()
                                .min_by(|(_, a), (_, b)| {
                                    let da = a.center().distance(viewport.position);
                                    let db = b.center().distance(viewport.position);
                                    da.partial_cmp(&db).unwrap()
                                })
                                .map(|(idx, _)| idx);
                        }
                        let (dx, dy) = viewport.movement;
                        let mut did_wrap = false;
                        let mut new_pos = viewport.position;
                        let mut new_idx = current_monitor_idx.unwrap_or(0);
                        match self.wrap_mode {
                            WrapMode::AllMonitorsSequential => {
                                if let Some(idx) = current_monitor_idx {
                                    if let Some((pos, idx2)) = monitor_info::wrap_sequential(
                                        viewport.position,
                                        idx,
                                        dx,
                                        dy,
                                        &monitor_rects,
                                    ) {
                                        new_pos = pos;
                                        new_idx = idx2;
                                        did_wrap = true;
                                    }
                                }
                            }
                            WrapMode::AllMonitorsGeometric => {
                                if let Some((pos, idx2)) = monitor_info::wrap_geometric(
                                    viewport.position,
                                    dx,
                                    dy,
                                    &monitor_rects,
                                ) {
                                    new_pos = pos;
                                    new_idx = idx2;
                                    did_wrap = true;
                                }
                            }
                            _ => {}
                        }
                        if did_wrap {
                            viewport.position = new_pos;
                            viewport.monitor_rect = monitor_rects[new_idx];
                        }
                        monitor_rects[new_idx]
                    }
                };
                #[cfg(debug_assertions)]
                {
                    // println!("[DEBUG] Viewport {i} {:?} wrap_mode={:?} wrap_rect: min=({:.1},{:.1}) max=({:.1},{:.1}) size=({:.1},{:.1}) pos=({:.1},{:.1})", viewport.direction, self.wrap_mode, wrap_rect.min.x, wrap_rect.min.y, wrap_rect.max.x, wrap_rect.max.y, wrap_rect.width(), wrap_rect.height(), viewport.position.x, viewport.position.y);
                }
                if viewport.position.x < wrap_rect.left() {
                    viewport.position.x = wrap_rect.right();
                }
                if viewport.position.x > wrap_rect.right() {
                    viewport.position.x = wrap_rect.left();
                }
                if viewport.position.y < wrap_rect.top() {
                    viewport.position.y = wrap_rect.bottom();
                }
                if viewport.position.y > wrap_rect.bottom() {
                    viewport.position.y = wrap_rect.top();
                }

                // Collision with parent window
                if self.collision_enabled {
                    let parent_center = parent_rect.center();
                    let dist = ((viewport.position.x - parent_center.x).powi(2)
                        + (viewport.position.y - parent_center.y).powi(2))
                    .sqrt();
                    // Draw collision circle in the parent window (main viewport)
                    if ctx.input(|i| i.viewport().parent.is_none()) {
                        ctx.layer_painter(egui::LayerId::new(
                            egui::Order::Foreground,
                            egui::Id::new("cardinal_collision"),
                        ))
                        .circle(
                            parent_center,
                            50.0,
                            egui::Color32::RED,
                            egui::Stroke::new(4.0, egui::Color32::RED),
                        );
                    }
                    if dist < 100.0 {
                        // Beep and close
                        #[cfg(target_os = "windows")]
                        {
                            unsafe { winapi::um::winuser::MessageBeep(0xFFFFFFFF) };
                        }
                        #[cfg(target_os = "linux")]
                        println!("\x07");
                        #[cfg(target_os = "macos")]
                        println!("\x07");
                        viewport.open = false;
                    }
                }

                let viewport_id = ViewportId::from_hash_of(format!("cardinal_{i}"));
                ctx.show_viewport_immediate(
                    viewport_id,
                    ViewportBuilder::default()
                        .with_title(format!("Viewport: {:?}", viewport.direction))
                        .with_inner_size([200.0, 100.0])
                        .with_position(viewport.position)
                        .with_decorations(false)
                        .with_always_on_top(),
                    move |ctx, class| {
                        if class == egui::ViewportClass::Embedded {
                            egui::Window::new("Cardinal Viewport").show(ctx, |ui| {
                                ui.label(
                                    "This egui integration does not support multiple viewports",
                                );
                            });
                        } else {
                            egui::CentralPanel::default().show(ctx, |ui| {
                                // Do not handle or draw focus for this viewport, making it non-focusable
                                ui.vertical_centered(|ui| {
                                    let dir_char = match viewport.direction {
                                        Direction::North => "N",
                                        Direction::South => "S",
                                        Direction::East => "E",
                                        Direction::West => "W",
                                    };
                                    ui.label(egui::RichText::new(dir_char).size(96.0).strong());
                                });
                            });
                        }
                    },
                );
            }
        }

        // --- UxnPanel grid config (panel_size, panel_scale) ---
        let mut panel_scale: f32 = 1.0;
        let panel_size = (680, 456);
        let panel_padding = 16.0;
        // TOP_PANEL_HEIGHT is already defined at module level
        // Handle Ctrl++ and Ctrl+- for scaling
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Plus)) {
            panel_scale *= 1.1;
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Minus)) {
            panel_scale /= 1.1;
        }
        // Clamp scale
        panel_scale = panel_scale.clamp(0.5, 3.0);
        // Calculate window size
        let window_width =
            self.grid_cols as f32 * (panel_size.0 as f32 * panel_scale + panel_padding);
        let window_height = self.grid_rows as f32
            * (panel_size.1 as f32 * panel_scale + panel_padding)
            + TOP_PANEL_HEIGHT;
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
            window_width,
            window_height,
        )));
        // --- USB Pedal Polling and Debug Panel ---
        #[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
        {
            // Try to poll pedal state from the first panel's controller if available
            if let Some(idx) = self.focused_panel {
                if let Some(panel_rc) = self.uxn_panels.get(idx) {
                    let mut panel = panel_rc.borrow_mut();
                    if let Some(varvara_controller) = panel
                        .stage
                        .dev
                        .controller
                        .as_any()
                        .downcast_mut::<varvara::controller_usb::ControllerUsb>(
                    ) {
                        if let Some(pedal) = varvara_controller.last_pedal {
                            self.last_usb_pedal = Some(pedal);
                            // Map pedal bits to panel focus: bits 1,2,4,8 -> panels 0-3, 7 (all pressed) -> panel 4
                            let pedal_bits = pedal & 0x0F;
                            let new_focus = match pedal_bits {
                                0x01 => Some(0),
                                0x02 => Some(1),
                                0x04 => Some(2),
                                0x07 => Some(3),
                                _ => self.focused_panel, // no change
                            };
                            if new_focus != self.focused_panel {
                                println!("[DEBUG][pedal] pedal=0x{pedal:02X} -> focus panel {new_focus:?}");
                                self.focused_panel = new_focus;
                                // Also request egui focus for the newly focused panel
                                // if let Some(idx) = new_focus {
                                //     if let Some(panel_rc) = self.uxn_panels.get(idx) {
                                //         // let panel = panel_rc.borrow();
                                //         //let response_id = panel.last_response_id();
                                //         // ctx.memory_mut(|mem| mem.request_focus(response_id));
                                //     }
                                // }
                            }
                        }
                    }
                }
            }
            egui::TopBottomPanel::bottom("usb_pedal_debug").show(ctx, |ui| {
                let panel_id_str = if let Some(idx) = self.focused_panel {
                    if let Some(panel_rc) = self.uxn_panels.get(idx) {
                        let panel = panel_rc.borrow();
                        use cardinal_gui::cardinal_orcas_symbols::cardinal_orcas_symbols::get_slice;
                        let bang = get_slice(panel.stage.vm.ram(), "*");
                        let x = get_slice(panel.stage.vm.ram(), "Mouse/x");
                        let posx = get_slice(panel.stage.vm.ram(), "cursor/x");
                        let grid = get_slice(panel.stage.vm.ram(), "grid/buf");
                        format!(
                            "{:?} bang  {:?} x {:?} posx {:?} grid {:?}",
                            panel.last_response_id(),
                            bang,
                            x,
                            posx,
                            grid
                        )
                    } else {
                        "None".to_string()
                    }
                } else {
                    "None".to_string()
                };
                egui::CollapsingHeader::new(format!(
                    "[USB] Last pedal event: state={:?} | Focused panel id: {panel_id_str}",
                    self.last_usb_pedal
                ))
                .default_open(false)
                .show(ui, |ui| {
                    if let Some(pedal) = self.last_usb_pedal {
                        ui.label(format!("[USB] Last pedal event: state=0x{pedal:02X}"));
                    } else {
                        ui.label("[USB] No pedal events received yet.");
                    }
                    ui.label(format!("[USB] Focused panel id: {panel_id_str}"));
                });
            });
        }

        // --- UxnPanel grid in main area ---
        let mut panel_scale: f32 = 1.0;
        let panel_size = (680, 456);
        let panel_padding = 16.0;
        const TOP_PANEL_HEIGHT: f32 = 64.0;
        // Handle Ctrl++ and Ctrl+- for scaling
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Plus)) {
            panel_scale *= 1.1;
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Minus)) {
            panel_scale /= 1.1;
        }
        // Clamp scale
        panel_scale = panel_scale.clamp(0.5, 3.0);
        // Calculate window size
        let window_width =
            self.grid_cols as f32 * (panel_size.0 as f32 * panel_scale + panel_padding);
        let window_height = self.grid_rows as f32
            * (panel_size.1 as f32 * panel_scale + panel_padding)
            + TOP_PANEL_HEIGHT;
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
            window_width,
            window_height,
        )));
        // --- UxnPanel grid in main area (refactored to use persistent PanelState) ---
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("uxn_grid")
                .num_columns(self.grid_cols)
                .show(ui, |ui| {
                    let input = ctx.input(|i| i.clone());
                    for (i, panel_rc) in self.uxn_panels.iter().enumerate() {
                        // First borrow: only extract needed fields
                        // First borrow: only extract needed fields
                        let (rect, response_id, clicked);
                        {
                            let mut panel = panel_rc.borrow_mut();
                            let response = panel.show(ui);
                            rect = response.rect;
                            response_id = response.id;
                            clicked = response.clicked();
                            let is_focused = self.focused_panel == Some(i);
                            if is_focused {
                                let painter = ui.painter();
                                // Inflate the rect by 2px for the border to avoid overlap
                                let border_rect = rect.expand(2.0);
                                painter.rect_stroke(
                                    border_rect,
                                    egui::CornerRadius::same(6),
                                    egui::Stroke::new(3.0, egui::Color32::from_rgb(0, 120, 255)),
                                    StrokeKind::Outside,
                                );
                                // Do NOT request egui focus every frame for the focused panel
                            }
                            if clicked {
                                println!("[DEBUG] Panel {i} clicked, requesting focus");
                                self.focused_panel = Some(i);
                                // Request egui focus for this panel only on click
                                response.request_focus();
                                ui.ctx().memory_mut(|mem| mem.request_focus(response_id));
                            }
                        }
                        let is_focused = self.focused_panel == Some(i);
                        let filtered_input = if self.all_panels_receive_input || is_focused {
                            input.clone()
                        } else {
                            let mut no_events = input.clone();
                            no_events.events.clear();
                            no_events
                        };
                        let filtered_input = if self.all_panels_receive_mouse {
                            if self.all_panels_receive_input || is_focused {
                                filtered_input
                            } else {
                                let pointer_pos = ctx.input(|i| i.pointer.latest_pos());
                                if let Some(global_pos) = pointer_pos {
                                    let local_x =
                                        (global_pos.x - rect.min.x).max(0.0).min(rect.width())
                                            + rect.min.x;
                                    let local_y =
                                        (global_pos.y - rect.min.y).max(0.0).min(rect.height())
                                            + rect.min.y;
                                    let local_pos = egui::Pos2::new(local_x, local_y);
                                    let pointer = ctx.input(|i| i.pointer.clone());
                                    let left = pointer.button_down(egui::PointerButton::Primary);
                                    let right = pointer.button_down(egui::PointerButton::Secondary);
                                    let middle = pointer.button_down(egui::PointerButton::Middle);
                                    let mut buttons = 0u8;
                                    if left {
                                        buttons |= 1;
                                    }
                                    if middle {
                                        buttons |= 2;
                                    }
                                    if right {
                                        buttons |= 4;
                                    }
                                    let mouse_state = varvara::MouseState {
                                        pos: (local_pos.x, local_pos.y),
                                        scroll: (0.0, 0.0),
                                        buttons,
                                    };
                                    // Mouse update borrow
                                    {
                                        let mut panel = panel_rc.borrow_mut();
                                        let stage = &mut panel.stage;
                                        stage.dev.mouse.set_active();
                                        stage.dev.mouse.update(&mut stage.vm, mouse_state);
                                    }
                                }
                                let mut no_mouse = filtered_input.clone();
                                no_mouse.events.retain(|e| {
                                    !matches!(
                                        e,
                                        egui::Event::PointerButton { .. }
                                            | egui::Event::PointerMoved(_)
                                            | egui::Event::PointerGone
                                    )
                                });
                                no_mouse
                            }
                        } else {
                            filtered_input
                        };
                        if self.all_panels_receive_input
                            || is_focused
                            || self.all_panels_receive_mouse
                        {
                            for event in &filtered_input.events {
                                if matches!(event, egui::Event::MouseMoved(_)) {
                                    continue;
                                }
                                if let egui::Event::Text(s) = event {
                                    println!("[DEBUG] Panel {i} egui::Event::Text: {s:?}");
                                }
                            }
                            // Redraw/update borrow
                            {
                                let mut panel = panel_rc.borrow_mut();
                                panel.handle_input(&filtered_input, rect);
                                let stage = &mut panel.stage;
                                stage.dev.redraw(&mut stage.vm);
                                stage.update_texture(ui.ctx());
                            }
                        }
                        if (i + 1) % self.grid_cols == 0 {
                            ui.end_row();
                        }
                    }
                }); // end Grid
        }); // end CentralPanel
            // After all panels are shown, if a focus is pending, request it now
        if let Some(idx) = self.pending_focus_panel.take() {
            if let Some(panel_rc) = self.uxn_panels.get(idx) {
                let panel = panel_rc.borrow();
                let response_id = panel.last_response_id();
                ctx.memory_mut(|mem| mem.request_focus(response_id));
            }
        }
        // ...existing code...
    }
}

// Embed the ROM and .sym file as byte arrays
const CARDINAL_ORCAS_ROM: &[u8] = include_bytes!("cardinal-orcas.rom");
const CARDINAL_ORCAS_SYM: &[u8] = include_bytes!("cardinal-orcas.rom.sym");

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Use grid size from app default
    let app_default = CardinalViewportsApp::default();
    let grid_cols = app_default.grid_cols;
    let grid_rows = app_default.grid_rows;
    let panel_size = (128, 128);
    let panel_padding = 16.0; // padding around each panel for blue rectangle
                              // top_panel_height is now defined as TOP_PANEL_HEIGHT constant above
                              // Total width: (panel width + padding) * columns + extra padding
    let window_width = grid_cols as f32 * (panel_size.0 as f32 + panel_padding) + panel_padding;
    // Total height: (panel height + padding) * rows + top panel height + extra padding
    let window_height = grid_rows as f32 * (panel_size.1 as f32) + panel_padding;
    let window_size = egui::Vec2::new(window_width, window_height);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(window_size),
        ..Default::default()
    };
    monitor_info::fill_monitor_rects();
    #[cfg(debug_assertions)]
    {
        let rects = monitor_info::MONITOR_RECTS.lock().unwrap();
        println!("[DEBUG] Filled MONITOR_RECTS: {} monitors", rects.len());
        for (i, r) in rects.iter().enumerate() {
            println!(
                "  Monitor {i}: min=({:.1},{:.1}) max=({:.1},{:.1}) size=({:.1},{:.1})",
                r.min.x,
                r.min.y,
                r.max.x,
                r.max.y,
                r.width(),
                r.height()
            );
        }
    }
    // --- External event loop pattern for monitor snarfing ---
    use eframe::UserEvent;
    use winit::event_loop::EventLoop;
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("Failed to build event loop");
    monitor_info::fill_monitor_rects();
    #[cfg(debug_assertions)]
    {
        let rects = monitor_info::MONITOR_RECTS.lock().unwrap();
        println!("[DEBUG] Filled MONITOR_RECTS: {} monitors", rects.len());
        for (i, r) in rects.iter().enumerate() {
            println!(
                "  Monitor {i}: min=({:.1},{:.1}) max=({:.1},{:.1}) size=({:.1},{:.1})",
                r.min.x,
                r.min.y,
                r.max.x,
                r.max.y,
                r.width(),
                r.height()
            );
        }
    }
    let mut app = eframe::create_native(
        "cardinal-orcas",
        options,
        Box::new(|_cc| Ok(Box::new(CardinalViewportsApp::default()))),
        &event_loop,
    );
    event_loop.run_app(&mut app).expect("eframe app failed");
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eprintln!("cardinal-orcas is not supported on wasm32 targets.");
}
// Map egui::Key and shift to Varvara::Key
#[allow(dead_code)]
fn decode_key(k: egui::Key, shift: bool) -> Option<varvara::Key> {
    use varvara::Key;
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
        (egui::Key::Num5, true) => Key::Char(b'%'),
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
