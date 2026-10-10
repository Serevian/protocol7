use crate::{ast, ptac};

pub struct PtacGenerator {
    ast: ast::Program,
    instructions: Vec<ptac::Instruction>,
    temp_counter: usize,
}

impl PtacGenerator {
    #[must_use]
    pub const fn new(ast: ast::Program) -> Self {
        Self {
            ast,
            instructions: Vec::new(),
            temp_counter: 0,
        }
    }

    #[must_use]
    pub fn generate(mut self) -> ptac::Program {
        // Now the borrow checker is happier, but a better solution would be to PtacGenerator to not own the ast and instead pass it to generate
        let items = std::mem::take(&mut self.ast.items);
        let mut functions = Vec::new();

        for item in &items {
            match item {
                ast::Item::Function(function) => functions.push(self.lower_function(function)),
            }
        }

        ptac::Program { functions }
    }

    fn lower_function(&mut self, function: &ast::Function) -> ptac::Function {
        // Clear state for new function
        self.instructions.clear();
        self.temp_counter = 0;

        for statement in &function.body.statements {
            self.lower_statement(statement);
        }

        if let Some(tail_expression) = &function.body.tail_expression {
            let result = self.lower_expression(tail_expression);

            // If the tail expression wasn't a explicit return, emit a Return instruction
            if !self.last_is_return() {
                self.emit(ptac::Instruction::Return(Some(result)));
            }
        }

        // Guarantee a return at the end of the function
        if !self.last_is_return() {
            self.emit(ptac::Instruction::Return(None));
        }

        ptac::Function {
            identifier: function.name.clone(),
            instructions: self.instructions.clone(),
        }
    }

    fn lower_statement(&mut self, statement: &ast::Statement) {
        match statement {
            ast::Statement::Expression(expression) => {
                self.lower_expression(expression);
            }
        }
    }

    fn lower_expression(&mut self, expression: &ast::Expression) -> ptac::Value {
        match expression {
            ast::Expression::Literal(literal) => match literal {
                ast::Literal::Int(int) => ptac::Value::Constant(*int),
            },
            ast::Expression::Unary {
                operator,
                expression,
            } => {
                let source = self.lower_expression(expression);
                let destination = self.new_temp();
                let operator = Self::lower_unary_operator(*operator);

                self.emit(ptac::Instruction::Unary {
                    operator,
                    source,
                    destination: destination.clone(),
                });

                destination
            }
            ast::Expression::Binary {
                left,
                operator,
                right,
            } => {
                let lhs = self.lower_expression(left);
                let rhs = self.lower_expression(right);
                let destination = self.new_temp();
                let operator = Self::lower_binary_operator(*operator);

                self.emit(ptac::Instruction::Binary {
                    operator,
                    source1: lhs,
                    source2: rhs,
                    destination: destination.clone(),
                });

                destination
            }
            ast::Expression::Return(expression) => {
                match expression {
                    // return <expr>
                    Some(value_expression) => {
                        let value = self.lower_expression(value_expression);
                        self.emit(ptac::Instruction::Return(Some(value)));
                    }
                    // return
                    None => {
                        self.emit(ptac::Instruction::Return(None));
                    }
                }
                // Return expression diverges, so its value is never read. Perhaps make a Never type?
                ptac::Value::Constant(0)
            }
        }
    }

    // Instead of doing t.x, maybe it can write function_name.x so collisions don't happen
    fn new_temp(&mut self) -> ptac::Value {
        let name = format!("t.{}", self.temp_counter);
        self.temp_counter += 1;
        ptac::Value::Var(name)
    }

    fn emit(&mut self, inst: ptac::Instruction) {
        self.instructions.push(inst);
    }

    fn last_is_return(&self) -> bool {
        matches!(self.instructions.last(), Some(ptac::Instruction::Return(_)))
    }

    const fn lower_unary_operator(op: ast::UnaryOperator) -> ptac::UnaryOperator {
        match op {
            ast::UnaryOperator::Negation => ptac::UnaryOperator::Negation,
        }
    }

    const fn lower_binary_operator(op: ast::BinaryOperator) -> ptac::BinaryOperator {
        match op {
            ast::BinaryOperator::Add => ptac::BinaryOperator::Add,
            ast::BinaryOperator::Subtract => ptac::BinaryOperator::Subtract,
            ast::BinaryOperator::Multiply => ptac::BinaryOperator::Multiply,
            ast::BinaryOperator::Divide => ptac::BinaryOperator::Divide,
            ast::BinaryOperator::Remainder => ptac::BinaryOperator::Remainder,
        }
    }
}
