use crate::{assembly_ast, ast};

pub struct AssemblyGenerator {
    ast: ast::Program,
}

impl AssemblyGenerator {
    pub fn new(ast: ast::Program) -> Self {
        Self { ast }
    }

    pub fn generate(&mut self) -> assembly_ast::Program {
        let mut functions = Vec::new();

        for func in &self.ast.items {
            match func {
                ast::Item::Function(function) => functions.push(self.generate_function(function)),
            }
        }

        assembly_ast::Program { functions }
    }

    fn generate_function(&self, func: &ast::Function) -> assembly_ast::Function {
        let mut instructions = Vec::new();

        for statement in &func.body.statements {
            self.generate_statement(statement, &mut instructions);
        }

        if let Some(tail_expression) = &func.body.tail_expression {
            self.generate_expression(tail_expression, &mut instructions);
            if !matches!(instructions.last(), Some(assembly_ast::Instruction::Ret)) {
                instructions.push(assembly_ast::Instruction::Ret);
            }
        }

        // Ensure functions always end with Ret
        // Handles Unit functions
        if !matches!(instructions.last(), Some(assembly_ast::Instruction::Ret)) {
            instructions.push(assembly_ast::Instruction::Ret);
        }

        assembly_ast::Function {
            identifier: func.name.clone(),
            instructions,
        }
    }

    fn generate_statement(
        &self,
        statement: &ast::Statement,
        instructions: &mut Vec<assembly_ast::Instruction>,
    ) {
        match statement {
            ast::Statement::Expression(expression) => {
                self.generate_expression(expression, instructions);
            }
            _ => todo!(),
        }
    }

    fn generate_expression(
        &self,
        expression: &ast::Expression,
        instructions: &mut Vec<assembly_ast::Instruction>,
    ) {
        match expression {
            ast::Expression::Literal(literal) => match literal {
                ast::Literal::Int(num) => {
                    instructions.push(assembly_ast::Instruction::Mov {
                        source: assembly_ast::Operand::Immediate(*num),
                        destination: assembly_ast::Operand::Register,
                    });
                }
            },
            ast::Expression::Return(expression) => {
                if let Some(expr) = expression {
                    self.generate_expression(expr, instructions);
                }
                instructions.push(assembly_ast::Instruction::Ret);
            }
            ast::Expression::Unary {
                operator,
                expression,
            } => todo!(),
        }
    }
}
