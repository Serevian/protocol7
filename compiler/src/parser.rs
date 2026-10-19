use crate::{
    ast::{
        BinaryOperator, Block, Expression, Function, Item, Literal, Program, Statement, Type,
        UnaryOperator,
    },
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
        self.skip_newlines();
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
            if self.check(&TokenKind::Let) || self.check(&TokenKind::Var) {
                statements.push(self.parse_variable_declaration()?);
                self.skip_newlines();
                continue;
            }

            // Otherwise it's an expression
            let expression = self.parse_expression()?;

            // Check for assignments
            if let Some(op) = self.match_assign_operator() {
                self.skip_newlines();

                match &expression {
                    Expression::Identifier(_) => {}
                    _ => return Err("Invalid assignment target: must be a variable".to_string()),
                }

                let value = self.parse_expression()?;
                match op {
                    AssignOperator::Simple => {
                        statements.push(Statement::Assignment {
                            target: expression,
                            value,
                        });
                    }
                    AssignOperator::Compound(binary_operator) => {
                        statements.push(Statement::CompoundAssignment {
                            operator: binary_operator,
                            target: expression,
                            value,
                        });
                    }
                }

                self.skip_newlines();
                continue;
            }

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

    fn parse_variable_declaration(&mut self) -> Result<Statement, String> {
        let is_mut = self.match_token(&TokenKind::Var);
        if !is_mut {
            self.expect(&TokenKind::Let)?;
        }

        let name = match self.advance().kind.clone() {
            TokenKind::Identifier(name) => name,
            other => return Err(format!("Expected variable name, found {other:?}")),
        };

        let ty = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let init = if self.match_token(&TokenKind::Equal) {
            self.skip_newlines();
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(Statement::VariableDeclaration {
            is_mut,
            name,
            ty,
            init,
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

        self.parse_expression_climbing(0)
    }

    fn parse_expression_climbing(&mut self, min_precedence: u8) -> Result<Expression, String> {
        // Parse left atoms, parens expressions or unary operators
        let mut lhs = self.parse_unary()?;
        while let Some((precedence, association, operator)) = Self::binary_op_info(self.peek()) {
            if precedence < min_precedence {
                break;
            }

            // Consume operator
            self.advance();
            // Now that it has consumed the operator, it can skip newlines
            self.skip_newlines();

            let next_min_precedence = match association {
                // For non-association, raise so chained comparisons will error
                Association::Left | Association::None => precedence + 1,
                Association::Right => precedence,
            };

            let rhs = self.parse_expression_climbing(next_min_precedence)?;

            // If the next token is another of association none, reject it
            if association == Association::None
                && let Some((next_precedence, _, _)) = Self::binary_op_info(self.peek())
                && next_precedence == precedence
            {
                return Err("Comparison operators can't be chained".to_string());
            }

            lhs = Expression::Binary {
                left: Box::new(lhs),
                operator,
                right: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    const fn binary_op_info(kind: &TokenKind) -> Option<(u8, Association, BinaryOperator)> {
        match kind {
            // Level 1
            TokenKind::Or => Some((1, Association::Left, BinaryOperator::Or)),

            // Level 2
            TokenKind::And => Some((2, Association::Left, BinaryOperator::And)),

            // Level 3 (Non-Associative)
            TokenKind::EqualEqual => Some((3, Association::None, BinaryOperator::EqualEqual)),
            TokenKind::NotEqual => Some((3, Association::None, BinaryOperator::NotEqual)),
            TokenKind::Less => Some((3, Association::None, BinaryOperator::Less)),
            TokenKind::LessEqual => Some((3, Association::None, BinaryOperator::LessEqual)),
            TokenKind::Greater => Some((3, Association::None, BinaryOperator::Greater)),
            TokenKind::GreaterEqual => Some((3, Association::None, BinaryOperator::GreaterEqual)),

            // Level 4
            TokenKind::Plus => Some((4, Association::Left, BinaryOperator::Add)),
            TokenKind::Minus => Some((4, Association::Left, BinaryOperator::Subtract)),

            // Level 5
            TokenKind::Star => Some((5, Association::Left, BinaryOperator::Multiply)),
            TokenKind::Slash => Some((5, Association::Left, BinaryOperator::Divide)),
            TokenKind::Percent => Some((5, Association::Left, BinaryOperator::Remainder)),

            _ => None,
        }
    }

    fn match_assign_operator(&mut self) -> Option<AssignOperator> {
        if self.match_token(&TokenKind::Equal) {
            Some(AssignOperator::Simple)
        } else if self.match_token(&TokenKind::PlusEqual) {
            Some(AssignOperator::Compound(BinaryOperator::Add))
        } else if self.match_token(&TokenKind::MinusEqual) {
            Some(AssignOperator::Compound(BinaryOperator::Subtract))
        } else if self.match_token(&TokenKind::StarEqual) {
            Some(AssignOperator::Compound(BinaryOperator::Multiply))
        } else if self.match_token(&TokenKind::SlashEqual) {
            Some(AssignOperator::Compound(BinaryOperator::Divide))
        } else if self.match_token(&TokenKind::PercentEqual) {
            Some(AssignOperator::Compound(BinaryOperator::Remainder))
        } else {
            None
        }
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
        if self.match_token(&TokenKind::Minus) {
            let inner = self.parse_unary()?;
            return Ok(Expression::Unary {
                operator: UnaryOperator::Negation,
                expression: Box::new(inner),
            });
        }
        if self.match_token(&TokenKind::Not) {
            let inner = self.parse_unary()?;
            return Ok(Expression::Unary {
                operator: UnaryOperator::Not,
                expression: Box::new(inner),
            });
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        if self.check(&TokenKind::While) {
            return self.parse_while_expression();
        }
        if self.check(&TokenKind::For) {
            return self.parse_for_expression();
        }
        if self.match_token(&TokenKind::Break) {
            return Ok(Expression::Break(None));
        }
        if self.match_token(&TokenKind::Continue) {
            return Ok(Expression::Continue(None));
        }
        if self.check(&TokenKind::If) {
            return self.parse_if_expression();
        }

        if self.check(&TokenKind::OpenBrace) {
            let block = self.parse_block()?;
            return Ok(Expression::Block(block));
        }

        let token = self.advance().clone();
        match token.kind {
            TokenKind::IntLiteral(num) => Ok(Expression::Literal(Literal::Int(num))),
            TokenKind::Identifier(name) => Ok(Expression::Identifier(name)),
            TokenKind::OpenParen => {
                let inner = self.parse_expression()?;
                self.expect(&TokenKind::CloseParen)?;
                Ok(inner)
            }
            other => Err(format!("Expected expression, found {other:?}")),
        }
    }

    fn parse_if_expression(&mut self) -> Result<Expression, String> {
        self.expect(&TokenKind::If)?;

        let condition = self.parse_expression()?;
        self.skip_newlines();

        let then_branch = self.parse_block()?;
        self.skip_newlines();

        let else_branch = if self.match_token(&TokenKind::Else) {
            self.skip_newlines();

            if self.check(&TokenKind::If) {
                Some(Box::new(self.parse_if_expression()?))
            } else if self.check(&TokenKind::OpenBrace) {
                let block = self.parse_block()?;
                Some(Box::new(Expression::Block(block)))
            } else {
                return Err("Expected '{' or 'if' after 'else'".to_string());
            }
        } else {
            None
        };

        Ok(Expression::If {
            condition: Box::new(condition),
            then_branch,
            else_branch,
        })
    }

    fn parse_while_expression(&mut self) -> Result<Expression, String> {
        self.expect(&TokenKind::While)?;

        let condition = self.parse_expression()?;
        self.skip_newlines();

        let body = self.parse_block()?;

        Ok(Expression::While {
            condition: Box::new(condition),
            body,
            label: None,
        })
    }

    fn parse_for_expression(&mut self) -> Result<Expression, String> {
        self.expect(&TokenKind::For)?;

        let var_name = match self.advance().kind.clone() {
            TokenKind::Identifier(name) => name,
            other => return Err(format!("Expected loop variable name, found {other:?}")),
        };

        self.expect(&TokenKind::In)?;

        let start = self.parse_expression_climbing(0)?;
        self.expect(&TokenKind::DotDot)?;
        let end = self.parse_expression_climbing(0)?;

        self.skip_newlines();
        let body = self.parse_block()?;

        Ok(Expression::For {
            var_name,
            start: Box::new(start),
            end: Box::new(end),
            body,
            label: None,
        })
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Association {
    Left,
    Right, // Still don't have any right association operators
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssignOperator {
    Simple,                   // =
    Compound(BinaryOperator), // +=, -=, *=, /=, %=
}
