use crate::token::{
    Token,
    TokenKind::{self},
};

pub struct Lexer {
    source: String,
    cursor: usize,
}

impl Lexer {
    #[must_use]
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            cursor: 0,
        }
    }

    #[must_use]
    pub fn tokenize(mut self) -> Vec<Token> {
        let mut vec = Vec::new();

        while let Some(ch) = self.advance() {
            let token = match ch {
                'a'..='z' | 'A'..='Z' => self.identifier_or_keyword(ch),
                '0'..='9' => self.number(ch),
                '+' => Token::new(TokenKind::Plus),
                '-' => match self.peek() {
                    Some('>') => {
                        let _ = self.advance();
                        Token::new(TokenKind::Arrow)
                    }
                    _ => Token::new(TokenKind::Minus),
                },
                '*' => Token::new(TokenKind::Star),
                '/' => match self.peek() {
                    Some('/') => {
                        let _ = self.advance();

                        while let Some(ch) = self.peek() {
                            // Don't advance past a newline, the tokenizer will handle it
                            if ch == '\n' {
                                break;
                            }
                            self.advance();
                        }
                        continue;
                    }
                    _ => Token::new(TokenKind::Slash),
                },
                '%' => Token::new(TokenKind::Percent),
                '(' => Token::new(TokenKind::OpenParen),
                ')' => Token::new(TokenKind::CloseParen),
                '{' => Token::new(TokenKind::OpenBrace),
                '}' => Token::new(TokenKind::CloseBrace),
                '\n' => self.handle_newlines(),
                ' ' | '\t' | '\r' => continue,
                _ => Token::new(TokenKind::Error(ch.to_string())),
            };

            vec.push(token);
        }
        vec.push(Token::new(TokenKind::EndOfFile));

        vec
    }

    fn peek(&self) -> Option<char> {
        self.source[self.cursor..].chars().next()
    }

    /// Returns None if EOF
    fn advance(&mut self) -> Option<char> {
        let remaining = &self.source[self.cursor..];
        let c = remaining.chars().next()?;
        self.cursor += c.len_utf8();

        Some(c)
    }

    fn identifier_or_keyword(&mut self, first_character: char) -> Token {
        let mut identifier = String::from(first_character);
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        let kind = match identifier.as_str() {
            "fun" => TokenKind::Fun,
            "return" => TokenKind::Return,
            _ => TokenKind::Identifier(identifier),
        };

        Token::new(kind)
    }

    // Doesn't support floating point numbers yet
    fn number(&mut self, first_number: char) -> Token {
        let mut number = String::from(first_number);
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '_' {
                number.push(self.advance().unwrap());
            } else if ch.is_alphabetic() {
                while let Some(invalid_ch) = self.peek() {
                    if invalid_ch.is_alphanumeric() || invalid_ch == '_' {
                        number.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                return Token::new(TokenKind::Error(number));
            } else {
                break;
            }
        }

        let int = number.parse::<i64>().unwrap();

        Token::new(TokenKind::IntLiteral(int))
    }

    fn handle_newlines(&mut self) -> Token {
        while let Some(ch) = self.peek() {
            let _ = match ch {
                '\n' | ' ' | '\t' | '\r' => self.advance(),
                _ => break,
            };
        }

        Token::new(TokenKind::Newline)
    }
}
