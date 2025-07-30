**Cardinal** is a fork of [Raven](https://github.com/mkeeter/raven) an independent re-implementation of the
[Uxn CPU](https://wiki.xxiivv.com/site/uxn.html)
and
[Varvara Ordinator](https://wiki.xxiivv.com/site/varvara.html).


The Uxn/Varvara ecosystem is a **personal computing stack**.

Cardinal is my personal stack for the Uxn/Varvara ecosystem.

For details on project origins, see [Raven's project writeup](https://mattkeeter.com/projects/raven).

--------------------------------------------------------------------------------

The `cardinal-uxn` crate includes two implementations of the Uxn CPU:

- The safe interpreter is a `#[no_std]` crate written in 100% safe Rust, with a
  single dependency (`zerocopy`).  It is 10-20% faster than
  the [reference implementation](https://git.sr.ht/~rabbits/uxn/tree/main/item/src)
  for CPU-heavy workloads (e.g.
  [`fib.tal`](https://git.sr.ht/~rabbits/uxn/tree/main/item/projects/examples/exercises/fib.tal),
  and
  [`mandelbrot.tal`](https://git.sr.ht/~rabbits/uxn/tree/main/item/projects/examples/demos/mandelbrot.tal)
- The unsafe ("native") interpreter is written in `aarch64` assembly (with Rust
  shims on either side), and runs 40-50% faster than the reference
  implementation

The native interpreter can be checked against the safe interpreter with fuzz
testing:

```console
cargo install cargo-fuzz # this only needs to be run once
cargo +nightly fuzz run --release fuzz-native
```

--------------------------------------------------------------------------------

The Varvara implementation (`cardinal-varvara`) includes all peripherals, and has
been tested on many of the
[flagship applications](https://wiki.xxiivv.com/site/roms.html)
(Left, Orca, Noodle, Potato).

--------------------------------------------------------------------------------

The repository includes two applications built on these libraries:

- `cardinal-cli` is a command-line application to run console-based ROMs
- `cardinal-gui` is a full-fledged GUI, which runs both as a native application and
  [Raven on the web](https://mattkeeter.com/projects/raven/demo)

The web demo is built with [`truck`](https://trunkrs.dev/), e.g.

```console
cargo install --locked trunk # this only needs to be run once
cd cardinal-gui
trunk build --release --public-url=/projects/cardinal/demo/ # edit this path
```

--------------------------------------------------------------------------------
**technology from the past come to save the future from itself**

July 2025 Changes
- stdout and stderr callbacks
- feat(audio): support dynamic sample rate selection between 48000 and 44100 Hz
- refactor(deps): update to the latest and greatest 7/25 for all dependencies!
- feat(cardinal-gui): add cardinal-demo binary with hot reload, inject, and rom cycling; cardinal-orcas binary added, wip.
- wasm notice: cardinal-gui supports wasm.  the others do not.  PRs welcome.
- [/notes/01.debug](https://github.com/davehorner/cardinal/tree/main/notes/01.debug/README.md) basic debug start.
- [/notes/02.duplicate_mouse_device](https://github.com/davehorner/cardinal/tree/main/notes/02.duplicate_mouse_device/README.md) demonstrate custom Tracker Mouse device.
- resolved 100+ clippy lints.  wasm, ubuntu, windows, and macos all show green.
- [/notes/03.hidusb_controllers](https://github.com/davehorner/cardinal/tree/main/notes/03.hidusb_controllers/README.md)
- [/notes/04.sym_file_vector_labeling](https://github.com/davehorner/cardinal/tree/main/notes/04.sym_file_vector_labeling/README.md) 
- [/notes/05.cardinal-orcas](https://github.com/davehorner/cardinal/tree/main/notes/05.cardinal-orcas/README.md)
- [/notes/06.hardware_verification](https://github.com/davehorner/cardinal/tree/main/notes/06.hardware_verification/readme.md)
- [/notes/07.xbox_controller_gilrs](https://github.com/davehorner/cardinal/tree/main/notes/07.xbox_controller_gilrs/README.md)

© 2024-2025 Matthew Keeter, David Horner
Released under the [Mozilla Public License 2.0](/LICENSE.txt)

The repository includes ROMs compiled from the `uxnemu` reference
implementation, which are © Devine Lu Linvega and released under the MIT
license; see the [`roms/`](roms/) subfolder for details.

**we do not know what exactly will come of it**