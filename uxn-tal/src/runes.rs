/// Represents a "Rune" in the assembler, which is a symbolic character used to denote
/// various syntactic and semantic elements in the language.
///
/// # Padding Runes
/// - `|` (`Absolute`): Absolute padding rune.
/// - `$` (`Relative`): Relative padding rune.
/// - `#` (`LiteralNumber`): Literal number rune.
///
/// # Label Runes
/// - `@` (`Parent`): Parent label rune.
/// - `&` (`Child`): Child label rune.
/// - `"` (`RawAscii`): Raw ASCII rune.
///
/// # Addressing Runes
/// - `,` (`LiteralRelative`): Literal relative addressing rune.
/// - `_` (`RawRelative`): Raw relative addressing rune.
/// - `(` (`CommentOpen`): Opens a comment.
/// - `)` (`CommentClose`): Closes a comment.
/// - `.` (`LiteralZeroPage`): Literal zero-page addressing rune.
/// - `-` (`RawZeroPage`): Raw zero-page addressing rune.
/// - `{` (`AnonymousOpen`): Opens an anonymous block.
/// - `}` (`AnonymousClose`): Closes an anonymous block.
/// - `;` (`LiteralAbsolute`): Literal absolute addressing rune.
/// - `=` (`RawAbsolute`): Raw absolute addressing rune.
/// - `[` (`IgnoredOpen`): Opens an ignored section.
/// - `]` (`IgnoredClose`): Closes an ignored section.
///
/// # Immediate Runes
/// - `!` (`Jmi`): JMI immediate rune.
/// - `?` (`Jci`): JCI immediate rune.
///
/// # Pre-processor Runes
/// - `%` (`MacroOpen`): Opens a macro.
/// - `}` (`MacroClose`): Closes a macro (note: also used for `AnonymousClose`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rune {
    // Padding Runes
    Absolute,      // '|'
    Relative,      // '$'
    LiteralNumber, // '#'

    // Label Runes
    Parent,   // '@'
    Child,    // '&'
    RawAscii, // '"'

    // Addressing Runes
    LiteralRelative, // ','
    RawRelative,     // '_'
    CommentOpen,     // '('
    CommentClose,    // ')'
    LiteralZeroPage, // '.'
    RawZeroPage,     // '-'
    AnonymousOpen,   // '{'
    AnonymousClose,  // '}'
    LiteralAbsolute, // ';'
    RawAbsolute,     // '='
    IgnoredOpen,     // '['
    IgnoredClose,    // ']'

    // Immediate Runes
    Jmi, // '!'
    Jci, // '?'

    // Pre-processor Runes
    MacroOpen,  // '%'
    MacroClose, // '}'

    None, // ' '
}

impl From<char> for Rune {
    fn from(c: char) -> Self {
        match c {
            // Padding Runes
            '|' => Rune::Absolute,
            '$' => Rune::Relative,
            '#' => Rune::LiteralNumber,

            // Label Runes
            '@' => Rune::Parent,
            '&' => Rune::Child,
            '"' => Rune::RawAscii,

            // Addressing Runes
            ',' => Rune::LiteralRelative,
            '_' => Rune::RawRelative,
            '(' => Rune::CommentOpen,
            ')' => Rune::CommentClose,
            '.' => Rune::LiteralZeroPage,
            '-' => Rune::RawZeroPage,
            '{' => Rune::AnonymousOpen,
            '}' => Rune::AnonymousClose,
            ';' => Rune::LiteralAbsolute,
            '=' => Rune::RawAbsolute,
            '[' => Rune::IgnoredOpen,
            ']' => Rune::IgnoredClose,

            // Immediate Runes
            '!' => Rune::Jmi,
            '?' => Rune::Jci,

            // Pre-processor Runes
            '%' => Rune::MacroOpen,

            _ => Rune::None,
        }
    }
}

impl ToString for Rune {
    fn to_string(&self) -> String {
        match self {
            // Padding Runes
            Rune::Absolute => "|",
            Rune::Relative => "$",
            Rune::LiteralNumber => "#",

            // Label Runes
            Rune::Parent => "@",
            Rune::Child => "&",
            Rune::RawAscii => "\"",

            // Addressing Runes
            Rune::LiteralRelative => ",",
            Rune::RawRelative => "_",
            Rune::CommentOpen => "(",
            Rune::CommentClose => ")",
            Rune::LiteralZeroPage => ".",
            Rune::RawZeroPage => "-",
            Rune::AnonymousOpen => "{",
            Rune::AnonymousClose => "}",
            Rune::LiteralAbsolute => ";",
            Rune::RawAbsolute => "=",
            Rune::IgnoredOpen => "[",
            Rune::IgnoredClose => "]",

            // Immediate Runes
            Rune::Jmi => "!",
            Rune::Jci => "?",

            // Pre-processor Runes
            Rune::MacroOpen => "%",
            Rune::MacroClose => "}", // Note: '}' is also used for AnonymousClose

            Rune::None => " ",
        }
        .to_string()
    }
}
