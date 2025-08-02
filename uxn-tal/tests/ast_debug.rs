use uxn_tal::lexer::Lexer;
use uxn_tal::parser::Parser;

fn main() {
    let source = r#"
        |0100
        ;data
        @data #42
    "#;

    println!("=== Lexing ===");
    let mut lexer = Lexer::new(source.to_string(), None);
    let tokens = lexer.tokenize().expect("Lexing failed");

    for (i, token) in tokens.iter().enumerate() {
        println!("{}: {:?}", i, token);
    }

    println!("\n=== Parsing ===");
    let path = "<test>".to_string();
    let mut parser =
        Parser::new_with_source(tokens, path.clone(), source.to_string());
    let ast = parser.parse().expect("Parsing failed");

    for (i, node) in ast.iter().enumerate() {
        println!("{}: {:?}", i, node);
    }
}
