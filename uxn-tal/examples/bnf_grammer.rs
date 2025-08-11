/// UXN-TAL BNF Grammar Tool
///
/// This tool provides utilities for working with the BNF and EBNF grammars of the UXN-TAL assembly language.
/// It supports parsing, random sentence generation, and parse tree explanation for UXN-TAL grammar definitions.
///
/// # Features
/// - Parse UXN-TAL grammar definitions from BNF or EBNF formats.
/// - Generate random sentences conforming to the grammar.
/// - Parse sentences and explain their parse trees in human-readable form.
/// - Command-line interface with selectable modes: `parse`, `generate`, and `parse-sentence`.
///
/// # Command-Line Arguments
/// - `--mode` / `-m`: Selects the operation mode (`parse`, `generate`, or `parse-sentence`).
/// - `--grammar` / `-g`: Optional path to a grammar file. If not provided, uses the built-in UXN-TAL grammar.
/// - `--sentence` / `-s`: Sentence to parse (used only in `parse-sentence` mode).
///
/// # Example
/// ```sh
/// cargo run -- --mode generate
/// cargo run -- --mode parse-sentence --sentence "LIT 2"
/// ```
///
/// # Grammar
/// The tool includes built-in EBNF and BNF representations of the UXN-TAL grammar for convenience.
///
/// # Dependencies
/// - `bnf`: For grammar parsing and sentence generation.
/// - `clap`: For command-line argument parsing.
///
/// # Main Functions
/// - `explain_parse_tree`: Recursively explains a parse tree node in human-readable terms.
/// - `main`: Handles argument parsing, grammar loading, and dispatches to the selected mode.
// use bnf::Grammar;
// use clap::Parser;
// const UXNTAL_GRAMMAR_EBNF: &str = r####"
// nibble = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "a" | "b" | "c" | "d" | "e" | "f" ;
// byte   = nibble , nibble ;
// short  = nibble , nibble , nibble , nibble ;
// number = nibble
//        | nibble , nibble
//        | nibble , nibble , nibble
//        | nibble , nibble , nibble , nibble ;

// op     = "LIT" | "INC" | "POP" | "NIP" | "SWP" | "ROT" | "DUP" | "OVR"
//        | "EQU" | "NEQ" | "GTH" | "LTH" | "JMP" | "JCN" | "JSR" | "STH"
//        | "LDZ" | "STZ" | "LDR" | "STR" | "LDA" | "STA" | "DEI" | "DEO"
//        | "ADD" | "SUB" | "MUL" | "DIV" | "AND" | "ORA" | "EOR" | "SFT" ;
// mode   = "2" | "k" | "r" ;
// opcode = "BRK"
//        | op , mode
//        | op , mode , mode
//        | op , mode , mode , mode ;

// rune   = "|" | "$" | "@" | "&" | "%" | "(" | "," | "_" | "." | "-" | ";" | "=" | "?" | "!" | "#" | "}" | "~" | "[" | "]" ;
// name   = string
//        | string , "/" , string
//        | "-" , number
//        | opcode
//        | rune , string ;
// addr   = name
//        | "/" , string
//        | "&" , string
//        | "{" ;

// macros   = "%" , name , "{" , string , "}" ;
// comment  = "(" , string , ")" ;
// ignore   = "[" , string , "]" ;
// toplab   = "@" , name ;
// padrel   = "$" , number | "$" , name ;
// padabd   = "|" , number | "|" , name ;
// lithex   = "#" , byte | "#" , short ;
// rawstr   = "\"" , string ;
// immjsi   = name | "/" , string | "{" ;
// immjmi   = "!" , addr ;
// immjci   = "?" , addr ;
// litzep   = "." , addr ;
// litrel   = "," , addr ;
// litabs   = ";" , addr ;
// rawzep   = "-" , addr ;
// rawrel   = "_" , addr ;
// rawabs   = "=" , addr ;
// "####;

// const UXNTAL_GRAMMAR_BNF: &str = r####"
// <program> ::= <line> | <line> <program>
// <line> ::= <number> | <opcode> | <rune> | <name> | <addr> | <macros> | <comment> | <ignore> | <toplab> | <padrel> | <padabd> | <lithex> | <rawstr> | <immjsi> | <immjmi> | <immjci> | <litzep> | <litrel> | <litabs> | <rawzep> | <rawrel> | <rawabs>
// <nibble> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "a" | "b" | "c" | "d" | "e" | "f"
// <byte> ::= <nibble> <nibble>
// <short> ::= <nibble> <nibble> <nibble> <nibble>
// <number> ::= <nibble>
//            | <nibble> <nibble>
//            | <nibble> <nibble> <nibble>
//            | <nibble> <nibble> <nibble> <nibble>
// <op> ::= "LIT" | "INC" | "POP" | "NIP" | "SWP" | "ROT" | "DUP" | "OVR"
//        | "EQU" | "NEQ" | "GTH" | "LTH" | "JMP" | "JCN" | "JSR" | "STH"
//        | "LDZ" | "STZ" | "LDR" | "STR" | "LDA" | "STA" | "DEI" | "DEO"
//        | "ADD" | "SUB" | "MUL" | "DIV" | "AND" | "ORA" | "EOR" | "SFT"
// <mode> ::= "2" | "k" | "r"
// <opcode> ::= "BRK"
//            | <op> <mode>
//            | <op> <mode> <mode>
//            | <op> <mode> <mode> <mode>
// <rune> ::= "|" | "$" | "@" | "&" | "%" | "(" | "," | "_" | "." | "-" | ";" | "=" | "?" | "!" | "#" | "}" | "~" | "[" | "]"
// <name> ::= <string>
//          | <string> "/" <string>
//          | "-" <number>
//          | <opcode>
//          | <rune> <string>
// <addr> ::= <name>
//          | "/" <string>
//          | "&" <string>
//          | "{"
// <macros> ::= "%" <name> "{" <string> "}"
// <comment> ::= "(" <string> ")"
// <ignore> ::= "[" <string> "]"
// <toplab> ::= "@" <name>
// <padrel> ::= "$" <number> | "$" <name>
// <padabd> ::= "|" <number> | "|" <name>
// <lithex> ::= "#" <byte> | "#" <short>
// <rawstr> ::= '"' <string> '"'
// <immjsi> ::= <name> | "/" <string> | "{"
// <immjmi> ::= "!" <addr>
// <immjci> ::= "?" <addr>
// <litzep> ::= "." <addr>
// <litrel> ::= "," <addr>
// <litabs> ::= ";" <addr>
// <rawzep> ::= "-" <addr>
// <rawrel> ::= "_" <addr>
// <rawabs> ::= "=" <addr>
// "####;

// #[derive(Parser, Debug)]
// #[command(author, version, about, long_about = None)]
// struct Args {
//     /// Mode: parse, generate, or parse-sentence
//     #[arg(short, long, value_parser = ["parse", "generate", "parse-sentence"])]
//     mode: String,

//     /// Input grammar file (optional, defaults to UXNTAL_GRAMMAR)
//     #[arg(short, long)]
//     grammar: Option<String>,

//     /// Sentence to parse (for parse-sentence mode)
//     #[arg(short, long)]
//     sentence: Option<String>,
// }

// fn explain_parse_tree(parse_tree: &bnf::ParseTree) -> String {
//     use bnf::ParseTreeNode;
//     let lhs = parse_tree.lhs;
//     let lhs = match lhs {
//         bnf::Term::Nonterminal(s) => s.as_str(),
//         bnf::Term::Terminal(i) => &format!("terminal_{}", i),
//     };
//     let mut children_desc = Vec::new();
//     for node in parse_tree.rhs_iter() {
//         match node {
//             ParseTreeNode::Terminal(s) => children_desc.push(format!("'{}'", s)),
//             ParseTreeNode::Nonterminal(child) => children_desc.push(explain_parse_tree(child)),
//         }
//     }
//     let desc = match lhs.to_string().as_str() {
//         "macros" => "a macro definition",
//         "immjsi" => "an immediate jump or string",
//         "rawstr" => "a raw string",
//         "comment" => "a comment",
//         "ignore" => "an ignored section",
//         "toplab" => "a top-level label",
//         "padrel" => "a relative pad",
//         "padabd" => "an absolute pad",
//         "lithex" => "a hex literal",
//         "immjmi" => "an immediate jump (minus)",
//         "immjci" => "an immediate jump (conditional)",
//         "litzep" => "a zero-page literal",
//         "litrel" => "a relative literal",
//         "litabs" => "an absolute literal",
//         "rawzep" => "a zero-page raw",
//         "rawrel" => "a relative raw",
//         "rawabs" => "an absolute raw",
//         "opcode" => "an opcode",
//         "number" => "a number",
//         "name" => "a name",
//         "addr" => "an address",
//         "rune" => "a rune",
//         _ => &lhs.to_string(),
//     };
//     if children_desc.is_empty() {
//         format!("{}", desc)
//     } else {
//         format!("{}: {}", desc, children_desc.join(" "))
//     }
// }

// fn main() {
//     let args = Args::parse();

//     let grammar_str = if let Some(ref path) = args.grammar {
//         std::fs::read_to_string(path).expect("Failed to read grammar file")
//     } else {
//         UXNTAL_GRAMMAR_BNF.to_string()
//     };
//     let result = ebnf::get_grammar(&UXNTAL_GRAMMAR_EBNF);
//     println!("ebnf::get_grammar result: {:#?}", result);
//     let grammar: Result<Grammar, _> = grammar_str.parse();

//     match args.mode.as_str() {
//         "parse" => {
//             match grammar {
//                 Ok(g) => println!("{:#?}", g),
//                 Err(e) => println!("Failed to make grammar from String: {}", e),
//             }
//         }
//         "generate" => {
//             match grammar {
//                 Ok(g) => match g.generate() {
//                     Ok(s) => {
//                         println!("random sentence: {}", s);
//                         // Try to parse and explain
//                         let mut parse_trees = g.parse_input(&s);
//                         match parse_trees.next() {
//                             Some(parse_tree) => {
//                                 println!("explanation: {}", explain_parse_tree(&parse_tree));
//                             }
//                             _ => println!("Grammar could not parse sentence"),
//                         }
//                     }
//                     Err(e) => println!("something went wrong: {}!", e),
//                 },
//                 Err(e) => println!("Failed to make grammar from String: {}", e),
//             }
//         }
//         "parse-sentence" => {
//             let sentence = args.sentence.as_deref().unwrap_or("");
//             match grammar {
//                 Ok(g) => {
//                     let mut parse_trees = g.parse_input(sentence);
//                     match parse_trees.next() {
//                         Some(parse_tree) => println!("{}", parse_tree),
//                         _ => println!("Grammar could not parse sentence"),
//                     }
//                 }
//                 Err(e) => println!("Failed to make grammar from String: {}", e),
//             }
//         }
//         _ => {
//             eprintln!("Unknown mode: {}", args.mode);
//         }
//     }
// }

fn main() {
    println!("This is just an unfinished code concept. Uncomment code related.");
}
// use uxn_tal::grammer::GrammarUxnTalCtx;

// fn main() {
//     // Use None to load the built-in grammar, or Some("path/to/grammar.bnf") for a custom grammar.
//     let ctx = GrammarUxnTalCtx::new(None).expect("Failed to load grammar");
//     println!("OPs: {:?}", ctx.op_list());
//     println!("Runes: {:?}", ctx.rune_list());
//     let first_ops = ctx.first_terminals_of("opcode");
// println!("FIRST(opcode): {:?}", first_ops);
//     let is_terminal = ctx.is_terminal_of("opcode", "LIT");
//     println!("Is 'LIT' a terminal of 'opcode'? {}", is_terminal);
// }