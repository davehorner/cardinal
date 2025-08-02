use std::env;
use std::fs;
use uxn_tal::{error::Result, lexer::Lexer};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input.tal>", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];
    let source = fs::read_to_string(input_file).map_err(|e| {
        uxn_tal::error::AssemblerError::SyntaxError {
            line: 0,
            message: format!("Failed to read file {}: {}", input_file, e),
            path: input_file.clone(),
            position: 0,
            source_line: String::new(),
        }
    })?;

    println!("Source: {:?}", source);

    let mut lexer = Lexer::new(source, Some(input_file.clone()));
    let tokens = lexer.tokenize()?;

    for (i, token) in tokens.iter().enumerate() {
        println!("Token {}: {:?}", i, token);
    }

    Ok(())
}
