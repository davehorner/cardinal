#[cfg(not(feature = "uses_gilrs"))]
compile_error!("controller_gilrs.rs should not be compiled unless the 'uses_gilrs' feature is enabled");

#[cfg(all(feature = "uses_gilrs", not(target_arch = "wasm32")))]
use super::controller::Controller;
#[cfg(all(feature = "uses_gilrs", not(target_arch = "wasm32")))]
use super::controller::{ControllerDevice, Key};
#[cfg(all(feature = "uses_gilrs", not(target_arch = "wasm32")))]
use crate::Event;
#[cfg(all(feature = "uses_gilrs", not(target_arch = "wasm32")))]
use std::any::Any;
#[cfg(all(feature = "uses_gilrs", not(target_arch = "wasm32")))]
use uxn::Uxn;

use gilrs::EventType;
#[cfg(all(feature = "uses_gilrs", not(target_arch = "wasm32")))]
use gilrs::{Axis, Button, Event as GilrsEvent, Gilrs};
#[cfg(all(feature = "uses_gilrs", not(target_arch = "wasm32")))]
use std::sync::mpsc;
#[cfg(all(feature = "uses_gilrs", not(target_arch = "wasm32")))]
use std::thread;

#[cfg(all(feature = "uses_gilrs", not(target_arch = "wasm32")))]
/// Message from Gilrs controller thread
#[derive(Debug)]
pub struct GilrsControllerMessage {
    /// Raw Gilrs event data (can be extended as needed)
    pub event: Option<GilrsEvent>,
}

#[cfg(all(feature = "uses_gilrs", not(target_arch = "wasm32")))]
/// Gilrs controller device for Varvara system.
pub struct ControllerGilrs {
    /// Receiver for Gilrs controller messages.
    pub rx: mpsc::Receiver<GilrsControllerMessage>,
    /// Internal controller state.
    pub controller: Controller,
}

#[cfg(all(feature = "uses_gilrs", not(target_arch = "wasm32")))]
impl ControllerDevice for ControllerGilrs {
    fn char(&mut self, vm: &mut Uxn, c: u8) -> Event {
        self.controller.char(vm, c)
    }
    fn pressed(&mut self, vm: &mut Uxn, k: Key, repeat: bool) -> Option<Event> {
        self.controller.pressed(vm, k, repeat)
    }
    fn released(&mut self, vm: &mut Uxn, k: Key) -> Option<Event> {
        self.controller.released(vm, k)
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(all(feature = "uses_gilrs", not(target_arch = "wasm32")))]
impl ControllerGilrs {
    /// Polls for Gilrs events and prints debug output for button/axis changes.
    pub fn poll_gilrs_event(&mut self, vm: &mut Uxn) {
        while let Ok(msg) = self.rx.try_recv() {
            if let Some(event) = msg.event {
                println!("[GILRS] Event: {event:?}");
                // You can add more detailed event handling here
                // Example mapping: map Gilrs button events to internal controller keys

                match event.event {
                    EventType::ButtonPressed(button, _) => {
                        // Map Gilrs button to internal Key
                        if let Some(key) = match button {
                            Button::South => Some(Key::Char(b'A')),
                            Button::East => Some(Key::Char(b'B')),
                            Button::North => Some(Key::Char(b'X')),
                            Button::West => Some(Key::Char(b'Y')),
                            Button::DPadUp => Some(Key::Up),
                            Button::DPadDown => Some(Key::Down),
                            Button::DPadLeft => Some(Key::Left),
                            Button::DPadRight => Some(Key::Right),
                            Button::LeftTrigger => Some(Key::Char(b'L')),
                            Button::RightTrigger => Some(Key::Char(b'R')),
                            Button::Start => Some(Key::Char(b'S')),
                            Button::Select => Some(Key::Char(b'E')),
                            _ => None,
                        } {
                            let _ = self.controller.pressed(vm, key, false);
                        }
                    }
                    EventType::ButtonReleased(button, _) => {
                        if let Some(key) = match button {
                            Button::South => Some(Key::Char(b'A')),
                            Button::East => Some(Key::Char(b'B')),
                            Button::North => Some(Key::Char(b'X')),
                            Button::West => Some(Key::Char(b'Y')),
                            Button::DPadUp => Some(Key::Up),
                            Button::DPadDown => Some(Key::Down),
                            Button::DPadLeft => Some(Key::Left),
                            Button::DPadRight => Some(Key::Right),
                            Button::LeftTrigger => Some(Key::Char(b'L')),
                            Button::RightTrigger => Some(Key::Char(b'R')),
                            Button::Start => Some(Key::Char(b'S')),
                            Button::Select => Some(Key::Char(b'E')),
                            _ => None,
                        } {
                            let _ = self.controller.released(vm, key);
                        }
                    }
                    EventType::AxisChanged(axis, value, _) => {
                        // Map DPad axes to arrow keys (for controllers that use axes for DPad)
                        let threshold = 0.5;
                        match axis {
                            Axis::DPadX => {
                                if value > threshold {
                                    let _ = self.controller.pressed(
                                        vm,
                                        Key::Right,
                                        false,
                                    );
                                    let _ =
                                        self.controller.released(vm, Key::Left);
                                } else if value < -threshold {
                                    let _ = self.controller.pressed(
                                        vm,
                                        Key::Left,
                                        false,
                                    );
                                    let _ = self
                                        .controller
                                        .released(vm, Key::Right);
                                } else {
                                    let _ =
                                        self.controller.released(vm, Key::Left);
                                    let _ = self
                                        .controller
                                        .released(vm, Key::Right);
                                }
                            }
                            Axis::DPadY => {
                                if value > threshold {
                                    let _ = self.controller.pressed(
                                        vm,
                                        Key::Down,
                                        false,
                                    );
                                    let _ =
                                        self.controller.released(vm, Key::Up);
                                } else if value < -threshold {
                                    let _ = self.controller.pressed(
                                        vm,
                                        Key::Up,
                                        false,
                                    );
                                    let _ =
                                        self.controller.released(vm, Key::Down);
                                } else {
                                    let _ =
                                        self.controller.released(vm, Key::Up);
                                    let _ =
                                        self.controller.released(vm, Key::Down);
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Helper to construct a ControllerGilrs with a running gilrs thread.
    pub fn new(controller: Controller) -> Self {
        let rx = spawn_gilrs_controller_thread();
        ControllerGilrs { rx, controller }
    }

    #[allow(dead_code)]
    fn check_buttons(&mut self, vm: &mut Uxn, repeat: bool) -> Option<Event> {
        self.controller.check_buttons(vm, repeat)
    }
}

/// the thread that spawns the Gilrs controller listener
/// This function is only available when the `uses_gilrs` feature is enabled
#[cfg(all(feature = "uses_gilrs", not(target_arch = "wasm32")))]
pub fn spawn_gilrs_controller_thread() -> mpsc::Receiver<GilrsControllerMessage>
{
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        println!("[GILRS] Starting Gilrs thread...");
        let mut gilrs = match Gilrs::new() {
            Ok(g) => g,
            Err(e) => {
                println!("[GILRS] Failed to create Gilrs instance: {e}");
                return;
            }
        };
        loop {
            while let Some(gilrs_event) = gilrs.next_event() {
                println!("[GILRS] Read event: {gilrs_event:?}");
                let _ = tx.send(GilrsControllerMessage {
                    event: Some(gilrs_event),
                });
            }
            // Sleep a bit to avoid busy loop
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });
    rx
}
