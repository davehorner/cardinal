# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
