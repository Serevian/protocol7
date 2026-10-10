use std::collections::HashMap;

use crate::{
    aast::{self, Operand::Immediate},
    ptac::{self},
};

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
            functions.push(Self::lower_function(function));
        }

        // Second pass: replace pseudoregisters
        for function in &mut functions {
            let stack_offset = Self::replace_pseudoregisters(function);

            // Third pass: fix functions
            Self::fix_function(function, stack_offset);
        }
        aast::Program { functions }
    }

    fn lower_function(function: &ptac::Function) -> aast::Function {
        let mut instructions = Vec::new();

        for instruction in &function.instructions {
            Self::lower_instructions(&mut instructions, instruction);
        }

        aast::Function {
            identifier: function.identifier.clone(),
            instructions,
        }
    }

    fn lower_instructions(
        instructions: &mut Vec<aast::Instruction>,
        instruction: &ptac::Instruction,
    ) {
        match instruction {
            ptac::Instruction::Unary {
                operator,
                source,
                destination,
            } => {
                let destination = Self::lower_value(destination);
                instructions.push(aast::Instruction::Move {
                    source: Self::lower_value(source),
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
                    instructions.push(aast::Instruction::Move {
                        source: Self::lower_value(v),
                        destination: aast::Operand::Register(aast::Register::AX),
                    });
                    instructions.push(aast::Instruction::Return);
                }
                None => {
                    instructions.push(aast::Instruction::Return);
                }
            },
            ptac::Instruction::Binary {
                operator,
                source1,
                source2,
                destination,
            } => {
                let source1 = Self::lower_value(source1);
                let source2 = Self::lower_value(source2);
                let destination = Self::lower_value(destination);
                match operator {
                    ptac::BinaryOperator::Add
                    | ptac::BinaryOperator::Subtract
                    | ptac::BinaryOperator::Multiply => {
                        instructions.push(aast::Instruction::Move {
                            source: source1,
                            destination: destination.clone(),
                        });
                        match operator {
                            ptac::BinaryOperator::Add => {
                                instructions.push(aast::Instruction::Binary {
                                    operator: aast::BinaryOperator::Add,
                                    source: source2,
                                    destination,
                                });
                            }
                            ptac::BinaryOperator::Subtract => {
                                instructions.push(aast::Instruction::Binary {
                                    operator: aast::BinaryOperator::Substract,
                                    source: source2,
                                    destination,
                                });
                            }
                            ptac::BinaryOperator::Multiply => {
                                instructions.push(aast::Instruction::Binary {
                                    operator: aast::BinaryOperator::Multiply,
                                    source: source2,
                                    destination,
                                });
                            }
                            _ => {}
                        }
                    }
                    ptac::BinaryOperator::Divide | ptac::BinaryOperator::Remainder => {
                        instructions.push(aast::Instruction::Move {
                            source: source1,
                            destination: aast::Operand::Register(aast::Register::AX),
                        });
                        instructions.push(aast::Instruction::SignExtension);
                        instructions.push(aast::Instruction::Division(source2));
                        match operator {
                            ptac::BinaryOperator::Divide => {
                                instructions.push(aast::Instruction::Move {
                                    source: aast::Operand::Register(aast::Register::AX),
                                    destination,
                                });
                            }
                            ptac::BinaryOperator::Remainder => {
                                instructions.push(aast::Instruction::Move {
                                    source: aast::Operand::Register(aast::Register::DX),
                                    destination,
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    fn lower_value(value: &ptac::Value) -> aast::Operand {
        match value {
            ptac::Value::Constant(int) => aast::Operand::Immediate(*int),
            ptac::Value::Var(string) => aast::Operand::PseudoRegister(string.clone()),
        }
    }

    fn replace_pseudoregisters(function: &mut aast::Function) -> i64 {
        // Reset per function: maps "t.0" -> -8, "t.1" -> -16, etc.
        let mut current_offset: i64 = 0;
        let mut stack_map: HashMap<String, i64> = HashMap::new();

        for instruction in &mut function.instructions {
            match instruction {
                aast::Instruction::Unary {
                    operator: _,
                    operand,
                } => Self::replace_operand(operand, &mut stack_map, &mut current_offset),
                aast::Instruction::Move {
                    source,
                    destination,
                }
                | aast::Instruction::Binary {
                    operator: _,
                    source,
                    destination,
                } => {
                    Self::replace_operand(source, &mut stack_map, &mut current_offset);
                    Self::replace_operand(destination, &mut stack_map, &mut current_offset);
                }
                aast::Instruction::Division(operand) => {
                    Self::replace_operand(operand, &mut stack_map, &mut current_offset);
                }
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

                *operand = aast::Operand::Stack(offset);
            }
        }
    }

    fn fix_function(function: &mut aast::Function, stack_offset: i64) {
        let old = std::mem::take(&mut function.instructions);
        let mut fixed = Vec::with_capacity(old.len() + 1);

        fixed.push(aast::Instruction::AllocateStack(stack_offset));

        for instruction in old {
            match instruction {
                // mov can't have two stack operands, so route through R10
                aast::Instruction::Move {
                    source: aast::Operand::Stack(source_stack),
                    destination: aast::Operand::Stack(destination_stack),
                } => {
                    fixed.push(aast::Instruction::Move {
                        source: aast::Operand::Stack(source_stack),
                        destination: aast::Operand::Register(aast::Register::R10),
                    });
                    fixed.push(aast::Instruction::Move {
                        source: aast::Operand::Register(aast::Register::R10),
                        destination: aast::Operand::Stack(destination_stack),
                    });
                }
                aast::Instruction::Binary {
                    operator: aast::BinaryOperator::Multiply,
                    source,
                    destination: aast::Operand::Stack(destination_stack),
                } => {
                    fixed.push(aast::Instruction::Move {
                        source: aast::Operand::Stack(destination_stack),
                        destination: aast::Operand::Register(aast::Register::R11),
                    });
                    fixed.push(aast::Instruction::Binary {
                        operator: aast::BinaryOperator::Multiply,
                        source,
                        destination: aast::Operand::Register(aast::Register::R11),
                    });
                    fixed.push(aast::Instruction::Move {
                        source: aast::Operand::Register(aast::Register::R11),
                        destination: aast::Operand::Stack(destination_stack),
                    });
                }
                aast::Instruction::Binary {
                    operator,
                    source: aast::Operand::Stack(source_stack),
                    destination: aast::Operand::Stack(destination_stack),
                } => match operator {
                    aast::BinaryOperator::Add | aast::BinaryOperator::Substract => {
                        fixed.push(aast::Instruction::Move {
                            source: aast::Operand::Stack(source_stack),
                            destination: aast::Operand::Register(aast::Register::R10),
                        });
                        fixed.push(aast::Instruction::Binary {
                            operator,
                            source: aast::Operand::Register(aast::Register::R10),
                            destination: aast::Operand::Stack(destination_stack),
                        });
                    }
                    aast::BinaryOperator::Multiply => {
                        unreachable!("handled by the Multiply arm above")
                    }
                },
                aast::Instruction::Division(operand) => {
                    if let Immediate(_) = operand {
                        fixed.push(aast::Instruction::Move {
                            source: operand,
                            destination: aast::Operand::Register(aast::Register::R10),
                        });
                        fixed.push(aast::Instruction::Division(aast::Operand::Register(
                            aast::Register::R10,
                        )));
                    } else {
                        fixed.push(aast::Instruction::Division(operand));
                    }
                }
                // everything else passes through unchanged
                other => fixed.push(other),
            }
        }

        function.instructions = fixed;
    }
}
