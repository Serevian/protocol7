use std::fmt::{self, Write as _};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Fun,
    Return,

    // Identifiers
    Identifier(String),
    IntLiteral(i64),

    Arrow,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

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
            Self::Identifier(_) => "Identifier",
            Self::IntLiteral(_) => "IntLiteral",
            Self::Arrow => "Arrow",
            Self::Plus => "Plus",
            Self::Minus => "Minus",
            Self::Star => "Star",
            Self::Slash => "Slash",
            Self::Percent => "Percent",
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
            Self::Identifier(s) => f.write_str(s),
            Self::IntLiteral(n) => write!(f, "{n}"),
            Self::Arrow => f.write_str("->"),
            Self::Plus => f.write_str("+"),
            Self::Minus => f.write_str("-"),
            Self::Star => f.write_str("*"),
            Self::Slash => f.write_str("/"),
            Self::Percent => f.write_str("%"),
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
///  #  KIND        LEXEME
/// ---------------------------
///  0  Fun         fun
///  1  Identifier  main
/// ```
#[must_use]
pub fn format_token_table(tokens: &[Token]) -> String {
    let rows: Vec<(String, &'static str, String)> = tokens
        .iter()
        .enumerate()
        .map(|(i, t)| (i.to_string(), t.kind.name(), t.kind.to_string()))
        .collect();

    let idx_w = rows.iter().map(|r| r.0.len()).max().unwrap_or(0).max(1);
    let kind_w = rows
        .iter()
        .map(|r| r.1.len())
        .max()
        .unwrap_or(0)
        .max("KIND".len());
    let lex_w = rows
        .iter()
        .map(|r| r.2.chars().count())
        .max()
        .unwrap_or(0)
        .max("LEXEME".len());

    let mut out = String::new();
    let _ = writeln!(out, "{:>idx_w$}  {:<kind_w$}  LEXEME", "#", "KIND");
    let _ = writeln!(out, "{}", "-".repeat(idx_w + kind_w + lex_w + 4));
    for (idx, kind, lexeme) in &rows {
        let _ = writeln!(out, "{idx:>idx_w$}  {kind:<kind_w$}  {lexeme}");
    }
    out
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
