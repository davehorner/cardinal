# Gilrs Gamepad Support in cardinal

## Overview

cardinal supports gamepad input via the [`gilrs`](https://gitlab.com/gilrs-project/gilrs) crate. This enables Xbox and other standard controllers to be used for input in cardinal-based applications.

## How It Works

- When the `uses_gilrs` feature is enabled, cardinal will spawn a background thread using `gilrs` to poll for controller events.
- Button presses (A, B, X, Y, DPad, triggers, Start, Select) are mapped to internal key events and sent to the VM.
- Both DPad and Left Stick axes are mapped to arrow keys (Up, Down, Left, Right). Axis events are translated to key presses/releases based on a threshold.
- All gilrs events are printed to the console for debugging if you run with logging enabled.

## Enabling Gilrs Support

- In `Cargo.toml`, ensure the `uses_gilrs` feature is enabled for both your crate and the `cardinal-varvara` dependency.
- Example for `cardinal-gui/Cargo.toml`:
  ```toml
  [features]
  uses_gilrs = ["dep:gilrs", "varvara/uses_gilrs"]
  ```

## Usage

- Plug in your Xbox or compatible controller.
- Run your cardinal application with the `uses_gilrs` feature enabled.
- You should see `[GILRS]` debug output in the console when you press buttons or move the sticks.
- The mapped keys will be sent to the VM as if they were keyboard/controller input.

## Notes

- If your controller uses axes for DPad, both DPad and stick axes are supported.
- Button mappings can be found in `controller_gilrs.rs`.
- If you want to customize mappings, edit the match statements in that file.

## Troubleshooting

- If you do not see `[GILRS]` output, ensure the `uses_gilrs` feature is enabled in both your crate and dependencies.
- Make sure your controller is supported by `gilrs`.
- Run with logging enabled to see debug output.


### Added
- gilrs gamepad support via new `ControllerGilrs`
- Optional `uses_gilrs` feature, enabled by default
- Chaining of `ControllerGilrs` into `ControllerUsb` where applicable

### Changed
- `poll_pedal_event` now requires `&mut Uxn` to support gilrs event injection
- Refactored controller instantiation in `Varvara::new()` and `reset()` to conditionally include gilrs support