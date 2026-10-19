use crate::{ast, ptac};

pub struct PtacGenerator {
    ast: ast::Program,
    instructions: Vec<ptac::Instruction>,
    temp_counter: usize,
    label_counter: usize,
}

impl PtacGenerator {
    #[must_use]
    pub const fn new(ast: ast::Program) -> Self {
        Self {
            ast,
            instructions: Vec::new(),
            temp_counter: 0,
            label_counter: 0,
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
        // This can be refactored to be a per function struct instead of mutable state
        self.instructions.clear();
        self.temp_counter = 0;
        self.label_counter = 0;

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
            ast::Statement::VariableDeclaration { name, init, .. } => {
                if let Some(expr) = init {
                    let source = self.lower_expression(expr);
                    self.emit(ptac::Instruction::Copy {
                        source,
                        destination: ptac::Value::Var(name.clone()),
                    });
                }
                // Else, don't do nothing
                // TODO: Implement a Definite Assigment Analysis in the semantic analyzer
            }
            ast::Statement::Assignment { target, value } => {
                let source = self.lower_expression(value);
                let dest_name = match target {
                    ast::Expression::Identifier(name) => name.clone(),
                    _ => unreachable!(
                        "Semantic analysis hopefully guarantees target is an identifier"
                    ),
                };

                self.emit(ptac::Instruction::Copy {
                    source,
                    destination: ptac::Value::Var(dest_name),
                });
            }
            ast::Statement::CompoundAssignment {
                operator,
                target,
                value,
            } => {
                let target_name = match target {
                    ast::Expression::Identifier(name) => name.clone(),
                    _ => unreachable!(
                        "Semantic analysis hopefully guarantees target is an identifier"
                    ),
                };

                let current_val = ptac::Value::Var(target_name.clone());

                let rhs_val = self.lower_expression(value);

                let temp = self.new_temp();
                let op = Self::lower_binary_operator(*operator);

                self.emit(ptac::Instruction::Binary {
                    operator: op,
                    source1: current_val,
                    source2: rhs_val,
                    destination: temp.clone(),
                });

                self.emit(ptac::Instruction::Copy {
                    source: temp,
                    destination: ptac::Value::Var(target_name),
                });
            }
        }
    }

    fn lower_expression(&mut self, expression: &ast::Expression) -> ptac::Value {
        match expression {
            ast::Expression::Literal(literal) => match literal {
                ast::Literal::Int(int) => ptac::Value::Constant(*int),
            },
            ast::Expression::Block(block) => self.lower_block_value(block),
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
            } => match operator {
                ast::BinaryOperator::And => self.lower_logical_and(left, right),
                ast::BinaryOperator::Or => self.lower_logical_or(left, right),
                _ => {
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
            },
            ast::Expression::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let end_label = self.new_label("if_end");
                let else_label = self.new_label("if_else");
                let destination = self.new_temp();

                let condition_value = self.lower_expression(condition);
                self.emit(ptac::Instruction::JumpIfZero {
                    condition: condition_value,
                    target: else_label.clone(),
                });

                let then_value = self.lower_block_value(then_branch);
                self.emit(ptac::Instruction::Copy {
                    source: then_value,
                    destination: destination.clone(),
                });
                self.emit(ptac::Instruction::Jump {
                    target: end_label.clone(),
                });

                self.emit(ptac::Instruction::Label(else_label));
                if let Some(else_expression) = else_branch {
                    let else_value = self.lower_expression(else_expression);
                    self.emit(ptac::Instruction::Copy {
                        source: else_value,
                        destination: destination.clone(),
                    });
                }

                self.emit(ptac::Instruction::Label(end_label));

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
            ast::Expression::Identifier(name) => ptac::Value::Var(name.clone()),
            ast::Expression::While {
                condition,
                body,
                label,
            } => {
                let loop_label = label.as_ref().expect("While loop must be labeled");
                let start_label = format!("{loop_label}_start");
                let continue_label = format!("{loop_label}_continue");
                let end_label = format!("{loop_label}_end");

                self.emit(ptac::Instruction::Label(start_label.clone()));
                self.emit(ptac::Instruction::Label(continue_label));

                let condition_value = self.lower_expression(condition);
                self.emit(ptac::Instruction::JumpIfZero {
                    condition: condition_value,
                    target: end_label.clone(),
                });

                self.lower_block_value(body);

                self.emit(ptac::Instruction::Jump {
                    target: start_label,
                });

                self.emit(ptac::Instruction::Label(end_label));

                ptac::Value::Constant(0)
            }
            ast::Expression::For {
                var_name,
                start,
                end,
                body,
                label,
            } => {
                let loop_label = label.as_ref().expect("While loop must be labeled");
                let start_label = format!("{loop_label}_start");
                let continue_label = format!("{loop_label}_continue");
                let end_label = format!("{loop_label}_end");

                let start_val = self.lower_expression(start);
                self.emit(ptac::Instruction::Copy {
                    source: start_val,
                    destination: ptac::Value::Var(var_name.clone()),
                });

                let end_val = self.lower_expression(end);
                let end_temp = self.new_temp();
                self.emit(ptac::Instruction::Copy {
                    source: end_val,
                    destination: end_temp.clone(),
                });

                self.emit(ptac::Instruction::Label(start_label.clone()));

                let cond_temp = self.new_temp();
                self.emit(ptac::Instruction::Binary {
                    operator: ptac::BinaryOperator::Less,
                    source1: ptac::Value::Var(var_name.clone()),
                    source2: end_temp,
                    destination: cond_temp.clone(),
                });
                self.emit(ptac::Instruction::JumpIfZero {
                    condition: cond_temp,
                    target: end_label.clone(),
                });

                self.lower_block_value(body);

                self.emit(ptac::Instruction::Label(continue_label));

                let inc_temp = self.new_temp();
                self.emit(ptac::Instruction::Binary {
                    operator: ptac::BinaryOperator::Add,
                    source1: ptac::Value::Var(var_name.clone()),
                    source2: ptac::Value::Constant(1),
                    destination: inc_temp.clone(),
                });
                self.emit(ptac::Instruction::Copy {
                    source: inc_temp,
                    destination: ptac::Value::Var(var_name.clone()),
                });

                self.emit(ptac::Instruction::Jump {
                    target: start_label,
                });

                self.emit(ptac::Instruction::Label(end_label));

                ptac::Value::Constant(0)
            }
            ast::Expression::Break(label) => {
                let loop_label = label.as_ref().expect("Break must be labeled");
                self.emit(ptac::Instruction::Jump {
                    target: format!("{loop_label}_end"),
                });
                ptac::Value::Constant(0)
            }
            ast::Expression::Continue(label) => {
                let loop_label = label.as_ref().expect("Continue must be labeled");
                self.emit(ptac::Instruction::Jump {
                    target: format!("{loop_label}_continue"),
                });
                ptac::Value::Constant(0)
            }
        }
    }

    fn lower_logical_and(
        &mut self,
        left: &ast::Expression,
        right: &ast::Expression,
    ) -> ptac::Value {
        let false_label = self.new_label("and_false");
        let end_label = self.new_label("and_end");
        let destination = self.new_temp();

        let v1 = self.lower_expression(left);
        self.emit(ptac::Instruction::JumpIfZero {
            condition: v1,
            target: false_label.clone(),
        });

        let v2 = self.lower_expression(right);
        self.emit(ptac::Instruction::JumpIfZero {
            condition: v2,
            target: false_label.clone(),
        });

        // Both sides are true
        self.emit(ptac::Instruction::Copy {
            source: ptac::Value::Constant(1),
            destination: destination.clone(),
        });
        self.emit(ptac::Instruction::Jump {
            target: end_label.clone(),
        });

        // At least one side was false
        self.emit(ptac::Instruction::Label(false_label));
        self.emit(ptac::Instruction::Copy {
            source: ptac::Value::Constant(0),
            destination: destination.clone(),
        });

        self.emit(ptac::Instruction::Label(end_label));

        destination
    }

    fn lower_logical_or(&mut self, left: &ast::Expression, right: &ast::Expression) -> ptac::Value {
        let true_label = self.new_label("or_true");
        let end_label = self.new_label("or_end");
        let destination = self.new_temp();

        let v1 = self.lower_expression(left);
        self.emit(ptac::Instruction::JumpIfNotZero {
            condition: v1,
            target: true_label.clone(),
        });

        let v2 = self.lower_expression(right);
        self.emit(ptac::Instruction::JumpIfNotZero {
            condition: v2,
            target: true_label.clone(),
        });

        // Both sides are false
        self.emit(ptac::Instruction::Copy {
            source: ptac::Value::Constant(0),
            destination: destination.clone(),
        });
        self.emit(ptac::Instruction::Jump {
            target: end_label.clone(),
        });

        // At least one side is true
        self.emit(ptac::Instruction::Label(true_label));
        self.emit(ptac::Instruction::Copy {
            source: ptac::Value::Constant(1),
            destination: destination.clone(),
        });

        self.emit(ptac::Instruction::Label(end_label));

        destination
    }

    // Instead of doing t.x, maybe it can write function_name.x so collisions don't happen
    fn new_temp(&mut self) -> ptac::Value {
        let name = format!("t.{}", self.temp_counter);
        self.temp_counter += 1;
        ptac::Value::Var(name)
    }

    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("{prefix}.{}", self.label_counter);
        self.label_counter += 1;
        label
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
            ast::UnaryOperator::Not => ptac::UnaryOperator::Not,
        }
    }

    fn lower_binary_operator(op: ast::BinaryOperator) -> ptac::BinaryOperator {
        match op {
            ast::BinaryOperator::Add => ptac::BinaryOperator::Add,
            ast::BinaryOperator::Subtract => ptac::BinaryOperator::Subtract,
            ast::BinaryOperator::Multiply => ptac::BinaryOperator::Multiply,
            ast::BinaryOperator::Divide => ptac::BinaryOperator::Divide,
            ast::BinaryOperator::Remainder => ptac::BinaryOperator::Remainder,
            ast::BinaryOperator::EqualEqual => ptac::BinaryOperator::EqualEqual,
            ast::BinaryOperator::NotEqual => ptac::BinaryOperator::NotEqual,
            ast::BinaryOperator::Less => ptac::BinaryOperator::Less,
            ast::BinaryOperator::LessEqual => ptac::BinaryOperator::LessEqual,
            ast::BinaryOperator::Greater => ptac::BinaryOperator::Greater,
            ast::BinaryOperator::GreaterEqual => ptac::BinaryOperator::GreaterEqual,
            ast::BinaryOperator::And | ast::BinaryOperator::Or => {
                unreachable!("And and Or are handled in lower_expression as control flow")
            }
        }
    }

    fn lower_block_value(&mut self, block: &ast::Block) -> ptac::Value {
        for statement in &block.statements {
            self.lower_statement(statement);
        }

        if let Some(tail) = &block.tail_expression {
            self.lower_expression(tail)
        } else {
            ptac::Value::Constant(0)
        }
    }
}
