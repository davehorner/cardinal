# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/davehorner/cardinal/compare/uxn-tal-defined-v0.3.0...uxn-tal-defined-v0.4.0) - 2025-11-05

### Added

- *(!arg1,!stdin,basic,file://)* add !stdin support, local file provider, and basic/orca mode handling. forced timeout and cli/gui determinations via heuristics.

## [0.3.0](https://github.com/davehorner/cardinal/compare/uxn-tal-defined-v0.2.0...uxn-tal-defined-v0.3.0) - 2025-11-02

### Added

- *(git@)* [**breaking**] add git@ URL support + enhanced protocol parsing; wire into resolver/CLI; expand tests and README

### Other

- *(compatibility_matrix)* README updates and compatibility matrix.
- *(orca)* fix to bootstrap orca.rom properly.
- *(.orca)* [**breaking**] add ORCA mode, shared common crate, cache-aware emulator launchers; add PatchStorage provider; bump deps; improve caching and CLI UX  Introduce uxn-tal-common crate to share cache and utility traits used by uxn-tal and uxn-tal-defined. Add first-class ORCA mode support across protocol parsing, fetch, and emulator launchers. Improve fetchers to reuse cached artifacts when present and add a PatchStorage provider for .orca downloads. Update workspace patches to use local cardinal-uxn/cardinal-varvara. Bump a number of dependencies and clean up unused ones.  Highlights Add ORCA mode Protocol: ProtocolParser now auto-sets orca=true when the URL ends with .orca; an explicit uxntal:orca://... also works. CLI: uxn-tal detects .orca inputs and launches the chosen emulator with the canonical orca.rom plus the .orca file. Emulators: UxnMapper, CuxnMapper, and BuxnMapper accept an injected RomCache to resolve/cache the canonical ORCA ROM and build the correct argv order for [orca.rom, file.orca].  Shared common crate New uxn-tal-common with: hash_url RomEntryResolver and RomCache traits default stub implementations get_or_write_cached_rom stub (real impl in uxn-tal::util::RealRomCache) UxnMapper::is_available_in_path helper  Fetcher improvements Add PatchStorage provider with HTML scrape of .orca download links. GitHub, sr.ht, Codeberg now reuse cached entry files when present. Downloader resolves uxntal:// or raw URLs consistently, creates a stable per-URL cache dir, and copies canonical orca.rom into cache when orca=true.  CLI and util updates uxn-tal wires RealRomCache into protocol flows and supports .orca early exit paths without attempting pre-processing/assembly. New util helpers to resolve/get cached canonical orca.rom, plus a real RomEntryResolver and RealRomCache (assemble .tal ΓåÆ .rom on demand, or copy .rom directly).  API changes (breaking) EmulatorLauncher now has a lifetime parameter and is constructed with an injected RomCache. get_emulator_launcher(result, rom_cache) and get_emulator_mapper(result, rom_cache) require a cache instance and return boxed trait objects with the correct lifetime. uxn-tal-defined emu mappers are now lifetime-parametric and use the injected cache to materialize orca.rom.  Dependency and workspace changes Add workspace member uxn-tal-common; patch crates.io for varvara, uxn-tal-defined, and uxn-tal-common. Bump many deps (examples): clap 4.5.51, assert_cmd 2.1.1, reqwest 0.12.24, display-info 0.5.7, winit 0.30.12, zerocopy 0.8.27, ICU crates, unicode-ident 1.0.22, writeable 0.6.2. Remove doc-comment; update ctrlc to use dispatch2; normalize some windows-* versions. Update all cardinal-uxn uses to 0.5.6 and prefer workspace patches.  Tests Add ORCA protocol var tests, ORCA emulator arg tests, ORCA resolution tests, PatchStorage tests, and end-to-end launcher tests for buxn-gui and cardinal-gui.

## [0.2.0](https://github.com/davehorner/cardinal/compare/uxn-tal-defined-v0.1.0...uxn-tal-defined-v0.2.0) - 2025-10-30

### Added

- *(.orca)* [**breaking**] add ORCA mode, shared common crate, cache-aware emulator launchers; add PatchStorage provider; bump deps; improve caching and CLI UX

## [0.1.0](https://github.com/davehorner/cardinal/releases/tag/uxn-tal-defined-v0.1.0) - 2025-10-30

### Added

- *(rom.txt)* [**breaking**] rom.txt extensions. break out protocol into sep. crate. tests demonstrating and issue in Raven's interaction with drifblim arguments.
