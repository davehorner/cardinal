# UXN-TAL Assembler and protocol

A fast and comprehensive Rust library for assembling TAL (Tal Assembly Language) files into UXN ROM files.  It is also a url protocol handler `uxntal://` which allows users to quickly run tal and rom files via URL.

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
    --r, --root[=DIR]     Set root directory for includes (default: current dir)
    --register            Register uxntal as a file handler
    --debug, -d           Show debug console and extra diagnostic output in the emulator (cardinal-gui windows only)
    --ontop[=true|false]  Make window always-on-top (widget implies ontop unless --ontop=false)
    --widget              Widget mode: implies --transparent=ffffff, --no-decorations, ctrl-move, and ontop
    --help, -h            Show this help

Behavior:
    If output.rom omitted, use input path with .rom extension, or 'out.rom' if reading from stdin.
    You can also pass /dev/stdin as the input filename to read from stdin.
    Rust interface file path: <output>.rom.symbols.rs
```

A few unique arguments to call out specifically are the `--rust-interface`, `--cmp`, and the `--register` arguments.

- `--rust-interface` generates a rust file that contains all of the labels, sizes, and offsets so that you can access that data via rust interface.  This means you can run a rom and access ram data via label.


- `--cmp` will attempt to build your tal file against a number of different asm backends.  It will use the asm backend on the host machine if it is in the path.  Otherwise, if you are running a docker daemon, it will create docker images and generate roms via docker.

- `--register` been tested on Windows, MacOS, and Linux.  `--register` will setup a protocol handler for `uxntal://` on your system.  It will also install the e_window and cardinal-gui crates as a dependency.  This feature allows you to place `uxntal://` in front of any http(s) url and uxntal will download, assemble, cache, and run the tal/rom file pointed to by url.
```
cargo install uxn-tal
uxntal --register
explorer uxntal://https://wiki.xxiivv.com/etc/catclock.tal.txt
```
The above will run a catclock from cmd.exe/pwsh.  You can prepend the uxntal:// to any valid tal url, or you can create a bookmarklet on your bookmark toolbar to launch the protocol on click of the bookmarklet.
```
javascript:(function(){window.open('uxntal://' + location.href);})();
```

This works for most webpages, however content from raw.githubusercontent.com/... is not a normal webpage and it’s delivered inside a sandboxed iframe by GitHub.  This means executing JavaScript from bookmarklets will not work on those pages.

To provide another means of opening sandboxed webpages, you can install the [open-in-uxntal](https://github.com/davehorner/cardinal/tree/main/uxn-tal/open-in-uxntal) chrome extension via [chrome://extensions](chrome://extensions) `"Load unpacked"` button and pointing to the open-in-uxntal folder.  The extension exposes a new right click context menu on the webpage for `Open in uxntal`.

You can just prefix any url with uxntal:// and it should work.  Extensions and other things constructing a custom url should use open and url encode so that urls are not invalid and munged when parsed.  The simple prefix is for the user without the extension or bookmarklet installed.  Other functionality such as selection of asm/emu/pre may be added in the future and controlled via variables in the preferred open encoded form.
```
uxntal://open?url=ENC or uxntal://open/?url=ENC
multiple query params (...&url=...)
uxntal://b64,<payload> (URL-safe base64)
percent-encoded full URLs after the scheme
over-slashed forms (https///, https//, etc.)
```

Using the extension and the bookmarklet, you will find a chrome dialog pop that asks if you want to run uxntal.exe to open an application.  Using the bookmarklet you sometimes have the option to allow always; the extension does not provide this option so you always have a second click to acknowledge the website opening the application.

The protocol handler now has a provider abstraction over Github/Codeberg/Sourcehut urls; this means that the bookmarklet will work on view/log/blame pages on these websites.  Additionally, the downloader now parses and downloads all the includes. 

For example `explorer uxntal://https://git.sr.ht/~rabbits/left/tree/main/item/src/left.tal`, which is a project with a few includes, runs fine.

If you are often viewing code from a site like github, using the bookmarklet on a view/blame/history page instead of the raw allows you to use the protocol without the permission dialog being raised.  It's possible uxntal.exe could register a NativeMessagingHosts endpoint so that the chrome extension isn't using the protocol handler but instead invoke chrome.runtime.sendNativeMessage to side step the additional chrome dialog.

On macOS, `--register` creates a minimal GUI `.app` bundle in your `~/Applications` folder that registers the `uxntal://` protocol. This bundle launches your installed `uxntal` binary with the URL as an argument.

**Requirements:**
- You must have the `uxntal` binary installed (e.g., via `cargo install uxn-tal`).
- You must have Xcode command line tools installed (`xcode-select --install`).

## `uxntal:variables:key^^value://` Protocol Handler Variable Support 

The protocol handler supports passing variables and flags directly in the protocol portion of URL using key-value pairs. These are used to select emulator or pass options.

Variables are specified separated by colons (`:`). Key-value pairs use either `^` or `^^` as separators.

`cardinal-gui` supports the `--widget` flag, which turns on transparency for white pixels, disables window decorations, enables ctrl+alt+click-and-drag to move the window, and sets the window always-on-top by default. If you want to disable always-on-top while using widget mode, pass `ontop^false` in the URL or use `--ontop=false` on the CLI.

[uxntal:widget://https://wiki.xxiivv.com/etc/catclock.tal.txt](uxntal:widget://https://wiki.xxiivv.com/etc/catclock.tal.txt) will open in `cardinal-gui` in --widget mode.  catclock will show in a transparent window with no application decorations and will be always-on-top unless you specify `ontop^false`.

In order to support different emulators, you can pass an `emu` variable, which currently supports buxn,cuxn,uxnemu emulators if they are within the PATH.

[uxntal:emu^^buxn://https://wiki.xxiivv.com/etc/catclock.tal.txt](uxntal:emu^^buxn://https://wiki.xxiivv.com/etc/catclock.tal.txt) will open in `buxn-gui`

You can use either `^` or `^^` as a separator. Both are supported and percent-encoded forms (e.g., `%5E`, `%5E%5E`) are also decoded automatically.
The reason for both `^` and `^^` is that on windows, you must escape `^` with another `^`, so if you want a string that can be pasted in cmd.exe; use the double `^^` form.

### Examples

- `uxntal:emu^uxn://https://wiki.xxiivv.com/etc/catclock.tal.txt` launches catclock in the `uxnemu` emulator.
- `uxntal:emu^buxn://https://wiki.xxiivv.com/etc/catclock.tal.txt` launches catclock in the `buxn-gui` emulator.
- `uxntal:widget://https://wiki.xxiivv.com/etc/catclock.tal.txt` passes the `--widget` flag to the emulator. (cardinal-gui is the only emu that supports this flag currently)
- `uxntal:widget:ontop^false://https://wiki.xxiivv.com/etc/catclock.tal.txt` passes the `--widget` flag but disables always-on-top, showing widget mode without ontop.
- `uxntal:widget:debug://https://wiki.xxiivv.com/etc/cccc.tal.txt` opens cccc in widget mode with debug console enabled. (windows only)

You might create multiple bookmarklets to launch urls in the emulator and with the settings you desire.  Right now, the variables are restricted to `widget`,`debug`,`ontop` and `emu` to limit arbitrary input on emu invocation.

## Warning

A single click protocol handler that assembles and runs arbitrary code is considered a dangerous activity. uxntal protocol handler 0.1.18 and earlier had a shell exploit that could allow someone to craft a url/website which could run arbitrary code on your machine.  This security concern has been addressed, in 0.2.0.  This disclaimer is here to educate users on the security concerns involved, to request additional eyes for security, and to remind the user to apply upgrades as they become available so that any new security concerns found can be patched.  

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

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
uxn-tal = "0.1.0"
```

### Basic Usage

```rust
use uxn_tal::Assembler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tal_source = r#"
        |0100 @main
            #48 #65 #6c #6c #6f  ( "Hello" )
            #20 #57 #6f #72 #6c #64 #21 #0a  ( " World!\n" )
            BRK
    "#;
    
    let mut assembler = Assembler::new();
    let rom = assembler.assemble(tal_source)?;
    
    // Save ROM file
    std::fs::write("hello.rom", rom)?;
    
    // Generate symbol file
    let symbols = assembler.generate_symbol_file();
    std::fs::write("hello.sym", symbols)?;
    
    Ok(())
}
```

### File-Based Assembly

```rust
use uxn_tal::assemble_file_with_symbols;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Assembles hello.tal -> hello.rom + hello.sym
    let (rom_path, sym_path, size) = assemble_file_with_symbols("hello.tal")?;
    
    println!("Generated {} bytes: {} + {}", 
        size, 
        rom_path.display(), 
        sym_path.display()
    );
    
    Ok(())
}
```

### Batch Processing

```rust
use uxn_tal::assemble_directory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Process all .tal files in a directory
    let results = assemble_directory("examples/", true)?; // true = generate symbols
    
    for (tal_path, rom_path, sym_path, size) in results {
        println!("✅ {} -> {} ({} bytes)", 
            tal_path.file_name().unwrap().to_string_lossy(),
            rom_path.file_name().unwrap().to_string_lossy(),
            size
        );
        if let Some(sym_path) = sym_path {
            println!("  + {}", sym_path.file_name().unwrap().to_string_lossy());
        }
    }
    
    Ok(())
}
```

## TAL Syntax Reference

### Literals
```tal
#12      ( hex byte literal )
#1234    ( hex short literal )
'A       ( character literal )
42       ( decimal literal )
#b101010 ( binary literal )
```

### Labels and References
```tal
@main           ( define main label )
&loop           ( define sublabel under current main label )
;main           ( absolute reference to main label )
,loop           ( relative reference to sublabel )
/target         ( relative reference to any label )
```

### Instructions with Mode Flags
```tal
ADD      ( basic instruction )
ADD2     ( short mode - operates on 16-bit values )
ADDr     ( return mode - operates on return stack )
ADDk     ( keep mode - keeps operands on stack )
ADD2rk   ( combined modes )
```

### Padding and Skipping
```tal
|0100    ( pad to absolute address 0x0100 )
$10      ( skip 16 bytes, filling with zeros )
```

### Raw Strings
```tal
"Hello World! 00  ( raw string with explicit null terminator )
```

## Symbol File Format

### Text Format (.sym)
```
0100 main
010A main/loop
0115 data
0125 end
```

### Binary Format
Compact binary format with:
- Address as little-endian `u16`
- Symbol name as null-terminated string
- Repeating for each symbol

## API Reference

### Core Types

#### `Assembler`
Main assembler instance for processing TAL source code.

```rust
impl Assembler {
    pub fn new() -> Self
    pub fn assemble(&mut self, source: &str) -> Result<Vec<u8>, AssemblerError>
    pub fn generate_symbol_file(&self) -> String
    pub fn generate_symbol_file_binary(&self) -> Vec<u8>
    pub fn symbols(&self) -> &HashMap<String, Symbol>
}
```

#### `AssemblerError`
Comprehensive error types with detailed messages:
- `SyntaxError { line: usize, message: String }`
- `UnknownOpcode { opcode: String }`
- `UndefinedLabel { label: String }`
- `DuplicateLabel { label: String }`
- `InvalidNumber { value: String }`
- `Io(std::io::Error)`

### Convenience Functions

```rust
// Assemble single file
pub fn assemble_file<P: AsRef<Path>>(input_path: P) -> Result<Vec<u8>, AssemblerError>

// Assemble file to ROM
pub fn assemble_file_to_rom<P: AsRef<Path>, Q: AsRef<Path>>(
    input_path: P, 
    output_path: Q
) -> Result<usize, AssemblerError>

// Auto-generate ROM with same basename
pub fn assemble_file_auto<P: AsRef<Path>>(
    input_path: P
) -> Result<(PathBuf, usize), AssemblerError>

// Assemble with symbol generation
pub fn assemble_file_with_symbols<P: AsRef<Path>>(
    input_path: P
) -> Result<(PathBuf, PathBuf, usize), AssemblerError>

// Batch process directory
pub fn assemble_directory<P: AsRef<Path>>(
    dir_path: P, 
    generate_symbols: bool
) -> Result<Vec<(PathBuf, PathBuf, Option<PathBuf>, usize)>, AssemblerError>
```

## Examples

The `examples/` directory contains comprehensive demonstrations:

- `comprehensive_demo.rs` - Complete feature showcase
- `batch_assembler.rs` - Batch processing with CLI options
- `test_symbols.rs` - Symbol generation testing
- `hello_world.rs` - Basic usage example

Run examples with:
```bash
cargo run --example comprehensive_demo
cargo run --example batch_assembler -- --symbols
```

## Testing

The library includes comprehensive tests covering all functionality:

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_skip_directive

# Run with verbose output
cargo test -- --nocapture
```

## Compatibility

### Verified Compatible
- ✅ `uxncli` - Command-line UXN emulator
- ✅ `uxnemu` - UXN emulator with GUI
- ✅ Official UXN opcode specification

### Generated ROMs
- Compatible with all standard UXN emulators
- Proper memory layout and addressing
- Correctly trimmed ROM files (excludes zero page padding)

## Performance

- **Fast Assembly**: Optimized two-pass assembler
- **Memory Efficient**: Minimal memory usage during assembly
- **Batch Processing**: Efficient directory processing
- **Error Recovery**: Detailed error reporting with line numbers


### Labels and References
- **Label Definition**: `@main`, `@loop`
- **Label Reference**: `;main`, `;data`
- **Sublabel Definition**: `&loop`, `&end`
- **Sublabel Reference**: `,loop`, `,end`
- **Relative Reference**: `/loop` (for short jumps)

### Instructions
All UXN instructions are supported with mode flags:
- **Base Instructions**: `ADD`, `SUB`, `MUL`, `DIV`, `JMP`, `JSR`, etc.
- **Short Mode**: `ADD2`, `LDA2`, `STA2` (operates on 16-bit values)
- **Return Mode**: `ADDr`, `LDAr` (uses return stack)
- **Keep Mode**: `ADDk`, `LDAk` (doesn't consume stack values)
- **Combined Modes**: `ADD2rk`, `LDA2k`, etc.

### Directives
- **Padding**: `|0100` (pad to address 0x0100)
- **Comments**: `( this is a comment )`

## Examples

### Hello World
```tal
( Hello World Program )

|0100 @reset
    ;hello-world print-string
BRK

@print-string ( string* -- )
    &loop
        LDAk #18 DEO INC2 
        LDAk ,&loop JCN
    POP2 JMP2r

@hello-world
    "Hello 20 "World! 0a 00
```

### Counter Program
```tal
( Counter Program )

|0100 @reset
    #00 ;counter STA
    
    &main-loop
        ;counter LDA 
        DUP #30 ADD #18 DEO ( print digit )
        #20 #18 DEO ( print space )
        
        INC DUP ;counter STA
        #0a LTH ,&main-loop JCN
        
        #0a #18 DEO ( newline )
BRK

@counter $1
```

## Running Examples

```bash
# Run the hello world example
cargo run --example hello_world

# Run the counter example  
cargo run --example counter
```

## Testing

Run the test suite:

```bash
cargo test
```

## Architecture

The assembler uses a two-pass approach:

1. **First Pass**: Tokenize source, parse into AST, collect label definitions, and generate initial code
2. **Second Pass**: Resolve forward references and patch addresses

### Modules

- **`lexer`**: Tokenizes TAL source code
- **`parser`**: Converts tokens into Abstract Syntax Tree
- **`opcodes`**: UXN instruction definitions and mode handling
- **`assembler`**: Main assembly logic with symbol table management
- **`rom`**: ROM file generation and binary output
- **`error`**: Comprehensive error types with detailed messages

## Error Handling

All errors include file name, line number, position, and source line for actionable debugging. Common error types:
- Syntax errors (with file, line, position, and source context)
- Undefined label references
- Duplicate label definitions
- Invalid number formats
- Unknown opcodes
- ROM size limits
- Invalid addressing modes

#### `Assembler`
Main assembler instance for processing TAL source code.

```rust
impl Assembler {
    pub fn new() -> Self
    pub fn assemble(&mut self, source: &str, context: Option<String>) -> Result<Vec<u8>, AssemblerError>
    pub fn generate_symbol_file(&self) -> String
    pub fn generate_symbol_file_binary(&self) -> Vec<u8>
    pub fn symbols(&self) -> &HashMap<String, Symbol>
}
```
#### `AssemblerError`
Comprehensive error types with detailed messages:
- `SyntaxError { path: String, line: usize, position: usize, message: String, source_line: String }`
- `UnknownOpcode { opcode: String }`
- `UndefinedLabel { label: String }`
- `DuplicateLabel { label: String }`
- `InvalidNumber { value: String }`
- `Io(std::io::Error)`

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
- [ruxnasm](https://github.com/bellinitte/ruxnasm) - Another Rust implementation

## Acknowledgments

- **Devine Lu Linvega** - Creator of UXN and TAL
- **UXN Community** - Documentation and examples
- **100 Rabbits** - UXN ecosystem development
- **Binary**: `#b10101010`
- **Character**: `'A`, `'B`
- **Strings**: `"Hello World"`
