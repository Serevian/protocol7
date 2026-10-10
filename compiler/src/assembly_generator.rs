use crate::{aast, ast};

pub struct AssemblyGenerator {
    ast: ast::Program,
}

impl AssemblyGenerator {
    #[must_use]
    pub const fn new(ast: ast::Program) -> Self {
        Self { ast }
    }

    pub fn generate(&mut self) -> aast::Program {
        let mut functions = Vec::new();

        for func in &self.ast.items {
            match func {
                ast::Item::Function(function) => functions.push(Self::generate_function(function)),
            }
        }

        aast::Program { functions }
    }

    fn generate_function(func: &ast::Function) -> aast::Function {
        let mut instructions = Vec::new();

        for statement in &func.body.statements {
            Self::generate_statement(statement, &mut instructions);
        }

        if let Some(tail_expression) = &func.body.tail_expression {
            Self::generate_expression(tail_expression, &mut instructions);
            if !matches!(instructions.last(), Some(aast::Instruction::Ret)) {
                instructions.push(aast::Instruction::Ret);
            }
        }

        // Ensure functions always end with Ret
        // Handles Unit functions
        if !matches!(instructions.last(), Some(aast::Instruction::Ret)) {
            instructions.push(aast::Instruction::Ret);
        }

        aast::Function {
            identifier: func.name.clone(),
            instructions,
        }
    }

    fn generate_statement(statement: &ast::Statement, instructions: &mut Vec<aast::Instruction>) {
        match statement {
            ast::Statement::Expression(expression) => {
                Self::generate_expression(expression, instructions);
            }
        }
    }

    fn generate_expression(
        expression: &ast::Expression,
        instructions: &mut Vec<aast::Instruction>,
    ) {
        match expression {
            ast::Expression::Literal(literal) => match literal {
                ast::Literal::Int(num) => {
                    instructions.push(aast::Instruction::Mov {
                        source: aast::Operand::Immediate(*num),
                        destination: aast::Operand::Register,
                    });
                }
            },
            ast::Expression::Return(expression) => {
                if let Some(expr) = expression {
                    Self::generate_expression(expr, instructions);
                }
                instructions.push(aast::Instruction::Ret);
            }
            ast::Expression::Unary {
                operator: _,
                expression: _,
            } => todo!(),
        }
    }
}
