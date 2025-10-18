
// Use the real types for compatibility
use crate::Event;
use uxn::Uxn;

/// Stub for ControllerPollEvents
#[allow(dead_code)]
pub struct ControllerPollEvents;
impl ControllerPollEvents {
    /// Stub for polling USB events (does nothing).
    pub fn poll_usb_events(_controller: &mut ControllerUsb, _vm: &mut Uxn) -> Vec<Event> {
        Vec::new()
    }
}
/// Stub for controller_usb when feature = "uses_usb" is not enabled or target_arch = "wasm32"
/// Stub struct for USB controller device.
#[allow(dead_code)]
pub struct ControllerUsb;

/// Stub struct for USB device configuration.
#[allow(dead_code)]
pub struct UsbDeviceConfig;

/// Stub function for spawning USB controller thread.
#[allow(dead_code)]
pub fn spawn_usb_controller_thread(_config: UsbDeviceConfig) {}

impl Default for UsbDeviceConfig {
    fn default() -> Self {
        UsbDeviceConfig
    }
}
