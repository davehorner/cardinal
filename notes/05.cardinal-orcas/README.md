# cardinal-orcas

![cardinal-orcas](../03.hidusb_controllers/hidusb_controller_orcas.png)

an extended orca livecoding language/environment, written in rust/tal by David Horner - "hornersOrca"

This document provides an overview of the `cardinal-orcas` application and rom, the features, architecture, and usage.

## Overview

`cardinal-orcas` is a multi-panel Uxn/Varvara application for `cardinal-orcas.rom`. It integrates multiple Uxn VMs (panels) in a grid, supports USB pedal input for hands-free panel switching, and provides advanced input and debugging features.  

## Motivation
- encourage new ways to use a single orca or many orcas.
- learn and expand the Varvara Uxn Tal machine.
- bang in time across the grid, viewports, and systems.
- shared variables between multiple orca.
- extend orca is ways other people may not; in contexts other than sound/live coding.
- share knowledge with others and hope for cool results.
- have fun recreational programming.

## Features
- Multiple Uxn VM panels in a grid layout
- USB pedal and keyboard input for panel focus and navigation
- Focus highlighting and visual feedback for active panel

## Architecture
- Each panel is a separate Uxn VM instance, managed by the GUI
- USB pedal events are mapped to panel navigation or custom actions

## Usage
- Run with USB pedal support (default):
  ```sh
  cargo run --bin cardinal-orcas
  ```
- Run without USB support:
  ```sh
  cargo run --bin cardinal-orcas --no-default-features
  ```
- Use the pedal or Tab key to switch focus between panels
- Drop a ROM file onto a panel to load it
- View debug and vector label info in the UI

## BROKEN
- letters and numbers don't come thru! ha.
- multi-mouse control doesn't work either.
- in other words, its a technical preview and isn't something you can bang to yet.

## See Also
- [03.hidusb_controllers](../03.hidusb_controllers/README.md)
- [04.sym_file_vector_labeling](../04.sym_file_vector_labeling/README.md)

## Acknowledgements and Credits

`cardinal-orcas` aka `hornersOrca` is released under the MIT
2025 © Devine Lu Linvega, David Horner

Origin: fork of [https://git.sr.ht/~rabbits/orca-toy/src/orca.tal](https://git.sr.ht/~rabbits/orca-toy/tree/3f03e6847870e90841b90e4c96acc37ae042f0d6/item/src/orca.tal) , [https://git.sr.ht/~rabbits/orca-toy/src/library.tal](https://git.sr.ht/~rabbits/orca-toy/tree/3f03e6847870e90841b90e4c96acc37ae042f0d6/item/src/library.tal) and [https://git.sr.ht/~rabbits/orca-toy/src/assets.tal](https://git.sr.ht/~rabbits/orca-toy/tree/3f03e6847870e90841b90e4c96acc37ae042f0d6/item/src/assets.tal)

© Devine Lu Linvega released under the MIT.

These files are committed in tree at #[3f03e684](https://github.com/davehorner/cardinal/tree/develop/roms/orca); the rom and sym files also are within the [roms](https://github.com/davehorner/cardinal/tree/develop/roms) folder.  The `carindal-orcas` binary embeds the rom and symbol files, so there's no need for additional loose files.

The rom will run on other emulators; but the cardinal-orcas application is purpose built for `cardinal-orcas.rom` and will provide functionality not found elsewhere.  It's possible this compatibility may go away to afford some additional feature. TBD.

[orca](https://git.sr.ht/~rabbits/orca-toy) source files are provided with Devine's permission.

I suggest supporting via the link below if you can. 
[https://hundredrabbits.itch.io/orca](https://hundredrabbits.itch.io/orca)

*Last updated: July 28, 2025*
David Horner


