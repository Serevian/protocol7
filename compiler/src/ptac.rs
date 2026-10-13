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
    /// dst = op src
    Unary {
        operator: UnaryOperator,
        source: Value,
        destination: Value,
    },
    Binary {
        operator: BinaryOperator,
        source1: Value,
        source2: Value,
        destination: Value,
    },
    Copy {
        source: Value,
        destination: Value,
    },
    Jump {
        target: String,
    },
    JumpIfZero {
        condition: Value,
        target: String,
    },
    JumpIfNotZero {
        condition: Value,
        target: String,
    },
    Label(String),
    Return(Option<Value>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Negation,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Constant(i64),
    Var(String),
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
        write!(f, "function {}:", self.identifier)?;
        for instruction in &self.instructions {
            write!(f, "\n{INDENT}{instruction}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unary {
                operator,
                source,
                destination,
            } => {
                write!(f, "{destination} = {operator}{source}")
            }
            Self::Binary {
                operator,
                source1,
                source2,
                destination,
            } => {
                write!(f, "{destination} = {source1} {operator} {source2}")
            }
            Self::Copy {
                source,
                destination,
            } => write!(f, "{destination} = {source}"),
            Self::Return(None) => write!(f, "return"),
            Self::Jump { target } => write!(f, "jump #{target}"),
            Self::JumpIfZero { condition, target } => {
                write!(f, "jump if {condition} is zero to #{target}")
            }
            Self::JumpIfNotZero { condition, target } => {
                write!(f, "jump if {condition} not zero to #{target}")
            }
            Self::Label(str) => write!(f, "\n#{str}:"),
            Self::Return(Some(value)) => write!(f, "return {value}"),
        }
    }
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negation => write!(f, "-"),
            Self::Not => write!(f, "!"),
        }
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Subtract => write!(f, "-"),
            Self::Multiply => write!(f, "*"),
            Self::Divide => write!(f, "/"),
            Self::Remainder => write!(f, "%"),
            Self::EqualEqual => write!(f, "=="),
            Self::NotEqual => write!(f, "!="),
            Self::Less => write!(f, "<"),
            Self::LessEqual => write!(f, "<="),
            Self::Greater => write!(f, ">"),
            Self::GreaterEqual => write!(f, ">="),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant(n) => write!(f, "{n}"),
            Self::Var(name) => write!(f, "{name}"),
        }
    }
}
