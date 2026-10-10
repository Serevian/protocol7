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
    Return(Option<Value>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Negation,
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
            Self::Return(None) => write!(f, "return"),
            Self::Return(Some(value)) => write!(f, "return {value}"),
        }
    }
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negation => write!(f, "-"),
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
