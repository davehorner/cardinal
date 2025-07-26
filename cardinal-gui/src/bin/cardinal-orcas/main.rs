//! Example: Cardinal Viewports
// Spawns animated viewport windows in N, S, W, E directions on key press.

use eframe::egui;
use egui::{ViewportBuilder, ViewportId};
// use std::sync::Mutex; // No longer needed here
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

pub struct CardinalViewportsApp {
    viewports: Vec<CardinalViewport>,
    collision_enabled: bool,
    wrap_mode: WrapMode,
}

#[derive(PartialEq, Eq, Debug)]
enum WrapMode {
    ParentRect,
    MonitorOfSpawn,
    AllMonitorsSequential,
    AllMonitorsGeometric,
}

impl Default for CardinalViewportsApp {
    fn default() -> Self {
        Self {
            viewports: Vec::new(),
            collision_enabled: true,
            wrap_mode: WrapMode::MonitorOfSpawn,
        }
    }
}

impl eframe::App for CardinalViewportsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Listen for key presses and button clicks
        let mut spawn_direction: Option<Direction> = None;
        egui::TopBottomPanel::top("cardinal_controls_top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Cardinal Viewports Example");
                ui.label("Press N, S, W, E or use the buttons below to spawn animated viewports in each direction.");
            });
            ui.horizontal(|ui| {
                if ui.button("North").clicked() {
                    spawn_direction = Some(Direction::North);
                }
                if ui.button("South").clicked() {
                    spawn_direction = Some(Direction::South);
                }
                if ui.button("West").clicked() {
                    spawn_direction = Some(Direction::West);
                }
                if ui.button("East").clicked() {
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
        });

        for key in [egui::Key::N, egui::Key::S, egui::Key::W, egui::Key::E] {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
                spawn_direction = Direction::from_key(key);
            }
        }

        if let Some(direction) = spawn_direction {
            if let Some(parent_rect) = ctx.input(|i| i.viewport().outer_rect) {
                let monitor_rects: Vec<egui::Rect> = unsafe {
                    monitor_info::MONITOR_RECTS.lock().unwrap().clone()
                };
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
                        monitor_rects.get(0).cloned().unwrap_or(
                            egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::Vec2::new(1920.0, 1080.0),
                            ),
                        )
                    });
                self.viewports.push(CardinalViewport {
                    direction,
                    position: start_pos,
                    velocity: 8.0,
                    open: true,
                    monitor_rect: spawn_monitor_rect,
                    movement: (dx, dy),
                });
            }
        }

        if let Some(parent_rect) = ctx.input(|i| i.viewport().outer_rect) {
            let monitor_rects =
                monitor_info::MONITOR_RECTS.lock().unwrap().clone();
            let all_monitors_rect = monitor_rects.iter().skip(1).fold(
                monitor_rects.first().cloned().unwrap_or(
                    egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::Vec2::new(1920.0, 1080.0),
                    ),
                ),
                |acc, r| acc.union(*r),
            );
            // Debug: print monitor rects
            #[cfg(debug_assertions)]
            {
                println!("[DEBUG] Monitor rects:");
                for (idx, r) in monitor_rects.iter().enumerate() {
                    println!(
                        " {} Monitor {idx}: min=({:.1},{:.1}) max=({:.1},{:.1}) size=({:.1},{:.1})",
                        idx,
                        r.min.x,
                        r.min.y,
                        r.max.x,
                        r.max.y,
                        r.width(),
                        r.height()
                    );
                }
                println!("[DEBUG] All monitors union: min=({:.1},{:.1}) max=({:.1},{:.1}) size=({:.1},{:.1})", all_monitors_rect.min.x, all_monitors_rect.min.y, all_monitors_rect.max.x, all_monitors_rect.max.y, all_monitors_rect.width(), all_monitors_rect.height());
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
                    WrapMode::AllMonitorsSequential
                    | WrapMode::AllMonitorsGeometric => {
                        let mut current_monitor_idx = monitor_rects
                            .iter()
                            .position(|r| r.contains(viewport.position));
                        if current_monitor_idx.is_none() {
                            // Snap to closest
                            current_monitor_idx = monitor_rects
                                .iter()
                                .enumerate()
                                .min_by(|(_, a), (_, b)| {
                                    let da =
                                        a.center().distance(viewport.position);
                                    let db =
                                        b.center().distance(viewport.position);
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
                                    if let Some((pos, idx2)) =
                                        monitor_info::wrap_sequential(
                                            viewport.position,
                                            idx,
                                            dx,
                                            dy,
                                            &monitor_rects,
                                        )
                                    {
                                        new_pos = pos;
                                        new_idx = idx2;
                                        did_wrap = true;
                                    }
                                }
                            }
                            WrapMode::AllMonitorsGeometric => {
                                if let Some((pos, idx2)) =
                                    monitor_info::wrap_geometric(
                                        viewport.position,
                                        dx,
                                        dy,
                                        &monitor_rects,
                                    )
                                {
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
                    println!("[DEBUG] Viewport {i} {:?} wrap_mode={:?} wrap_rect: min=({:.1},{:.1}) max=({:.1},{:.1}) size=({:.1},{:.1}) pos=({:.1},{:.1})", viewport.direction, self.wrap_mode, wrap_rect.min.x, wrap_rect.min.y, wrap_rect.max.x, wrap_rect.max.y, wrap_rect.width(), wrap_rect.height(), viewport.position.x, viewport.position.y);
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
                    let dist = ((viewport.position.x - parent_center.x)
                        .powi(2)
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
                            unsafe {
                                winapi::um::winuser::MessageBeep(0xFFFFFFFF)
                            };
                        }
                        #[cfg(target_os = "linux")]
                        println!("\x07");
                        #[cfg(target_os = "macos")]
                        println!("\x07");
                        viewport.open = false;
                    }
                }

                let viewport_id =
                    ViewportId::from_hash_of(format!("cardinal_{i}"));
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

        // ...existing code...
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let options = eframe::NativeOptions::default();
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
        "Cardinal Viewports",
        options,
        Box::new(|_cc| Ok(Box::new(CardinalViewportsApp::default()))),
        &event_loop,
    );
    event_loop.run_app(&mut app).expect("eframe app failed");
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eprintln!("Cardinal Viewports is not supported on wasm32 targets.");
}
