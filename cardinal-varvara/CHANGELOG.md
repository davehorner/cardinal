# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.2](https://github.com/davehorner/cardinal/compare/cardinal-varvara-v0.7.1...cardinal-varvara-v0.7.2) - 2025-10-18

### Other

- *(clippy)* 4 the clippy purists

## [0.7.1](https://github.com/davehorner/cardinal/compare/cardinal-varvara-v0.7.0...cardinal-varvara-v0.7.1) - 2025-10-18

### Other

- *(clippy)* 4 the clippy purists

## [0.7.0](https://github.com/davehorner/cardinal/compare/cardinal-varvara-v0.6.0...cardinal-varvara-v0.7.0) - 2025-10-18

### Other

- *(macos)* [**breaking**] macOS protocol handler + safer exec; docs/deps  - macOS .app registers uxntal:// and forwards to uxntal - run-after uses direct exec with PATH lookup - docs updated; minor refactors - deps refreshed (no specifics)  BREAKING CHANGE: fix shell exploit.  # Multi-crate LAST_RELEASE

## [0.6.0](https://github.com/davehorner/cardinal/compare/cardinal-varvara-v0.5.0...cardinal-varvara-v0.6.0) - 2025-10-18

### Added

- *(macos)* [**breaking**] macOS protocol handler + safer exec; docs/deps

## [0.5.0](https://github.com/davehorner/cardinal/compare/cardinal-varvara-v0.4.9...cardinal-varvara-v0.5.0) - 2025-10-13

### Added

- *(unx-tal)* initial developer release of unx-tal
- *(gui)* add USB pedal support and uxn panel grid to cardinal-orcas
- *(tracker)* add Tracker device for parallel mouse input support - demonstration.
- *(tracker)* add Tracker device for parallel mouse input support - demonstration.
- *(gui)* support wasm32 target on cardinal-gui, add `cardinal-orcas` bin.
- *(cardinal-gui)* add cardinal-demo binary with hot reload, inject, and rom cycling
- *(audio)* [**breaking**] support dynamic sample rate selection between 48000 and 44100 Hz

### Other

- *(uxn-tal)* add Chocolatal preprocessor and integrate with assembler/CLI  - Introduce `chocolatal` module for TAL preprocessing (file includes, prefix expansion, lambda/loop labels, etc.) like https://codeberg.org/davehorner/deluge-domain/src/branch/main/preprocess-tal.sh - Add `--no-pre`, `--preprocess`, `--no-intermediate`, `--stdin`, and `--cmp-pp` flags to `uxntal` CLI - Write preprocessed `.pre.tal` intermediates and remove unless disabled - Expose `Uxn` internals (`ram`, `dev`, stacks, backend) with accessor methods - Refactor `Assembler` and `Reference` fields to be `pub` for external use - Move `generate_rust_interface_module` helper to `lib.rs` - Add `cardinal_orcas_symbols` for symbol slice lookups in `cardinal-gui` - Update `Stage` with `get_bang` helper for symbol access - Add `glob` dependency and Taskfile for https://git.sr.ht/~angelwood/combee  # Multi-crate LAST_RELEASE
- *(gui+build)* add e_midi support and aarch64 cross-compilation via Docker  - Enabled `uses_e_midi` feature in `cardinal-demo` (enabled by default). - Added `e_midi.rs` module with MidiPlayerThread lifecycle management. - Cleanly shuts down MIDI thread on GUI close via `AppWithClose` wrapper.  NOT TRUE - CTRL+C to EXIT.  TODO. - Reduced USB controller logging noise for cleaner output. - Added `Dockerfile.aarch64` and Windows `build_aarch64.cmd` to support   cross-compilation of cardinal-orcas for `aarch64-unknown-linux-gnu`.
- *(input)* hot pluggable xbox controller support and letters/numbers now work.  shared variables are also working between the grids.  the cardinal-demo supports the buttons on the xbox but the dpad doesn't work.  analog pads and repeat are not yet working.  this is functional enough that someone could actually use it.
- support loading and using .sym symbol files alongside ROMs.  orcas is not yet functional in terms of input.  it is in an interesting state where the letters and numbers show up;  but it is flashing blank frames and clearing the state.  - Automatically detects and loads .sym files matching downloaded or static ROMs. - Added RomData struct to pair ROMs with optional symbol metadata. - Enhanced cardinal-demo to fetch, persist, and use symbol files from GitHub. - Updated auto ROM cycling to preserve and load associated .sym files. - Integrated symbol loading into Varvara and UxnApp lifecycle. - Added detailed debug output for ROM and symbol file handling. - Improved controller USB char handling to emit complete events. - Added handle_usb_input method and refined input event forwarding.
- *(varvara)* add gilrs gamepad support via ControllerGilrs  - Adds gilrs support as an optional feature to cardinal-varvara and cardinal-gui - Chains ControllerGilrs with ControllerUsb when enabled - Updates poll_pedal_event to support Uxn VM injection - Enables gilrs by default in cardinal-varvara - Updates workspace Cargo.toml and lock file accordingly - Refactors release.ps1 to support multi-crate LAST_RELEASE generation  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE
- *(docs)* verification typo
- disable piano, controller, and audio tests for release
- release varvara
- release-plz
- suppress unused warnings and remove redundant code in uxn and demo
- *(deps)* [**breaking**] update to the latest and greatest for all dependencies!
- add changelogs for cardinal-uxn and cardinal-varvara projects
- add Release-plz GitHub Actions workflow configuration
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

## [0.4.9](https://github.com/davehorner/cardinal/compare/cardinal-varvara-v0.4.8...cardinal-varvara-v0.4.9) - 2025-08-08

### Added

- *(unx-tal)* initial developer release of unx-tal

### Other

- *(gui+build)* add e_midi support and aarch64 cross-compilation via Docker  - Enabled `uses_e_midi` feature in `cardinal-demo` (enabled by default). - Added `e_midi.rs` module with MidiPlayerThread lifecycle management. - Cleanly shuts down MIDI thread on GUI close via `AppWithClose` wrapper.  NOT TRUE - CTRL+C to EXIT.  TODO. - Reduced USB controller logging noise for cleaner output. - Added `Dockerfile.aarch64` and Windows `build_aarch64.cmd` to support   cross-compilation of cardinal-orcas for `aarch64-unknown-linux-gnu`.

## [0.4.8](https://github.com/davehorner/cardinal/compare/cardinal-varvara-v0.4.7...cardinal-varvara-v0.4.8) - 2025-07-31

### Added

- *(gui+build)* add e_midi support and aarch64 cross-compilation via Docker

### Other

- *(input)* hot pluggable xbox controller support and letters/numbers now work.  shared variables are also working between the grids.  the cardinal-demo supports the buttons on the xbox but the dpad doesn't work.  analog pads and repeat are not yet working.  this is functional enough that someone could actually use it.

## [0.4.7](https://github.com/davehorner/cardinal/compare/cardinal-varvara-v0.4.6...cardinal-varvara-v0.4.7) - 2025-07-31

### Added

- *(input)* hot pluggable xbox controller support and letters/numbers now work.  shared variables are also working between the grids.  the cardinal-demo supports the buttons on the xbox but the dpad doesn't work.  analog pads and repeat are not yet working.  this is functional enough that someone could actually use it.

### Other

- support loading and using .sym symbol files alongside ROMs.  orcas is not yet functional in terms of input.  it is in an interesting state where the letters and numbers show up;  but it is flashing blank frames and clearing the state.  - Automatically detects and loads .sym files matching downloaded or static ROMs. - Added RomData struct to pair ROMs with optional symbol metadata. - Enhanced cardinal-demo to fetch, persist, and use symbol files from GitHub. - Updated auto ROM cycling to preserve and load associated .sym files. - Integrated symbol loading into Varvara and UxnApp lifecycle. - Added detailed debug output for ROM and symbol file handling. - Improved controller USB char handling to emit complete events. - Added handle_usb_input method and refined input event forwarding.

## [0.4.6](https://github.com/davehorner/cardinal/compare/cardinal-varvara-v0.4.5...cardinal-varvara-v0.4.6) - 2025-07-30

### Added

- support loading and using .sym symbol files alongside ROMs.  orcas is not yet functional in terms of input.  it is in an interesting state where the letters and numbers show up;  but it is flashing blank frames and clearing the state.

### Other

- *(varvara)* add gilrs gamepad support via ControllerGilrs  - Adds gilrs support as an optional feature to cardinal-varvara and cardinal-gui - Chains ControllerGilrs with ControllerUsb when enabled - Updates poll_pedal_event to support Uxn VM injection - Enables gilrs by default in cardinal-varvara - Updates workspace Cargo.toml and lock file accordingly - Refactors release.ps1 to support multi-crate LAST_RELEASE generation  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE

## [0.4.5](https://github.com/davehorner/cardinal/compare/cardinal-varvara-v0.4.5...cardinal-varvara-v0.4.6) - 2025-07-29

### Other

- *(varvara)* add gilrs gamepad support via ControllerGilrs  - Adds gilrs support as an optional feature to cardinal-varvara and cardinal-gui - Chains ControllerGilrs with ControllerUsb when enabled - Updates poll_pedal_event to support Uxn VM injection - Enables gilrs by default in cardinal-varvara - Updates workspace Cargo.toml and lock file accordingly - Refactors release.ps1 to support multi-crate LAST_RELEASE generation  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE  # Multi-crate LAST_RELEASE

## [0.4.4](https://github.com/davehorner/cardinal/compare/cardinal-varvara-v0.4.3...cardinal-varvara-v0.4.4) - 2025-07-28

### Added

- *(gui)* add USB pedal support and uxn panel grid to cardinal-orcas
- *(tracker)* add Tracker device for parallel mouse input support - demonstration.

## [0.4.3](https://github.com/davehorner/cardinal/compare/cardinal-varvara-v0.4.2...cardinal-varvara-v0.4.3) - 2025-07-27

### Added

- *(tracker)* add Tracker device for parallel mouse input support - demonstration.

### Other

- release-plz

## [0.4.2](https://github.com/davehorner/cardinal/compare/cardinal-varvara-v0.4.1...cardinal-varvara-v0.4.2) - 2025-07-26

### Other

- update Cargo.toml dependencies

## [0.4.1](https://github.com/davehorner/cardinal/compare/cardinal-varvara-v0.4.0...cardinal-varvara-v0.4.1) - 2025-07-26

### Added

- *(gui)* support wasm32 target on cardinal-gui, add `cardinal-orcas` bin.

### Other

- suppress unused warnings and remove redundant code in uxn and demo

## [0.4.0](https://github.com/davehorner/cardinal/compare/cardinal-varvara-v0.3.0...cardinal-varvara-v0.4.0) - 2025-07-26

### Other

- *(deps)* [**breaking**] update to the latest and greatest for all dependencies!

## [0.3.0](https://github.com/davehorner/cardinal/compare/cardinal-varvara-v0.2.0...cardinal-varvara-v0.3.0) - 2025-07-26

### Added

- *(audio)* [**breaking**] support dynamic sample rate selection between 48000 and 44100 Hz

### Other

- *(deps)* [**breaking**] update to the latest and greatest for all dependencies!

## [0.2.0](https://github.com/davehorner/cardinal/compare/cardinal-varvara-v0.1.0...cardinal-varvara-v0.2.0) - 2025-07-26

### Added

- *(audio)* [**breaking**] support dynamic sample rate selection between 48000 and 44100 Hz

## [0.1.0](https://github.com/davehorner/cardinal/releases/tag/cardinal-varvara-v0.1.0) - 2025-07-25

### Other

- add Release-plz GitHub Actions workflow configuration
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
