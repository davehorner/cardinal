#[cfg(not(feature = "uses_gilrs"))]
compile_error!(
    "controller_gilrs.rs should not be compiled unless the 'uses_gilrs' feature is enabled"
);

use super::controller::Controller;
use super::controller::{ControllerDevice, Key};
use crate::Event;
use std::any::Any;
use uxn::Uxn;

use gilrs::EventType;
#[cfg(feature = "uses_gilrs")]
use gilrs::{Axis, Button, Event as GilrsEvent, Gilrs};
use std::sync::mpsc;
use std::thread;

#[cfg(feature = "uses_gilrs")]
/// Message from Gilrs controller thread
#[derive(Debug)]
pub struct GilrsControllerMessage {
    /// Raw Gilrs event data (can be extended as needed)
    pub event: Option<GilrsEvent>,
}

#[cfg(feature = "uses_gilrs")]
/// Gilrs controller device for Varvara system.
pub struct ControllerGilrs {
    /// Receiver for Gilrs controller messages.
    pub rx: mpsc::Receiver<GilrsControllerMessage>,
    /// Internal controller state.
    pub controller: Controller,
}

#[cfg(feature = "uses_gilrs")]
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

#[cfg(feature = "uses_gilrs")]
impl ControllerGilrs {
    /// Polls for Gilrs events and prints debug output for button/axis changes.
    pub fn poll_gilrs_event(&mut self, vm: &mut Uxn) -> Option<Vec<Event>> {
        let mut events = Vec::new();
        while let Ok(msg) = self.rx.try_recv() {
            if let Some(event) = msg.event {
                println!("[GILRS] Event: {event:?}");
                match event.event {
                    EventType::ButtonPressed(button, _) => {
                        let maybe_event = match button {
                            Button::South => self.controller.pressed(vm, Key::Char(b'A'), false),
                            Button::East => self.controller.pressed(vm, Key::Char(b'B'), false),
                            Button::North => self.controller.pressed(vm, Key::Char(b'X'), false),
                            Button::West => self.controller.pressed(vm, Key::Char(b'Y'), false),
                            Button::DPadUp => self.controller.pressed(vm, Key::Up, true),
                            Button::DPadDown => self.controller.pressed(vm, Key::Down, true),
                            Button::DPadLeft => self.controller.pressed(vm, Key::Left, true),
                            Button::DPadRight => self.controller.pressed(vm, Key::Right, true),
                            Button::LeftTrigger => {
                                self.controller.pressed(vm, Key::Char(b'L'), false)
                            }
                            Button::RightTrigger => {
                                self.controller.pressed(vm, Key::Char(b'R'), false)
                            }
                            Button::Start => self.controller.pressed(vm, Key::Char(b'S'), false),
                            Button::Select => self.controller.pressed(vm, Key::Char(b'E'), false),
                            _ => None,
                        };
                        if let Some(ev) = maybe_event {
                            events.push(ev);
                        }
                    }
                    EventType::ButtonReleased(button, _) => {
                        let maybe_event = match button {
                            Button::South => self.controller.released(vm, Key::Char(b'A')),
                            Button::East => self.controller.released(vm, Key::Char(b'B')),
                            Button::North => self.controller.released(vm, Key::Char(b'X')),
                            Button::West => self.controller.released(vm, Key::Char(b'Y')),
                            Button::DPadUp => self.controller.released(vm, Key::Up),
                            Button::DPadDown => self.controller.released(vm, Key::Down),
                            Button::DPadLeft => self.controller.released(vm, Key::Left),
                            Button::DPadRight => self.controller.released(vm, Key::Right),
                            Button::LeftTrigger => self.controller.released(vm, Key::Char(b'L')),
                            Button::RightTrigger => self.controller.released(vm, Key::Char(b'R')),
                            Button::Start => self.controller.released(vm, Key::Char(b'S')),
                            Button::Select => self.controller.released(vm, Key::Char(b'E')),
                            _ => None,
                        };
                        if let Some(ev) = maybe_event {
                            events.push(ev);
                        }
                    }
                    EventType::AxisChanged(axis, value, _) => {
                        let threshold = 0.5;
                        match axis {
                            Axis::DPadX => {
                                if value > threshold {
                                    if let Some(ev) = self.controller.pressed(vm, Key::Right, false)
                                    {
                                        events.push(ev);
                                    }
                                    if let Some(ev) = self.controller.released(vm, Key::Left) {
                                        events.push(ev);
                                    }
                                } else if value < -threshold {
                                    if let Some(ev) = self.controller.pressed(vm, Key::Left, false)
                                    {
                                        events.push(ev);
                                    }
                                    if let Some(ev) = self.controller.released(vm, Key::Right) {
                                        events.push(ev);
                                    }
                                } else {
                                    if let Some(ev) = self.controller.released(vm, Key::Left) {
                                        events.push(ev);
                                    }
                                    if let Some(ev) = self.controller.released(vm, Key::Right) {
                                        events.push(ev);
                                    }
                                }
                            }
                            Axis::DPadY => {
                                if value > threshold {
                                    if let Some(ev) = self.controller.pressed(vm, Key::Down, false)
                                    {
                                        events.push(ev);
                                    }
                                    // Optionally handle release of Up
                                } else if value < -threshold {
                                    if let Some(ev) = self.controller.pressed(vm, Key::Up, false) {
                                        events.push(ev);
                                    }
                                    // Optionally handle release of Down
                                } else {
                                    // Optionally handle release of Up/Down
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
        if events.is_empty() {
            None
        } else {
            Some(events)
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
#[cfg(feature = "uses_gilrs")]
pub fn spawn_gilrs_controller_thread() -> mpsc::Receiver<GilrsControllerMessage> {
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
