# uxntal Protocol Specification Defined

This module documents the `uxntal://` protocol for launching TAL/ROM files via URL.  

### Quick Start
[uxn-tal](https://crates.io/crates/uxn-tal) uses this module to map urls to emulators.

```
cargo install uxn-tal
uxntal --register
uxntal uxntal://https://wiki.xxiivv.com/etc/catclock.tal.txt
```
The above will run a catclock from Windows, MacOS, and Linux.

- `--register` will setup a protocol handler for `uxntal://` on your system.  It will also ask you to install the e_window and cardinal-gui crates as a dependency.  This feature allows you to place `uxntal://` in front of any http(s) url and uxntal will download, assemble, cache, and run the tal/rom file pointed to by url.  `uxntal --register` been tested on Windows, MacOS, and Linux.  On macOS, `--register` creates a minimal GUI `.app` bundle in your `~/Applications` folder that registers the `uxntal://` protocol. This bundle launches your installed `uxntal` binary with the URL as an argument.

- `uxntal --unregister` will remove and disable `uxntal://` urls.

### Bookmarklets

You can prepend the uxntal:// to any valid tal url, or you can create a bookmarklet on your bookmark toolbar to launch the protocol on click of the bookmarklet.
```
javascript:(function(){window.open('uxntal://' + location.href);})();
```

This works for most webpages, however content from raw.githubusercontent.com/... is not a normal webpage and it’s delivered inside a sandboxed iframe by GitHub.  This means executing JavaScript from bookmarklets will not work on those pages.

To provide another means of opening sandboxed webpages, you can install the [open-in-uxntal](https://github.com/davehorner/cardinal/tree/main/uxn-tal/open-in-uxntal) chrome extension via [chrome://extensions](chrome://extensions) `"Load unpacked"` button and pointing to the open-in-uxntal folder.  The extension exposes a new right click context menu on the webpage for `Open in uxntal`.

You can just prefix any url with uxntal:// and it should work.  Extensions and other things constructing a custom url should use open and url encode so that urls are not invalid and munged when parsed.  The simple prefix is for the user without the extension or bookmarklet installed.  Other functionality such as selection of asm/emu/pre may be added in the future and controlled via variables in the preferred open encoded form.
```
uxntal://open?url=ENC or uxntal://open/?url=ENC
multiple query params (...&url=...)
percent-encoded full URLs after the scheme
over-slashed forms (https///, https//, etc.)
```

Using the extension and the bookmarklet, you will find a chrome dialog pop that asks if you want to run uxntal.exe to open an application.  Using the bookmarklet you sometimes have the option to allow always; the extension does not provide this option so you always have a second click to acknowledge the website opening the application.

The protocol handler now has a provider abstraction over Github/Codeberg/Sourcehut urls; this means that the bookmarklet will work on view/log/blame pages on these websites.  Additionally, the downloader now parses and downloads all the includes. 

For example `uxntal uxntal://https://git.sr.ht/~rabbits/left/tree/main/item/src/left.tal`, which is a project with a few includes, runs fine.

If you are often viewing code from a site like github, using the bookmarklet on a view/blame/history page instead of the raw allows you to use the protocol without the permission dialog being raised.  It's possible uxntal.exe could register a NativeMessagingHosts endpoint so that the chrome extension isn't using the protocol handler but instead invoke chrome.runtime.sendNativeMessage to side step the additional chrome dialog.

### `uxntal:variables:key^^value://` Protocol Handler Variable Support 

The protocol handler supports passing variables and flags directly in the protocol portion of URL using key-value pairs. These are used to select emulator or pass options.

Variables are specified separated by colons (`:`). Key-value pairs use either `^` or `^^` as separators.

`cardinal-gui` supports the `--widget` flag, which turns on transparency for white pixels, disables window decorations, enables ctrl+alt+click-and-drag to move the window, and sets the window always-on-top by default. If you want to disable always-on-top while using widget mode, pass `ontop^false` in the URL or use `--ontop=false` on the CLI.

[uxntal:widget://https://wiki.xxiivv.com/etc/catclock.tal.txt](uxntal:widget://https://wiki.xxiivv.com/etc/catclock.tal.txt) will open in `cardinal-gui` in --widget mode.  catclock will show in a transparent window with no application decorations and will be always-on-top unless you specify `ontop^false`.

In order to support different emulators, you can pass an `emu` variable, which currently supports buxn,cuxn,uxnemu emulators if they are within the PATH.

[uxntal:emu^^buxn://https://wiki.xxiivv.com/etc/catclock.tal.txt](uxntal:emu^^buxn://https://wiki.xxiivv.com/etc/catclock.tal.txt) will open in `buxn-gui`

You can use either `^` or `^^` as a separator. Both are supported and percent-encoded forms (e.g., `%5E`, `%5E%5E`) are also decoded automatically.
The reason for both `^` and `^^` is that on windows, you must escape `^` with another `^`, so if you want a string that can be pasted in cmd.exe; use the double `^^` form.

## Protocol Format

URLs are parsed as:

```text
uxntal:var1:var2^val2:var3^^val3://actual_url
```

- Variables are separated by `:`
- Key-value pairs use `^` or `^^` as separators (double `^^` for Windows shell escaping)
- The actual URL is after the `://`


### Supported File Extensions

The protocol handler supports the following file extensions for remote and local files:

- `.tal`/`.tal.txt` — TAL source files (assembled to .rom on the fly)
- `.rom` — Binary UXN ROM files (used as-is)
- `.rom.txt` — Hex-encoded ROM files (automatically converted to binary .rom before use)
- `.orca` — Orca pattern files (run with canonical orca ROM, not assembled as TAL)

When you provide a URL or file ending in `.rom.txt`, the handler will automatically fetch and convert the hex text to a binary `.rom` file using the same logic as `xxd -r -p`. This allows you to share ROMs as plain text for easy inspection, versioning, and diffing, while still running them directly via the protocol.

## Orca Mode (`.orca` files)

The protocol handler supports `.orca` files, which are not assembled as TAL but instead are run directly with the canonical orca ROM. When you provide a URL or file ending in `.orca`, the handler will:

- Download and cache the `.orca` file if remote.
- Launch the emulator with the canonical orca ROM and the `.orca` file as arguments.
- Never assemble `.orca` files as TAL or treat them as source code.

This ensures `.orca` files are always run in the correct environment, matching the canonical orca UXN experience.

**Example:**
```
uxntal://https://git.sr.ht/~rabbits/orca-examples/tree/master/item/basics/u.orca
```
This will launch the canonical orca ROM with the specified `.orca` file in widget mode.  You can always change the emu to run it in your preferred emulator.

## Protocol Section Variables

| Name | Type | Description | Example |
|------|------|-------------|---------|
| `transparent` | String | Transparent color for widget/background (hex RGB, e.g. ffffff for white) | `transparent^ff00ff` |
| `scale` | Float | Scale factor for the window (float) | `scale^2.0` |
| `fit` | String | Fit mode for ROM display (none, contain, cover, stretch) | `fit^cover` |
| `timeout` | Float | Timeout in seconds before the emulator exits (alias: t) | `timeout^60` |
| `t` | Float | Timeout in seconds before the emulator exits (alias for timeout) | `t^60` |
| `emu` | Enum (buxn, cuxn, uxn) | Select emulator backend | `emu^^buxn` |
| `widget` | Bool | Enable widget mode (transparent, no decorations, always-on-top) | `widget` |
| `ontop` | Bool | Control always-on-top (true/false) | `ontop^false` |
| `debug` | Bool | Enable debug console (Windows only) | `debug` |
| `efx` | String | Effect name or identifier for emulator (string) | `efx^invert` |
| `efxmode` | String | Effect mode for emulator (string) | `efxmode^blend` |
| `orca` | Bool | Orca mode: run the orca ROM with the given .orca file. Automatically set if the URL ends with .orca. | `orca` |
| `x` | String | Window X position (pixels or percent or complex) | `x^100` |
| `y` | String | Window Y position (pixels or percent or complex) | `y^100` |
| `w` | String | Window width (pixels or percent or complex) | `w^800` |
| `h` | String | Window height (pixels or percent or complex) | `h^600` |

## Bang Query Variables

| Name | Type | Description | Example |
|------|------|-------------|---------|
| `fit` | Enum (none, contain, cover, stretch) | Fit mode for ROM display (none, contain, cover, stretch) | `!fit=cover` |
| `timeout` | Float | Timeout in seconds before the emulator exits (alias: t) | `!timeout=60` |
| `t` | Float | Timeout in seconds before the emulator exits (alias for timeout) | `!t=60` |
| `x` | String | Window X position (pixels or percent or complex) | `!x=100` |
| `y` | String | Window Y position (pixels or percent or complex) | `!y=100` |
| `w` | String | Window width (pixels or percent or complex) | `!w=800` |
| `h` | String | Window height (pixels or percent or complex) | `!h=600` |

## Emulator Compatibility Matrix

This table shows which protocol/bang variables affect the command-line arguments for each supported emulator. An `X` means the variable is mapped to CLI args for that emulator.

| Variable | buxn | uxn | cuxn | Example | Type 
|---|---|---|---|---|---|
| `transparent` |   |    |  X |  transparent^ff00ff | proto |
| `scale` |   |    |  X |  scale^2.0 | proto |
| `fit`/`!fit` |   |    |  X |  fit^cover | both |
| `timeout`/`!timeout` |   |    |  X |  timeout^60 | both |
| `t`/`!t` |   |    |  X |  t^60 | both |
| `emu` | X |  X |  X |  emu^^buxn | proto |
| `widget` |   |    |  X |  widget | proto |
| `ontop` |   |    |  X |  ontop^false | proto |
| `debug` |   |    |  X |  debug | proto |
| `efx` |   |    |  X |  efx^invert | proto |
| `efxmode` |   |    |  X |  efxmode^blend | proto |
| `orca` | X |  X |  X |  orca | proto |
| `x`/`!x` |   |    |  X |  x^100 | both |
| `y`/`!y` |   |    |  X |  y^100 | both |
| `w`/`!w` |   |    |  X |  w^800 | both |
| `h`/`!h` |   |    |  X |  h^600 | both |


**Note:** If both a protocol variable (e.g. `x`) and a bang variable (e.g. `!x`) are provided, the protocol variable typically takes precedence and overrides the bang variable.

### Examples

- `uxntal:emu^uxn://https://wiki.xxiivv.com/etc/catclock.tal.txt` launches catclock in the `uxnemu` emulator.
- `uxntal:emu^buxn://https://wiki.xxiivv.com/etc/catclock.tal.txt` launches catclock in the `buxn-gui` emulator.
- `uxntal:widget://https://wiki.xxiivv.com/etc/catclock.tal.txt` passes the `--widget` flag to the emulator. (cardinal-gui is the only emu that supports this flag currently)
- `uxntal:widget:ontop^false://https://wiki.xxiivv.com/etc/catclock.tal.txt` passes the `--widget` flag but disables always-on-top, showing widget mode without ontop.
- `uxntal:widget:debug://https://wiki.xxiivv.com/etc/cccc.tal.txt` opens cccc in widget mode with debug console enabled. (windows only)

You might create multiple bookmarklets to launch urls in the emulator and with the settings you desire.  Right now, the variables are restricted to `widget`,`debug`,`ontop` and `emu` to limit arbitrary input on emu invocation.

## Git Repository Support

The protocol handler supports direct git repository cloning and file access using `git@` URLs. This allows you to run TAL files with supporting files directly from git repositories.

### Supported Git URL Formats

- **SSH Format**: `uxntal://git@host:owner/repo/tree/branch/path/file.tal`
- **HTTPS Format**: `uxntal://git@https://host/owner/repo/blob/branch/path/file.tal`
- **Git .git Format**: `uxntal://git@host:owner/repo.git/path/file.tal`

### Supported Git Providers

- **GitHub**: `git@github.com:` or `git@https://github.com/`
- **SourceHut**: `git@git.sr.ht:` or `git@https://git.sr.ht/`
- **Codeberg**: `git@https://codeberg.org/` (HTTPS only, SSH not supported)

### How Git Integration Works

When you use a `git@` URL, the protocol handler will:

1. **Parse the URL** to extract repository information (host, owner, repo, branch, file path)
2. **Clone or update** the repository to a local cache directory
3. **Checkout the specified branch** (defaults to `main` if not specified)


### Git URL Examples

```bash
# GitHub SSH format
uxntal://git@github.com:davehorner/uxn-games/tree/main/flap/flap.tal

# GitHub HTTPS format  
uxntal://git@https://github.com/davehorner/uxn-cats/blob/main/catclock.tal

# GitHub .git format
uxntal://git@github.com:davehorner/uxn-cats.git/catclock.tal

# SourceHut SSH format
uxntal://git@git.sr.ht:~rabbits/noodle/main/src/noodle.tal

# SourceHut HTTPS format
uxntal://git@https://git.sr.ht/~rabbits/noodle/main/src/noodle.tal

# Codeberg HTTPS format
uxntal://git@https://codeberg.org/yorshex/minesweeper-uxn/tree/main/minesweeper.tal
```

### Benefits of Git Integration

- **Supporting files**: `git@` urls will include supporting files that would not ordinarily come down based on simple tal include resolution.

### Cache Location

Git repositories and downloads are cached in your local uxntal cache directory:
- **Windows**: `%USERPROFILE%\.uxntal\roms\`
- **macOS/Linux**: `~/.uxntal/roms/`

Each repository gets its own hash-based subdirectory for isolation.

## Warning

A single click protocol handler that assembles and runs arbitrary code is a **dangerous activity**. uxntal protocol handler 0.1.18 and earlier had a shell exploit that could allow someone to craft a url/website which could run arbitrary code on your machine.  This specific shell exploit concern has been addressed in 0.2.0 but similiar issues may develop or be uncovered.

uxntal protocol is for recreational development and might represent a risk to your data if abused by others.  Consider `--register` at the start of your activities and then `--unregister` when you are done;  or you can assume others aren't crafting uxntal urls to do bad things and enjoy the protocol.  I hope it is used for good fun.

This disclaimer is here to educate users on the security concerns involved, to request additional eyes for security, and to remind the user to apply upgrades as they become available so that any new security concerns found can be patched.  
