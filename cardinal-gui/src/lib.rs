use log::{error, info};

#[cfg_attr(target_arch = "wasm32", path = "web.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path = "native.rs")]
mod core_entry;

/// Shared entry point for both binaries
pub fn entry() -> anyhow::Result<()> {
    let out = core_entry::run();
    match &out {
        Ok(()) => info!("core::run() completed successfully"),
        Err(e) => error!("core::run() failed: {e:?}"),
    };
    out
}
pub mod stage;
pub mod uxn;
pub mod uxn_panel;

pub mod cardinal_orcas_symbols;
#[cfg(feature = "uses_e_midi")]
pub mod e_midi;
