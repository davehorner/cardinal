# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.2.1...uxn-tal-v0.2.2) - 2025-10-18

### Added

- *(rom)* support .rom files as direct inputs and fetch targets like uxntal://https://github.com/davehorner/cardinal/blob/main/roms/audio.rom

## [0.2.1](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.2.0...uxn-tal-v0.2.1) - 2025-10-18

### Fixed

- *(macos)* which does not resolve cardinal-gui when used from chrome protocol handler.  attempt to use user's .cargo/bin/ path if which does not resolve.

## [0.2.0](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.18...uxn-tal-v0.2.0) - 2025-10-18

### Added

- *(macos)* [**breaking**] macOS protocol handler + safer exec; docs/deps

## [0.1.18](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.17...uxn-tal-v0.1.18) - 2025-10-13

### Added

- *(wayland)* cardinal-gui works on wayland now; uxntal protocol handler works on ubuntu. run --register; then test with xdg-open uxntal://https///wiki.xxiivv.com/etc/cccc.tal.txt

## [0.1.17](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.16...uxn-tal-v0.1.17) - 2025-10-10

### Added

- *(fetch,cli,url)* repo-aware uxntal:// resolver + GitHub/Codeberg/sr.ht fetch with BFS includes

## [0.1.16](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.15...uxn-tal-v0.1.16) - 2025-10-10

### Added

- enhance uxntal URL handling with new chrome extension and README updates; use "Open in uxntal" context menu to run tal on sanboxed pages like github raw and sourcehut raw wepages.

## [0.1.15](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.14...uxn-tal-v0.1.15) - 2025-10-09

### Fixed

- *(protocol_handler)* uxntal://http(s) urls were not being properly rewritten when munged by being a non-standard url.

## [0.1.14](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.13...uxn-tal-v0.1.14) - 2025-10-09

### Added

- *(docs, protocol_handler)* update README and enhance uxntal command-line interface with URL handling and protocol registration features on windows.

## [0.1.13](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.12...uxn-tal-v0.1.13) - 2025-10-09

### Fixed

- *(acid)* [someWord and [someOtherWords are to be considered nothing/comments.  this resolves the acid.tal file in buxn that has a [test

## [0.1.12](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.11...uxn-tal-v0.1.12) - 2025-10-09

### Added

- *(macos)* macos support is working.

## [0.1.11](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.10...uxn-tal-v0.1.11) - 2025-10-08

### Added

- buxn,uxn,uxn38 docker support.  --cmp now creates docker images for each assembler and outputs roms and disassembly for each assembler.

## [0.1.10](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.9...uxn-tal-v0.1.10) - 2025-10-07

### Fixed

- preprocessor is not done so it should not be on by default!  a lot of changes to get binary equality with drifblim tal assemblers.  --cmp has been extended to test drif against uxnasm and uxntal.  acid.tal, opctest.tal, and drif tal asm to roms identically with drif asms.

## [0.1.9](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.8...uxn-tal-v0.1.9) - 2025-10-03

### Added

- *(chocolatal)* add standalone CLI support and update README

### Fixed

- DEC2 in https://github.com/davehorner/uxn-minesweeper/ with the help of  @." soxfox" in uxn discord, hex literals should be restricted to lowercase/digit. also checkout https://codeberg.org/yorshex/minesweeper-uxn.git

## [0.1.8](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.7...uxn-tal-v0.1.8) - 2025-08-24

### Other

- *(uxn-tal)* add Chocolatal preprocessor and integrate with assembler/CLI  - Introduce `chocolatal` module for TAL preprocessing (file includes, prefix expansion, lambda/loop labels, etc.) like https://codeberg.org/davehorner/deluge-domain/src/branch/main/preprocess-tal.sh - Add `--no-pre`, `--preprocess`, `--no-intermediate`, `--stdin`, and `--cmp-pp` flags to `uxntal` CLI - Write preprocessed `.pre.tal` intermediates and remove unless disabled - Expose `Uxn` internals (`ram`, `dev`, stacks, backend) with accessor methods - Refactor `Assembler` and `Reference` fields to be `pub` for external use - Move `generate_rust_interface_module` helper to `lib.rs` - Add `cardinal_orcas_symbols` for symbol slice lookups in `cardinal-gui` - Update `Stage` with `get_bang` helper for symbol access - Add `glob` dependency and Taskfile for https://git.sr.ht/~angelwood/combee

## [0.1.7](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.6...uxn-tal-v0.1.7) - 2025-08-13

### Added

- deluge.tal assembles. \n macro, use of _ name, sublabel registration on a padding label, \t cant be skipped

## [0.1.6](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.5...uxn-tal-v0.1.6) - 2025-08-11

### Added

- orca, left, and polycat all assemble.

## [0.1.5](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.4...uxn-tal-v0.1.5) - 2025-08-09

### Added

- relativepadding,relativepaddinglabel,bracekind brace_stack. improvements to debug_assemble and batch_assembler.  now most roms assemble!

### Fixed

- adjust label reference handling (MACROS) and enhance comment reading logic in assembler and lexer modules

## [0.1.4](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.3...uxn-tal-v0.1.4) - 2025-08-08

### Added

- update assembler structure to include lambda stack and counter, modify lexer for conditional references, and enhance parser for new AST node types. Added validation files for TAL and opcode tests.

### Other

- improve assembler error handling, add scope tracking and correct ROM size management

## [0.1.3](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.2...uxn-tal-v0.1.3) - 2025-08-06

### Added

- tak.tal

## [0.1.2](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.1...uxn-tal-v0.1.2) - 2025-08-06

### Added

- fizzbuzz and pig work at the same time.

## [0.1.1](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.1.0...uxn-tal-v0.1.1) - 2025-08-05

### Added

- *(unx-tal)* at least 4 working roms. :)
