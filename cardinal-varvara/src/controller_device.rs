use crate::{Event, Uxn};

/// Trait for controller device implementations for the Varvara system.
pub trait ControllerDevice: Send {
    /// Returns a mutable reference to self as `Any` for downcasting.
    fn as_any(&mut self) -> &mut dyn std::any::Any;
    /// Handles a character input event.
    fn char(&mut self, vm: &mut Uxn, c: u8) -> Event;
    /// Handles a key press event.
    fn pressed(
        &mut self,
        vm: &mut Uxn,
        k: super::controller::Key,
        repeat: bool,
    ) -> Option<Event>;
    /// Handles a key release event.
    fn released(
        &mut self,
        vm: &mut Uxn,
        k: super::controller::Key,
    ) -> Option<Event>;
}

impl ControllerDevice for super::controller::Controller {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn char(&mut self, vm: &mut Uxn, c: u8) -> Event {
        self.char(vm, c)
    }
    fn pressed(
        &mut self,
        vm: &mut Uxn,
        k: super::controller::Key,
        repeat: bool,
    ) -> Option<Event> {
        self.pressed(vm, k, repeat)
    }
    fn released(
        &mut self,
        vm: &mut Uxn,
        k: super::controller::Key,
    ) -> Option<Event> {
        self.released(vm, k)
    }
}
