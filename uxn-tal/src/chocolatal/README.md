# Chocolatal: TAL Preprocessor for uxn-tal

`chocolatal.rs` is a Rust module providing preprocessing for Uxn TAL assembly files, it attempts to be a faithful implementation of the original [deluge-domain](https://codeberg.org/davehorner/deluge-domain) `preprocess-tal.sh` shell script by [@notchoc](https://codeberg.org/notchoc). It is designed to be used as a standalone module within the `uxn-tal` assembler, located in the `mod/` directory.

## Features

- **File Inclusion:** Handles `~file.tal` and recursively includes and preprocesses other TAL files.
- **Token Normalization:** Normalizes whitespace, indentation, and token prefixes for consistent parsing.
- **Macro-like Label and Prefix Expansion:** Expands and rewrites label and prefix tokens (e.g., `&foo`, `/foo`, etc.) to their canonical forms.
- **Lambda/Loop Label Generation:** Automatically generates unique labels for lambda/loop constructs (e.g., `'>{'`, `|}`/`?}`/`}`) and manages their scope.
- **Path and Prefix Rewriting:** Rewrites paths and prefixes for included files and labels, ensuring correct label resolution.
- **Special Token Rewrites:** Handles special-case token rewrites (e.g., `|/` becomes the current directory as a label).
- **Comment and Parenthesis Handling:** Correctly processes comments and nested parentheses.

## Usage

1. **Integration:**
   - In your main assembler pipeline, call the preprocessor before lexing:
     ```rust
     let preprocessed = chocolatal::preprocess(&input, &path)?;
     let mut lexer = Lexer::new(preprocessed, Some(path.to_string()));
     ```

2. **API:**
   - The module exposes:
     ```rust
     pub fn preprocess(input: &str, path: &str) -> Result<String>
     ```
   - This function returns a preprocessed TAL source string suitable for lexing and parsing.

3. **Testing:**
   - Unit tests for `chocolatal.rs` should cover file inclusion, label rewriting, lambda/loop handling, and token normalization.

4. **uxntal integration:**
   - `uxntal` implements chocolatal preprocessing by default, use it for your assembling needs and start using chocolatal syntax in your tal.

## Design Notes

- The preprocessor is a direct Rust port of the original (`preprocess-tal.sh`)[https://codeberg.org/notchoc/deluge-domain/src/branch/main/preprocess-tal.sh] script, preserving its logic and behavior.
- All preprocessing is performed before lexing and parsing, keeping the assembler pipeline modular and maintainable.
- The module is self-contained and does not depend on the rest of the assembler internals.
- uxntal includes preprocessing comparsion for this rust implemention against the deluge preprocessor; something like the following might work on your machine.
    `C:\w\cardinal\uxn-tal\deluge-domain>cargo e -- --cmp-pp deluge\main.tal` using [cargo-e](https://crates.io/crates/cargo-e)
- `docker` needs to be installed to perform comparison tests.  deluge is developed using alpine and busybox ash syntax.
- ```
    git clone https://codeberg.org/davehorner/deluge-domain.git
    cd deluge-domain
    docker run --rm -v $(pwd):/workspace -w /workspace alpine sh preprocess-tal.sh deluge/main.tal
    - or cmd.exe -
    docker run --rm -v %cd%:/workspace -w /workspace alpine sh preprocess-tal.sh deluge/main.tal
    - or powershell -
    docker run --rm -v ${PWD}:/workspace -w /workspace alpine sh preprocess-tal.sh deluge/main.tal
    uxntal --cmp-pp deluge\main.tal
  ```
- **Standalone Usage with `rustc` and `rustscript`**  
    You can compile and run `mod.rs` directly using rustc or rustscript. Details at the bottom.

## License

This module is part of the `uxn-tal` project and is distributed under the same license.
8/25 David Horner

Special thanks to [@notchoc](https://codeberg.org/notchoc) and the uxn discord community.
