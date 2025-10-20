#![cfg_attr(windows, windows_subsystem = "windows")]
// Prevents a console window from opening on Windows systems
fn main() -> anyhow::Result<()> {
    cardinal_gui::entry()
}
