#[cfg(all(feature = "uses_e_midi", not(target_arch = "wasm32")))]
use e_midi::MidiPlayer;
#[cfg(feature = "uses_e_midi")]
pub fn init() {
    // Initialize e_midi functionality here
}

#[cfg(feature = "uses_e_midi")]
#[allow(unused_imports)]
use std::sync::{mpsc, Arc, atomic::{AtomicBool, Ordering}};
#[cfg(feature = "uses_e_midi")]
use std::thread;

#[cfg(feature = "uses_e_midi")]
#[allow(dead_code)]
pub struct MidiPlayerThread {
    handle: thread::JoinHandle<()>,
    shutdown_tx: mpsc::Sender<()>,
}

#[cfg(all(feature = "uses_e_midi", not(target_arch = "wasm32")))]
impl MidiPlayerThread {
    pub fn start() -> Self {
        let (shutdown_tx, shutdown_rx) = mpsc::channel();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let handle = thread::spawn(move || {
            let mut player = match MidiPlayer::new() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to create MidiPlayer: {e}");
                    return;
                }
            };
            let numsongs = player.get_total_song_count();
            println!("Total songs: {numsongs}");

            'outer: for i in 0..numsongs {
                println!("Playing song {}", i + 1);
                if let Err(e) = player.play_song(i, None, None) {
                    eprintln!("Failed to play song {}: {e}", i + 1);
                    continue;
                }
                // Wait until the song finishes or shutdown is requested
                loop {
                    // Check for shutdown signal
                    if shutdown_rx.try_recv().is_ok() {
                        running_clone.store(false, Ordering::SeqCst);
                        // Stop playback immediately if playing
                        // if player.is_playing() {
                        player.stop_playback();
                        // }
                        break 'outer;
                    }
                    if !player.is_playing() {
                        break;
                    }
                    std::thread::yield_now();
                    // std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
            // Optionally: stop playback if still playing
            if player.is_playing() {
                player.stop_playback();
            }
        });

        MidiPlayerThread { handle, shutdown_tx }
    }

    pub fn shutdown(self) {
        println!("Shutting down MidiPlayerThread...");
        let _ = self.shutdown_tx.send(());
        // Wait a short time to allow the thread to break out of its loop
        std::thread::sleep(std::time::Duration::from_millis(200));
        let _ = self.handle.join();
        println!("MidiPlayerThread has shut down.");
        // std::process::exit(0); // Remove this, let the main thread exit naturally
    }
}
