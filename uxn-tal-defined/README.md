# uxntal Protocol Specification Defined

This module documents the `uxntal://` protocol for launching TAL/ROM files via URL.  

### Quick Start
[uxn-tal](https://crates.io/crates/uxn-tal) uses this module to map urls to emulators.

```
cargo install uxn-tal
uxntal --register
explorer uxntal://https://wiki.xxiivv.com/etc/catclock.tal.txt
```
The above will run a catclock from cmd.exe/pwsh.

- `--register` will setup a protocol handler for `uxntal://` on your system.  It will also install the e_window and cardinal-gui crates as a dependency.  This feature allows you to place `uxntal://` in front of any http(s) url and uxntal will download, assemble, cache, and run the tal/rom file pointed to by url.  `uxntal --register` been tested on Windows, MacOS, and Linux.  On macOS, `--register` creates a minimal GUI `.app` bundle in your `~/Applications` folder that registers the `uxntal://` protocol. This bundle launches your installed `uxntal` binary with the URL as an argument.

- `uxntal --unregister` will remove and disable `uxntal://` urls.


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
uxntal:widget://https://git.sr.ht/~rabbits/orca-examples/tree/master/item/basics/u.orca
```
This will launch the canonical orca ROM with the specified `.orca` file in widget mode.  You can always change the emu to run it in your preferred emulator.

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

For example `explorer uxntal://https://git.sr.ht/~rabbits/left/tree/main/item/src/left.tal`, which is a project with a few includes, runs fine.

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

### Examples

- `uxntal:emu^uxn://https://wiki.xxiivv.com/etc/catclock.tal.txt` launches catclock in the `uxnemu` emulator.
- `uxntal:emu^buxn://https://wiki.xxiivv.com/etc/catclock.tal.txt` launches catclock in the `buxn-gui` emulator.
- `uxntal:widget://https://wiki.xxiivv.com/etc/catclock.tal.txt` passes the `--widget` flag to the emulator. (cardinal-gui is the only emu that supports this flag currently)
- `uxntal:widget:ontop^false://https://wiki.xxiivv.com/etc/catclock.tal.txt` passes the `--widget` flag but disables always-on-top, showing widget mode without ontop.
- `uxntal:widget:debug://https://wiki.xxiivv.com/etc/cccc.tal.txt` opens cccc in widget mode with debug console enabled. (windows only)

You might create multiple bookmarklets to launch urls in the emulator and with the settings you desire.  Right now, the variables are restricted to `widget`,`debug`,`ontop` and `emu` to limit arbitrary input on emu invocation.

## Protocol Format

URLs are parsed as:

```text
uxntal:var1:var2^val2:var3^^val3://actual_url
```

- Variables are separated by `:`
- Key-value pairs use `^` or `^^` as separators (double `^^` for Windows shell escaping)
- The actual TAL/ROM file URL is after the `://`

## Supported Variables

- `emu`    : Select emulator backend (`buxn`, `cuxn`, `uxn`). Example: `emu^^buxn`
- `widget` : Enable widget mode (transparent, no decorations, always-on-top). Example: `widget`
- `ontop`  : Control always-on-top (`ontop^false` disables it in widget mode)
- `debug`  : Enable debug console (Windows only). Example: `debug`

## Examples

- `uxntal:emu^uxn://https://wiki.xxiivv.com/etc/catclock.tal.txt` launches catclock in `uxnemu` emulator
- `uxntal:widget://https://wiki.xxiivv.com/etc/catclock.tal.txt` passes `--widget` flag to emulator
- `uxntal:widget:ontop^false://...` disables always-on-top in widget mode
- `uxntal:widget:debug://...` enables debug console (Windows only)

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

## Example URL

```text
uxntal:emu^^buxn:widget://https://example.com/rom.tal?!fit=cover&!timeout=60
```
