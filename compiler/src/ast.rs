use std::fmt::{self};

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
    VariableDeclaration {
        is_mut: bool, // false for 'let', true for 'var'
        name: String,
        ty: Option<Type>,
        init: Option<Expression>,
    },
    // Later this will become an expression that returns the Unit type
    Assignment {
        target: Expression,
        value: Expression,
    },
    CompoundAssignment {
        operator: BinaryOperator,
        target: Expression,
        value: Expression,
    },
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    Block(Block),
    Unary {
        operator: UnaryOperator,
        expression: Box<Self>,
    },
    Binary {
        left: Box<Self>,
        operator: BinaryOperator,
        right: Box<Self>,
    },
    If {
        condition: Box<Self>,
        then_branch: Block,
        // None: no else branch
        // Some(Block): final else
        // Some (If): Else if
        else_branch: Option<Box<Self>>,
    },
    For {
        var_name: String,
        start: Box<Self>,
        end: Box<Self>,
        body: Block,
        label: Option<String>,
    },
    While {
        condition: Box<Self>,
        body: Block,
        label: Option<String>,
    },
    Break(Option<String>),
    Continue(Option<String>),
    Return(Option<Box<Self>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Int(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Named(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Negation, // -
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    And,
    Or,
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ ")?;
        for stmt in &self.statements {
            write!(f, "{stmt}; ")?;
        }
        if let Some(tail) = &self.tail_expression {
            write!(f, "{tail} ")?;
        }
        write!(f, "}}")
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(Literal::Int(n)) => write!(f, "{n}"),
            Self::Identifier(name) => write!(f, "{name}"),
            Self::Block(block) => write!(f, "{block}"),
            Self::Unary {
                operator,
                expression,
            } => {
                write!(f, "({operator}{expression})")
            }
            Self::Binary {
                left,
                operator,
                right,
            } => {
                // Outer parens make precedence immediately obvious!
                write!(f, "({left} {operator} {right})")
            }
            Self::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if let Some(else_expr) = else_branch {
                    write!(f, "if {condition} then {then_branch} else {else_expr}")
                } else {
                    write!(f, "if {condition} then {then_branch}")
                }
            }
            Self::For {
                var_name,
                start,
                end,
                body,
                label,
            } => {
                if let Some(label_str) = label {
                    write!(
                        f,
                        "'{label_str} for {var_name} in {start}..{end} then {body}"
                    )
                } else {
                    write!(f, " for {var_name} in {start}..{end} then {body}")
                }
            }
            Self::While {
                condition,
                body,
                label,
            } => {
                if let Some(label_str) = label {
                    write!(f, "'{label_str} while {condition} then {body}")
                } else {
                    write!(f, "while {condition} then {body}")
                }
            }
            Self::Break(str) => {
                if let Some(label) = str {
                    write!(f, "'{label} break")
                } else {
                    write!(f, "break")
                }
            }
            Self::Continue(str) => {
                if let Some(label) = str {
                    write!(f, "'{label} continue")
                } else {
                    write!(f, "continue")
                }
            }
            Self::Return(Some(expr)) => write!(f, "return {expr}"),
            Self::Return(None) => write!(f, "return"),
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VariableDeclaration {
                is_mut,
                name,
                ty,
                init,
            } => {
                let kw = if *is_mut { "var" } else { "let" };
                write!(f, "{kw} {name}")?;
                if let Some(t) = ty {
                    write!(f, ": {t}")?;
                }
                if let Some(e) = init {
                    write!(f, " = {e}")?;
                }
                Ok(())
            }
            Self::Assignment { target, value } => {
                write!(f, "{target} = {value}")
            }
            Self::CompoundAssignment {
                operator,
                target,
                value,
            } => {
                let op = match operator {
                    BinaryOperator::Add => "+=",
                    BinaryOperator::Subtract => "-=",
                    BinaryOperator::Multiply => "*=",
                    BinaryOperator::Divide => "/=",
                    BinaryOperator::Remainder => "%=",
                    _ => unreachable!(),
                };
                write!(f, "{target} {op} {value}")
            }
            Self::Expression(e) => write!(f, "{e}"),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Named(name) => write!(f, "{name}"),
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
            Self::And => write!(f, "&&"),
            Self::Or => write!(f, "||"),
            Self::EqualEqual => write!(f, "=="),
            Self::NotEqual => write!(f, "!="),
            Self::Less => write!(f, "<"),
            Self::LessEqual => write!(f, "<="),
            Self::Greater => write!(f, ">"),
            Self::GreaterEqual => write!(f, ">="),
        }
    }
}

struct Node {
    label: String,
    children: Vec<Self>,
}

impl Node {
    fn new(label: impl Into<String>, children: Vec<Self>) -> Self {
        Self {
            label: label.into(),
            children,
        }
    }

    fn render(&self, out: &mut String) {
        out.push_str(&self.label);
        out.push('\n');
        self.render_children(out, "");
    }

    fn render_children(&self, out: &mut String, prefix: &str) {
        let last = self.children.len().saturating_sub(1);
        for (i, child) in self.children.iter().enumerate() {
            let is_last = i == last;

            let connector = if is_last { "└─ " } else { "├─ " };
            out.push_str(prefix);
            out.push_str(connector);
            out.push_str(&child.label);
            out.push('\n');

            let next_prefix = format!("{prefix}{}", if is_last { "   " } else { "│  " });
            child.render_children(out, &next_prefix);
        }
    }
}

impl Program {
    pub fn dump_tree(&self) -> String {
        let mut out = String::new();
        self.to_node().render(&mut out);
        out
    }

    fn to_node(&self) -> Node {
        Node::new("Program", self.items.iter().map(Item::to_node).collect())
    }
}

impl Item {
    fn to_node(&self) -> Node {
        match self {
            Self::Function(func) => func.to_node(),
        }
    }
}

impl Function {
    fn to_node(&self) -> Node {
        let return_str = self
            .return_type
            .as_ref()
            .map_or_else(String::new, |t| format!(" -> {t}"));

        Node::new(
            format!("Function \"{}\"{}", self.name, return_str),
            vec![self.body.to_node()],
        )
    }
}

impl Block {
    fn to_node(&self) -> Node {
        let mut children: Vec<Node> = self.statements.iter().map(Statement::to_node).collect();

        if let Some(tail) = &self.tail_expression {
            match &**tail {
                Expression::Block(nested) => {
                    let mut node = nested.to_node();
                    node.label = format!("[tail] {}", node.label);
                    children.push(node);
                }
                other => {
                    children.push(Node::new(format!("[tail] {other}"), vec![]));
                }
            }
        }

        Node::new("Block", children)
    }
}

impl Statement {
    fn to_node(&self) -> Node {
        match self {
            Self::Expression(Expression::Block(block)) => block.to_node(),
            other => Node::new(other.to_string(), vec![]),
        }
    }
}
