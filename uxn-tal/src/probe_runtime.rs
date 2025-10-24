//! Probes runtime behavior of a TAL/UXN ROM via dry run or simulation.

/// Result of runtime probing, e.g. device I/O, blocking, file prompts, etc.
#[derive(Debug, Clone)]
pub struct ProbeRuntimeResult {
    pub expects_path: bool,
    pub uses_console: bool,
    pub uses_gui: bool,
    pub device_accesses: Vec<String>, // e.g. ["File/name", "Console/vector"]
    pub blocked_on_input: bool,
    pub tick_count: usize,
    pub stdout: String,
    pub stderr: String,
    // Add more runtime features as needed
}

/// Runs a dry simulation of the ROM and returns runtime features.
/// This is a stub; implement with your emulator core.
pub fn probe_runtime(rom_bytes: &[u8], max_ticks: usize) -> ProbeRuntimeResult {
    use varvara::Varvara;

    // Create RAM and load ROM
    let mut ram = Box::new([0u8; 65536]);
    let rom_len = rom_bytes.len().min(65536);
    ram[..rom_len].copy_from_slice(&rom_bytes[..rom_len]);

    // Instantiate Varvara and Uxn VM
    let mut varvara = Varvara::default();
    let mut vm = uxn::Uxn::new(&mut ram, uxn::Backend::Interpreter);

    // Reset peripherals and VM
    varvara.reset(&[]);

    // Run the ROM (simulate for max_ticks)
    let mut ticks = 0;
    while ticks < max_ticks {
        // For now, just call redraw (screen vector) as a tick
        varvara.redraw(&mut vm);
        ticks += 1;
        // TODO: Add more sophisticated tick logic, input simulation, blocking detection
    }

    // Capture output
    let stdout = String::from_utf8_lossy(&varvara.console.stdout()).to_string();
    let stderr = String::from_utf8_lossy(&varvara.console.stderr()).to_string();

    ProbeRuntimeResult {
        expects_path: false, // TODO: Detect from device_accesses
        uses_console: !stdout.is_empty() || !stderr.is_empty(),
        uses_gui: false,         // TODO: Detect from device_accesses
        device_accesses: vec![], // TODO: Track device I/O
        blocked_on_input: false, // TODO: Detect blocking
        tick_count: ticks,
        stdout,
        stderr,
    }
}
