# Cardinal-Orcas Example

This example demonstrates how to spawn and animate viewport windows in the cardinal directions (North, South, East, West) using [egui](https://github.com/emilk/egui) and [eframe](https://github.com/emilk/egui/tree/main/crates/eframe). It is designed to showcase advanced multi-monitor support and various strategies for wrapping viewports across monitor boundaries.

## Features
- Spawns animated viewports in any cardinal direction via keyboard (N, S, E, W) or UI buttons.
- Supports multiple monitors, using real monitor geometry (via the `display-info` crate).
- Multiple wrap modes for controlling how viewports traverse and wrap around monitor edges.
- Collision detection with the parent window.

## Wrap Modes
The behavior of viewports when they move beyond the edge of their current area is controlled by the **Wrap Mode**. You can select the wrap mode from the UI at runtime.

### 1. Parent Rect
- **Description:** The viewport wraps within the bounds of the parent window only.
- **Behavior:** When a viewport moves past the edge of the parent window, it reappears on the opposite side of the parent window.

### 2. Monitor of Spawn
- **Description:** The viewport wraps within the monitor where it was originally spawned.
- **Behavior:** When a viewport moves past the edge of its spawn monitor, it reappears on the opposite side of that same monitor, regardless of the global monitor layout.

### 3. All Monitors (Sequential)
- **Description:** The viewport wraps across all monitors in a fixed, sequential order (by index).
- **Behavior:**
    - When a viewport moves past the edge of its current monitor, it jumps to the next (or previous) monitor in the list, wrapping around if necessary.
    - The relative position (e.g., vertical offset for left/right, horizontal offset for top/bottom) is preserved.
    - This mode is useful for setups where you want predictable, index-based traversal between monitors.

### 4. All Monitors (Geometric)
- **Description:** The viewport wraps across all monitors based on their geometric arrangement.
- **Behavior:**
    - When a viewport moves past the edge of its current monitor, it attempts to find a monitor that is physically adjacent in the direction of movement.
    - If no monitor exists in that direction, it wraps to the farthest monitor in that direction.
    - The relative position is preserved as much as possible.
    - This mode is useful for setups with non-linear or irregular monitor arrangements.

## How It Works
- Monitor geometry is collected at startup using the `display-info` crate and stored in a global static.
- Each viewport tracks its position, direction, and the monitor it is currently on.
- Wrapping logic is handled in `monitor_info.rs`, with separate functions for sequential and geometric strategies.
- The UI allows you to select the wrap mode and spawn new viewports interactively.

## Running the Example
1. Ensure you have Rust and the required dependencies installed.
2. Run the example from the `egui/examples/cardinal_viewports` directory:
   ```sh
   cargo run --example cardinal_viewports
   ```
3. Use the UI or keyboard shortcuts (N, S, E, W) to spawn and move viewports.
4. Experiment with different wrap modes to see how viewports traverse your monitor setup.

## Notes
- The example is designed to be non-destructive and additive; you can extend or modify the wrap logic as needed.
- The code is cross-platform, but monitor geometry is only as accurate as reported by your OS and the `display-info` crate.

---

For more information, see the source code in `main.rs` and `monitor_info.rs`.
