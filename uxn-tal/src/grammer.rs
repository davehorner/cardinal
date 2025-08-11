//! UXN-TAL BNF Grammar Tool Context Utilities
//!
//! This module provides a context struct for working with the parsed UXN-TAL grammar,
//! including utilities to extract the list of <op> and <rune> terminals.
use crate::lexer::Token;
use crate::opcodes::Opcodes;
use bnf::Grammar;
use clap::Parser;
const UXNTAL_GRAMMAR_EBNF: &str = r####"
nibble = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "a" | "b" | "c" | "d" | "e" | "f" ;
byte   = nibble , nibble ;
short  = nibble , nibble , nibble , nibble ;
number = nibble
       | nibble , nibble
       | nibble , nibble , nibble
       | nibble , nibble , nibble , nibble ;

op     = "LIT" | "INC" | "POP" | "NIP" | "SWP" | "ROT" | "DUP" | "OVR"
       | "EQU" | "NEQ" | "GTH" | "LTH" | "JMP" | "JCN" | "JSR" | "STH"
       | "LDZ" | "STZ" | "LDR" | "STR" | "LDA" | "STA" | "DEI" | "DEO"
       | "ADD" | "SUB" | "MUL" | "DIV" | "AND" | "ORA" | "EOR" | "SFT" ;
mode   = "2" | "k" | "r" ;
opcode = "BRK"
       | op , mode
       | op , mode , mode
       | op , mode , mode , mode ;

rune   = "|" | "$" | "@" | "&" | "%" | "(" | "," | "_" | "." | "-" | ";" | "=" | "?" | "!" | "#" | "}" | "~" | "[" | "]" ;
name   = string
       | string , "/" , string
       | "-" , number
       | opcode
       | rune , string ;
addr   = name
       | "/" , string
       | "&" , string
       | "{" ;

macros   = "%" , name , "{" , string , "}" ;
comment  = "(" , string , ")" ;
ignore   = "[" , string , "]" ;
toplab   = "@" , name ;
padrel   = "$" , number | "$" , name ;
padabd   = "|" , number | "|" , name ;
lithex   = "#" , byte | "#" , short ;
rawstr   = "\"" , string ;
immjsi   = name | "/" , string | "{" ;
immjmi   = "!" , addr ;
immjci   = "?" , addr ;
litzep   = "." , addr ;
litrel   = "," , addr ;
litabs   = ";" , addr ;
rawzep   = "-" , addr ;
rawrel   = "_" , addr ;
rawabs   = "=" , addr ;
"####;

const UXNTAL_GRAMMAR_BNF: &str = r####"
<program> ::= <line> | <line> <program>
<line> ::= <number> | <opcode> | <rune> | <name> | <addr> | <macros> | <comment> | <ignore> | <toplab> | <padrel> | <padabd> | <lithex> | <rawstr> | <immjsi> | <immjmi> | <immjci> | <litzep> | <litrel> | <litabs> | <rawzep> | <rawrel> | <rawabs>
<nibble> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "a" | "b" | "c" | "d" | "e" | "f"
<byte> ::= <nibble> <nibble>
<short> ::= <nibble> <nibble> <nibble> <nibble>
<number> ::= <nibble>
           | <nibble> <nibble>
           | <nibble> <nibble> <nibble>
           | <nibble> <nibble> <nibble> <nibble>
<op> ::= "LIT" | "INC" | "POP" | "NIP" | "SWP" | "ROT" | "DUP" | "OVR"
       | "EQU" | "NEQ" | "GTH" | "LTH" | "JMP" | "JCN" | "JSR" | "STH"
       | "LDZ" | "STZ" | "LDR" | "STR" | "LDA" | "STA" | "DEI" | "DEO"
       | "ADD" | "SUB" | "MUL" | "DIV" | "AND" | "ORA" | "EOR" | "SFT"
<mode> ::= "2" | "k" | "r"
<opcode> ::= "BRK"
           | <op> <mode>
           | <op> <mode> <mode>
           | <op> <mode> <mode> <mode>
<rune> ::= "|" | "$" | "@" | "&" | "%" | "(" | "," | "_" | "." | "-" | ";" | "=" | "?" | "!" | "#" | "}" | "~" | "[" | "]"
<name> ::= <string>
         | <string> "/" <string>
         | "-" <number>
         | <opcode>
         | <rune> <string>
<addr> ::= <name>
         | "/" <string>
         | "&" <string>
         | "{"
<macros> ::= "%" <name> "{" <string> "}"
<comment> ::= "(" <string> ")"
<ignore> ::= "[" <string> "]"
<toplab> ::= "@" <name>
<padrel> ::= "$" <number> | "$" <name>
<padabd> ::= "|" <number> | "|" <name>
<lithex> ::= "#" <byte> | "#" <short>
<rawstr> ::= '"' <string> '"'
<immjsi> ::= <name> | "/" <string> | "{"
<immjmi> ::= "!" <addr>
<immjci> ::= "?" <addr>
<litzep> ::= "." <addr>
<litrel> ::= "," <addr>
<litabs> ::= ";" <addr>
<rawzep> ::= "-" <addr>
<rawrel> ::= "_" <addr>
<rawabs> ::= "=" <addr>
"####;


fn explain_parse_tree(parse_tree: &bnf::ParseTree) -> String {
    use bnf::ParseTreeNode;
    let lhs = parse_tree.lhs;
    let lhs = match lhs {
        bnf::Term::Nonterminal(s) => s.as_str(),
        bnf::Term::Terminal(i) => &format!("terminal_{}", i),
    };
    let mut children_desc = Vec::new();
    for node in parse_tree.rhs_iter() {
        match node {
            ParseTreeNode::Terminal(s) => children_desc.push(format!("'{}'", s)),
            ParseTreeNode::Nonterminal(child) => children_desc.push(explain_parse_tree(child)),
        }
    }
    let desc = match lhs.to_string().as_str() {
        "macros" => "a macro definition",
        "immjsi" => "an immediate jump or string",
        "rawstr" => "a raw string",
        "comment" => "a comment",
        "ignore" => "an ignored section",
        "toplab" => "a top-level label",
        "padrel" => "a relative pad",
        "padabd" => "an absolute pad",
        "lithex" => "a hex literal",
        "immjmi" => "an immediate jump (minus)",
        "immjci" => "an immediate jump (conditional)",
        "litzep" => "a zero-page literal",
        "litrel" => "a relative literal",
        "litabs" => "an absolute literal",
        "rawzep" => "a zero-page raw",
        "rawrel" => "a relative raw",
        "rawabs" => "an absolute raw",
        "opcode" => "an opcode",
        "number" => "a number",
        "name" => "a name",
        "addr" => "an address",
        "rune" => "a rune",
        _ => &lhs.to_string(),
    };
    if children_desc.is_empty() {
        format!("{}", desc)
    } else {
        format!("{}: {}", desc, children_desc.join(" "))
    }
}

use std::collections::HashSet;

/// Context for a parsed UXN-TAL Grammar, with helpers for extracting terminals.
pub struct GrammarUxnTalCtx {
    pub grammar: bnf::Grammar,
}

impl GrammarUxnTalCtx {
    /// Create a new context from an optional grammar string or file path.
    /// If `grammar_path_or_str` is Some, tries to read from file or parse as grammar string.
    /// If None, uses the built-in UXNTAL_GRAMMAR_BNF.
    pub fn new(grammar_path_or_str: Option<&str>) -> Result<Self, String> {
        let grammar_str = if let Some(path_or_str) = grammar_path_or_str {
            // Try to read as file, fallback to using as grammar string
            match std::fs::read_to_string(path_or_str) {
                Ok(contents) => contents,
                Err(_) => path_or_str.to_string(),
            }
        } else {
            UXNTAL_GRAMMAR_BNF.to_string()
        };
        let grammar: Grammar = grammar_str
            .parse()
            .map_err(|e| format!("Failed to parse grammar: {e}"))?;
        let r=Self { grammar };
        r.validate_opcodes_vs_grammar()?;
        Ok(r)
    }

    /// Returns the list of <op> terminals + BRK as strings.
pub fn op_list(&self) -> Vec<String> {
    let mut ops = self.get_terminals_of("op");
    if self.get_terminals_of("opcode").contains(&"BRK".to_string()) && !ops.contains(&"BRK".to_string()) {
        ops.push("BRK".to_string());
    }
    ops.sort();
    ops
}

pub fn get_full_op_list(&self) -> Vec<String> {
    let mut full_ops = vec![];

    // Add BRK if present
    if self.get_terminals_of("opcode").contains(&"BRK".to_string()) {
        full_ops.push("BRK".to_string());
    }

    let ops = self.get_terminals_of("op");
    let suffixes = self.mode_suffixes();

    for op in &ops {
        for suffix in &suffixes {
            if suffix.is_empty() {
                full_ops.push(op.clone());
            } else {
                full_ops.push(format!("{}{}", op, suffix));
            }
        }
    }

    full_ops.sort();
    full_ops.dedup();
    full_ops
}

    /// Returns the list of <rune> terminals as strings.
    pub fn rune_list(&self) -> Vec<String> {
        self.get_terminals_of("rune")
    }

    /// Helper: get all terminal strings for a given nonterminal.
    fn get_terminals_of(&self, nt: &str) -> Vec<String> {
        let mut result = HashSet::new();
        for prod in self.grammar.productions_iter() {
            if let bnf::Term::Nonterminal(ref lhs) = prod.lhs {
                if lhs == nt {
                    for expr in prod.rhs_iter() {
                        for term in expr.terms_iter() {
                            if let bnf::Term::Terminal(ref s) = term {
                                // Remove quotes if present
                                let trimmed = s.trim_matches('"').to_string();
                                result.insert(trimmed);
                            }
                        }
                    }
                }
            }
        }
        let mut v: Vec<_> = result.into_iter().collect();
        v.sort();
        v
    }

        /// Returns true if `candidate` is a valid terminal for the given nonterminal.
    pub fn is_terminal_of(&self, nt: &str, candidate: &str) -> bool {
        self.get_terminals_of(nt).contains(&candidate.to_string())
    }

    /// Returns all nonterminals in the grammar.
    pub fn all_nonterminals(&self) -> Vec<String> {
        let mut nts = HashSet::new();
        for prod in self.grammar.productions_iter() {
            if let bnf::Term::Nonterminal(ref lhs) = prod.lhs {
                nts.insert(lhs.clone());
            }
        }
        let mut v: Vec<_> = nts.into_iter().collect();
        v.sort();
        v
    }

        /// Returns the set of terminals that can appear first for the given nonterminal.
    pub fn first_terminals_of(&self, nt: &str) -> Vec<String> {
        let mut result = HashSet::new();
        let mut visited = HashSet::new();
        self.first_terminals_of_inner(nt, &mut result, &mut visited);
        let mut v: Vec<_> = result.into_iter().collect();
        v.sort();
        v
    }

//     For each production of the given nonterminal, look at the first term of each alternative.
// If it’s a terminal, add it to the result.
// If it’s a nonterminal, recursively collect its FIRST terminals.
// Uses a visited set to avoid infinite recursion on cycles.
    fn first_terminals_of_inner(
        &self,
        nt: &str,
        result: &mut HashSet<String>,
        visited: &mut HashSet<String>,
    ) {
        if !visited.insert(nt.to_string()) {
            // Prevent infinite recursion on cycles
            return;
        }
        for prod in self.grammar.productions_iter() {
            if let bnf::Term::Nonterminal(ref lhs) = prod.lhs {
                if lhs == nt {
                    for expr in prod.rhs_iter() {
                        if let Some(first_term) = expr.terms_iter().next() {
                            match first_term {
                                bnf::Term::Terminal(ref s) => {
                                    result.insert(s.trim_matches('"').to_string());
                                }
                                bnf::Term::Nonterminal(ref nt2) => {
                                    self.first_terminals_of_inner(nt2, result, visited);
                                }
                            }
                        }
                    }
                }
            }
        }
    }


    /// Returns the list of valid <op> names from the opcode table.
    pub fn op_list_from_table(&self) -> Vec<String> {
        let opcodes = Opcodes::new();
        opcodes
            .opcodes
            .keys()
            .map(|s| s.to_ascii_uppercase())
            .collect::<std::collections::HashSet<_>>() // deduplicate
            .into_iter()
            .collect::<Vec<_>>()
    }

fn terminal_of(tok: &Token) -> Option<&'static str> {
    Some(match tok {
        Token::Instruction(_)       => "opcode",
        Token::HexLiteral(_)        => "number",
        Token::DecLiteral(_)        => "number",
        Token::BinLiteral(_)        => "number",
        Token::CharLiteral(_)       => "number",
        Token::LabelDef(_)          => "toplab",     // or treat as "name" in some contexts
        Token::LabelRef(_)          => "name",
        Token::SublabelDef(_)       => "name",
        Token::SublabelRef(_)       => "name",
        Token::RelativeRef(_)       => "addr",
        Token::ConditionalRef(_)    => "?",          // if it’s the rune variant
        Token::DotRef(_)            => ".",
        Token::SemicolonRef(_)      => ";",
        Token::EqualsRef(_)         => "=",
        Token::CommaRef(_)          => ",",
        Token::UnderscoreRef(_)     => "_",
        Token::QuestionRef(_)       => "?",
        Token::ExclamationRef(_)    => "!",
        Token::RawAddressRef(_)     => "addr",
        Token::JSRRef(_)            => "addr",
        Token::HyphenRef(_)         => "-",
        Token::Padding(_)           => "padrel",
        Token::PaddingLabel(_)      => "padrel",
        Token::Skip(_)              => "ignore",     // or its own nonterminal if you have one
        Token::DeviceAccess(_,_)    => "name",
        Token::MacroDef(_)          => "macros",
        Token::MacroCall(_)         => "name",
        Token::RawString(_)         => "rawstr",
        Token::Include(_)           => "ignore",
        Token::BracketOpen          => "[",
        Token::BracketClose         => "]",
        Token::BraceOpen            => "{",
        Token::BraceClose           => "}",
        Token::Comment(_)           => "comment",
        Token::Newline              => "",           // skip
        Token::Eof                  => "$eof",
        _ => return None,
    })
}

pub fn validate_opcodes_vs_grammar(&self) -> Result<(), String> {
    let grammar_ops = self.get_full_op_list();
    let table_ops = self.op_list_from_table();

    let missing_in_table: Vec<_> = grammar_ops
        .iter()
        .filter(|op| !table_ops.contains(op))
        .collect();
    let missing_in_grammar: Vec<_> = table_ops
        .iter()
        .filter(|op| !grammar_ops.contains(op))
        .collect();

    if !missing_in_table.is_empty() || !missing_in_grammar.is_empty() {
        return Err(format!(
            "Opcode mismatch:\nMissing in table: {:?}\nMissing in grammar: {:?}",
            missing_in_table, missing_in_grammar
        ));
    }
    Ok(())
}

    /// Generate all valid UXN mode suffixes (ordered, non-repeating) from the grammar's <mode> terminals.
    pub fn mode_suffixes(&self) -> Vec<String> {
        let modes = self.get_terminals_of("mode");
        let mut suffixes = vec![String::new()];

        // Generate all ordered, non-repeating combinations (subsets) of modes
        for i in 0..modes.len() {
            suffixes.push(modes[i].clone());
            for j in (i+1)..modes.len() {
                suffixes.push(format!("{}{}", modes[i], modes[j]));
                for k in (j+1)..modes.len() {
                    suffixes.push(format!("{}{}{}", modes[i], modes[j], modes[k]));
                }
            }
        }
        suffixes.sort();
        suffixes.dedup();
        suffixes
    }
}
