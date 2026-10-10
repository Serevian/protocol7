use std::{collections::HashMap, hash::Hash};

use crate::{aast, ast, ptac};

pub struct AssemblyGenerator {
    ast: ptac::Program,
}

impl AssemblyGenerator {
    #[must_use]
    pub const fn new(ptac: ptac::Program) -> Self {
        Self { ast: ptac }
    }

    pub fn generate(&mut self) -> aast::Program {
        let mut functions = Vec::new();

        // First pass
        for function in &self.ast.functions {
            functions.push(self.lower_function(function));
        }

        // Second pass: replace pseudoregisters
        for function in &mut functions {
            let stack_offset = self.replace_pseudoregisters(function);

            // Third pass: fix functions
            self.fix_function(function, stack_offset);
        }
        aast::Program { functions }
    }

    fn lower_function(&self, function: &ptac::Function) -> aast::Function {
        let mut instructions = Vec::new();

        for instruction in &function.instructions {
            self.lower_instructions(&mut instructions, instruction);
        }

        aast::Function {
            identifier: function.identifier.clone(),
            instructions,
        }
    }

    fn lower_instructions(
        &self,
        instructions: &mut Vec<aast::Instruction>,
        instruction: &ptac::Instruction,
    ) {
        match instruction {
            ptac::Instruction::Unary {
                operator,
                source,
                destination,
            } => {
                let destination = self.lower_value(destination);
                instructions.push(aast::Instruction::Mov {
                    source: self.lower_value(source),
                    destination: destination.clone(),
                });
                match operator {
                    ptac::UnaryOperator::Negation => {
                        instructions.push(aast::Instruction::Unary {
                            operator: aast::UnaryOperator::Negation,
                            operand: destination,
                        });
                    }
                }
            }
            ptac::Instruction::Return(value) => match value {
                Some(v) => {
                    instructions.push(aast::Instruction::Mov {
                        source: self.lower_value(v),
                        destination: aast::Operand::Register(aast::Register::AX),
                    });
                    instructions.push(aast::Instruction::Ret);
                }
                None => {
                    instructions.push(aast::Instruction::Ret);
                }
            },
        }
    }

    fn lower_value(&self, value: &ptac::Value) -> aast::Operand {
        match value {
            ptac::Value::Constant(int) => aast::Operand::Immediate(*int),
            ptac::Value::Var(string) => aast::Operand::PseudoRegister(string.clone()),
        }
    }

    fn replace_pseudoregisters(&self, function: &mut aast::Function) -> i64 {
        // Reset per function: maps "t.0" -> -8, "t.1" -> -16, etc.
        let mut current_offset: i64 = 0;
        let mut stack_map: HashMap<String, i64> = HashMap::new();

        for instruction in &mut function.instructions {
            match instruction {
                aast::Instruction::Mov {
                    source,
                    destination,
                } => {
                    Self::replace_operand(source, &mut stack_map, &mut current_offset);
                    Self::replace_operand(destination, &mut stack_map, &mut current_offset);
                }
                aast::Instruction::Unary {
                    operator: _,
                    operand,
                } => Self::replace_operand(operand, &mut stack_map, &mut current_offset),
                _ => {}
            }
        }

        // Total stack size allocated
        current_offset.abs()
    }

    fn replace_operand(
        operand: &mut aast::Operand,
        stack_map: &mut HashMap<String, i64>,
        current_offset: &mut i64,
    ) {
        match operand {
            aast::Operand::Immediate(_) | aast::Operand::Register(_) | aast::Operand::Stack(_) => {}
            aast::Operand::PseudoRegister(name) => {
                let offset = *stack_map.entry(name.clone()).or_insert_with(|| {
                    // In x86-64, variablesl ive below %rbp
                    // -4 for 32bit values or -8 for 64-bit
                    *current_offset -= 8;
                    *current_offset
                });

                *operand = aast::Operand::Stack(offset)
            }
        }
    }

    fn fix_function(&self, function: &mut aast::Function, stack_offset: i64) {
        let old = std::mem::take(&mut function.instructions);
        let mut fixed = Vec::with_capacity(old.len() + 1);

        fixed.push(aast::Instruction::AllocateStack(stack_offset));

        for instruction in old {
            match instruction {
                // mov can't have two stack operands, so route through R10
                aast::Instruction::Mov {
                    source: aast::Operand::Stack(source_stack),
                    destination: aast::Operand::Stack(destination_stack),
                } => {
                    fixed.push(aast::Instruction::Mov {
                        source: aast::Operand::Stack(source_stack),
                        destination: aast::Operand::Register(aast::Register::R10),
                    });
                    fixed.push(aast::Instruction::Mov {
                        source: aast::Operand::Register(aast::Register::R10),
                        destination: aast::Operand::Stack(destination_stack),
                    });
                }
                // everything else passes through unchanged
                other => fixed.push(other),
            }
        }

        function.instructions = fixed;
    }
}
