# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.3](https://github.com/davehorner/cardinal/compare/cardinal-varvara-v0.4.2...cardinal-varvara-v0.4.3) - 2025-07-26

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
