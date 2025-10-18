// hexrev.rs
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Reversible hex <-> bytes helper.
/// - `hex_to_bin` ignores non-hex (whitespace/newlines/etc.) like `xxd -r -p`.
/// - `hex_to_bin_strict` errors on ANY non-hex character.
/// - `bin_to_hex` emits lowercase by default; can uppercase and wrap lines.
pub struct HexRev;

impl HexRev {
    #[inline]
    fn hex_val(b: u8) -> Option<u8> {
        match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            b'A'..=b'F' => Some(b - b'A' + 10),
            _ => None,
        }
    }

    /// Streaming plain-hex -> bytes, *ignoring* non-hex (matches `xxd -r -p` feel).
    /// Errors on odd number of hex digits.
    pub fn hex_to_bin<R: Read, W: Write>(mut r: R, mut w: W) -> io::Result<()> {
        let mut buf = [0u8; 8192];
        let mut have_high = false;
        let mut high = 0u8;

        loop {
            let n = r.read(&mut buf)?;
            if n == 0 {
                break;
            }
            for &b in &buf[..n] {
                if let Some(v) = Self::hex_val(b) {
                    if !have_high {
                        high = v << 4;
                        have_high = true;
                    } else {
                        w.write_all(&[high | v])?;
                        have_high = false;
                    }
                } // else: ignore
            }
        }

        if have_high {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "odd number of hex digits in input",
            ));
        }
        Ok(())
    }

    /// Streaming plain-hex -> bytes in **strict** mode:
    /// - Only [0-9a-fA-F] are allowed; ANY other byte (including whitespace) is an error.
    /// - Errors on odd number of hex digits.
    pub fn hex_to_bin_strict<R: Read, W: Write>(mut r: R, mut w: W) -> io::Result<()> {
        let mut buf = [0u8; 8192];
        let mut have_high = false;
        let mut high = 0u8;

        loop {
            let n = r.read(&mut buf)?;
            if n == 0 {
                break;
            }
            for &b in &buf[..n] {
                let v = match Self::hex_val(b) {
                    Some(v) => v,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("non-hex character: 0x{b:02X}"),
                        ))
                    }
                };
                if !have_high {
                    high = v << 4;
                    have_high = true;
                } else {
                    w.write_all(&[high | v])?;
                    have_high = false;
                }
            }
        }

        if have_high {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "odd number of hex digits in input",
            ));
        }
        Ok(())
    }

    /// Convenience: hex text file -> binary file (permissive).
    pub fn hex_to_bin_paths(input: &Path, output: &Path) -> io::Result<()> {
        let rdr = BufReader::new(File::open(input)?);
        if let Some(parent) = output.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let wtr = BufWriter::new(File::create(output)?);
        Self::hex_to_bin(rdr, wtr)
    }

    /// Convenience: hex text file -> binary file (strict).
    pub fn hex_to_bin_paths_strict(input: &Path, output: &Path) -> io::Result<()> {
        let rdr = BufReader::new(File::open(input)?);
        if let Some(parent) = output.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let wtr = BufWriter::new(File::create(output)?);
        Self::hex_to_bin_strict(rdr, wtr)
    }

    /// Streaming bytes -> plain hex.
    ///
    /// - `uppercase`: A–F if true, a–f if false.
    /// - `line_width_bytes`: if `Some(n)`, insert '\n' after every `n` bytes (like `xxd -p`, which
    ///   uses 60 by default). If `None`, writes one long line with no trailing newline.
    pub fn bin_to_hex<R: Read, W: Write>(
        mut r: R,
        mut w: W,
        uppercase: bool,
        line_width_bytes: Option<usize>,
    ) -> io::Result<()> {
        let mut buf = [0u8; 8192];
        let mut line_count = 0usize;

        let lut_lower = *b"0123456789abcdef";
        let lut_upper = *b"0123456789ABCDEF";
        let lut = if uppercase { lut_upper } else { lut_lower };

        loop {
            let n = r.read(&mut buf)?;
            if n == 0 {
                break;
            }
            for &byte in &buf[..n] {
                let hi = (byte >> 4) as usize;
                let lo = (byte & 0x0F) as usize;
                w.write_all(&[lut[hi], lut[lo]])?;
                line_count += 1;

                if let Some(wrap) = line_width_bytes {
                    if wrap > 0 && line_count == wrap {
                        w.write_all(b"\n")?;
                        line_count = 0;
                    }
                }
            }
        }
        Ok(())
    }

    /// Convenience: binary file -> hex text file.
    pub fn bin_to_hex_paths(
        input: &Path,
        output: &Path,
        uppercase: bool,
        line_width_bytes: Option<usize>,
    ) -> io::Result<()> {
        let rdr = BufReader::new(File::open(input)?);
        if let Some(parent) = output.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let mut wtr = BufWriter::new(File::create(output)?);
        Self::bin_to_hex(rdr, &mut wtr, uppercase, line_width_bytes)?;
        // If line wrapping was used and the last line wasn't empty, add final newline to match common tools.
        if line_width_bytes.is_some() {
            wtr.write_all(b"\n")?;
        }
        Ok(())
    }
}

//
// -------- Optional CLI entrypoint (same file) --------
// Enable by compiling with the feature flag `bin`:
//
//   rustc --cfg feature="bin" hexrev.rs -O -o hexrev
// or inside Cargo: cargo run --features bin -- <subcommand> ...
//

// #[cfg(feature = "bin")]
#[cfg_attr(not(test), allow(dead_code))]
pub fn main() -> io::Result<()> {
    use std::env;

    fn usage() -> ! {
        eprintln!("Usage:");
        eprintln!("  hexrev to-bin   [--strict] <input_hex.txt|- > <output_bin|- >");
        eprintln!("  hexrev to-hex   [--upper]  [--width BYTES] <input_bin|- > <output_hex|- >");
        eprintln!();
        eprintln!("Defaults:");
        eprintln!("  to-hex width: 60 bytes per line (like `xxd -p`).");
        std::process::exit(2);
    }

    let mut args = env::args().skip(1);
    let Some(cmd) = args.next() else { usage() };

    match cmd.as_str() {
        "to-bin" => {
            let mut strict = false;
            // Parse flags (only --strict)
            let mut next = args.next();
            while let Some(a) = next.clone() {
                if a == "--strict" {
                    strict = true;
                    next = args.next();
                } else {
                    break;
                }
            }
            let input = next.unwrap_or_else(|| usage());
            let output = args.next().unwrap_or_else(|| usage());
            if args.next().is_some() {
                usage()
            }

            // stdin/stdout support
            let reader: Box<dyn Read> = if input == "-" {
                Box::new(io::stdin())
            } else {
                Box::new(BufReader::new(File::open(&input)?))
            };
            let writer: Box<dyn Write> = if output == "-" {
                Box::new(io::stdout())
            } else {
                let p = Path::new(&output);
                if let Some(parent) = p.parent() {
                    if !parent.as_os_str().is_empty() {
                        fs::create_dir_all(parent)?;
                    }
                }
                Box::new(BufWriter::new(File::create(p)?))
            };

            if strict {
                HexRev::hex_to_bin_strict(reader, writer)
            } else {
                HexRev::hex_to_bin(reader, writer)
            }
        }

        "to-hex" => {
            let mut uppercase = false;
            let mut width: Option<usize> = Some(60); // match xxd -p default-ish
                                                     // Parse flags: --upper, --width N
            while let Some(a) = args.next() {
                if a == "--upper" {
                    uppercase = true;
                } else if a == "--width" {
                    let n = args.next().unwrap_or_else(|| {
                        eprintln!("--width requires a number");
                        std::process::exit(2);
                    });
                    let parsed = n.parse::<usize>().unwrap_or_else(|_| {
                        eprintln!("invalid width: {n}");
                        std::process::exit(2);
                    });
                    width = Some(parsed);
                } else {
                    // We've reached positional args
                    let input = a;
                    let output = args.next().unwrap_or_else(|| usage());
                    if args.next().is_some() {
                        usage()
                    }

                    let reader: Box<dyn Read> = if input == "-" {
                        Box::new(io::stdin())
                    } else {
                        Box::new(BufReader::new(File::open(&input)?))
                    };
                    let writer: Box<dyn Write> = if output == "-" {
                        Box::new(io::stdout())
                    } else {
                        let p = Path::new(&output);
                        if let Some(parent) = p.parent() {
                            if !parent.as_os_str().is_empty() {
                                fs::create_dir_all(parent)?;
                            }
                        }
                        Box::new(BufWriter::new(File::create(p)?))
                    };

                    return HexRev::bin_to_hex(reader, writer, uppercase, width);
                }
            }
            usage()
        }
        _ => usage(),
    }
}
