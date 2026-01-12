use std::path::Path;
use std::fs;
use uxn_tal::assemble_file_with_symbols;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Try to find a checked-out `uxneyes.rom/eyes.smal` by walking parents from CWD.
    let mut cwd = std::env::current_dir()?.canonicalize()?;
    let mut uxne_found: Option<std::path::PathBuf> = None;
    loop {
        let candidate = cwd.join("uxneyes.rom").join("eyes.smal");
        if candidate.exists() {
            uxne_found = Some(candidate);
            break;
        }
        if !cwd.pop() { break; }
    }

    // Prefer the checked-out uxneyes example if present, otherwise write a small sample.
    let sample_path = if let Some(uxne_eyes) = uxne_found {
        uxne_eyes
    } else {
        let workspace_root = Path::new("..").canonicalize()?;
        let sample_path = workspace_root.join("sample.smal");
        let sample = r#"// Load
fun t1( *byte    -- byte    ) { load }
fun t2( *short   -- short   ) { load }
fun t3( *fun(->) -- fun(->) ) { load }
fun t4( ^byte    -- byte    ) { load }
fun t5( ^short   -- short   ) { load }
fun t6( ^fun(->) -- fun(->) ) { load }

fun t7 ( *^byte  -- ^byte  ) { load }
fun t8 ( *^byte  -- byte   ) { load load }
fun t9 ( ^*short -- *short ) { load }
fun t10( ^*short -- short  ) { load load }

// Store
fun t11( byte         *byte         -- ) { store }
fun t12( short        *short        -- ) { store }
fun t13( fun(->)      *fun(->)      -- ) { store }
fun t14( fun(-- byte) *fun(-- byte) -- ) { store }
fun t15( byte         ^byte         -- ) { store }
fun t16( short        ^short        -- ) { store }
fun t17( fun(->)      ^fun(->)      -- ) { store }
fun t18( fun(-- byte) ^fun(-- byte) -- ) { store }

fun t19( *byte  **byte  -- ) { store }
fun t20( *short **short -- ) { store }
fun t21( *byte  ^*byte  -- ) { store }
fun t22( *short ^*short -- ) { store }

// Input
fun t23( byte -- byte  ) { input }
fun t24( byte -- short ) { input2 }

// Output
fun t25( byte         byte -- ) { output }
fun t26( short        byte -- ) { output }
fun t27( *byte        byte -- ) { output }
fun t28( ^short       byte -- ) { output }
fun t29( fun(->)      byte -- ) { output }
fun t30( fun(-- byte) byte -- ) { output }
fun t31( *^fun(--)    byte -- ) { output }

alias enum byte Dev { a b c }

fun enum-dev-input1 ( Dev -- byte  ) { input }
fun enum-dev-input2 ( Dev -- short ) { input2 }
fun enum-dev-input3 ( -- byte  ) { Dev.a input }
fun enum-dev-input4 ( -- short ) { Dev.a input2 }

fun enum-dev-output1 ( byte    Dev -- ) { output }
fun enum-dev-output2 ( short   Dev -- ) { output }
fun enum-dev-output3 ( fun(->) Dev -- ) { output }
fun enum-dev-output4 ( *byte   Dev -- ) { output }
fun enum-dev-output5 ( ^byte   Dev -- ) { output }
fun enum-dev-output6 ( byte        -- ) { Dev.a output }
fun enum-dev-output7 ( *byte       -- ) { Dev.a output }
"#;
        fs::write(&sample_path, sample)?;
        println!("Wrote sample.smal to {}", sample_path.display());
        sample_path
    };

    // If we found a uxneyes example, switch CWD to its directory so `include` paths resolve.
    let old_cwd = std::env::current_dir()?;
    if let Some(parent) = sample_path.parent() {
        std::env::set_current_dir(parent)?;
    }

    // `assemble_file_with_symbols` now returns (rom_path, sym_path, size)
    let (rom_path, sym_path, size) = assemble_file_with_symbols(&sample_path)?;

    // Restore original working directory.
    std::env::set_current_dir(old_cwd)?;
    println!("Wrote ROM to {} ({} bytes)", rom_path.display(), size);
    println!("Wrote SYM to {}", sym_path.display());

    Ok(())
}
