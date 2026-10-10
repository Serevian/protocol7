#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub identifier: String,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Move {
        source: Operand,
        destination: Operand,
    },
    Unary {
        operator: UnaryOperator,
        operand: Operand,
    },
    Binary {
        operator: BinaryOperator,
        source: Operand,
        destination: Operand,
    },
    Division(Operand),
    SignExtension, // CD
    AllocateStack(i64),
    Return,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Negation,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Substract,
    Multiply,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    Immediate(i64),
    Register(Register),
    PseudoRegister(String),
    Stack(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    AX,
    DX,
    R10,
    R11,
}

use std::fmt;

const INDENT: &str = "    ";

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, function) in self.functions.iter().enumerate() {
            if i > 0 {
                f.write_str("\n\n")?; // blank line between functions
            }
            write!(f, "{function}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:", self.identifier)?;
        for instruction in &self.instructions {
            write!(f, "\n{INDENT}{instruction}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Intel order: destination first
            Self::Move {
                source,
                destination,
            } => {
                write!(f, "mov {destination}, {source}")
            }
            Self::Unary { operator, operand } => write!(f, "{operator} {operand}"),
            Self::Binary {
                operator,
                source,
                destination,
            } => write!(f, "{operator} {destination}, {source}"),
            Self::Division(operand) => write!(f, "idivl {operand}"),
            Self::SignExtension => write!(f, "cd"),
            Self::AllocateStack(bytes) => write!(f, "sub rsp, {bytes}"),
            Self::Return => write!(f, "ret"),
        }
    }
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negation => write!(f, "neg"),
            Self::Not => write!(f, "not"),
        }
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add => write!(f, "add"),
            Self::Substract => write!(f, "sub"),
            Self::Multiply => write!(f, "imul"),
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Immediate(n) => write!(f, "{n}"),
            Self::Register(register) => write!(f, "{register}"),
            // Should be gone after pseudo-register replacement; `%` makes leaks obvious
            Self::PseudoRegister(name) => write!(f, "%{name}"),
            Self::Stack(0) => write!(f, "DWORD PTR [rbp]"),
            // `{:+}` always prints the sign: [rbp-4] / [rbp+8]
            Self::Stack(offset) => write!(f, "DWORD PTR [rbp{offset:+}]"),
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 32-bit names, matching the DWORD PTR memory operands
        match self {
            Self::AX => write!(f, "eax"),
            Self::DX => write!(f, "edx"),
            Self::R10 => write!(f, "r10d"),
            Self::R11 => write!(f, "r11d"),
        }
    }
}
