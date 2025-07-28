# .sym File Vector Labeling for Cardinal

This document describes the `.sym` file format and its use for runtime vector labeling and debug tracing in the Cardinal project.

## Overview

A `.sym` file provides symbolic labels for vectors (addresses, functions, or events) in a Uxn/Varvara ROM or running system. These labels are loaded at runtime to enhance debugging, tracing, and UI display by mapping numeric vector addresses to human-readable names.

## Format

- Each line in a `.sym` file represents a label mapping.
- The format is typically:

```
<address> <label>
```

- `<address>`: Hexadecimal or decimal address of the vector/function/event.
- `<label>`: Human-readable name for the address.
- Lines starting with `#` are treated as comments and ignored.

### Example

```
# Example .sym file for orca.rom
0020 main_loop
0030 input_handler
0040 render_panel
00A0 midi_out
# ...
```

## Usage in Cardinal

- When a `.sym` file is loaded, the system parses each line and builds a mapping from address to label.
- During runtime, when a vector is invoked or traced, the label is displayed instead of (or alongside) the raw address.
- This improves debugging, error reporting, and UI clarity.
- impl Varvara tries to load sym files if they exist when loading via a path.

```rust
  /// Loads a .sym file and returns a map of address -> label
    pub fn load_symbols(path: &str) -> io::Result<HashMap<u16, String>> {
        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let mut map = HashMap::new();
        let mut i = 0;
        while i + 2 < buf.len() {
            let addr = ((buf[i] as u16) << 8) | (buf[i + 1] as u16);
            i += 2;
            let mut end = i;
            while end < buf.len() && buf[end] != 0 {
                end += 1;
            }
            let label = String::from_utf8_lossy(&buf[i..end]).to_string();
            map.insert(addr, label);
            i = end + 1;
        }
        Ok(map)
    }
```
The above is how the sym file is used to build up a HashMap.

Using it looks something like:
```rust
    /// Processes a single vector event
    ///
    /// Events with an unassigned vector (i.e. 0) are ignored
    fn process_event(&mut self, vm: &mut Uxn, e: Event) {
        if e.vector != 0 {
            let label = self.vector_to_label(e.vector);
            if self.last_vector != e.vector {
                println!("[VARVARA][process_event] vector: 0x{:04x} [{}], data: {:?}", e.vector, label, e.data);
                self.last_vector = e.vector;
            }

            if let Some(d) = e.data {
                println!("[VARVARA][process_event] write_dev_mem addr: 0x{:02x}, value: 0x{:02x} ('{}')", d.addr, d.value, d.value as char);
                vm.write_dev_mem(d.addr, d.value);
            }
            vm.run(self, e.vector);
            if let Some(d) = e.data {
                if d.clear {
                    println!("[VARVARA][process_event] clear addr: 0x{:02x}", d.addr);
                    vm.write_dev_mem(d.addr, 0);
                }
            }
        }
    }
```

text labels made for humans.
```
[VARVARA][process_event] vector: 0x0aa7 [timer/on-play], data: None
```

## Features
- Supports comments and blank lines.
- Can be reloaded at runtime for live debugging.
- Used in both CLI and GUI for vector labeling.

## Future Extensions
- Integration for better source-level debugging tools.

## See Also
- [Symbols page on the XXIIVV wiki](https://wiki.xxiivv.com/site/symbols.html)
- [Varvara documentation](https://wiki.xxiivv.com/site/varvara.html)
- [Uxn project](https://wiki.xxiivv.com/site/uxn.html)

*Last updated: July 28, 2025*
David Horner