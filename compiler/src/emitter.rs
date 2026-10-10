use crate::aast::{
    BinaryOperator, Function, Instruction, Operand, Program, Register, UnaryOperator,
};

pub struct Emitter {
    output: String,
}

impl Emitter {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    #[must_use]
    pub fn emit_program(mut self, program: &Program) -> String {
        self.writeln(".intel_syntax noprefix");

        for func in &program.functions {
            self.emit_function(func);
        }

        // In Linux it's best to mark the stack as non-executable
        #[cfg(target_os = "linux")]
        self.writeln(".section .note.GNU-stack,\"\",@progbits");

        self.output
    }

    fn emit_function(&mut self, function: &Function) {
        // macOS C runtime requires an underscore prefix (_main)
        let name = if cfg!(target_os = "macos") {
            format!("_{}", function.identifier)
        } else {
            function.identifier.clone()
        };

        self.writeln(&format!(".globl {name}"));
        self.writeln(&format!("{name}:"));
        self.writeln("\tpush rbp");
        self.writeln("\tmov rbp, rsp");

        for inst in &function.instructions {
            self.emit_instruction(inst);
        }
    }

    fn emit_instruction(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::Move {
                source,
                destination,
            } => {
                let destination_str = Self::format_operand(destination);
                let source_str = Self::format_operand(source);
                self.writeln(&format!("\tmov {destination_str}, {source_str}"));
            }
            Instruction::Unary { operator, operand } => {
                let operand = Self::format_operand(operand);
                match operator {
                    UnaryOperator::Negation => {
                        self.writeln(&format!("\tneg {operand}"));
                    }
                    UnaryOperator::Not => {
                        self.writeln(&format!("\tnot {operand}"));
                    }
                }
            }
            Instruction::Binary {
                operator,
                source,
                destination,
            } => {
                let source = Self::format_operand(source);
                let destination = Self::format_operand(destination);
                match operator {
                    BinaryOperator::Add => self.writeln(&format!("\tadd {destination}, {source}")),
                    BinaryOperator::Substract => {
                        self.writeln(&format!("\tsub {destination}, {source}"));
                    }
                    BinaryOperator::Multiply => {
                        self.writeln(&format!("\timul {destination}, {source}"));
                    }
                }
            }
            Instruction::Division(operand) => self.writeln(&format!("\tidiv {operand}")),
            // Sign extends from EAX to EDX:EAX
            Instruction::SignExtension => self.writeln("\tcdq"),
            Instruction::AllocateStack(int) => self.writeln(&format!("\tsub rsp, {int}")),
            Instruction::Return => {
                self.writeln("\tmov rsp, rbp");
                self.writeln("\tpop rbp");
                self.writeln("\tret");
            }
        }
    }

    fn format_operand(operand: &Operand) -> String {
        match operand {
            Operand::Immediate(n) => n.to_string(),
            // For now maps to 32bit registers
            Operand::Register(register) => match register {
                Register::AX => "eax".to_string(),
                Register::DX => "edx".to_string(),
                Register::R10 => "r10d".to_string(),
                Register::R11 => "r11d".to_string(),
            },
            Operand::PseudoRegister(_) => {
                panic!("PSEUDO REGISTER IN EMITTER!!!")
            }
            Operand::Stack(offset) => {
                if *offset < 0 {
                    format!("DWORD PTR [rbp - {}]", offset.abs())
                } else {
                    format!("DWORD PTR [rbp + {offset}]")
                }
            }
        }
    }

    fn writeln(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }
}

impl Default for Emitter {
    fn default() -> Self {
        Self::new()
    }
}
