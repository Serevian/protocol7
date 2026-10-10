#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Fun,
    Return,

    // Identifiers
    Identifier(String),
    IntLiteral(i64),

    Arrow,
    Minus,

    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,

    Newline,
    EndOfFile,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
}

impl Token {
    pub fn new(kind: TokenKind) -> Self {
        Self { kind }
    }
}
