use crate::{
    ast::{Block, Expression, Function, Item, Literal, Program, Statement, Type, UnaryOperator},
    token::{Token, TokenKind},
};

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    #[must_use]
    pub const fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, cursor: 0 }
    }

    /// # Errors
    ///
    /// Will return `Err` if the parser finds a non conformant expression/statement
    pub fn parse(&mut self) -> Result<Program, String> {
        let mut items = Vec::new();
        self.skip_newlines();

        while !self.is_at_end() {
            items.push(self.parse_item()?);
            self.skip_newlines();
        }

        Ok(Program { items })
    }

    fn parse_item(&mut self) -> Result<Item, String> {
        match self.peek() {
            TokenKind::Fun => self.parse_function().map(Item::Function),
            other => Err(format!("Expected top-level item, found {other:?}")),
        }
    }

    // <function> ::= "fun" <identifier> "(" ")" [ "->" <type> ] "{" <block> "}"
    fn parse_function(&mut self) -> Result<Function, String> {
        self.expect(&TokenKind::Fun)?;

        let name = match self.advance().kind.clone() {
            TokenKind::Identifier(name) => name,
            other => return Err(format!("Expected function name, found {other:?}")),
        };

        // ( ... )
        self.expect(&TokenKind::OpenParen)?;
        // Parse parameters
        self.expect(&TokenKind::CloseParen)?;

        // Optional -> Type
        let return_type = if self.match_token(&TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // This way '{' can appear either after Type or on the next line
        self.skip_newlines();

        let body = self.parse_block()?;

        Ok(Function {
            name,
            return_type,
            body,
        })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        match self.advance().kind.clone() {
            TokenKind::Identifier(name) => Ok(Type::Named(name)),
            other => Err(format!("Expected type name, found {other:?}")),
        }
    }

    fn parse_block(&mut self) -> Result<Block, String> {
        self.expect(&TokenKind::OpenBrace)?;
        self.skip_newlines();

        let mut statements = Vec::new();
        let mut tail_expression = None;

        while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
            // Parse statements (variable declaration)

            // Otherwise it's an expression
            let expression = self.parse_expression()?;
            self.skip_newlines();

            // If the next token is a "}", then it's the tail expression return
            if self.check(&TokenKind::CloseBrace) {
                tail_expression = Some(Box::new(expression));
                break;
            }

            statements.push(Statement::Expression(expression));
        }

        self.expect(&TokenKind::CloseBrace)?;
        Ok(Block {
            statements,
            tail_expression,
        })
    }

    // Lowest precedence, return, break, continue, etc
    fn parse_expression(&mut self) -> Result<Expression, String> {
        if self.match_token(&TokenKind::Return) {
            if self.check(&TokenKind::Newline)
                || self.check(&TokenKind::CloseBrace)
                || self.is_at_end()
            {
                return Ok(Expression::Return(None));
            }

            let expression = self.parse_expression()?;
            return Ok(Expression::Return(Some(Box::new(expression))));
        }

        self.parse_unary()
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
        if self.match_token(&TokenKind::Minus) {
            let inner = self.parse_unary()?;
            return Ok(Expression::Unary {
                operator: UnaryOperator::Negation,
                expression: Box::new(inner),
            });
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::IntLiteral(num) => Ok(Expression::Literal(Literal::Int(num))),
            TokenKind::OpenParen => {
                let inner = self.parse_expression()?;
                self.expect(&TokenKind::CloseParen)?;
                Ok(inner)
            }
            other => Err(format!("Expected expression, found {other:?}")),
        }
    }

    fn peek(&self) -> &TokenKind {
        self.tokens
            .get(self.cursor)
            .map_or(&TokenKind::EndOfFile, |t| &t.kind)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.cursor += 1;
        }
        &self.tokens[self.cursor - 1]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), TokenKind::EndOfFile)
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.peek() == kind
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<&Token, String> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(format!("Expected {:?}, found {:?}", kind, self.peek()))
        }
    }

    fn skip_newlines(&mut self) {
        while self.check(&TokenKind::Newline) {
            self.advance();
        }
    }
}
