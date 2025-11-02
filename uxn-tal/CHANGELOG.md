# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.2](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.6.1...uxn-tal-v0.6.2) - 2025-11-02

### Other

- *(compatibility_matrix)* README updates and compatibility matrix.

## [0.6.1](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.6.0...uxn-tal-v0.6.1) - 2025-10-30

### Fixed

- *(orca)* fix to bootstrap orca.rom properly.

## [0.6.0](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.5.0...uxn-tal-v0.6.0) - 2025-10-30

### Added

- *(.orca)* [**breaking**] add ORCA mode, shared common crate, cache-aware emulator launchers; add PatchStorage provider; bump deps; improve caching and CLI UX

## [0.5.0](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.4.0...uxn-tal-v0.5.0) - 2025-10-30

### Added

- *(rom.txt)* [**breaking**] rom.txt extensions. break out protocol into sep. crate. tests demonstrating and issue in Raven's interaction with drifblim arguments.

## [0.4.0](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.3.0...uxn-tal-v0.4.0) - 2025-10-24

### Fixed

- *(regression)* uxntal://https://git.sr.ht/~rabbits/left/tree/main/item/src/left.tal stopped working after removing the obvious < hack.  default arm now returns a LabelRef instead of Word.

### Other

- *(uxn-tal)* [**breaking**] uxntal string lexing passes basic.tal rom strings.  speed improvement via less nth(). UTF-8/BOM-aware source loading, richer errors, probing modules, and CLI polish.  fix(cardinal-varvara): replace panics on poisoned audio stream locks with error logs; align wasm/native controller logs  refactor(lexer): track byte and char positions, tighten string and identifier lexing, improve position reporting, and stabilize token cloning  chore(cardinal-gui): silence unused cc warning in web runner  chore(emu_*): tidy imports and cfg-gated Docker-on-WASM errors  refactor(cardinal-varvara): cfg-gated controller wiring and logging cleanup  feat(uxn-tal): add --debug acceptance, Windows console pause helpers, and early show_console(); upgrade file-read path resolution errors to structured AssemblerError::FileReadError  feat(uxn-tal): introduce probe_tal and probe_runtime modules for heuristics and dry-run analysis  BREAKING CHANGE: Lexer behavior around quoted strings and identifier parsing is stricter and position accounting now uses both byte and char indices; error enum expanded (e.g., Utf8Error, FileReadError, LabelReferenceError) which may affect downstream matches.

## [0.3.0](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.2.16...uxn-tal-v0.3.0) - 2025-10-24

### Added

- *(uxn-tal)* [**breaking**] uxntal string lexing passes basic.tal rom strings.  speed improvement via less nth(). UTF-8/BOM-aware source loading, richer errors, probing modules, and CLI polish.

## [0.2.16](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.2.15...uxn-tal-v0.2.16) - 2025-10-23

### Fixed

- correct README instructions and refactor TAL include parsing with lexer extraction.  this fixes an issue with identifers being expected at EOF.  this also fixed uxntal://https://github.com/davehorner/uxn-cats/blob/main/catclock.tal which was resolving includes via regex.  the actual lexer is used to resolve includes now. :-)

## [0.2.15](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.2.14...uxn-tal-v0.2.15) - 2025-10-22

### Other

- *(wasm)* return issues after adding logging.

## [0.2.14](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.2.13...uxn-tal-v0.2.14) - 2025-10-22

### Other

- *(gui)* A lot of changes to summarize.  Add screen effects and extended window controls. uxntal_protocol.rs attempts to start defining the valid protocol/query parameters across emulators.  wasm cardinal-varvara demo works, uxntal lib now supports wasm.  cargo make serve-wasm in cardinal-gui.  the effects are loose and things are changing rapidly.  `uxntal uxntal:widget::efx^^random://https://wiki.xxiivv.com/etc/catclock.tal.txt`  Introduces a new `effects` module with various screen effects (plasma, rainbow, noise, etc.) and blending modes.  Adds CLI arguments for: - Selecting a screen effect (`--efx`, `--efxmode`, `--efxt`). - Controlling window size, position, and fit mode (`-x`, `-y`, `-w`, `-h`, `--fit`). - Configuring mouse behavior for window drag (`--mouse`) and resize (`--mouse-resize`).  # Multi-crate LAST_RELEASE

## [0.2.13](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.2.12...uxn-tal-v0.2.13) - 2025-10-21

### Fixed

- *(fetch)* update resolver to handle legacy redirect messages and improve error handling in URL resolution process

## [0.2.12](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.2.11...uxn-tal-v0.2.12) - 2025-10-21

### Added

- *(linkedin-redirects)* uxntal uxntal:widget://https://lnkd.in/ec8ySaDV redirects to catclock.  I am looking for new work; visit my linkedin if that interests you. https://www.linkedin.com/in/mrdavidhorner/

## [0.2.11](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.2.10...uxn-tal-v0.2.11) - 2025-10-20

### Added

- *(windows_console)* changes to support widgets and roms without the default console flashing.  crate now ships with a cardinal-gui-win binary that is windows subsystem, uxntal also is now gui window subsystem and allocs a console when --debug -d are passed.  --widget is now ontop.

## [0.2.10](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.2.9...uxn-tal-v0.2.10) - 2025-10-20

### Fixed

- *(protocol)* uxntal:// urls were broken in last release.

## [0.2.9](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.2.8...uxn-tal-v0.2.9) - 2025-10-20

### Added

- *(cardinal-gui, uxn-tal)* add --widget/transparent windows, ctrl-alt-drag move, ctrl+c exit, f2 debug, and uxntal proto URL variables (emu, widget); pass flags through to emulator; update README `uxntal uxntal:widget://https://wiki.xxiivv.com/etc/catclock.tal.txt`

### Other

- *(svitlyna)* smarter digit-start tokens and defined bare-word label refs  Previously any token starting with a digit was eagerly parsed as a hex number. Complex forms like 1+foo or names beginning with a digit were misclassified, and simple bare words that werenΓÇÖt macros or instructions caused a generic ΓÇ£not a macro or instructionΓÇ¥ error.  In the lexer, digit-start tokens now use a lightweight lookahead. Simple 2- or 4-char lowercase hex like ff or 1234 still become RawHex. Known instruction mnemonics are emitted as instructions. Everything else is treated as a label reference, which preserves numeric-prefixed identifiers and expressions instead of misparsing them as hex.  In the parser, a bare word that isnΓÇÖt a macro or instruction is accepted as a label reference when a matching LabelDef exists in the token stream. If no such label is present, the error message is upgraded to ΓÇ£Label reference '<name>' is not defined,ΓÇ¥ pointing to the exact line and position.  This improves correctness for numeric-prefixed identifiers, avoids accidental hex tokenization, recognizes instruction names reliably, and replaces a vague parse error with a precise undefined-label diagnostic.  git clone https://github.com/gardenappl/svitlyna.git cd svitlyna uxntal --cmp svitlyna.tal cardinal-gui svitlyna.tal_uxntal.rom showcase\autumn.qoi  this fixes: Syntax error at svitlyna.tal:1715:34 Unexpected word '+8s-clamp': not a macro or instruction  btw all the other assemblers did not work drifloon stderr: Number invalid: add32 in part-get-median/loop Invalid number: Invalid number of hex digits (`add32`) buxn

## [0.2.8](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.2.7...uxn-tal-v0.2.8) - 2025-10-19

### Fixed

- *(svitlyna)* smarter digit-start tokens and defined bare-word label refs

## [0.2.7](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.2.6...uxn-tal-v0.2.7) - 2025-10-18

### Fixed

- *(macro_context)* ?& and !& scope to the macro invocation context.  varaboy https://github.com/tbsp/varaboy/blob/main/src/varaboy.tal#L42 is the source of this fix.  varaboy runs test_roms OK.

## [0.2.6](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.2.5...uxn-tal-v0.2.6) - 2025-10-18

### Fixed

- *(fmt)* 4 the cargo fmt ppl.

## [0.2.5](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.2.4...uxn-tal-v0.2.5) - 2025-10-18

### Fixed

- *(clippy)* 4 the clippy purists

## [0.2.4](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.2.3...uxn-tal-v0.2.4) - 2025-10-18

### Fixed

- *(fetch)* make include resolution robust; re-enable pause_on_error

## [0.2.3](https://github.com/davehorner/cardinal/compare/uxn-tal-v0.2.2...uxn-tal-v0.2.3) - 2025-10-18

### Fixed

- *(includes)* additional include multi-resolution improvements.  the resolution is very forgiving, it will attempt to resolve the file at top level.  this fixes ~invalid/named.tal, which resolves to the top level via filename and root of repo.

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
