#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Function(Function),
    // Struct
    // Protocol
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    // list of parameters
    pub return_type: Option<Type>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub tail_expression: Option<Box<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Return(Option<Box<Expression>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Named(String),
}
