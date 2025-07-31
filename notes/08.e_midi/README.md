# e_midi Support Summary

- `e_midi` integration allows cardinal to play MIDI files using the `e_midi` library.
- `cardinal-demo` will hopefully "entertain" you when `e_midi` is enabled.
- Enable with the `uses_e_midi` feature flag in Cargo.  Default flag.
- When enabled, a background thread plays all available MIDI songs in sequence.
- The thread can be gracefully shut down when the application exits.  TODO.  Use CTRL+C to exit.
- See `cardinal-gui/src/e_midi.rs` for implementation details.

**musical things should make noise out of the box**