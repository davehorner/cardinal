# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.1](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.7.0...cardinal-gui-v0.7.1) - 2025-11-05

### Added

- *(!arg1,!stdin,basic,file://)* add !stdin support, local file provider, and basic/orca mode handling. forced timeout and cli/gui determinations via heuristics.

## [0.7.0](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.6.0...cardinal-gui-v0.7.0) - 2025-10-30

### Added

- *(.orca)* [**breaking**] add ORCA mode, shared common crate, cache-aware emulator launchers; add PatchStorage provider; bump deps; improve caching and CLI UX

## [0.6.0](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.5.0...cardinal-gui-v0.6.0) - 2025-10-30

### Added

- *(rom.txt)* [**breaking**] rom.txt extensions. break out protocol into sep. crate. tests demonstrating and issue in Raven's interaction with drifblim arguments.

## [0.5.0](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.4.9...cardinal-gui-v0.5.0) - 2025-10-24

### Added

- *(uxn-tal)* [**breaking**] uxntal string lexing passes basic.tal rom strings.  speed improvement via less nth(). UTF-8/BOM-aware source loading, richer errors, probing modules, and CLI polish.

### Other

- correct README instructions and refactor TAL include parsing with lexer extraction.  this fixes an issue with identifers being expected at EOF.  this also fixed uxntal://https://github.com/davehorner/uxn-cats/blob/main/catclock.tal which was resolving includes via regex.  the actual lexer is used to resolve includes now. :-)

## [0.4.9](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.4.8...cardinal-gui-v0.4.9) - 2025-10-22

### Other

- *(wasm)* return issues after adding logging.

## [0.4.8](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.4.7...cardinal-gui-v0.4.8) - 2025-10-22

### Other

- *(gui)* A lot of changes to summarize.  Add screen effects and extended window controls. uxntal_protocol.rs attempts to start defining the valid protocol/query parameters across emulators.  wasm cardinal-varvara demo works, uxntal lib now supports wasm.  cargo make serve-wasm in cardinal-gui.  the effects are loose and things are changing rapidly.  `uxntal uxntal:widget::efx^^random://https://wiki.xxiivv.com/etc/catclock.tal.txt`  Introduces a new `effects` module with various screen effects (plasma, rainbow, noise, etc.) and blending modes.  Adds CLI arguments for: - Selecting a screen effect (`--efx`, `--efxmode`, `--efxt`). - Controlling window size, position, and fit mode (`-x`, `-y`, `-w`, `-h`, `--fit`). - Configuring mouse behavior for window drag (`--mouse`) and resize (`--mouse-resize`).  # Multi-crate LAST_RELEASE

## [0.4.7](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.4.6...cardinal-gui-v0.4.7) - 2025-10-21

### Added

- *(linkedin-redirects)* uxntal uxntal:widget://https://lnkd.in/ec8ySaDV redirects to catclock.  I am looking for new work; visit my linkedin if that interests you. https://www.linkedin.com/in/mrdavidhorner/

## [0.4.6](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.4.5...cardinal-gui-v0.4.6) - 2025-10-20

### Added

- *(windows_console)* changes to support widgets and roms without the default console flashing.  crate now ships with a cardinal-gui-win binary that is windows subsystem, uxntal also is now gui window subsystem and allocs a console when --debug -d are passed.  --widget is now ontop.

## [0.4.5](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.4.4...cardinal-gui-v0.4.5) - 2025-10-20

### Fixed

- *(protocol)* uxntal:// urls were broken in last release.

## [0.4.4](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.4.3...cardinal-gui-v0.4.4) - 2025-10-20

### Added

- *(cardinal-gui, uxn-tal)* add --widget/transparent windows, ctrl-alt-drag move, ctrl+c exit, f2 debug, and uxntal proto URL variables (emu, widget); pass flags through to emulator; update README `uxntal uxntal:widget://https://wiki.xxiivv.com/etc/catclock.tal.txt`

## [0.4.3](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.4.2...cardinal-gui-v0.4.3) - 2025-10-18

### Fixed

- *(fmt)* 4 the cargo fmt ppl.

## [0.4.2](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.4.1...cardinal-gui-v0.4.2) - 2025-10-18

### Other

- *(clippy)* 4 the clippy purists

## [0.4.1](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.4.0...cardinal-gui-v0.4.1) - 2025-10-18

### Fixed

- *(fetch)* make include resolution robust; re-enable pause_on_error

## [0.4.0](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.3.11...cardinal-gui-v0.4.0) - 2025-10-18

### Other

- *(macos)* [**breaking**] macOS protocol handler + safer exec; docs/deps  - macOS .app registers uxntal:// and forwards to uxntal - run-after uses direct exec with PATH lookup - docs updated; minor refactors - deps refreshed (no specifics)  BREAKING CHANGE: fix shell exploit.  # Multi-crate LAST_RELEASE

## [0.3.11](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.3.10...cardinal-gui-v0.3.11) - 2025-10-13

### Added

- *(wayland)* cardinal-gui works on wayland now; uxntal protocol handler works on ubuntu. run --register; then test with xdg-open uxntal://https///wiki.xxiivv.com/etc/cccc.tal.txt

## [0.3.10](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.3.9...cardinal-gui-v0.3.10) - 2025-08-24

### Added

- *(unx-tal)* initial developer release of unx-tal

### Other

- *(uxn-tal)* add Chocolatal preprocessor and integrate with assembler/CLI  - Introduce `chocolatal` module for TAL preprocessing (file includes, prefix expansion, lambda/loop labels, etc.) like https://codeberg.org/davehorner/deluge-domain/src/branch/main/preprocess-tal.sh - Add `--no-pre`, `--preprocess`, `--no-intermediate`, `--stdin`, and `--cmp-pp` flags to `uxntal` CLI - Write preprocessed `.pre.tal` intermediates and remove unless disabled - Expose `Uxn` internals (`ram`, `dev`, stacks, backend) with accessor methods - Refactor `Assembler` and `Reference` fields to be `pub` for external use - Move `generate_rust_interface_module` helper to `lib.rs` - Add `cardinal_orcas_symbols` for symbol slice lookups in `cardinal-gui` - Update `Stage` with `get_bang` helper for symbol access - Add `glob` dependency and Taskfile for https://git.sr.ht/~angelwood/combee

## [0.3.9](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.3.8...cardinal-gui-v0.3.9) - 2025-07-31

### Added

- *(gui+build)* add e_midi support and aarch64 cross-compilation via Docker

### Other

- *(input)* hot pluggable xbox controller support and letters/numbers now work.  shared variables are also working between the grids.  the cardinal-demo supports the buttons on the xbox but the dpad doesn't work.  analog pads and repeat are not yet working.  this is functional enough that someone could actually use it.

## [0.3.8](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.3.7...cardinal-gui-v0.3.8) - 2025-07-31

### Added

- *(input)* hot pluggable xbox controller support and letters/numbers now work.  shared variables are also working between the grids.  the cardinal-demo supports the buttons on the xbox but the dpad doesn't work.  analog pads and repeat are not yet working.  this is functional enough that someone could actually use it.

## [0.3.7](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.3.6...cardinal-gui-v0.3.7) - 2025-07-30

### Added

- support loading and using .sym symbol files alongside ROMs.  orcas is not yet functional in terms of input.  it is in an interesting state where the letters and numbers show up;  but it is flashing blank frames and clearing the state.

## [0.3.5](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.3.4...cardinal-gui-v0.3.5) - 2025-07-28

### Added

- *(gui)* add USB pedal support and uxn panel grid to cardinal-orcas

## [0.3.4](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.3.3...cardinal-gui-v0.3.4) - 2025-07-27

### Added

- *(tracker)* add Tracker device for parallel mouse input support - demonstration.

## [0.3.3](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.3.2...cardinal-gui-v0.3.3) - 2025-07-26

### Other

- suppress unused warnings and remove redundant code in uxn and demo

## [0.3.2](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.3.1...cardinal-gui-v0.3.2) - 2025-07-26

### Added

- *(gui)* support wasm32 target on cardinal-gui, add `cardinal-orcas` bin.

## [0.3.1](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.3.0...cardinal-gui-v0.3.1) - 2025-07-26

### Added

- *(cardinal-gui)* add cardinal-demo binary with hot reload, inject, and rom cycling

## [0.2.0](https://github.com/davehorner/cardinal/compare/cardinal-gui-v0.1.0...cardinal-gui-v0.2.0) - 2025-07-26

### Other

- *(deps)* [**breaking**] update to the latest and greatest for all dependencies!

## [0.1.0](https://github.com/davehorner/cardinal/releases/tag/cardinal-gui-v0.1.0) - 2025-07-26

### Added

- *(audio)* [**breaking**] support dynamic sample rate selection between 48000 and 44100 Hz

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
