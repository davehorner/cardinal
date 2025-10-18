//! The Varvara computer system
#![warn(missing_docs)]
#[cfg(feature = "uses_gilrs")]
use crate::controller_gilrs::ControllerGilrs;
use log::warn;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::{
    io::Write,
    sync::{Arc, Mutex},
};
/// Audio handler implementation
mod audio;
mod console;
/// Controller device and input handling for the Varvara system.
pub mod controller;
mod controller_device;
/// Gilrs controller device support for the Varvara system (enabled with the `uses_gilrs` feature).
#[cfg(feature = "uses_gilrs")]
pub mod controller_gilrs;
/// USB controller device support for the Varvara system (enabled with the `uses_usb` feature).
#[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
pub mod controller_usb;

// #[cfg(not(all(feature = "uses_usb", not(target_arch = "wasm32"))))]
// /// USB controller device stub for the Varvara system (when `uses_usb` is not enabled or on wasm32).
// #[path = "controller_usb_stub.rs"]
// pub mod controller_usb;
#[cfg(all(feature = "uses_usb", target_arch = "wasm32"))]
/// USB controller device stub for the Varvara system (when `uses_usb` is enabled on wasm32 target).
#[path = "controller_usb_stub.rs"]
pub mod controller_usb;

mod datetime;
mod file;
mod mouse;
mod screen;
mod system;
mod tracker;

pub use audio::set_sample_rate;
pub use audio::StreamData;
pub use audio::CHANNELS as AUDIO_CHANNELS;
pub use console::spawn_worker as spawn_console_worker;
pub use controller::Key;
pub use mouse::MouseState;
pub use tracker::TrackerState;

use uxn::{Device, Ports, Uxn};

/// Holds ROM data and optional symbol information for Uxn.
#[derive(Clone)]
pub struct RomData {
    /// The ROM data as a vector of bytes.
    pub rom: Vec<u8>,
    /// Optional symbol information as a vector of bytes.
    pub sym: Option<Vec<u8>>,
}

/// Write to execute before calling the event vector
#[derive(Copy, Clone, Debug)]
pub struct EventData {
    /// The device address associated with the event.
    pub addr: u8,
    /// The value associated with the event (e.g., key code or pedal state).
    pub value: u8,
    /// Whether to clear the event after handling.
    pub clear: bool,
}

/// Internal events, accumulated by devices then applied to the CPU
#[derive(Copy, Clone, Debug, Default)]
pub struct Event {
    /// Tuple of `(address, value)` to write in in device memory
    pub data: Option<EventData>,

    /// Vector to trigger
    pub vector: u16,
}

/// Output from Varvara::update, which may modify the GUI
pub struct Output<'a> {
    /// Current window size
    pub size: (u16, u16),

    /// Current screen contents, as RGBA values
    pub frame: &'a [u8],

    /// The system's mouse cursor should be hidden
    pub hide_mouse: bool,

    /// Outgoing console characters sent to the `write` port
    pub stdout: Vec<u8>,

    /// Outgoing console characters sent to the `error` port
    pub stderr: Vec<u8>,

    /// Request to exit with the given error code
    pub exit: Option<i32>,
}

impl Output<'_> {
    /// Prints `stdout` and `stderr` to the console
    pub fn print(&self) -> std::io::Result<()> {
        if !self.stdout.is_empty() {
            let mut stdout = std::io::stdout().lock();
            stdout.write_all(&self.stdout)?;
            stdout.flush()?;
        }
        if !self.stderr.is_empty() {
            let mut stderr = std::io::stderr().lock();
            stderr.write_all(&self.stderr)?;
            stderr.flush()?;
        }
        Ok(())
    }

    /// Checks the results
    ///
    /// `stdout` and `stderr` are printed, and `exit(..)` is called if it has
    /// been requested by the VM.
    pub fn check(&self) -> std::io::Result<()> {
        self.print()?;
        if let Some(e) = self.exit {
            log::info!("requested exit ({e})");

            #[cfg(not(target_arch = "wasm32"))]
            std::process::exit(e);

            #[cfg(target_arch = "wasm32")]
            return Err(std::io::Error::other("exit requested"));
        }
        Ok(())
    }
}

/// Handle to the Varvara system
pub struct Varvara {
    /// System device (timers, exit, etc)
    pub system: system::System,
    /// Console device (stdout, stderr, args)
    pub console: console::Console,
    /// Datetime device (clock, date)
    pub datetime: datetime::Datetime,
    /// Audio device (sound output)
    pub audio: audio::Audio,
    /// Screen device (framebuffer, size)
    pub screen: screen::Screen,
    /// Mouse device (position, buttons, scroll)
    pub mouse: mouse::Mouse,
    /// File device (file I/O)
    pub file: file::File,
    /// Controller device (keyboard input)
    pub controller: Box<dyn controller::ControllerDevice>,
    /// Tracker device (position, buttons, scroll)
    pub tracker: tracker::Tracker,
    /// Flags indicating if we've already printed a warning about a missing dev
    pub already_warned: [bool; 16],
    /// Use USB controller (only present if feature = "uses_usb")
    #[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
    pub uses_usb: bool,
    /// Optional symbol map for vector labels
    pub symbols: Option<HashMap<u16, String>>,
    /// Last processed vector for deduplication
    pub last_vector: u16,
}

impl Default for Varvara {
    fn default() -> Self {
        #[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
        {
            let mut v = Self::new(true);
            v.symbols = None;
            v.last_vector = 0;
            v
        }
        #[cfg(any(not(feature = "uses_usb"), target_arch = "wasm32"))]
        {
            let mut v = Self::new();
            v.symbols = None;
            v.last_vector = 0;
            v
        }
    }
}

impl Device for Varvara {
    fn deo(&mut self, vm: &mut Uxn, target: u8) -> bool {
        match target & 0xF0 {
            system::SystemPorts::BASE => self.system.deo(vm, target),
            console::ConsolePorts::BASE => self.console.deo(vm, target),
            datetime::DatetimePorts::BASE => self.datetime.deo(vm, target),
            screen::ScreenPorts::BASE => self.screen.deo(vm, target),
            mouse::MousePorts::BASE => self.mouse.set_active(),
            tracker::TrackerPorts::BASE => self.tracker.set_active(),
            f if file::FilePorts::matches(f) => self.file.deo(vm, target),
            controller::ControllerPorts::BASE => (),
            a if audio::AudioPorts::matches(a) => self.audio.deo(vm, target),

            // Default case
            t => self.warn_missing(t),
        }
        !self.system.should_exit()
    }
    fn dei(&mut self, vm: &mut Uxn, target: u8) {
        match target & 0xF0 {
            system::SystemPorts::BASE => self.system.dei(vm, target),
            console::ConsolePorts::BASE => self.console.dei(vm, target),
            datetime::DatetimePorts::BASE => self.datetime.dei(vm, target),
            screen::ScreenPorts::BASE => self.screen.dei(vm, target),
            mouse::MousePorts::BASE => self.mouse.set_active(),
            tracker::TrackerPorts::BASE => self.tracker.set_active(),
            f if file::FilePorts::matches(f) => (),
            controller::ControllerPorts::BASE => (),
            a if audio::AudioPorts::matches(a) => self.audio.dei(vm, target),

            // Default case
            t => self.warn_missing(t),
        }
    }
}

impl Varvara {
    /// Loads a .sym file into the Varvara instance
    pub fn load_symbols_into_self(&mut self, path: &str) -> std::io::Result<()> {
        let map = Self::load_symbols(path)?;
        self.symbols = Some(map);
        Ok(())
    }

    /// Load a ROM from a file path and attempt to load a .sys symbol file if present
    pub fn load_sym_with_rom_path<P: AsRef<std::path::Path>>(&mut self, path: P) {
        let path = path.as_ref();
        println!("[DEBUG][VARVARA] Loading symbols from {path:?}");
        if path.exists() {
            println!("[DEBUG][VARVARA] Found ROM at {path:?}");
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
                println!("[DEBUG][VARVARA] Attempting to load symbols from {sys_path_str}");
                if sys_path.exists() {
                    let _ = self.load_symbols_into_self(&sys_path_str);
                    println!("[DEBUG][VARVARA] Loaded symbols from {sys_path_str}");
                }
            }
        }
    }

    /// Loads a .sym file from a byte vector and returns a map of address -> label
    pub fn load_symbols_from_vec(&mut self, data: &[u8]) -> io::Result<HashMap<u16, String>> {
        let map = Self::parse_symbols_from_bytes(data)?;
        self.symbols = Some(map.clone());
        Ok(map)
    }
    /// Returns a mutable reference to the USB controller, if it exists.
    pub fn controller_usb_mut(&mut self) -> Option<&mut controller_usb::ControllerUsb> {
        self.controller
            .as_mut()
            .as_any()
            .downcast_mut::<controller_usb::ControllerUsb>()
    }

    /// Looks up a label for a given vector address
    pub fn vector_to_label(&self, vector: u16) -> &str {
        if let Some(ref map) = self.symbols {
            map.get(&vector).map(|s| s.as_str()).unwrap_or("unknown")
        } else {
            "unknown"
        }
    }
    /// Builds a new instance of the Varvara peripherals
    #[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
    pub fn new(uses_usb: bool) -> Self {
        #[cfg(feature = "uses_gilrs")]
        {}
        let controller: Box<dyn controller::ControllerDevice> = if uses_usb {
            #[cfg(feature = "uses_gilrs")]
            {
                Box::new(controller_usb::ControllerUsb {
                    rx: controller_usb::spawn_usb_controller_thread(
                        controller_usb::UsbDeviceConfig::default(),
                    ),
                    last_pedal: None,
                    controller: controller::Controller::default(),
                    gilrs: Some(ControllerGilrs::new(controller::Controller::default())),
                })
            }
            #[cfg(not(feature = "uses_gilrs"))]
            {
                Box::new(controller_usb::ControllerUsb {
                    rx: controller_usb::spawn_usb_controller_thread(
                        controller_usb::UsbDeviceConfig::default(),
                    ),
                    last_pedal: None,
                    controller: controller::Controller::default(),
                })
            }
        } else {
            #[cfg(feature = "uses_gilrs")]
            {
                use crate::controller_gilrs::ControllerGilrs;

                Box::new(ControllerGilrs::new(controller::Controller::default()))
            }
            #[cfg(not(feature = "uses_gilrs"))]
            {
                Box::new(controller::Controller::default())
            }
        };
        Self {
            console: console::Console::new(),
            system: system::System::new(),
            datetime: datetime::Datetime,
            audio: audio::Audio::new(),
            screen: screen::Screen::new(),
            mouse: mouse::Mouse::new(),
            file: file::File::new(),
            controller,
            tracker: tracker::Tracker::new(),
            already_warned: [false; 16],
            uses_usb,
            symbols: None,
            last_vector: 0,
        }
    }

    /// Builds a new instance of the Varvara peripherals (non-USB/wasm version).
    #[cfg(any(not(feature = "uses_usb"), target_arch = "wasm32"))]
    pub fn new() -> Self {
        #[cfg(feature = "uses_gilrs")]
        {
            #[allow(unused_imports)]
            use controller_gilrs::ControllerGilrs;
        }
        Self {
            console: console::Console::new(),
            system: system::System::new(),
            datetime: datetime::Datetime,
            audio: audio::Audio::new(),
            screen: screen::Screen::new(),
            mouse: mouse::Mouse::new(),
            file: file::File::new(),
            controller: {
                #[cfg(feature = "uses_gilrs")]
                {
                    Box::new(ControllerGilrs::new(controller::Controller::default()))
                }
                #[cfg(not(feature = "uses_gilrs"))]
                {
                    Box::new(controller::Controller::default())
                }
            },
            tracker: tracker::Tracker::new(),
            already_warned: [false; 16],
            symbols: None,
            last_vector: 0,
        }
    }

    /// Resets the CPU, loading extra data into expansion memory
    #[cfg(all(feature = "uses_usb", not(target_arch = "wasm32")))]
    pub fn reset(&mut self, extra: &[u8]) {
        self.system.reset(extra);
        self.console = console::Console::new();
        self.audio.reset();
        self.screen = screen::Screen::new();
        self.mouse = mouse::Mouse::new();
        self.file = file::File::new();

        self.controller = if self.uses_usb {
            #[cfg(feature = "uses_gilrs")]
            {
                Box::new(controller_usb::ControllerUsb {
                    rx: controller_usb::spawn_usb_controller_thread(
                        controller_usb::UsbDeviceConfig::default(),
                    ),
                    last_pedal: None,
                    controller: controller::Controller::default(),
                    gilrs: Some(ControllerGilrs::new(controller::Controller::default())),
                })
            }
            #[cfg(not(feature = "uses_gilrs"))]
            {
                Box::new(controller_usb::ControllerUsb {
                    rx: controller_usb::spawn_usb_controller_thread(
                        controller_usb::UsbDeviceConfig::default(),
                    ),
                    last_pedal: None,
                    controller: controller::Controller::default(),
                })
            }
        } else {
            #[cfg(feature = "uses_gilrs")]
            {
                Box::new(ControllerGilrs::new(controller::Controller::default()))
            }
            #[cfg(not(feature = "uses_gilrs"))]
            {
                Box::new(controller::Controller::default())
            }
        };
        self.tracker = tracker::Tracker::new();
        self.already_warned.fill(false);
    }

    /// Resets the CPU, loading extra data into expansion memory.
    #[cfg(any(not(feature = "uses_usb"), target_arch = "wasm32"))]
    pub fn reset(&mut self, extra: &[u8]) {
        self.system.reset(extra);
        self.console = console::Console::new();
        self.audio.reset();
        self.screen = screen::Screen::new();
        self.mouse = mouse::Mouse::new();
        self.file = file::File::new();

        self.controller = {
            #[cfg(feature = "uses_gilrs")]
            {
                Box::new(ControllerGilrs::new(controller::Controller::default()))
            }
            #[cfg(not(feature = "uses_gilrs"))]
            {
                Box::new(controller::Controller::default())
            }
        };
        self.tracker = tracker::Tracker::new();
        self.already_warned.fill(false);
    }

    /// Checks whether the SHIFT key is currently down
    fn warn_missing(&mut self, t: u8) {
        if !self.already_warned[usize::from(t >> 4)] {
            warn!("unimplemented device {t:#02x}");
            self.already_warned[usize::from(t >> 4)] = true;
        }
    }

    /// Calls the screen vector
    ///
    /// This function must be called at 60 Hz
    pub fn redraw(&mut self, vm: &mut Uxn) {
        let e = self.screen.update(vm);
        self.process_event(vm, e);
    }

    /// Sets initial value for `Console/type` based on the presense of arguments
    ///
    /// This should be called before running the reset vector
    pub fn init_args(&mut self, vm: &mut Uxn, args: &[String]) {
        self.console.set_has_args(vm, !args.is_empty());
    }

    /// Returns the current output state of the system
    ///
    /// This is not idempotent; the output is taken from various accumulators
    /// and will be empty if this is called multiple times.
    #[must_use]
    pub fn output(&mut self, vm: &Uxn) -> Output<'_> {
        Output {
            size: self.screen.size(),
            frame: self.screen.frame(vm),
            hide_mouse: self.mouse.active(),
            stdout: self.console.stdout(),
            stderr: self.console.stderr(),
            exit: self.system.exit(),
        }
    }

    /// Sends arguments to the console device
    ///
    /// Leaves the console type set to `stdin`, and returns the current output
    /// state of the system
    pub fn send_args(&mut self, vm: &mut Uxn, args: &[String]) -> Output<'_> {
        for (i, a) in args.iter().enumerate() {
            self.console.set_type(vm, console::Type::Argument);
            for c in a.bytes() {
                self.process_event(vm, self.console.update(vm, c));
            }

            let ty = if i == args.len() - 1 {
                console::Type::ArgumentEnd
            } else {
                console::Type::ArgumentSpacer
            };
            self.console.set_type(vm, ty);
            self.process_event(vm, self.console.update(vm, b'\n'));
        }
        self.console.set_type(vm, console::Type::Stdin);
        self.output(vm)
    }

    /// Send a character from the keyboard (controller) device
    /// Send a character from the keyboard (controller) device
    pub fn char(&mut self, vm: &mut Uxn, k: u8) {
        println!("Sending character: {k}");
        let e = self.controller.char(vm, k);
        println!("Processing event: {e:?}");
        self.process_event(vm, e);
    }

    /// Press a key on the controller device
    /// Press a key on the controller device
    pub fn pressed(&mut self, vm: &mut Uxn, k: Key, repeat: bool) {
        // Only send pressed events for non-character keys
        println!("Pressed key: {k:?}");
        if let Key::Char(k) = k {
            // Do nothing, character keys are handled by char()
            self.char(vm, k);
        } else if let Some(e) = self.controller.pressed(vm, k, repeat) {
            println!("Processing event: {e:?}");
            self.process_event(vm, e);
        }
    }

    /// Release a key on the controller device
    pub fn released(&mut self, vm: &mut Uxn, k: Key) {
        if let Some(e) = self.controller.released(vm, k) {
            self.process_event(vm, e);
        }
    }

    /// Send a character from the console device
    pub fn console(&mut self, vm: &mut Uxn, c: u8) {
        let e = self.console.update(vm, c);
        self.process_event(vm, e);
    }

    /// Updates the mouse state
    pub fn mouse(&mut self, vm: &mut Uxn, m: MouseState) {
        if let Some(e) = self.mouse.update(vm, m) {
            self.process_event(vm, e);
        }
    }

    /// Processes pending audio events
    pub fn audio(&mut self, vm: &mut Uxn) {
        for i in 0..audio::DEV_COUNT {
            if let Some(e) = self.audio.update(vm, usize::from(i)) {
                self.process_event(vm, e);
            }
        }
    }

    /// Updates the tracker state
    pub fn tracker(&mut self, vm: &mut Uxn, m: tracker::TrackerState) {
        if let Some(e) = self.tracker.update(vm, m) {
            self.process_event(vm, e);
        }
    }

    /// Processes a single vector event
    ///
    /// Events with an unassigned vector (i.e. 0) are ignored
    pub fn process_event(&mut self, vm: &mut Uxn, e: Event) {
        if e.vector != 0 {
            let label = self.vector_to_label(e.vector);
            let skip_labels = ["timer/on-play", "on-mouse"];
            let skip_print = skip_labels.contains(&label);

            if self.last_vector != e.vector && !skip_print {
                println!(
                    "[VARVARA][process_event] vector: 0x{:04x} [{}], data: {:?}",
                    e.vector, label, e.data
                );
                self.last_vector = e.vector;
            }

            if let Some(d) = e.data {
                if !skip_print {
                    println!("[VARVARA][process_event] write_dev_mem addr: 0x{:02x}, value: 0x{:02x} ('{}')", d.addr, d.value, d.value as char);
                }
                vm.write_dev_mem(d.addr, d.value);
            }
            vm.run(self, e.vector);
            if let Some(d) = e.data {
                if d.clear {
                    if !skip_print {
                        println!("[VARVARA][process_event] clear addr: 0x{:02x}", d.addr);
                    }
                    vm.write_dev_mem(d.addr, 0);
                }
            }
        }
    }

    /// Returns the set of audio stream data handles
    pub fn audio_streams(&self) -> [Arc<Mutex<audio::StreamData>>; 4] {
        [0, 1, 2, 3].map(|i| self.audio.stream(i))
    }

    /// Sets the global mute flag for audio
    pub fn audio_set_muted(&mut self, m: bool) {
        self.audio.set_muted(m)
    }

    /// Loads a .sym file and returns a map of address -> label
    pub fn load_symbols(path: &str) -> io::Result<HashMap<u16, String>> {
        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        println!(
            "[DEBUG] Loaded symbols from {path:?}, length: {}",
            buf.len()
        );
        let mut map = HashMap::new();
        let mut i = 0;
        while i + 2 < buf.len() {
            let addr = ((buf[i] as u16) << 8) | (buf[i + 1] as u16);
            i += 2;
            let mut end = i;
            while end < buf.len() && buf[end] != 0 {
                end += 1;
            }
            let label = String::from_utf8_lossy(&buf[i..end]).to_string();
            map.insert(addr, label);
            i = end + 1;
        }
        Ok(map)
    }

    /// Parse a .sym file from a byte slice and return a map of address -> label
    pub fn parse_symbols_from_bytes(
        data: &[u8],
    ) -> std::io::Result<std::collections::HashMap<u16, String>> {
        let mut map = std::collections::HashMap::new();
        let mut i = 0;
        while i + 2 < data.len() {
            let addr = ((data[i] as u16) << 8) | (data[i + 1] as u16);
            i += 2;
            let mut end = i;
            while end < data.len() && data[end] != 0 {
                end += 1;
            }
            let label = String::from_utf8_lossy(&data[i..end]).to_string();
            map.insert(addr, label);
            i = end + 1;
        }
        Ok(map)
    }
}
