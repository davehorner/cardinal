# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0](https://github.com/davehorner/cardinal/compare/cardinal-uxn-v0.4.5...cardinal-uxn-v0.5.0) - 2025-10-18

### Added

- *(macos)* [**breaking**] macOS protocol handler + safer exec; docs/deps

## [0.4.5](https://github.com/davehorner/cardinal/compare/cardinal-uxn-v0.4.4...cardinal-uxn-v0.4.5) - 2025-08-08

### Other

- *(gui+build)* add e_midi support and aarch64 cross-compilation via Docker  - Enabled `uses_e_midi` feature in `cardinal-demo` (enabled by default). - Added `e_midi.rs` module with MidiPlayerThread lifecycle management. - Cleanly shuts down MIDI thread on GUI close via `AppWithClose` wrapper.  NOT TRUE - CTRL+C to EXIT.  TODO. - Reduced USB controller logging noise for cleaner output. - Added `Dockerfile.aarch64` and Windows `build_aarch64.cmd` to support   cross-compilation of cardinal-orcas for `aarch64-unknown-linux-gnu`.

## [0.4.4](https://github.com/davehorner/cardinal/compare/cardinal-uxn-v0.4.3...cardinal-uxn-v0.4.4) - 2025-07-30

### Added

- support loading and using .sym symbol files alongside ROMs.  orcas is not yet functional in terms of input.  it is in an interesting state where the letters and numbers show up;  but it is flashing blank frames and clearing the state.

### Other

- *(varvara)* add gilrs gamepad support via ControllerGilrs  - Adds gilrs support as an optional feature to cardinal-varvara and cardinal-gui - Chains ControllerGilrs with ControllerUsb when enabled - Updates poll_pedal_event to support Uxn VM injection - Enables gilrs by default in cardinal-varvara - Updates workspace Cargo.toml and lock file accordingly - Refactors release.ps1 to support multi-crate LAST_RELEASE generation  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE

## [0.4.3](https://github.com/davehorner/cardinal/compare/v0.4.2...v0.4.3) - 2025-07-29

### Other

- *(varvara)* add gilrs gamepad support via ControllerGilrs  - Adds gilrs support as an optional feature to cardinal-varvara and cardinal-gui - Chains ControllerGilrs with ControllerUsb when enabled - Updates poll_pedal_event to support Uxn VM injection - Enables gilrs by default in cardinal-varvara - Updates workspace Cargo.toml and lock file accordingly - Refactors release.ps1 to support multi-crate LAST_RELEASE generation  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE
- *(docs)* verification typo

## [0.4.2](https://github.com/davehorner/cardinal/compare/cardinal-uxn-v0.4.1...cardinal-uxn-v0.4.2) - 2025-07-28

### Added

- *(tracker)* add Tracker device for parallel mouse input support - demonstration.
- *(tracker)* add Tracker device for parallel mouse input support - demonstration.

## [0.4.1](https://github.com/davehorner/cardinal/compare/cardinal-uxn-v0.4.0...cardinal-uxn-v0.4.1) - 2025-07-26

### Added

- *(gui)* support wasm32 target on cardinal-gui, add `cardinal-orcas` bin.

### Other

- suppress unused warnings and remove redundant code in uxn and demo

## [0.4.0](https://github.com/davehorner/cardinal/compare/cardinal-uxn-v0.3.0...cardinal-uxn-v0.4.0) - 2025-07-26

### Other

- *(deps)* [**breaking**] update to the latest and greatest for all dependencies!

## [0.3.0](https://github.com/davehorner/cardinal/compare/cardinal-uxn-v0.2.0...cardinal-uxn-v0.3.0) - 2025-07-26

### Added

- *(audio)* [**breaking**] support dynamic sample rate selection between 48000 and 44100 Hz

### Other

- *(deps)* [**breaking**] update to the latest and greatest for all dependencies!

## [0.2.0](https://github.com/davehorner/cardinal/compare/cardinal-uxn-v0.1.0...cardinal-uxn-v0.2.0) - 2025-07-26

### Added

- *(audio)* [**breaking**] support dynamic sample rate selection between 48000 and 44100 Hz

## [0.1.0](https://github.com/davehorner/cardinal/releases/tag/cardinal-uxn-v0.1.0) - 2025-07-25

### Other

- rename raven project to cardinal and update related dependencies and documentation
- Add notes on building the web GUI ([#19](https://github.com/davehorner/cardinal/pull/19))
- Add fuzzing, fix things that were discovered ([#13](https://github.com/davehorner/cardinal/pull/13))
- tweak text
- Add license and stuff
- Updating README to link to project page
- Update README
- Make README more accurate
- Fix directory listing in Potato
- More README updates
- Implement arguments
- Update implementation notes
- Remove GUI loop from Varvara crate (!)
- Make Varvara audio implementation-agnostic
- use to/from_le_bytes instead
- Working aroud bad codegen
- Staring at assembly
- Begin updating README
