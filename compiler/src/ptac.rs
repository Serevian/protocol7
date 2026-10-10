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
