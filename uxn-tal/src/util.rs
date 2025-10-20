// src/util.rs
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Non-cryptographic, stable hash for cache bucketing.
pub fn hash_url(url: &str) -> u64 {
    let mut h = DefaultHasher::new();
    url.hash(&mut h);
    h.finish()
}

// Helper: pause for 15 seconds on error
pub fn pause_on_error() {
    use std::io::IsTerminal;
    if !std::io::stderr().is_terminal() && !std::io::stdout().is_terminal() {
        // no console attached — don't pause
        return;
    }
    use std::{thread, time};
    eprintln!("\n---\nKeeping window open for 15 seconds so you can read the above. Press Enter to continue...");
    use std::sync::mpsc;
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut _buf = String::new();
        let _ = std::io::stdin().read_line(&mut _buf);
        let _ = tx.send(());
    });
    let _ = rx.recv_timeout(time::Duration::from_secs(15));
}
