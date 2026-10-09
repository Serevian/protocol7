pub struct Program {
    pub functions: Vec<Function>,
}

pub struct Function {
    pub identifier: String,
    pub instructions: Vec<Instruction>,
}

pub enum Instruction {
    Mov {
        source: Operand,
        destination: Operand,
    },
    Ret,
}

pub enum Operand {
    Immediate(i64),
    Register,
}
