pub use crate::controller_device::ControllerDevice;

use crate::{Event, EventData};
use std::{collections::HashSet, mem::offset_of};
use uxn::{Ports, Uxn};
use zerocopy::{BigEndian, U16};

/// Helper to map pedal bits to controller key events
pub fn inject_pedal_keys(controller: &mut Controller, vm: &mut Uxn, prev: u8, pedal: u8) {
    let pedal_map = [
        (0, Key::Shift),
        (1, Key::Ctrl),
        (2, Key::Alt),
        (3, Key::Home),
        (4, Key::Up),
        (5, Key::Down),
        (6, Key::Left),
        (7, Key::Right),
    ];
    for (bit, key) in pedal_map.iter() {
        let was_down = (prev & (1 << bit)) != 0;
        let is_down = (pedal & (1 << bit)) != 0;
        if !was_down && is_down {
            controller.pressed(vm, *key, false);
        } else if was_down && !is_down {
            controller.released(vm, *key);
        }
    }
}

#[derive(zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable, zerocopy::KnownLayout)]
#[repr(C)]
/// Controller port mappings for the Varvara system.
pub struct ControllerPorts {
    /// The vector address for controller events.
    pub vector: U16<BigEndian>,
    /// The button state.
    pub button: u8,
    /// The key code.
    pub key: u8,
    /// Padding bytes.
    pub _pad: [u8; 12],
}

impl Ports for ControllerPorts {
    const BASE: u8 = 0x80;
}

impl ControllerPorts {
    /// Offset for the key field in the controller port.
    pub const KEY: u8 = Self::BASE | offset_of!(Self, key) as u8;
}

#[derive(Default)]
/// Main controller device for the Varvara system.
pub struct Controller {
    /// Keys that are currently held down
    pub down: HashSet<Key>,

    /// Current button state
    pub buttons: u8,
}

/// Key input to the controller device
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Key {
    Shift,
    Ctrl,
    Alt,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    Char(u8),
}

impl Controller {
    /// Builds a new controller with no keys held
    pub fn new() -> Self {
        Self::default()
    }

    /// Sends a single character event
    pub fn char(&mut self, vm: &mut Uxn, c: u8) -> Event {
        let p = vm.dev::<ControllerPorts>();
        let event = Event {
            vector: p.vector.get(),
            data: Some(EventData {
                addr: ControllerPorts::KEY,
                value: c,
                clear: true,
            }),
        };
        println!(
            "[og CONTROLLER][char] char: '{}' (0x{:02x}), vector: 0x{:04x}, addr: 0x{:02x}",
            c as char,
            c,
            event.vector,
            ControllerPorts::KEY
        );
        event
    }

    /// Send the given key event, returning an event if needed
    pub fn pressed(&mut self, vm: &mut Uxn, k: Key, repeat: bool) -> Option<Event> {
        if let Key::Char(k) = k {
            Some(self.char(vm, k))
        } else {
            self.down.insert(k);
            self.check_buttons(vm, repeat)
        }
    }

    /// Indicate that the given key has been released
    ///
    /// This may change our button state and return an event
    pub fn released(&mut self, vm: &mut Uxn, k: Key) -> Option<Event> {
        if !matches!(k, Key::Char(..)) {
            self.down.remove(&k);
            self.check_buttons(vm, false)
        } else {
            None
        }
    }

    /// Checks the current button states and returns an event if any button state changed.
    pub fn check_buttons(&mut self, vm: &mut Uxn, repeat: bool) -> Option<Event> {
        let mut buttons = 0;
        for (i, k) in [
            Key::Ctrl,
            Key::Alt,
            Key::Shift,
            Key::Home,
            Key::Up,
            Key::Down,
            Key::Left,
            Key::Right,
        ]
        .iter()
        .enumerate()
        {
            if self.down.contains(k) {
                buttons |= 1 << i;
            }
        }

        // We'll return this event in case we don't have a keypress event;
        // otherwise, the keypress event will call the vector (at least once)
        if buttons != self.buttons || repeat {
            let p = vm.dev_mut::<ControllerPorts>();
            self.buttons = buttons;
            p.button = buttons;
            Some(Event {
                vector: p.vector.get(),
                data: None,
            })
        } else {
            None
        }
    }
}
