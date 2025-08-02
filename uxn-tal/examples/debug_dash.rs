use uxn_tal::{error::Result, lexer::Lexer};

fn main() -> Result<()> {
    let source = "|0000\n@direction $1\n\n|0100\n-direction\nBRK".to_string();
    println!("Source: {:?}", source);

    let mut lexer = Lexer::new(source, None);
    let tokens = lexer.tokenize()?;

    for (i, token) in tokens.iter().enumerate() {
        println!("Token {}: {:?}", i, token);
    }

    Ok(())
}
