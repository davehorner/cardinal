use uxn_tal::lexer::{Lexer, Token};

fn main() {
    // Test different cases
    let test_cases = vec![
        "'A", "'A 'B", "'A'", // This might be wrong format
    ];

    for (i, source) in test_cases.iter().enumerate() {
        println!("\nTest case {}: \"{}\"", i + 1, source);
        let mut lexer = Lexer::new(source.to_string(), None);

        loop {
            match lexer.next_token() {
                Ok(Token::Eof) => {
                    println!("  EOF");
                    break;
                }
                Ok(token) => println!("  {:?}", token),
                Err(e) => {
                    println!("  Error: {}", e);
                    break;
                }
            }
        }
    }
}
