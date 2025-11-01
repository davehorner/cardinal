# UXN-TAL Assembler and protocol

`uxntal` a url protocol handler `uxntal://` which allows users to quickly run tal, rom, and orca files via URL.  It is also comprehensive Rust library for assembling TAL (Tal Assembly Language) files into UXN ROM files.

This library provides functionality to parse TAL source code and generate bytecode compatible with the UXN virtual machine, with full symbol generation support for debugging. Unlike drif assemblers it includes line:col information in the error messages so you can ctrl+click to your source.  

It was written by reading the source for uxn/uxnasm.c; building a comparison framework to compare the output of assemblers, and lots of comparison and LLM queries.  The tools are verbose by default and not yet optimized for speed or non-development purposes.  It is a goal to be able to assemble the drif assemblers and produce identical output as the drif assemblers.  If you find something doesn't work or match up, please submit an issue.

uxn-tal and uxntal are names for the technology and a poor name for a specific project.  Given the name is published, I am going to continue with the uxl-tal and uxntal names.  The spirit of the cardinal project is a personal computing stack; and uxn-tal/uxntal crate will hopefully be used to faciliate wider usage of different assemblers, emulators, pre-processors, extensions in the UXN ecosystem.

The assembler will be referred to as cuxn in the future (Cardinal UXN).  I haven't yet broken out a cuxnasm or made any moves to rename things.

Today the primary binary is:
```
Usage:
    uxntal [flags] <input.tal|/dev/stdin> [output.rom]

Flags:
    --version, -V         Show version and exit
    --verbose, -v         Verbose output
    --rust-interface[=M]  Emit Rust symbols module (default module name: symbols)
    --cmp                 Compare disassembly for all backends
    --stdin               Read input.tal from stdin
    --cmp-pp              Compare preprocessor output (Rust vs deluge)
    --pre                 Enable preprocessing
    --preprocess          Print preprocessed output and exit
    --drif, --drifblim    Enable drifblim-compatible mode (optimizations, reference resolution)
    --debug               Enable debug output
    --r, --root[=DIR]     Set root directory for includes (default: current dir)
    --register            Register uxntal as a file handler (Windows only)
    --unregister          Unregister uxntal as a file handler (Windows only)
    --help, -h            Show this help

Behavior:
    If output.rom omitted, use input path with .rom extension, or 'out.rom' if reading from stdin.
    You can also pass /dev/stdin as the input filename to read from stdin.
    Rust interface file path: <output>.rom.symbols.rs
```

A few unique arguments to call out specifically are the `--rust-interface`, `--cmp`, and the `--register` arguments.

- `--rust-interface` generates a rust file that contains all of the labels, sizes, and offsets so that you can access that data via rust interface.  This means you can run a rom and access ram data via label.


- `--cmp` will attempt to build your tal file against a number of different asm backends.  It will use the asm backend on the host machine if it is in the path.  Otherwise, if you are running a docker daemon, it will create docker images and generate roms via docker.

- `--register` will setup a protocol handler for `uxntal://` on your system.  It will also ask you to install the e_window and cardinal-gui crates as a dependency.  This feature allows you to place `uxntal://` in front of any http(s) url and uxntal will download, assemble, cache, and run the tal/rom file pointed to by url.
```
cargo install uxn-tal
uxntal --register
uxntal uxntal://https://wiki.xxiivv.com/etc/catclock.tal.txt
```
The above will run a catclock on Windows, MacOS, and Linux.  You can prepend the uxntal:// to any valid tal url, or you can create a bookmarklet on your bookmark toolbar to launch the protocol on click of a bookmarklet.  See [uxn-tal-defined](https://crates.io/crates/uxn-tal-defined) for more details.


## cuxn Assembler Features

### ✅ Complete UXN Support
- **Errors report with line numbers!**
- **All UXN Opcodes**: Full support for all 256 UXN instructions
- **Mode Flags**: Short mode (`2`), return mode (`r`), and keep mode (`k`)
- **Verified Compatibility**: Embedded official opcode table from UXN specification [uxntal_reference.html](https://wiki.xxiivv.com/site/uxntal_reference.html)
- **ROM Generation**: Compatible with `uxncli`, `uxnemu`, and other UXN emulators

### ✅ TAL Syntax Support
- **Literals**: Hex (`#12`, `#1234`), character (`'A'`), decimal, and binary
- **Labels**: Main labels (`@main`) and sublabels (`&loop`)
- **References**: Absolute (`;main`), relative (`,loop`), and sublabel references
- **Padding**: Address padding (`|0100`) and byte skipping (`$10`)
- **Raw Strings**: String literals with automatic null termination
- **Comments**: Parenthetical comments `( like this )`

### ✅ Symbol File Generation
- **Text Format**: Human-readable address-symbol pairs for debugging
- **Binary Format**: Compact binary format for tool integration
- **Debug Support**: Full symbol table extraction for development tools

### ✅ Ergonomic API
- **Single File Assembly**: Simple one-function assembly with symbol generation
- **Batch Processing**: Process entire directories of TAL files
- **Flexible Options**: With or without symbol file generation
- **Comprehensive Error Handling**: Detailed error messages with line numbers


## Compatibility

### Verified Compatible
- ✅ `uxncli` - Command-line UXN emulator
- ✅ `uxnemu` - UXN emulator with GUI
- ✅ Official UXN opcode specification

### Generated ROMs
- Compatible with all standard UXN emulators
- Proper memory layout and addressing
- Correctly trimmed ROM files (excludes zero page padding)


## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

The repository includes ROMs and TAL files from the `uxn` reference
implementation, which are © Devine Lu Linvega and released under the MIT
license

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Related Projects

- [UXN](https://100r.co/site/uxn.html) - The UXN virtual machine
- [TAL](https://wiki.xxiivv.com/site/tal.html) - TAL assembly language documentation
- [uxnasm](https://git.sr.ht/~rabbits/uxn) - Original C implementation

## Acknowledgments

- **Devine Lu Linvega** - Creator of UXN and TAL
- **UXN Community** - Documentation and examples
- **100 Rabbits** - UXN ecosystem development
- **Binary**: `#b10101010`
- **Character**: `'A`, `'B`
- **Strings**: `"Hello World"`
