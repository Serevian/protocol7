use crate::assembly_ast::*;

pub struct Emitter {
    output: String,
}

impl Emitter {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

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

        for inst in &function.instructions {
            self.emit_instruction(inst);
        }
    }

    fn emit_instruction(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::Mov {
                source,
                destination,
            } => {
                let destination_str = self.format_operand(destination);
                let source_str = self.format_operand(source);
                self.writeln(&format!("\tmov {destination_str}, {source_str}"));
            }
            Instruction::Ret => self.writeln("\tret"),
        }
    }

    fn format_operand(&self, operand: &Operand) -> String {
        match operand {
            Operand::Immediate(n) => n.to_string(),
            // For now maps to the 32-bit return register `eax`
            Operand::Register => "eax".to_string(),
        }
    }

    fn writeln(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }
}
