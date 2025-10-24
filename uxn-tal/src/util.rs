// src/util.rs
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::IsTerminal;

/// Non-cryptographic, stable hash for cache bucketing.
pub fn hash_url(url: &str) -> u64 {
    let mut h = DefaultHasher::new();
    url.hash(&mut h);
    h.finish()
}

// Helper: pause for 15 seconds on error
pub fn pause_on_error() {
    if !std::io::stderr().is_terminal() && !std::io::stdout().is_terminal() {
        // no console attached — don't pause
        return;
    }
    use std::{thread, time};
    eprintln!("\n---\nKeeping window open for 15 seconds so you can read the above. Press Enter to continue...");
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();
    #[cfg(not(target_arch = "wasm32"))]
    thread::spawn(move || {
        let mut _buf = String::new();
        let _ = std::io::stdin().read_line(&mut _buf);
        let _ = tx.send(());
    });
    let _ = rx.recv_timeout(time::Duration::from_secs(15));
}
// Helper: pause for Windows console
pub fn pause_for_windows() {
    #[cfg(target_os = "windows")]
    {
        if std::io::stdout().is_terminal() || std::io::stderr().is_terminal() {
            use std::io::Write;

            print!("Press Enter to continue...");
            let _ = std::io::stdout().flush();
            let mut _buf = String::new();
            let _ = std::io::stdin().read_line(&mut _buf);
        }
    }
}
