#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub identifier: String,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    Mov {
        source: Operand,
        destination: Operand,
    },
    Ret,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    Immediate(i64),
    Register,
}
