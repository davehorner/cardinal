# Chocolatal: TAL Preprocessor for uxn-tal

`chocolatal` is a Rust module providing preprocessing for Uxn TAL assembly files, inspired by the original `preprocess-tal.sh` shell script. It is designed to be used as a standalone module within the `uxn-tal` assembler, located in the `src/chocolatal/` directory.

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
   - Place the module in the `src/chocolatal/` directory of your `uxn-tal` project.
   - In your main assembler pipeline, call the preprocessor before lexing:
     ```rust
     mod chocolatal;
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
   - Unit tests for `chocolatal` should cover file inclusion, label rewriting, lambda/loop handling, and token normalization.

## Design Notes

- The preprocessor is a direct Rust port of the original `preprocess-tal.sh` script, preserving its logic and behavior.
- All preprocessing is performed before lexing and parsing, keeping the assembler pipeline modular and maintainable.
- The module is self-contained and does not depend on the rest of the assembler internals.

## License

This module is part of the `uxn-tal` project and is distributed under the same license.
