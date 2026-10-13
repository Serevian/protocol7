use std::fmt::{self, Write as _};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Fun,
    Return,
    And,
    Or,
    Not,
    Let,
    Var,

    // Identifiers
    Identifier(String),
    IntLiteral(i64),

    Equal,
    EqualEqual,
    NotEqual,
    Plus,
    PlusEqual,
    Minus,
    MinusEqual,
    Arrow,
    Star,
    StarEqual,
    Slash,
    SlashEqual,
    Percent,
    PercentEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Colon,

    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,

    Newline,
    EndOfFile,
    Error(String),
}

impl TokenKind {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Fun => "Fun",
            Self::Return => "Return",
            Self::And => "And",
            Self::Or => "Or",
            Self::Not => "Not",
            Self::Let => "Let",
            Self::Var => "Var",
            Self::Identifier(_) => "Identifier",
            Self::IntLiteral(_) => "IntLiteral",
            Self::Equal => "Equal",
            Self::EqualEqual => "EqualEqual",
            Self::NotEqual => "NotEqual",
            Self::Plus => "Plus",
            Self::PlusEqual => "PlusEqual",
            Self::Minus => "Minus",
            Self::MinusEqual => "MinusEqual",
            Self::Arrow => "Arrow",
            Self::Star => "Star",
            Self::StarEqual => "StarEqual",
            Self::Slash => "Slash",
            Self::SlashEqual => "SlashEqual",
            Self::Percent => "Percent",
            Self::PercentEqual => "PercentEqual",
            Self::Less => "Less",
            Self::LessEqual => "LessEqual",
            Self::Greater => "Greater",
            Self::GreaterEqual => "GreaterEqual",
            Self::Colon => "Colon",
            Self::OpenParen => "OpenParen",
            Self::CloseParen => "CloseParen",
            Self::OpenBrace => "OpenBrace",
            Self::CloseBrace => "CloseBrace",
            Self::Newline => "Newline",
            Self::EndOfFile => "EndOfFile",
            Self::Error(_) => "Error",
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fun => f.write_str("fun"),
            Self::Return => f.write_str("return"),
            Self::And => f.write_str("and"),
            Self::Or => f.write_str("or"),
            Self::Not => f.write_str("not"),
            Self::Let => f.write_str("let"),
            Self::Var => f.write_str("var"),
            Self::Identifier(s) => f.write_str(s),
            Self::IntLiteral(n) => write!(f, "{n}"),
            Self::Equal => write!(f, "="),
            Self::EqualEqual => f.write_str("=="),
            Self::NotEqual => f.write_str("!="),
            Self::Plus => f.write_str("+"),
            Self::PlusEqual => f.write_str("+="),
            Self::Minus => f.write_str("-"),
            Self::MinusEqual => f.write_str("-="),
            Self::Arrow => f.write_str("->"),
            Self::Star => f.write_str("*"),
            Self::StarEqual => f.write_str("*="),
            Self::Slash => f.write_str("/"),
            Self::SlashEqual => f.write_str("/="),
            Self::Percent => f.write_str("%"),
            Self::PercentEqual => f.write_str("%="),
            Self::Less => f.write_str("<"),
            Self::LessEqual => f.write_str("<="),
            Self::Greater => f.write_str(">"),
            Self::GreaterEqual => f.write_str(">="),
            Self::Colon => f.write_str(":"),
            Self::OpenParen => f.write_str("("),
            Self::CloseParen => f.write_str(")"),
            Self::OpenBrace => f.write_str("{"),
            Self::CloseBrace => f.write_str("}"),
            Self::Newline => f.write_str("\\n"),
            Self::EndOfFile => f.write_str("<eof>"),
            Self::Error(msg) => write!(f, "{}", msg.escape_debug()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
}

impl Token {
    #[must_use]
    pub const fn new(kind: TokenKind) -> Self {
        Self { kind }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TokenKind::Identifier(_) | TokenKind::IntLiteral(_) | TokenKind::Error(_) => {
                write!(f, "{}({})", self.kind.name(), self.kind)
            }
            other => f.write_str(other.name()),
        }
    }
}

/// ```text
/// 1 | [fun] [main] [(] [)] [->] [{] [\n]
/// 2 |     [return] [1] [+] [2] [\n]
/// ```
#[must_use]
pub fn format_token_lines(tokens: &[Token]) -> String {
    let mut out = String::new();
    let mut line = 1usize;
    let mut at_line_start = true;

    for tok in tokens {
        if at_line_start {
            let _ = write!(out, "{line:>3} | ");
            at_line_start = false;
        }
        match tok.kind {
            TokenKind::Newline => {
                out.push_str("[\\n]\n");
                line += 1;
                at_line_start = true;
            }
            TokenKind::EndOfFile => out.push_str("[<eof>]"),
            _ => {
                let _ = write!(out, "[{}] ", tok.kind);
            }
        }
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}
