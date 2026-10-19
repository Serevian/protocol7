use std::collections::HashMap;

use crate::ast::{
    Block, Expression, Function, Item, Program,
    Statement::{self},
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct VarSymbol {
    unique_name: String,
    is_mut: bool,
}

pub struct SemanticAnalyzer {
    /// Stack of lexical scopes.
    scopes: Vec<HashMap<String, VarSymbol>>,
    var_counter: usize,
    loop_stack: Vec<String>,
    loop_counter: usize,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            var_counter: 0,
            loop_stack: Vec::new(),
            loop_counter: 0,
        }
    }

    pub fn analyze(&mut self, program: &mut Program) -> Result<(), String> {
        // Variable resolving
        for item in &mut program.items {
            match item {
                Item::Function(function) => self.resolve_function(function)?,
            }
        }

        // Loop labelling
        for item in &mut program.items {
            match item {
                Item::Function(func) => self.label_function(func)?,
            }
        }
        Ok(())
    }

    fn resolve_function(&mut self, function: &mut Function) -> Result<(), String> {
        self.var_counter = 0;
        self.scopes.clear();

        self.resolve_block(&mut function.body)
    }

    fn resolve_block(&mut self, block: &mut Block) -> Result<(), String> {
        self.enter_scope();

        for statement in &mut block.statements {
            self.resolve_statement(statement)?;
        }

        if let Some(tail_expression) = &mut block.tail_expression {
            self.resolve_expression(tail_expression)?;
        }
        self.exit_scope();
        Ok(())
    }

    fn resolve_statement(&mut self, statement: &mut Statement) -> Result<(), String> {
        match statement {
            Statement::VariableDeclaration {
                is_mut, name, init, ..
            } => {
                // Let's make sure the variable doesn't read itself
                // Example: let x = x + 1
                if let Some(expression) = init {
                    self.resolve_expression(expression)?;
                }

                let unique_name = self.declare_variable(name, *is_mut)?;

                *name = unique_name;
            }
            Statement::Assignment { target, value } => {
                // Let's make sure the target is a variable
                let Expression::Identifier(var_name) = target else {
                    return Err("Invalid assignment target: must be a variable".to_string());
                };

                let symbol = self.lookup(var_name)?;

                if !symbol.is_mut {
                    return Err(format!("Cannot assign to immutable variable '{var_name}'"));
                }

                *var_name = symbol.unique_name.clone();

                self.resolve_expression(value)?;
            }
            Statement::CompoundAssignment {
                operator: _,
                target,
                value,
            } => {
                let Expression::Identifier(var_name) = target else {
                    return Err("Invalid assignment target: must be a variable".to_string());
                };

                let symbol = self.lookup(var_name)?;

                if !symbol.is_mut {
                    return Err(format!("Cannot assign to immutable variable '{var_name}'"));
                }

                *var_name = symbol.unique_name.clone();

                self.resolve_expression(value)?;
            }
            Statement::Expression(expr) => {
                self.resolve_expression(expr)?;
            }
        }

        Ok(())
    }

    fn resolve_expression(&mut self, expression: &mut Expression) -> Result<(), String> {
        match expression {
            Expression::Identifier(name) => {
                let symbol = self.lookup(name)?;
                *name = symbol.unique_name.clone();
            }
            Expression::Block(block) => {
                self.resolve_block(block)?;
            }
            Expression::Unary { expression, .. } => self.resolve_expression(expression)?,
            Expression::Binary { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }
            Expression::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(condition)?;
                self.resolve_block(then_branch)?;

                if let Some(else_expression) = else_branch {
                    self.resolve_expression(else_expression)?;
                }
            }
            Expression::Return(expression) => {
                if let Some(inner) = expression {
                    self.resolve_expression(inner)?;
                }
            }
            Expression::For {
                var_name,
                start,
                end,
                body,
                ..
            } => {
                self.resolve_expression(start)?;
                self.resolve_expression(end)?;

                // Can't use resolve_block here due to the existence of var_name
                self.enter_scope();
                let unique_var = self.declare_variable(var_name, true)?;
                *var_name = unique_var;

                for stmt in &mut body.statements {
                    self.resolve_statement(stmt)?;
                }
                if let Some(tail) = &mut body.tail_expression {
                    self.resolve_expression(tail)?;
                }
                self.exit_scope();
            }
            Expression::While {
                condition, body, ..
            } => {
                self.resolve_expression(condition)?;
                self.resolve_block(body)?;
            }
            Expression::Literal(_) | Expression::Break(_) | Expression::Continue(_) => {}
        }
        Ok(())
    }

    fn label_function(&mut self, function: &mut Function) -> Result<(), String> {
        self.loop_stack.clear();
        self.label_block(&mut function.body)
    }

    fn label_block(&mut self, block: &mut Block) -> Result<(), String> {
        for statement in &mut block.statements {
            self.label_statement(statement)?;
        }

        if let Some(tail) = &mut block.tail_expression {
            self.label_expression(tail)?;
        }

        Ok(())
    }

    fn label_statement(&mut self, stmt: &mut Statement) -> Result<(), String> {
        match stmt {
            Statement::VariableDeclaration { init, .. } => {
                if let Some(expr) = init {
                    self.label_expression(expr)?;
                }
            }
            Statement::Assignment { target, value }
            | Statement::CompoundAssignment { target, value, .. } => {
                self.label_expression(target)?;
                self.label_expression(value)?;
            }
            Statement::Expression(expr) => {
                self.label_expression(expr)?;
            }
        }
        Ok(())
    }

    fn label_expression(&mut self, expr: &mut Expression) -> Result<(), String> {
        match expr {
            Expression::While {
                condition,
                body,
                label,
            } => {
                // Generate unique label
                let loop_label = format!("loop.{}", self.loop_counter);
                self.loop_counter += 1;

                *label = Some(loop_label.clone());

                self.loop_stack.push(loop_label);

                self.label_expression(condition)?;
                self.label_block(body)?;

                self.loop_stack.pop();
            }
            Expression::For {
                start,
                end,
                body,
                label,
                ..
            } => {
                let loop_label = format!("loop.{}", self.loop_counter);
                self.loop_counter += 1;
                *label = Some(loop_label.clone());

                self.loop_stack.push(loop_label);
                self.label_expression(start)?;
                self.label_expression(end)?;
                self.label_block(body)?;
                self.loop_stack.pop();
            }
            Expression::Break(target_id) => {
                let current_loop_id = self
                    .loop_stack
                    .last()
                    .ok_or_else(|| "Cannot 'break' outside of a loop".to_string())?;

                *target_id = Some(current_loop_id.clone());
            }

            Expression::Continue(target_id) => {
                let current_loop_id = self
                    .loop_stack
                    .last()
                    .ok_or_else(|| "Cannot 'continue' outside of a loop".to_string())?;

                *target_id = Some(current_loop_id.clone());
            }

            Expression::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.label_expression(condition)?;
                self.label_block(then_branch)?;
                if let Some(else_expr) = else_branch {
                    self.label_expression(else_expr)?;
                }
            }

            Expression::Block(block) => {
                self.label_block(block)?;
            }

            Expression::Unary { expression, .. } => {
                self.label_expression(expression)?;
            }

            Expression::Binary { left, right, .. } => {
                self.label_expression(left)?;
                self.label_expression(right)?;
            }

            Expression::Return(opt_expr) => {
                if let Some(inner) = opt_expr {
                    self.label_expression(inner)?;
                }
            }

            Expression::Literal(_) | Expression::Identifier(_) => {}
        }
        Ok(())
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop().expect("Scope stack underflow");
    }

    fn declare_variable(&mut self, name: &str, is_mut: bool) -> Result<String, String> {
        let current_scope = self.scopes.last_mut().expect("No active scope");

        // Let's not allow shadowing in the same scope
        if current_scope.contains_key(name) {
            return Err(format!(
                "Variable '{name}' is already declared in this scope"
            ));
        }

        let unique_name = format!("{name}.{}", self.var_counter);
        self.var_counter += 1;

        current_scope.insert(
            name.to_string(),
            VarSymbol {
                unique_name: unique_name.clone(),
                is_mut,
            },
        );

        Ok(unique_name)
    }

    fn lookup(&self, name: &str) -> Result<&VarSymbol, String> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Ok(symbol);
            }
        }
        Err(format!("Undeclared variable '{name}'"))
    }
}
