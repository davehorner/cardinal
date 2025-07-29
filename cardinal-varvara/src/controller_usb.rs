#[cfg(not(feature = "uses_usb"))]
compile_error!("controller_usb.rs should not be compiled unless the 'uses_usb' feature is enabled");

#[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
use super::controller::Controller;
#[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
use super::controller::{ControllerDevice, Key};
#[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
use crate::Event;
#[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
use std::any::Any;
#[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
use uxn::Uxn;

/// Only import and use ControllerGilrs if uses_gilrs feature is enabled
#[cfg(feature = "uses_gilrs")]
use super::controller_gilrs::ControllerGilrs;

#[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
/// USB controller device for Varvara system.
pub struct ControllerUsb {
    /// Receiver for USB controller messages.
    pub rx: std::sync::mpsc::Receiver<UsbControllerMessage>,
    /// Last pedal state received from the USB device.
    pub last_pedal: Option<u8>,
    /// Internal controller state.
    pub controller: Controller,
    /// Optional chained Gilrs controller (only if uses_gilrs)
    #[cfg(feature = "uses_gilrs")]
    pub gilrs: Option<ControllerGilrs>,
}

#[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
impl ControllerDevice for ControllerUsb {
    /// Sends a single character event
    fn char(&mut self, vm: &mut Uxn, c: u8) -> Event {
        let event = self.controller.char(vm, c);
        #[cfg(feature = "uses_gilrs")]
        if let Some(gilrs) = &mut self.gilrs {
            gilrs.char(vm, c);
        }
        event
    }

    /// Send the given key event, returning an event if needed
    fn pressed(&mut self, vm: &mut Uxn, k: Key, repeat: bool) -> Option<Event> {
        let event = self.controller.pressed(vm, k, repeat);
        #[cfg(feature = "uses_gilrs")]
        if let Some(gilrs) = &mut self.gilrs {
            gilrs.pressed(vm, k, repeat);
        }
        event
    }

    /// Indicate that the given key has been released
    ///
    /// This may change our button state and return an event
    fn released(&mut self, vm: &mut Uxn, k: Key) -> Option<Event> {
        let event = self.controller.released(vm, k);
        #[cfg(feature = "uses_gilrs")]
        if let Some(gilrs) = &mut self.gilrs {
            gilrs.released(vm, k);
        }
        event
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
impl ControllerUsb {
    /// Polls for pedal events and prints debug output for press/release changes.
    /// Call this in your main loop to process pedal events from the USB device.
    pub fn poll_pedal_event(&mut self, vm: &mut Uxn) {
        // Chain gilrs polling first, only if uses_gilrs
        #[cfg(feature = "uses_gilrs")]
        if let Some(gilrs) = &mut self.gilrs {
            gilrs.poll_gilrs_event(vm);
        }
        while let Ok(msg) = self.rx.try_recv() {
            // VEC Footpedal: pedal state is in the first byte of the message
            if let Some(&pedal_byte) = msg.data.first() {
                match self.last_pedal {
                    Some(prev) => {
                        let changed = pedal_byte ^ prev;
                        for i in 0..8 {
                            let mask = 1 << i;
                            if changed & mask != 0 {
                                if pedal_byte & mask != 0 {
                                    println!(
                                        "[USB] Pedal {i} pressed (bit {mask:02b})"
                                    );
                                } else {
                                    println!(
                                        "[USB] Pedal {i} released (bit {mask:02b})"
                                    );
                                }
                            }
                        }
                    }
                    None => {
                        println!(
                            "[USB] Initial pedal state: 0x{pedal_byte:02x}"
                        );
                    }
                }
                self.last_pedal = Some(pedal_byte);
            } else {
                println!("[USB] No pedal byte in message: {msg:?}");
            }
        }
    }

    /// Helper to construct a ControllerUsb with optional gilrs chaining.
    #[cfg(feature = "uses_gilrs")]
    pub fn new(
        controller: Controller,
        rx: std::sync::mpsc::Receiver<UsbControllerMessage>,
        gilrs: Option<ControllerGilrs>,
    ) -> Self {
        ControllerUsb {
            rx,
            last_pedal: None,
            controller,
            gilrs,
        }
    }

    #[cfg(not(feature = "uses_gilrs"))]
    pub fn new(
        controller: Controller,
        rx: std::sync::mpsc::Receiver<UsbControllerMessage>,
    ) -> Self {
        ControllerUsb {
            rx,
            last_pedal: None,
            controller,
        }
    }

    #[allow(dead_code)]
    /// Checks the current button states and returns an event if any button state changed.
    fn check_buttons(&mut self, vm: &mut Uxn, repeat: bool) -> Option<Event> {
        self.controller.check_buttons(vm, repeat)
    }
}

#[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
use hidapi::HidApi;
#[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
use std::sync::mpsc;
#[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
use std::thread;

#[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
/// Message from USB controller thread
#[derive(Debug)]
pub struct UsbControllerMessage {
    /// Raw data received from the USB device.
    pub data: Vec<u8>,
}

#[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
/// USB device configuration for controller
#[derive(Clone, Copy, Debug)]
pub struct UsbDeviceConfig {
    /// USB vendor ID
    pub vendor_id: u16,
    /// USB product ID
    pub product_id: u16,
}

#[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
impl Default for UsbDeviceConfig {
    /// Default to VEC Footpedal: idVendor = 0x05f3, idProduct = 0x00ff
    fn default() -> Self {
        // Default to VEC Footpedal: idVendor = 0x05f3, idProduct = 0x00ff
        // Alternative: Microsoft device: idVendor = 0x045e, idProduct = 0x0b13
        // Change which is returned by commenting/uncommenting as needed

        // VEC Footpedal (default)
        // UsbDeviceConfig {
        //     vendor_id: 0x05f3,
        //     product_id: 0x00ff,
        // }

        // Microsoft device (alternative)
        // UsbDeviceConfig {
        //     vendor_id: 0x045e,
        //     product_id: 0x0b13,
        // }

        // By default, return VEC Footpedal
        UsbDeviceConfig {
            vendor_id: 0x05f3,
            product_id: 0x00ff,
        }
    }
}

#[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
/// Spawns a background thread to read from a HID device and sends data over a channel
pub fn spawn_usb_controller_thread(
    config: UsbDeviceConfig,
) -> mpsc::Receiver<UsbControllerMessage> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        println!(
            "[USB] Starting HID thread, attempting to create API instance..."
        );
        let api = match HidApi::new() {
            Ok(api) => api,
            Err(e) => {
                println!("[USB] Failed to create HID API instance: {e}");
                return;
            }
        };
        println!("[USB] HID API instance created. Attempting to open device {:04x}:{:04x}...", config.vendor_id, config.product_id);
        let joystick = match api.open(config.vendor_id, config.product_id) {
            Ok(dev) => {
                println!("[USB] Device opened successfully.");
                dev
            }
            Err(e) => {
                println!("[USB] Failed to open device: {e}");
                return;
            }
        };
        loop {
            let mut buf = [0u8; 256];
            match joystick.read(&mut buf[..]) {
                Ok(res) => {
                    let data = buf[..res].to_vec();
                    println!("[USB] Read {res} bytes: {data:?}");
                    let _ = tx.send(UsbControllerMessage { data });
                }
                Err(e) => {
                    println!("[USB] Error reading from device: {e}");
                }
            }
        }
    });
    rx
}
